use crate::protocol::RpcError;
use ctxpack_compiler::{compile_context_pack_from_plan_for_agent, render_pack_markdown};
use ctxpack_core::{ContextPack, RepoRoot};
use ctxpack_index::{
    dependency_edges, load_or_refresh_inventory, read_safe_source, symbol_search, test_map,
    DependencyOptions, InventoryOptions, SourceReadStatus, SymbolOptions, SOURCE_READ_MAX_BYTES,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadResourceParams {
    uri: String,
}

pub(crate) fn read_resource(params: Value) -> Result<Value, RpcError> {
    let params: ReadResourceParams = serde_json::from_value(params).map_err(|error| {
        RpcError::invalid_params(format!("invalid resources/read params: {error}"))
    })?;

    let content = match params.uri.as_str() {
        "ctxpack://repo/summary" => {
            let repo = discover_repo(None)?;
            resource_json(&repo_summary(&repo.path)?)
        }
        "ctxpack://repo/test-map" => {
            let repo = discover_repo(None)?;
            resource_json(&repo_test_map(&repo.path)?)
        }
        "ctxpack://repo/dependency-graph" => {
            let repo = discover_repo(None)?;
            resource_json(&repo_dependency_graph(&repo.path)?)
        }
        "ctxpack://pack/guide" => pack_guide_markdown(),
        uri if uri.starts_with("ctxpack://pack/") => read_pack_resource(uri)?,
        uri if uri.starts_with("ctxpack://file/") => {
            let repo = discover_repo(None)?;
            read_file_resource(&repo.path, uri)?
        }
        uri if uri.starts_with("ctxpack://symbol/") => {
            let repo = discover_repo(None)?;
            read_symbol_resource(&repo.path, uri)?
        }
        uri => {
            return Err(RpcError::invalid_params(format!(
                "unsupported resource URI: {uri}"
            )))
        }
    };

    Ok(json!({
        "contents": [{
            "uri": params.uri,
            "mimeType": content.mime_type,
            "text": content.text
        }]
    }))
}

struct ResourceContent {
    mime_type: &'static str,
    text: String,
}

#[derive(Debug, Clone)]
struct CachedResource {
    mime_type: &'static str,
    text: String,
}

const MAX_PACK_RESOURCE_CACHE_ENTRIES: usize = 24;

#[derive(Debug, Default)]
struct PackResourceCache {
    entries: BTreeMap<String, CachedResource>,
    insertion_order: VecDeque<String>,
}

fn pack_resource_cache() -> &'static Mutex<PackResourceCache> {
    static CACHE: OnceLock<Mutex<PackResourceCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(PackResourceCache::default()))
}

#[cfg(test)]
pub(crate) fn clear_pack_resource_cache() {
    let mut cache = pack_resource_cache().lock().unwrap();
    cache.entries.clear();
    cache.insertion_order.clear();
}

#[cfg(test)]
pub(crate) fn pack_resource_cache_len() -> usize {
    pack_resource_cache().lock().unwrap().entries.len()
}

#[cfg(test)]
pub(crate) fn pack_resource_cache_limit() -> usize {
    MAX_PACK_RESOURCE_CACHE_ENTRIES
}

pub(crate) fn cache_pack_resources(
    repo: &Path,
    task: &str,
    plan: &ctxpack_core::ContextPlan,
    target_agent: &str,
) -> Result<(), RpcError> {
    let mut cache = pack_resource_cache()
        .lock()
        .map_err(|_| RpcError::invalid_params("pack resource cache is unavailable"))?;
    for option in &plan.pack_options {
        let pack = compile_context_pack_from_plan_for_agent(
            repo,
            task,
            plan,
            option.budget.clone(),
            target_agent,
        );
        cache_context_pack(&mut cache, &option.resource_uri, &pack)?;
    }
    Ok(())
}

fn cache_context_pack(
    cache: &mut PackResourceCache,
    uri: &str,
    pack: &ContextPack,
) -> Result<(), RpcError> {
    let markdown = render_pack_markdown(pack);
    insert_cached_pack_resource(
        cache,
        uri.to_string(),
        CachedResource {
            mime_type: "text/markdown",
            text: markdown,
        },
    );
    insert_cached_pack_resource(
        cache,
        format!("{uri}.json"),
        CachedResource {
            mime_type: "application/json",
            text: serde_json::to_string_pretty(pack).map_err(|error| {
                RpcError::invalid_params(format!("failed to serialize cached pack: {error}"))
            })?,
        },
    );
    Ok(())
}

fn insert_cached_pack_resource(
    cache: &mut PackResourceCache,
    uri: String,
    resource: CachedResource,
) {
    if cache.entries.insert(uri.clone(), resource).is_some() {
        cache
            .insertion_order
            .retain(|cached_uri| cached_uri != &uri);
    }
    cache.insertion_order.push_back(uri);
    evict_oldest_pack_resources(cache);
}

fn evict_oldest_pack_resources(cache: &mut PackResourceCache) {
    while cache.entries.len() > MAX_PACK_RESOURCE_CACHE_ENTRIES {
        let Some(uri) = cache.insertion_order.pop_front() else {
            break;
        };
        cache.entries.remove(&uri);
    }
}

fn read_pack_resource(uri: &str) -> Result<ResourceContent, RpcError> {
    let cache = pack_resource_cache()
        .lock()
        .map_err(|_| RpcError::invalid_params("pack resource cache is unavailable"))?;
    let Some(resource) = cache.entries.get(uri) else {
        return Err(RpcError::invalid_params(format!(
            "pack resource is session-scoped and is only available after prepare_task in the same MCP server process; call prepare_task first in this session, or call get_pack again after reconnect/restart: {uri}"
        )));
    };
    Ok(ResourceContent {
        mime_type: resource.mime_type,
        text: resource.text.clone(),
    })
}

fn resource_json(value: &Value) -> ResourceContent {
    ResourceContent {
        mime_type: "application/json",
        text: serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string()),
    }
}

fn repo_summary(repo: &Path) -> Result<Value, RpcError> {
    let report = load_or_refresh_inventory(repo, &InventoryOptions::default())
        .map_err(|error| RpcError::invalid_params(format!("failed to load inventory: {error}")))?;
    let inventory = report.inventory;
    let mut roles = BTreeMap::<String, usize>::new();
    for file in &inventory.files {
        *roles.entry(format!("{:?}", file.role)).or_default() += 1;
    }

    Ok(json!({
        "repoId": inventory.repo_id,
        "fileCount": inventory.files.len(),
        "generatedCount": inventory.generated_count,
        "sensitiveCount": inventory.sensitive_count,
        "roles": roles,
        "diagnostics": report.diagnostics,
        "cacheStatus": report.cache_status,
        "privacyStatus": {
            "localOnly": true,
            "remoteEmbeddingsUsed": false,
            "remoteRerankingUsed": false
        }
    }))
}

fn repo_test_map(repo: &Path) -> Result<Value, RpcError> {
    let tests = test_map(repo)
        .map_err(|error| RpcError::invalid_params(format!("failed to build test map: {error}")))?;

    Ok(json!({ "tests": tests }))
}

fn repo_dependency_graph(repo: &Path) -> Result<Value, RpcError> {
    let edges = dependency_edges(repo, &DependencyOptions { limit: 200 }).map_err(|error| {
        RpcError::invalid_params(format!("failed to build dependency graph: {error}"))
    })?;

    Ok(json!({ "edges": edges }))
}

fn read_file_resource(repo: &Path, uri: &str) -> Result<ResourceContent, RpcError> {
    let (path, lines) = parse_file_uri(uri)?;
    let report = load_or_refresh_inventory(repo, &InventoryOptions::default())
        .map_err(|error| RpcError::invalid_params(format!("failed to load inventory: {error}")))?;
    let source = read_safe_source(repo, &report.inventory, &path, SOURCE_READ_MAX_BYTES).map_err(
        |error| RpcError::invalid_params(format!("failed to read safe source: {error}")),
    )?;
    let SourceReadStatus::Read = source.status else {
        return Err(RpcError::invalid_params(format!(
            "file is not in current safe inventory: {path}"
        )));
    };
    let content = source.text.unwrap_or_default();
    let text = render_line_slice(&content, lines.unwrap_or((1, 120)));
    Ok(ResourceContent {
        mime_type: "text/plain",
        text,
    })
}

fn read_symbol_resource(repo: &Path, uri: &str) -> Result<ResourceContent, RpcError> {
    let symbol = uri.trim_start_matches("ctxpack://symbol/").trim();
    if symbol.is_empty() {
        return Err(RpcError::invalid_params(
            "symbol resource requires a symbol",
        ));
    }
    let results = symbol_search(repo, symbol, &SymbolOptions { limit: 10 })
        .map_err(|error| RpcError::invalid_params(format!("failed to search symbol: {error}")))?;
    Ok(ResourceContent {
        mime_type: "application/json",
        text: serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".to_string()),
    })
}

fn pack_guide_markdown() -> ResourceContent {
    ResourceContent {
        mime_type: "text/markdown",
        text: "Use the `ctxpack.get_pack` MCP tool with `task`, optional `mode`, and `budget` to compile a task-conditioned context pack. Pack resource URIs returned by `prepare_task` are MCP-session scoped: they work only in the same MCP server process that created them. After reconnect or server restart, call `get_pack` for the durable reconnect-safe way to materialize a pack, or call `prepare_task` again to mint fresh session resource URIs. Packs are generated on demand so they reflect the current safe inventory and git history.".to_string(),
    }
}

fn parse_file_uri(uri: &str) -> Result<(String, Option<(usize, usize)>), RpcError> {
    let rest = uri.trim_start_matches("ctxpack://file/");
    let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
    let path = path.trim_start_matches('/').to_string();
    if path.is_empty() {
        return Err(RpcError::invalid_params("file resource requires a path"));
    }
    let lines = query
        .split('&')
        .find_map(|part| part.strip_prefix("lines="))
        .and_then(|range| {
            let (start, end) = range.split_once('-')?;
            Some((start.parse::<usize>().ok()?, end.parse::<usize>().ok()?))
        })
        .map(|(start, end)| (start.max(1), end.max(start).min(start + 500)));
    Ok((path, lines))
}

fn render_line_slice(content: &str, lines: (usize, usize)) -> String {
    let (start, end) = lines;
    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line_no = index + 1;
            (line_no >= start && line_no <= end).then(|| format!("{line_no:>4}: {line}"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn discover_repo(repo: Option<PathBuf>) -> Result<RepoRoot, RpcError> {
    let start = match repo {
        Some(path) => path,
        None => std::env::current_dir()
            .map_err(|error| RpcError::invalid_params(format!("failed to read cwd: {error}")))?,
    };
    RepoRoot::discover_from(&start)
        .map_err(|error| RpcError::invalid_params(format!("failed to discover repo: {error}")))
}
