use crate::protocol::RpcError;
use ctxpack_compiler::{compile_context_pack_from_plan_for_agent, render_pack_markdown};
use ctxpack_core::{
    context_area_for_path, context_area_resource_uri, decode_context_area_resource_uri,
    ContextPack, FileRole, RepoRoot,
};
use ctxpack_index::{
    dependency_edges, list_memory_cards, load_or_refresh_inventory, read_safe_source,
    shared_artifact_manifest_path, symbol_search, test_map, workspace_inventory_status,
    DependencyOptions, InventoryOptions, SourceReadStatus, StoreConfig, SymbolOptions,
    SOURCE_READ_MAX_BYTES,
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
        "ctxpack://repo/memory" => {
            let repo = discover_repo(None)?;
            resource_json(&repo_memory(&repo.path)?)
        }
        "ctxpack://repo/context-areas" => {
            let repo = discover_repo(None)?;
            resource_json(&repo_context_areas(&repo.path)?)
        }
        "ctxpack://workspace/status" => {
            let repo = discover_repo(None)?;
            resource_json(&workspace_status_resource(&repo.path)?)
        }
        "ctxpack://workspace/shared-artifacts" => {
            let repo = discover_repo(None)?;
            resource_json(&shared_artifacts_resource(&repo.path)?)
        }
        "ctxpack://pack/guide" => pack_guide_markdown(),
        uri if uri.starts_with("ctxpack://repo/context-area/") => {
            let repo = discover_repo(None)?;
            resource_json(&repo_context_area(&repo.path, uri)?)
        }
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

fn repo_memory(repo: &Path) -> Result<Value, RpcError> {
    let cards = list_memory_cards(repo, &StoreConfig::default(), false).map_err(|error| {
        RpcError::invalid_params(format!("failed to list memory cards: {error}"))
    })?;

    Ok(json!({
        "cards": cards,
        "privacyStatus": {
            "localOnly": true,
            "remoteEmbeddingsUsed": false,
            "remoteRerankingUsed": false
        }
    }))
}

fn repo_context_areas(repo: &Path) -> Result<Value, RpcError> {
    let report = load_or_refresh_inventory(repo, &InventoryOptions::default())
        .map_err(|error| RpcError::invalid_params(format!("failed to load inventory: {error}")))?;
    let mut areas = BTreeMap::<String, AreaResourceAccumulator>::new();
    for file in &report.inventory.files {
        if !context_area_resource_role(&file.role) {
            continue;
        }
        areas
            .entry(context_area_for_path(&file.path))
            .or_default()
            .record(&file.path, &file.role);
    }

    let areas = areas
        .into_iter()
        .map(|(area, accumulator)| {
            json!({
                "area": area,
                "resourceUri": context_area_resource_uri(&area),
                "pathCount": accumulator.path_count,
                "roleCounts": accumulator.role_counts,
                "pathFamilies": accumulator.path_families_json(8),
                "representativePaths": accumulator.representative_paths,
                "sourceTextLogged": false
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "repoId": report.inventory.repo_id,
        "areaCount": areas.len(),
        "areas": areas,
        "diagnostics": report.diagnostics,
        "cacheStatus": report.cache_status,
        "sourceTextLogged": false,
        "privacyStatus": {
            "localOnly": true,
            "remoteEmbeddingsUsed": false,
            "remoteRerankingUsed": false
        }
    }))
}

fn repo_context_area(repo: &Path, uri: &str) -> Result<Value, RpcError> {
    let Some(area) = decode_context_area_resource_uri(uri) else {
        return Err(RpcError::invalid_params(format!(
            "invalid context area resource URI: {uri}"
        )));
    };
    let report = load_or_refresh_inventory(repo, &InventoryOptions::default())
        .map_err(|error| RpcError::invalid_params(format!("failed to load inventory: {error}")))?;
    let mut accumulator = AreaResourceAccumulator::default();
    for file in &report.inventory.files {
        if !context_area_resource_role(&file.role) {
            continue;
        }
        if context_area_for_path(&file.path) == area {
            accumulator.record(&file.path, &file.role);
        }
    }

    let diagnostics = if accumulator.path_count == 0 {
        json!([{
            "code": "context_area_not_found",
            "severity": "warning",
            "message": "No safe inventory paths are currently grouped under this context area.",
            "paths": [],
            "count": 0
        }])
    } else {
        json!([])
    };

    Ok(json!({
        "repoId": report.inventory.repo_id,
        "area": area,
        "resourceUri": context_area_resource_uri(&area),
        "pathCount": accumulator.path_count,
        "roleCounts": accumulator.role_counts,
        "pathFamilies": accumulator.path_families_json(16),
        "representativePaths": accumulator.representative_paths,
        "roleBuckets": accumulator.role_buckets_json(24),
        "nextReadBatches": accumulator.next_read_batches_json(),
        "sourceTextLogged": false,
        "diagnostics": diagnostics,
        "privacyStatus": {
            "localOnly": true,
            "remoteEmbeddingsUsed": false,
            "remoteRerankingUsed": false
        }
    }))
}

#[derive(Default)]
struct AreaResourceAccumulator {
    path_count: usize,
    role_counts: BTreeMap<String, usize>,
    role_paths: BTreeMap<String, Vec<String>>,
    path_families: BTreeMap<String, usize>,
    representative_paths: Vec<String>,
}

impl AreaResourceAccumulator {
    fn record(&mut self, path: &str, role: &FileRole) {
        let role_key = context_area_role_key(role);
        self.path_count += 1;
        *self.role_counts.entry(role_key.clone()).or_default() += 1;
        *self.path_families.entry(path_family(path)).or_default() += 1;
        self.role_paths
            .entry(role_key)
            .or_default()
            .push(path.to_string());
        if self.representative_paths.len() < 20 {
            self.representative_paths.push(path.to_string());
        }
    }

    fn path_families_json(&self, limit: usize) -> Vec<Value> {
        let mut families = self
            .path_families
            .iter()
            .map(|(family, path_count)| (family.as_str(), *path_count))
            .collect::<Vec<_>>();
        families.sort_by(|(left_family, left_count), (right_family, right_count)| {
            right_count
                .cmp(left_count)
                .then_with(|| left_family.cmp(right_family))
        });
        families
            .into_iter()
            .take(limit)
            .map(|(family, path_count)| {
                json!({
                    "family": family,
                    "pathCount": path_count
                })
            })
            .collect()
    }

    fn role_buckets_json(&self, limit_per_role: usize) -> Value {
        let mut buckets = serde_json::Map::new();
        for role in ["source", "config", "schema", "test", "docs"] {
            let paths = self
                .role_paths
                .get(role)
                .map(|paths| {
                    paths
                        .iter()
                        .take(limit_per_role)
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            buckets.insert(role.to_string(), json!(paths));
        }
        Value::Object(buckets)
    }

    fn next_read_batches_json(&self) -> Vec<Value> {
        let primary = self.paths_for_roles(&["source", "config", "schema"], 12);
        let validation = self.paths_for_roles(&["test"], 8);
        let docs = self.paths_for_roles(&["docs"], 8);
        [
            (
                "primary",
                "Inspect these source/config/schema paths first when this area is relevant.",
                primary,
            ),
            (
                "validation",
                "Use these tests after the primary paths explain the likely change.",
                validation,
            ),
            (
                "docs",
                "Use these docs for architecture or release constraints.",
                docs,
            ),
        ]
        .into_iter()
        .filter(|(_, _, paths)| !paths.is_empty())
        .map(|(kind, reason, paths)| {
            json!({
                "kind": kind,
                "reason": reason,
                "paths": paths,
                "sourceTextLogged": false
            })
        })
        .collect()
    }

    fn paths_for_roles(&self, roles: &[&str], limit: usize) -> Vec<String> {
        let mut paths = Vec::new();
        for role in roles {
            let Some(role_paths) = self.role_paths.get(*role) else {
                continue;
            };
            for path in role_paths {
                if paths.len() >= limit {
                    return paths;
                }
                if !paths.contains(path) {
                    paths.push(path.clone());
                }
            }
        }
        paths
    }
}

fn context_area_resource_role(role: &FileRole) -> bool {
    matches!(
        role,
        FileRole::Source | FileRole::Test | FileRole::Config | FileRole::Schema | FileRole::Docs
    )
}

fn context_area_role_key(role: &FileRole) -> String {
    match role {
        FileRole::Source => "source",
        FileRole::Test => "test",
        FileRole::Config => "config",
        FileRole::Schema => "schema",
        FileRole::Docs => "docs",
        FileRole::Generated => "generated",
        FileRole::Sensitive => "sensitive",
        FileRole::Unknown => "unknown",
    }
    .to_string()
}

fn path_family(path: &str) -> String {
    let (parent, file_name) = path.rsplit_once('/').unwrap_or((".", path));
    let extension = file_name.rsplit_once('.').map(|(_, extension)| extension);
    match extension {
        Some(extension) if !extension.is_empty() => format!("{parent}/*.{extension}"),
        _ => format!("{parent}/*"),
    }
}

fn workspace_status_resource(repo: &Path) -> Result<Value, RpcError> {
    let report = workspace_inventory_status(repo, None).map_err(|error| {
        RpcError::invalid_params(format!("failed to inspect workspace status: {error}"))
    })?;
    serde_json::to_value(report).map_err(|error| {
        RpcError::invalid_params(format!("failed to serialize workspace status: {error}"))
    })
}

fn shared_artifacts_resource(repo: &Path) -> Result<Value, RpcError> {
    let path = shared_artifact_manifest_path(repo);
    if !path.exists() {
        return Err(RpcError::invalid_params(format!(
            "shared artifact manifest does not exist; run `ctxpack workspace artifacts export --repo {}` first",
            repo.display()
        )));
    }
    let report = ctxpack_index::inspect_shared_artifact_manifest(path).map_err(|error| {
        RpcError::invalid_params(format!(
            "failed to inspect shared artifact manifest: {error}"
        ))
    })?;
    serde_json::to_value(report).map_err(|error| {
        RpcError::invalid_params(format!(
            "failed to serialize shared artifact manifest inspection: {error}"
        ))
    })
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
