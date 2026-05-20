use crate::protocol::RpcError;
use crate::resources::{cache_pack_resources, discover_repo};
use ctxpack_compiler::{
    compile_context_pack_with_plan_and_paths_for_agent_and_semantic_provider, eval_trace_for_pack,
    eval_trace_for_plan, prepare_context_plan_with_paths_and_semantic_provider,
    render_pack_markdown,
};
use ctxpack_core::{CacheStatus, Diagnostic, PackBudget, TaskType};
use ctxpack_index::{
    co_change_hints_report, current_diff_summary, current_diff_summary_report,
    lexical_search_report, related_dependency_edges_report, related_tests_report,
    symbol_search_report, try_append_eval_trace, CoChangeOptions, CurrentDiffOptions,
    DependencyOptions, SearchOptions, SemanticProviderConfig, SymbolOptions,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CallToolParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]

struct PrepareTaskArgs {
    task: String,
    #[serde(default)]
    repo: Option<PathBuf>,
    #[serde(default)]
    mode: Option<TaskType>,
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    include_current_diff: bool,
    #[serde(default)]
    target_agent: Option<String>,
    #[serde(default)]
    semantic: bool,
    #[serde(default)]
    semantic_provider: Option<String>,
    #[serde(default)]
    semantic_model: Option<String>,
    #[serde(default)]
    semantic_dimensions: Option<usize>,
    #[serde(default = "default_record_trace")]
    record_trace: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetPackArgs {
    task: String,
    #[serde(default)]
    repo: Option<PathBuf>,
    #[serde(default)]
    mode: Option<TaskType>,
    #[serde(default)]
    budget: Option<PackBudget>,
    #[serde(default)]
    format: Option<PackFormat>,
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    include_current_diff: bool,
    #[serde(default)]
    target_agent: Option<String>,
    #[serde(default)]
    semantic: bool,
    #[serde(default)]
    semantic_provider: Option<String>,
    #[serde(default)]
    semantic_model: Option<String>,
    #[serde(default)]
    semantic_dimensions: Option<usize>,
    #[serde(default = "default_record_trace")]
    record_trace: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchArgs {
    query: String,
    #[serde(default)]
    repo: Option<PathBuf>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    semantic: bool,
    #[serde(default)]
    semantic_provider: Option<String>,
    #[serde(default)]
    semantic_model: Option<String>,
    #[serde(default)]
    semantic_dimensions: Option<usize>,
    #[serde(default)]
    kinds: Vec<SearchKind>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum SearchKind {
    File,
    Symbol,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelatedArgs {
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    repo: Option<PathBuf>,
    #[serde(default)]
    include_current_diff: bool,
    #[serde(default)]
    include: Vec<RelatedInclude>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum RelatedInclude {
    Tests,
    Commits,
    Dependencies,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelatedTestsArgs {
    paths: Vec<String>,
    #[serde(default)]
    repo: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrentDiffArgs {
    #[serde(default)]
    repo: Option<PathBuf>,
    #[serde(default)]
    include_untracked: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum PackFormat {
    Markdown,
    Json,
}

pub(crate) fn call_tool(params: Value) -> Result<Value, RpcError> {
    let params: CallToolParams = serde_json::from_value(params)
        .map_err(|error| RpcError::invalid_params(format!("invalid tools/call params: {error}")))?;

    match params.name.as_str() {
        "prepare_task" => call_prepare_task(params.arguments),
        "search" => call_search(params.arguments),
        "related" => call_related(params.arguments),
        "get_pack" => call_get_pack(params.arguments),
        "related_tests" => call_related_tests(params.arguments),
        "current_diff" => call_current_diff(params.arguments),
        name => Err(RpcError::invalid_params(format!(
            "tool is not implemented: {name}"
        ))),
    }
}

fn call_prepare_task(arguments: Value) -> Result<Value, RpcError> {
    let args: PrepareTaskArgs = serde_json::from_value(arguments).map_err(|error| {
        RpcError::invalid_params(format!("invalid prepare_task arguments: {error}"))
    })?;

    if args.task.trim().is_empty() {
        return Err(RpcError::invalid_params("task must not be empty"));
    }

    let repo = discover_repo(args.repo)?;
    let paths = context_anchor_paths(&repo.path, args.paths, args.include_current_diff)?;
    let semantic_provider = semantic_provider_config(
        args.semantic_provider.as_deref(),
        args.semantic_model.as_deref(),
        args.semantic_dimensions,
    );
    let mut plan = prepare_context_plan_with_paths_and_semantic_provider(
        &repo.path,
        &args.task,
        args.mode.unwrap_or(TaskType::Explain),
        &paths,
        args.semantic,
        semantic_provider,
    )
    .map_err(|error| RpcError::invalid_params(format!("failed to prepare task: {error}")))?;
    if args.record_trace {
        let trace = eval_trace_for_plan(
            &repo.path,
            &args.task,
            args.target_agent.as_deref().unwrap_or("generic"),
            &plan,
        );
        plan.diagnostics
            .extend(try_append_eval_trace(&repo.path, &trace).diagnostics);
    }
    cache_pack_resources(
        &repo.path,
        &args.task,
        &plan,
        args.target_agent.as_deref().unwrap_or("generic"),
    )?;
    let structured = serde_json::to_value(&plan)
        .map_err(|error| RpcError::invalid_params(format!("failed to serialize plan: {error}")))?;
    let text = serde_json::to_string_pretty(&plan)
        .map_err(|error| RpcError::invalid_params(format!("failed to serialize plan: {error}")))?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }],
        "structuredContent": structured,
        "isError": false
    }))
}

fn call_search(arguments: Value) -> Result<Value, RpcError> {
    let args: SearchArgs = serde_json::from_value(arguments)
        .map_err(|error| RpcError::invalid_params(format!("invalid search arguments: {error}")))?;

    if args.query.trim().is_empty() {
        return Err(RpcError::invalid_params("query must not be empty"));
    }

    let repo = discover_repo(args.repo)?;
    let limit = bounded_limit(args.limit, 10);
    let include_files = args.kinds.is_empty() || args.kinds.contains(&SearchKind::File);
    let include_symbols = args.kinds.is_empty() || args.kinds.contains(&SearchKind::Symbol);

    let (files, semantic_files, mut diagnostics, cache_status, semantic_status) = if include_files {
        if args.semantic {
            let semantic_provider = semantic_provider_config(
                args.semantic_provider.as_deref(),
                args.semantic_model.as_deref(),
                args.semantic_dimensions,
            );
            let report = ctxpack_index::semantic_search_report(
                &repo.path,
                &args.query,
                &ctxpack_index::SemanticOptions {
                    enabled: true,
                    limit,
                    provider: semantic_provider,
                    ..ctxpack_index::SemanticOptions::default()
                },
            )
            .map_err(|error| {
                RpcError::invalid_params(format!("failed to search repo semantically: {error}"))
            })?;
            (
                Vec::new(),
                report.results,
                report.diagnostics,
                Some(report.cache_status),
                Some(report.provider),
            )
        } else {
            let report = lexical_search_report(&repo.path, &args.query, &SearchOptions { limit })
                .map_err(|error| {
                RpcError::invalid_params(format!("failed to search repo: {error}"))
            })?;
            (
                report.results,
                Vec::new(),
                report.diagnostics,
                Some(report.cache_status),
                None,
            )
        }
    } else {
        (Vec::new(), Vec::new(), Vec::new(), None, None)
    };
    let (symbols, symbol_cache_status) = if include_symbols {
        let report = symbol_search_report(&repo.path, &args.query, &SymbolOptions { limit })
            .map_err(|error| {
                RpcError::invalid_params(format!("failed to search repo symbols: {error}"))
            })?;
        diagnostics.extend(report.diagnostics);
        (report.results, Some(report.cache_status))
    } else {
        (Vec::new(), None)
    };
    let cache_status = cache_status.or(symbol_cache_status);

    tool_json_result(json!({
        "query": args.query.trim(),
        "files": files,
        "semanticFiles": semantic_files,
        "symbols": symbols,
        "diagnostics": diagnostics,
        "cacheStatus": cache_status,
        "semanticProvider": semantic_status,
        "privacyStatus": {
            "localOnly": true,
            "sourceTextReturned": false
        }
    }))
}

fn call_related(arguments: Value) -> Result<Value, RpcError> {
    let args: RelatedArgs = serde_json::from_value(arguments)
        .map_err(|error| RpcError::invalid_params(format!("invalid related arguments: {error}")))?;

    let repo = discover_repo(args.repo)?;
    let limit = bounded_limit(args.limit, 10);
    let (paths, symbol_matches, mut diagnostics) = related_anchor_paths(
        &repo.path,
        args.path,
        args.symbol,
        args.include_current_diff,
        limit,
    )?;
    let include_tests = args.include.is_empty() || args.include.contains(&RelatedInclude::Tests);
    let include_commits =
        args.include.is_empty() || args.include.contains(&RelatedInclude::Commits);
    let include_dependencies =
        args.include.is_empty() || args.include.contains(&RelatedInclude::Dependencies);

    let tests = if include_tests {
        let report = related_tests_report(&repo.path, &paths)
            .map_err(|error| RpcError::invalid_params(format!("failed to find tests: {error}")))?;
        diagnostics.extend(report.diagnostics);
        report.results.into_iter().take(limit).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let co_changes = if include_commits {
        let report = co_change_hints_report(&repo.path, &paths, &CoChangeOptions { limit })
            .map_err(|error| {
                RpcError::invalid_params(format!("failed to find co-change hints: {error}"))
            })?;
        diagnostics.extend(report.diagnostics);
        report.hints
    } else {
        Vec::new()
    };
    let dependency_edges = if include_dependencies {
        let report =
            related_dependency_edges_report(&repo.path, &paths, &DependencyOptions { limit })
                .map_err(|error| {
                    RpcError::invalid_params(format!("failed to find dependency edges: {error}"))
                })?;
        diagnostics.extend(report.diagnostics);
        report.edges
    } else {
        Vec::new()
    };
    let warnings = diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic.code.as_str(),
                "history_partial" | "git_missing" | "git_timeout" | "git_unavailable"
            )
        })
        .map(|diagnostic| {
            format!(
                "Local git co-change hints were unavailable; continuing without history signal: {}",
                diagnostic.message
            )
        })
        .collect::<Vec<_>>();

    tool_json_result(json!({
        "resolvedPaths": paths,
        "symbolMatches": symbol_matches,
        "relatedTests": tests,
        "coChangeHints": co_changes,
        "dependencyEdges": dependency_edges,
        "warnings": warnings,
        "diagnostics": diagnostics
    }))
}

fn related_anchor_paths(
    repo: &Path,
    path: Option<String>,
    symbol: Option<String>,
    include_current_diff: bool,
    limit: usize,
) -> Result<(Vec<String>, Value, Vec<Diagnostic>), RpcError> {
    let mut paths = Vec::new();
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();

    if let Some(path) = path.map(|value| value.trim().to_string()) {
        if !path.is_empty() && seen.insert(path.clone()) {
            paths.push(path);
        }
    }

    let symbol_matches = if let Some(symbol) = symbol.map(|value| value.trim().to_string()) {
        if symbol.is_empty() {
            Value::Array(Vec::new())
        } else {
            let report =
                symbol_search_report(repo, &symbol, &SymbolOptions { limit }).map_err(|error| {
                    RpcError::invalid_params(format!("failed to resolve related symbol: {error}"))
                })?;
            diagnostics.extend(report.diagnostics);
            for result in &report.results {
                let path = result.symbol.path.clone();
                if seen.insert(path.clone()) {
                    paths.push(path);
                }
            }
            serde_json::to_value(report.results).map_err(|error| {
                RpcError::invalid_params(format!("failed to serialize symbol matches: {error}"))
            })?
        }
    } else {
        Value::Array(Vec::new())
    };

    if include_current_diff {
        let diff = current_diff_summary_report(
            repo,
            &CurrentDiffOptions {
                include_untracked: true,
            },
        )
        .map_err(|error| {
            RpcError::invalid_params(format!("failed to collect current diff anchors: {error}"))
        })?;
        diagnostics.extend(diff.diagnostics);
        for path in diff
            .summary
            .staged
            .into_iter()
            .chain(diff.summary.unstaged.into_iter())
            .chain(diff.summary.untracked.into_iter())
        {
            if seen.insert(path.clone()) {
                paths.push(path);
            }
        }
    }

    if paths.is_empty() {
        return Err(RpcError::invalid_params(
            "related requires a non-empty path, symbol, or current diff anchor",
        ));
    }

    Ok((paths, symbol_matches, diagnostics))
}

fn context_anchor_paths(
    repo: &Path,
    explicit_paths: Vec<String>,
    include_current_diff: bool,
) -> Result<Vec<String>, RpcError> {
    let mut paths = Vec::new();
    let mut seen = BTreeSet::new();
    for path in explicit_paths {
        let path = path.trim();
        if !path.is_empty() && seen.insert(path.to_string()) {
            paths.push(path.to_string());
        }
    }

    if include_current_diff {
        let diff = current_diff_summary(
            repo,
            &CurrentDiffOptions {
                include_untracked: true,
            },
        )
        .map_err(|error| {
            RpcError::invalid_params(format!("failed to collect current diff anchors: {error}"))
        })?;
        for path in diff
            .staged
            .into_iter()
            .chain(diff.unstaged.into_iter())
            .chain(diff.untracked.into_iter())
        {
            if seen.insert(path.clone()) {
                paths.push(path);
            }
        }
    }

    Ok(paths)
}

fn call_get_pack(arguments: Value) -> Result<Value, RpcError> {
    let args: GetPackArgs = serde_json::from_value(arguments).map_err(|error| {
        RpcError::invalid_params(format!("invalid get_pack arguments: {error}"))
    })?;

    if args.task.trim().is_empty() {
        return Err(RpcError::invalid_params("task must not be empty"));
    }

    let repo = discover_repo(args.repo)?;
    let paths = context_anchor_paths(&repo.path, args.paths, args.include_current_diff)?;
    let budget = args.budget.unwrap_or(PackBudget::Brief);
    let target_agent = args.target_agent.as_deref().unwrap_or("generic");
    let semantic_provider = semantic_provider_config(
        args.semantic_provider.as_deref(),
        args.semantic_model.as_deref(),
        args.semantic_dimensions,
    );
    let (plan, mut pack) =
        compile_context_pack_with_plan_and_paths_for_agent_and_semantic_provider(
            &repo.path,
            &args.task,
            args.mode.unwrap_or(TaskType::Explain),
            budget,
            &paths,
            target_agent,
            args.semantic,
            semantic_provider,
        )
        .map_err(|error| RpcError::invalid_params(format!("failed to compile pack: {error}")))?;
    if args.record_trace {
        let trace = eval_trace_for_pack(&repo.path, &args.task, target_agent, &plan, &pack);
        pack.diagnostics
            .extend(try_append_eval_trace(&repo.path, &trace).diagnostics);
    }

    let structured = serde_json::to_value(&pack)
        .map_err(|error| RpcError::invalid_params(format!("failed to serialize pack: {error}")))?;
    let text = match args.format.unwrap_or(PackFormat::Markdown) {
        PackFormat::Markdown => render_pack_markdown(&pack),
        PackFormat::Json => serde_json::to_string_pretty(&pack).map_err(|error| {
            RpcError::invalid_params(format!("failed to serialize pack: {error}"))
        })?,
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }],
        "structuredContent": structured,
        "isError": false
    }))
}

fn call_related_tests(arguments: Value) -> Result<Value, RpcError> {
    let args: RelatedTestsArgs = serde_json::from_value(arguments).map_err(|error| {
        RpcError::invalid_params(format!("invalid related_tests arguments: {error}"))
    })?;

    if args.paths.is_empty() {
        return Err(RpcError::invalid_params("paths must not be empty"));
    }

    let repo = discover_repo(args.repo)?;
    let report = related_tests_report(&repo.path, &args.paths)
        .map_err(|error| RpcError::invalid_params(format!("failed to find tests: {error}")))?;

    tool_json_result_with_diagnostics(report.results, report.diagnostics, report.cache_status)
}

fn call_current_diff(arguments: Value) -> Result<Value, RpcError> {
    let args: CurrentDiffArgs = serde_json::from_value(arguments).map_err(|error| {
        RpcError::invalid_params(format!("invalid current_diff arguments: {error}"))
    })?;

    let repo = discover_repo(args.repo)?;
    let report = current_diff_summary_report(
        &repo.path,
        &CurrentDiffOptions {
            include_untracked: args.include_untracked,
        },
    )
    .map_err(|error| RpcError::invalid_params(format!("failed to read current diff: {error}")))?;

    let mut structured = serde_json::to_value(report.summary).map_err(|error| {
        RpcError::invalid_params(format!("failed to serialize current diff: {error}"))
    })?;
    insert_diagnostics(
        &mut structured,
        report.diagnostics,
        Some(report.cache_status),
    );
    tool_json_result(structured)
}

fn bounded_limit(limit: Option<usize>, default: usize) -> usize {
    limit.unwrap_or(default).clamp(1, 50)
}

fn default_record_trace() -> bool {
    true
}

fn semantic_provider_config(
    provider: Option<&str>,
    model: Option<&str>,
    dimensions: Option<usize>,
) -> SemanticProviderConfig {
    let default = SemanticProviderConfig::default();
    SemanticProviderConfig {
        provider: provider
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(&default.provider)
            .to_string(),
        model: model
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(&default.model)
            .to_string(),
        dimensions: dimensions.unwrap_or(default.dimensions),
        distance_metric: default.distance_metric,
        provider_role: default.provider_role,
        quality_backend: default.quality_backend,
        local_only: true,
        available: true,
    }
}

fn tool_json_result(value: impl serde::Serialize) -> Result<Value, RpcError> {
    let structured = serde_json::to_value(value).map_err(|error| {
        RpcError::invalid_params(format!("failed to serialize tool result: {error}"))
    })?;
    let text = serde_json::to_string_pretty(&structured).map_err(|error| {
        RpcError::invalid_params(format!("failed to serialize tool result: {error}"))
    })?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }],
        "structuredContent": structured,
        "isError": false
    }))
}

fn tool_json_result_with_diagnostics(
    value: impl serde::Serialize,
    diagnostics: Vec<Diagnostic>,
    cache_status: CacheStatus,
) -> Result<Value, RpcError> {
    let structured = serde_json::to_value(value).map_err(|error| {
        RpcError::invalid_params(format!("failed to serialize tool result: {error}"))
    })?;
    let text = serde_json::to_string_pretty(&structured).map_err(|error| {
        RpcError::invalid_params(format!("failed to serialize tool result: {error}"))
    })?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }],
        "structuredContent": structured,
        "diagnostics": diagnostics,
        "cacheStatus": cache_status,
        "isError": false
    }))
}

fn insert_diagnostics(
    value: &mut Value,
    diagnostics: Vec<Diagnostic>,
    cache_status: Option<CacheStatus>,
) {
    if let Some(object) = value.as_object_mut() {
        object.insert("diagnostics".to_string(), json!(diagnostics));
        object.insert("cacheStatus".to_string(), json!(cache_status));
    }
}
