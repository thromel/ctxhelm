use ctxpack_compiler::{
    compile_context_pack_from_plan, compile_context_pack_with_plan_and_paths, eval_trace_for_pack,
    eval_trace_for_plan, prepare_context_plan_with_paths, render_pack_markdown,
};
use ctxpack_core::{ContextPack, FileRole, PackBudget, RepoRoot, TaskType};
use ctxpack_index::{
    append_eval_trace, build_inventory, co_change_hints, dependency_edges, lexical_search,
    load_or_build_inventory, related_dependency_edges, related_tests, symbol_search, test_map,
    CoChangeOptions, DependencyOptions, InventoryOptions, SearchOptions, SymbolOptions,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

pub const PLANNED_MCP_TOOL_NAMES: &[&str] = &[
    "prepare_task",
    "search",
    "related",
    "get_pack",
    "related_tests",
    "current_diff",
];

pub const IMPLEMENTED_MCP_TOOL_NAMES: &[&str] = &[
    "prepare_task",
    "search",
    "related",
    "get_pack",
    "related_tests",
    "current_diff",
];

const JSONRPC_VERSION: &str = "2.0";
const MCP_PROTOCOL_VERSION: &str = "2025-11-25";

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CallToolParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadResourceParams {
    uri: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetPromptParams {
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
    target_agent: Option<String>,
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
    target_agent: Option<String>,
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

#[derive(Debug)]
struct RpcError {
    code: i32,
    message: String,
}

impl RpcError {
    fn parse_error(message: impl Into<String>) -> Self {
        Self {
            code: -32700,
            message: message.into(),
        }
    }

    fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: message.into(),
        }
    }

    fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("method not found: {method}"),
        }
    }
}

pub fn run_stdio_server() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_server(stdin.lock(), stdout.lock())
}

pub fn run_server<R, W>(reader: R, mut writer: W) -> io::Result<()>
where
    R: BufRead,
    W: Write,
{
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        if let Some(response) = handle_line(&line) {
            serde_json::to_writer(&mut writer, &response)?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
    }

    Ok(())
}

fn handle_line(line: &str) -> Option<Value> {
    let parsed = match serde_json::from_str::<JsonRpcRequest>(line) {
        Ok(request) => request,
        Err(error) => {
            return Some(error_response(
                Value::Null,
                RpcError::parse_error(format!("invalid JSON-RPC request: {error}")),
            ));
        }
    };

    let id = parsed.id.clone()?;

    match handle_request(&parsed) {
        Ok(result) => Some(success_response(id, result)),
        Err(error) => Some(error_response(id, error)),
    }
}

fn handle_request(request: &JsonRpcRequest) -> Result<Value, RpcError> {
    match request.method.as_str() {
        "initialize" => Ok(initialize_result()),
        "tools/list" => Ok(tools_list_result()),
        "tools/call" => call_tool(request.params.clone()),
        "resources/list" => Ok(resources_list_result()),
        "resources/read" => read_resource(request.params.clone()),
        "prompts/list" => Ok(prompts_list_result()),
        "prompts/get" => get_prompt(request.params.clone()),
        method => Err(RpcError::method_not_found(method)),
    }
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "capabilities": {
            "tools": {
                "listChanged": false
            },
            "resources": {
                "listChanged": false
            },
            "prompts": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": "ctxpack",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

fn resources_list_result() -> Value {
    json!({
        "resources": [
            {
                "uri": "ctxpack://repo/summary",
                "name": "Repository Summary",
                "description": "Safe inventory summary for the current repository.",
                "mimeType": "application/json"
            },
            {
                "uri": "ctxpack://repo/test-map",
                "name": "Repository Test Map",
                "description": "Test files and inferred targeted commands.",
                "mimeType": "application/json"
            },
            {
                "uri": "ctxpack://repo/dependency-graph",
                "name": "Repository Dependency Graph",
                "description": "Safe local import edges inferred from source and test files.",
                "mimeType": "application/json"
            },
            {
                "uri": "ctxpack://pack/guide",
                "name": "Context Pack Guide",
                "description": "How to request task-conditioned context packs with ctxpack.get_pack.",
                "mimeType": "text/markdown"
            }
        ]
    })
}

fn prompts_list_result() -> Value {
    json!({
        "prompts": [
            prompt_descriptor("bugfix", "Prepare and solve a bug fix with targeted repo context."),
            prompt_descriptor("feature", "Prepare and implement a feature using analogous repo context."),
            prompt_descriptor("refactor", "Plan a refactor using callers, tests, and constraints."),
            prompt_descriptor("review_diff", "Review the current diff with repo-aware context."),
            prompt_descriptor("write_tests", "Find source context and write focused tests."),
            prompt_descriptor("explain_area", "Explain an area of the codebase with grounded files.")
        ]
    })
}

fn prompt_descriptor(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "description": description,
        "arguments": [{
            "name": "task",
            "description": "The developer task or area to work on.",
            "required": name != "review_diff"
        }]
    })
}

fn tools_list_result() -> Value {
    json!({
        "tools": [
            {
                "name": "prepare_task",
                "title": "Prepare Task Context",
                "description": "Return a compact, local-only ContextPlan for a coding task.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task": {
                            "type": "string",
                            "description": "The developer task to prepare context for."
                        },
                        "repo": {
                            "type": "string",
                            "description": "Optional repository path. Pass the active workspace path when known; defaults to the MCP server working directory."
                        },
                        "mode": {
                            "type": "string",
                            "description": "Optional task type override.",
                            "enum": ["bug_fix", "feature", "refactor", "review", "test", "explain"]
                        },
                        "paths": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Optional active/open repo-relative or absolute paths to pin as context anchors."
                        },
                        "targetAgent": {
                            "type": "string",
                            "description": "Optional host agent label for local eval traces."
                        }
                    },
                    "required": ["task"],
                    "additionalProperties": false
                }
            },
            {
                "name": "search",
                "title": "Search Repository Context",
                "description": "Run compact local search over safe inventoried repository files and symbols.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Identifier, path fragment, error text, or concept to search for."
                        },
                        "repo": {
                            "type": "string",
                            "description": "Optional repository path. Pass the active workspace path when known; defaults to the MCP server working directory."
                        },
                        "limit": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 50,
                            "description": "Maximum result count."
                        },
                        "kinds": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["file", "symbol"]
                            },
                            "description": "Optional result kinds. Defaults to file and symbol matches."
                        }
                    },
                    "required": ["query"],
                    "additionalProperties": false
                }
            },
            {
                "name": "related",
                "title": "Related Repository Context",
                "description": "Expand around a path or symbol with related tests, dependency edges, and local git co-change hints.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Repository-relative or absolute file path to expand from."
                        },
                        "symbol": {
                            "type": "string",
                            "description": "Symbol name or query to resolve first, then expand from matching symbol paths."
                        },
                        "repo": {
                            "type": "string",
                            "description": "Optional repository path. Pass the active workspace path when known; defaults to the MCP server working directory."
                        },
                        "include": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["tests", "commits", "dependencies"]
                            },
                            "description": "Optional expansion categories. Defaults to tests, commits, and dependencies."
                        },
                        "limit": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 50,
                            "description": "Maximum count for each expansion category."
                        }
                    },
                    "additionalProperties": false
                }
            },
            {
                "name": "get_pack",
                "title": "Get Context Pack",
                "description": "Return a budgeted, local-only ContextPack for a coding task.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task": {
                            "type": "string",
                            "description": "The developer task to compile context for."
                        },
                        "repo": {
                            "type": "string",
                            "description": "Optional repository path. Pass the active workspace path when known; defaults to the MCP server working directory."
                        },
                        "mode": {
                            "type": "string",
                            "description": "Optional task type override.",
                            "enum": ["bug_fix", "feature", "refactor", "review", "test", "explain"]
                        },
                        "budget": {
                            "type": "string",
                            "description": "Context budget.",
                            "enum": ["brief", "standard", "deep"]
                        },
                        "format": {
                            "type": "string",
                            "description": "Text response format.",
                            "enum": ["markdown", "json"]
                        },
                        "paths": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Optional active/open repo-relative or absolute paths to pin as context anchors."
                        },
                        "targetAgent": {
                            "type": "string",
                            "description": "Optional host agent label for local eval traces."
                        }
                    },
                    "required": ["task"],
                    "additionalProperties": false
                }
            },
            {
                "name": "related_tests",
                "title": "Find Related Tests",
                "description": "Find likely test files and targeted commands for source paths.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "paths": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Repository-relative or absolute source paths."
                        },
                        "repo": {
                            "type": "string",
                            "description": "Optional repository path. Pass the active workspace path when known; defaults to the MCP server working directory."
                        }
                    },
                    "required": ["paths"],
                    "additionalProperties": false
                }
            },
            {
                "name": "current_diff",
                "title": "Current Diff Summary",
                "description": "Return safe changed path lists from the local git working tree without returning source content.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "repo": {
                            "type": "string",
                            "description": "Optional repository path. Pass the active workspace path when known; defaults to the MCP server working directory."
                        },
                        "includeUntracked": {
                            "type": "boolean",
                            "description": "Include untracked non-ignored paths."
                        }
                    },
                    "additionalProperties": false
                }
            }
        ]
    })
}

fn read_resource(params: Value) -> Result<Value, RpcError> {
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

fn get_prompt(params: Value) -> Result<Value, RpcError> {
    let params: GetPromptParams = serde_json::from_value(params).map_err(|error| {
        RpcError::invalid_params(format!("invalid prompts/get params: {error}"))
    })?;
    let task = params
        .arguments
        .get("task")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();

    let text = match params.name.as_str() {
        "bugfix" => workflow_prompt(
            "bug_fix",
            task,
            "Call ctxpack.prepare_task first, read the returned target files, request ctxpack.get_pack only if needed, make the smallest source change, then run the related test command.",
        ),
        "feature" => workflow_prompt(
            "feature",
            task,
            "Call ctxpack.prepare_task, inspect analogous target files and tests, request a standard pack when examples are needed, then implement within existing repo patterns.",
        ),
        "refactor" => workflow_prompt(
            "refactor",
            task,
            "Call ctxpack.prepare_task, expand with ctxpack.related around the affected files, preserve behavior, and validate with related tests.",
        ),
        "review_diff" => workflow_prompt(
            "review",
            task,
            "Call ctxpack.current_diff, inspect changed paths, use ctxpack.related for risky files, then report findings ordered by severity.",
        ),
        "write_tests" => workflow_prompt(
            "test",
            task,
            "Call ctxpack.prepare_task and ctxpack.related_tests, inspect the source under test and existing test style, then add focused tests.",
        ),
        "explain_area" => workflow_prompt(
            "explain",
            task,
            "Call ctxpack.prepare_task and use ctxpack.search for named concepts, then explain only from files actually read or returned by ctxpack.",
        ),
        name => {
            return Err(RpcError::invalid_params(format!(
                "prompt is not implemented: {name}"
            )))
        }
    };

    Ok(json!({
        "description": format!("ctxpack {} workflow", params.name),
        "messages": [{
            "role": "user",
            "content": {
                "type": "text",
                "text": text
            }
        }]
    }))
}

fn call_tool(params: Value) -> Result<Value, RpcError> {
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
    let plan = prepare_context_plan_with_paths(
        &repo.path,
        &args.task,
        args.mode.unwrap_or(TaskType::Explain),
        &args.paths,
    )
    .map_err(|error| RpcError::invalid_params(format!("failed to prepare task: {error}")))?;
    let trace = eval_trace_for_plan(
        &repo.path,
        &args.task,
        args.target_agent.as_deref().unwrap_or("generic"),
        &plan,
    );
    append_eval_trace(&repo.path, &trace).map_err(|error| {
        RpcError::invalid_params(format!("failed to record eval trace: {error}"))
    })?;
    cache_pack_resources(&repo.path, &args.task, &plan)?;
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

struct ResourceContent {
    mime_type: &'static str,
    text: String,
}

#[derive(Debug, Clone)]
struct CachedResource {
    mime_type: &'static str,
    text: String,
}

fn pack_resource_cache() -> &'static Mutex<BTreeMap<String, CachedResource>> {
    static CACHE: OnceLock<Mutex<BTreeMap<String, CachedResource>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn cache_pack_resources(
    repo: &Path,
    task: &str,
    plan: &ctxpack_core::ContextPlan,
) -> Result<(), RpcError> {
    let mut cache = pack_resource_cache()
        .lock()
        .map_err(|_| RpcError::invalid_params("pack resource cache is unavailable"))?;
    for option in &plan.pack_options {
        let pack = compile_context_pack_from_plan(repo, task, plan, option.budget.clone());
        cache_context_pack(&mut cache, &option.resource_uri, &pack)?;
    }
    Ok(())
}

fn cache_context_pack(
    cache: &mut BTreeMap<String, CachedResource>,
    uri: &str,
    pack: &ContextPack,
) -> Result<(), RpcError> {
    let markdown = render_pack_markdown(pack);
    cache.insert(
        uri.to_string(),
        CachedResource {
            mime_type: "text/markdown",
            text: markdown,
        },
    );
    cache.insert(
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

fn read_pack_resource(uri: &str) -> Result<ResourceContent, RpcError> {
    let cache = pack_resource_cache()
        .lock()
        .map_err(|_| RpcError::invalid_params("pack resource cache is unavailable"))?;
    let Some(resource) = cache.get(uri) else {
        return Err(RpcError::invalid_params(format!(
            "pack resource is not available in this MCP session; call prepare_task first: {uri}"
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
    let inventory = load_or_build_inventory(repo, &InventoryOptions::default())
        .map_err(|error| RpcError::invalid_params(format!("failed to load inventory: {error}")))?;
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
    let inventory = load_or_build_inventory(repo, &InventoryOptions::default())
        .map_err(|error| RpcError::invalid_params(format!("failed to load inventory: {error}")))?;
    let Some(file) = inventory.files.into_iter().find(|file| file.path == path) else {
        return Err(RpcError::invalid_params(format!(
            "file is not in safe inventory: {path}"
        )));
    };
    if file.generated || file.ignored || file.role == FileRole::Sensitive {
        return Err(RpcError::invalid_params(format!(
            "file is excluded by ctxpack policy: {path}"
        )));
    }

    let content = fs::read_to_string(repo.join(&file.path))
        .map_err(|error| RpcError::invalid_params(format!("failed to read file: {error}")))?;
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
        text: "Use the `ctxpack.get_pack` MCP tool with `task`, optional `mode`, and `budget` to compile a task-conditioned context pack. Packs are generated on demand so they reflect the current safe inventory and git history.".to_string(),
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

fn workflow_prompt(mode: &str, task: &str, instruction: &str) -> String {
    let task_line = if task.is_empty() {
        "Task: use the user's current request.".to_string()
    } else {
        format!("Task: {task}")
    };
    format!("{task_line}\nMode: {mode}\n\n{instruction}\n\nWhen the active workspace path is known, pass it as the ctxpack `repo` argument so the MCP server does not infer the wrong working directory.\n\nKeep ctxpack read-only: use it for context and use the host agent's native tools for file reads, edits, and validation commands.")
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

    let files = if include_files {
        lexical_search(&repo.path, &args.query, &SearchOptions { limit })
            .map_err(|error| RpcError::invalid_params(format!("failed to search repo: {error}")))?
    } else {
        Vec::new()
    };
    let symbols = if include_symbols {
        symbol_search(&repo.path, &args.query, &SymbolOptions { limit }).map_err(|error| {
            RpcError::invalid_params(format!("failed to search repo symbols: {error}"))
        })?
    } else {
        Vec::new()
    };

    tool_json_result(json!({
        "query": args.query.trim(),
        "files": files,
        "symbols": symbols,
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
    let (paths, symbol_matches) = related_anchor_paths(&repo.path, args.path, args.symbol, limit)?;
    let include_tests = args.include.is_empty() || args.include.contains(&RelatedInclude::Tests);
    let include_commits =
        args.include.is_empty() || args.include.contains(&RelatedInclude::Commits);
    let include_dependencies =
        args.include.is_empty() || args.include.contains(&RelatedInclude::Dependencies);
    let mut warnings = Vec::new();

    let tests = if include_tests {
        related_tests(&repo.path, &paths)
            .map_err(|error| RpcError::invalid_params(format!("failed to find tests: {error}")))?
            .into_iter()
            .take(limit)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let co_changes = if include_commits {
        match co_change_hints(&repo.path, &paths, &CoChangeOptions { limit }) {
            Ok(hints) => hints,
            Err(error) => {
                warnings.push(format!(
                    "Local git co-change hints were unavailable; continuing without history signal: {error}"
                ));
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };
    let dependency_edges = if include_dependencies {
        related_dependency_edges(&repo.path, &paths, &DependencyOptions { limit }).map_err(
            |error| RpcError::invalid_params(format!("failed to find dependency edges: {error}")),
        )?
    } else {
        Vec::new()
    };

    tool_json_result(json!({
        "resolvedPaths": paths,
        "symbolMatches": symbol_matches,
        "relatedTests": tests,
        "coChangeHints": co_changes,
        "dependencyEdges": dependency_edges,
        "warnings": warnings
    }))
}

fn related_anchor_paths(
    repo: &Path,
    path: Option<String>,
    symbol: Option<String>,
    limit: usize,
) -> Result<(Vec<String>, Value), RpcError> {
    let mut paths = Vec::new();
    let mut seen = BTreeSet::new();

    if let Some(path) = path.map(|value| value.trim().to_string()) {
        if !path.is_empty() && seen.insert(path.clone()) {
            paths.push(path);
        }
    }

    let symbol_matches = if let Some(symbol) = symbol.map(|value| value.trim().to_string()) {
        if symbol.is_empty() {
            Value::Array(Vec::new())
        } else {
            let results =
                symbol_search(repo, &symbol, &SymbolOptions { limit }).map_err(|error| {
                    RpcError::invalid_params(format!("failed to resolve related symbol: {error}"))
                })?;
            for result in &results {
                let path = result.symbol.path.clone();
                if seen.insert(path.clone()) {
                    paths.push(path);
                }
            }
            serde_json::to_value(results).map_err(|error| {
                RpcError::invalid_params(format!("failed to serialize symbol matches: {error}"))
            })?
        }
    } else {
        Value::Array(Vec::new())
    };

    if paths.is_empty() {
        return Err(RpcError::invalid_params(
            "related requires a non-empty path or symbol",
        ));
    }

    Ok((paths, symbol_matches))
}

fn call_get_pack(arguments: Value) -> Result<Value, RpcError> {
    let args: GetPackArgs = serde_json::from_value(arguments).map_err(|error| {
        RpcError::invalid_params(format!("invalid get_pack arguments: {error}"))
    })?;

    if args.task.trim().is_empty() {
        return Err(RpcError::invalid_params("task must not be empty"));
    }

    let repo = discover_repo(args.repo)?;
    let budget = args.budget.unwrap_or(PackBudget::Brief);
    let (plan, pack) = compile_context_pack_with_plan_and_paths(
        &repo.path,
        &args.task,
        args.mode.unwrap_or(TaskType::Explain),
        budget,
        &args.paths,
    )
    .map_err(|error| RpcError::invalid_params(format!("failed to compile pack: {error}")))?;
    let trace = eval_trace_for_pack(
        &repo.path,
        &args.task,
        args.target_agent.as_deref().unwrap_or("generic"),
        &plan,
        &pack,
    );
    append_eval_trace(&repo.path, &trace).map_err(|error| {
        RpcError::invalid_params(format!("failed to record eval trace: {error}"))
    })?;

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
    let results = related_tests(&repo.path, &args.paths)
        .map_err(|error| RpcError::invalid_params(format!("failed to find tests: {error}")))?;

    tool_json_result(results)
}

fn call_current_diff(arguments: Value) -> Result<Value, RpcError> {
    let args: CurrentDiffArgs = serde_json::from_value(arguments).map_err(|error| {
        RpcError::invalid_params(format!("invalid current_diff arguments: {error}"))
    })?;

    let repo = discover_repo(args.repo)?;
    let unstaged = git_lines(&repo.path, &["diff", "--name-only"])?;
    let staged = git_lines(&repo.path, &["diff", "--cached", "--name-only"])?;
    let untracked = if args.include_untracked {
        git_lines(&repo.path, &["ls-files", "--others", "--exclude-standard"])?
    } else {
        Vec::new()
    };
    let (unstaged, excluded_unstaged) = safe_diff_paths(&repo.path, unstaged)?;
    let (staged, excluded_staged) = safe_diff_paths(&repo.path, staged)?;
    let (untracked, excluded_untracked) = safe_diff_paths(&repo.path, untracked)?;

    tool_json_result(json!({
        "unstaged": unstaged,
        "staged": staged,
        "untracked": untracked,
        "excluded": {
            "unstaged": excluded_unstaged,
            "staged": excluded_staged,
            "untracked": excluded_untracked,
            "reason": "paths excluded by safe inventory policy; source content was not returned"
        },
        "privacyStatus": {
            "localOnly": true,
            "sourceTextReturned": false
        }
    }))
}

fn discover_repo(repo: Option<PathBuf>) -> Result<RepoRoot, RpcError> {
    let start = match repo {
        Some(path) => path,
        None => std::env::current_dir()
            .map_err(|error| RpcError::invalid_params(format!("failed to read cwd: {error}")))?,
    };
    RepoRoot::discover_from(&start)
        .map_err(|error| RpcError::invalid_params(format!("failed to discover repo: {error}")))
}

fn bounded_limit(limit: Option<usize>, default: usize) -> usize {
    limit.unwrap_or(default).clamp(1, 50)
}

fn safe_diff_paths(repo: &Path, paths: Vec<String>) -> Result<(Vec<String>, usize), RpcError> {
    if paths.is_empty() {
        return Ok((Vec::new(), 0));
    }
    let inventory = build_inventory(repo, &InventoryOptions::default()).map_err(|error| {
        RpcError::invalid_params(format!("failed to apply safe inventory policy: {error}"))
    })?;
    let safe_paths = inventory
        .files
        .into_iter()
        .map(|file| file.path)
        .collect::<BTreeSet<_>>();
    let mut safe = Vec::new();
    let mut excluded = 0;
    for path in paths {
        if safe_paths.contains(&path) {
            safe.push(path);
        } else {
            excluded += 1;
        }
    }
    Ok((safe, excluded))
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

fn git_lines(repo: &std::path::Path, args: &[&str]) -> Result<Vec<String>, RpcError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|error| RpcError::invalid_params(format!("failed to run git: {error}")))?;
    if !output.status.success() {
        return Err(RpcError::invalid_params(format!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id,
        "result": result
    })
}

fn error_response(id: Value, error: RpcError) -> Value {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id,
        "error": {
            "code": error.code,
            "message": error.message
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::sync::{Mutex, OnceLock};

    struct FixtureRepo {
        _temp: tempfile::TempDir,
        repo: std::path::PathBuf,
        home: std::path::PathBuf,
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn planned_mcp_tool_surface_stays_small() {
        assert_eq!(PLANNED_MCP_TOOL_NAMES.len(), 6);
    }

    #[test]
    fn implemented_tool_surface_stays_small() {
        assert_eq!(IMPLEMENTED_MCP_TOOL_NAMES, PLANNED_MCP_TOOL_NAMES);
    }

    #[test]
    fn initialize_returns_tool_capability() {
        let response =
            handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#).unwrap();

        assert_eq!(response["result"]["serverInfo"]["name"], "ctxpack");
        assert_eq!(
            response["result"]["capabilities"]["tools"]["listChanged"],
            false
        );
        assert_eq!(
            response["result"]["capabilities"]["resources"]["listChanged"],
            false
        );
        assert_eq!(
            response["result"]["capabilities"]["prompts"]["listChanged"],
            false
        );
    }

    #[test]
    fn initialized_notification_returns_no_response() {
        let response =
            handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#);

        assert!(response.is_none());
    }

    #[test]
    fn tools_list_only_exposes_implemented_tools() {
        let response =
            handle_line(r#"{"jsonrpc":"2.0","id":"tools","method":"tools/list","params":{}}"#)
                .unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();

        assert_eq!(tools.len(), 6);
        assert_eq!(tools[0]["name"], "prepare_task");
        assert_eq!(tools[0]["inputSchema"]["required"][0], "task");
        assert_eq!(
            tools[0]["inputSchema"]["properties"]["paths"]["items"]["type"],
            "string"
        );
        assert_eq!(tools[1]["name"], "search");
        assert_eq!(tools[2]["name"], "related");
        assert_eq!(tools[3]["name"], "get_pack");
        assert_eq!(tools[3]["inputSchema"]["required"][0], "task");
        assert_eq!(
            tools[3]["inputSchema"]["properties"]["paths"]["items"]["type"],
            "string"
        );
        assert_eq!(tools[4]["name"], "related_tests");
        assert_eq!(tools[5]["name"], "current_diff");
    }

    #[test]
    fn resources_list_exposes_repo_resources() {
        let response = handle_line(
            r#"{"jsonrpc":"2.0","id":"resources","method":"resources/list","params":{}}"#,
        )
        .unwrap();
        let resources = response["result"]["resources"].as_array().unwrap();

        assert_eq!(resources.len(), 4);
        assert_eq!(resources[0]["uri"], "ctxpack://repo/summary");
        assert_eq!(resources[1]["uri"], "ctxpack://repo/test-map");
        assert_eq!(resources[2]["uri"], "ctxpack://repo/dependency-graph");
        assert_eq!(resources[3]["uri"], "ctxpack://pack/guide");
    }

    #[test]
    fn prompts_list_exposes_workflow_prompts() {
        let response =
            handle_line(r#"{"jsonrpc":"2.0","id":"prompts","method":"prompts/list","params":{}}"#)
                .unwrap();
        let prompts = response["result"]["prompts"].as_array().unwrap();

        assert_eq!(prompts.len(), 6);
        assert_eq!(prompts[0]["name"], "bugfix");
        assert_eq!(prompts[5]["name"], "explain_area");
    }

    #[test]
    fn prepare_task_call_returns_structured_context_plan() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"prepare_task","arguments":{{"task":"fix requireSession bug","repo":"{}","mode":"bug_fix"}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["taskType"],
            "bug_fix"
        );
        assert_eq!(
            response["result"]["structuredContent"]["privacyStatus"]["localOnly"],
            true
        );
        assert_eq!(
            response["result"]["structuredContent"]["targetFiles"][0]["path"],
            "src/auth/session.ts"
        );
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("\"taskType\": \"bug_fix\""));
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prepare_task_call_caches_pack_resources_for_session_reads() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":18,"method":"tools/call","params":{{"name":"prepare_task","arguments":{{"task":"fix requireSession bug","repo":"{}","mode":"bug_fix"}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();
        let task_id = response["result"]["structuredContent"]["taskId"]
            .as_str()
            .unwrap();
        let pack_uri = response["result"]["structuredContent"]["packOptions"][0]["resourceUri"]
            .as_str()
            .unwrap();
        let resource_request = format!(
            r#"{{"jsonrpc":"2.0","id":19,"method":"resources/read","params":{{"uri":"{pack_uri}"}}}}"#
        );
        let resource_response = handle_line(&resource_request).unwrap();
        let text = resource_response["result"]["contents"][0]["text"]
            .as_str()
            .unwrap();

        assert_eq!(
            resource_response["result"]["contents"][0]["mimeType"],
            "text/markdown"
        );
        assert!(text.contains("# Context Pack"));
        assert!(text.contains("src/auth/session.ts"));
        assert!(text.contains("tests/auth/session.test.ts"));
        assert!(text.contains(&format!("Task ID: `{task_id}`")));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prepare_task_call_prefers_path_anchor() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":15,"method":"tools/call","params":{{"name":"prepare_task","arguments":{{"task":"explain unrelated area","repo":"{}","mode":"explain","paths":["src/auth/session.ts"]}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["targetFiles"][0]["path"],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["targetFiles"][0]["reason"],
            "explicit path anchor from active context"
        );
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prepare_task_requires_task_argument() {
        let response = handle_line(
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"prepare_task","arguments":{"mode":"explain"}}}"#,
        )
        .unwrap();

        assert_eq!(response["error"]["code"], -32602);
        assert!(response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("missing field `task`"));
    }

    #[test]
    fn resource_read_returns_repo_summary_and_test_map() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo.repo).unwrap();

        let summary = handle_line(
            r#"{"jsonrpc":"2.0","id":10,"method":"resources/read","params":{"uri":"ctxpack://repo/summary"}}"#,
        )
        .unwrap();
        let test_map = handle_line(
            r#"{"jsonrpc":"2.0","id":11,"method":"resources/read","params":{"uri":"ctxpack://repo/test-map"}}"#,
        )
        .unwrap();
        let dependency_graph = handle_line(
            r#"{"jsonrpc":"2.0","id":17,"method":"resources/read","params":{"uri":"ctxpack://repo/dependency-graph"}}"#,
        )
        .unwrap();

        assert!(summary["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("\"fileCount\""));
        assert!(test_map["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("tests/auth/session.test.ts"));
        assert!(dependency_graph["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("src/auth/cookies.ts"));

        std::env::set_current_dir(cwd).unwrap();
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn resource_test_map_uses_package_aware_commands() {
        let _guard = env_lock();
        let repo = fixture_repo();
        fs::write(
            repo.repo.join("package.json"),
            r#"{"scripts":{"test":"vitest run"}}"#,
        )
        .unwrap();
        fs::write(repo.repo.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo.repo).unwrap();

        let test_map = handle_line(
            r#"{"jsonrpc":"2.0","id":20,"method":"resources/read","params":{"uri":"ctxpack://repo/test-map"}}"#,
        )
        .unwrap();
        let text = test_map["result"]["contents"][0]["text"].as_str().unwrap();

        assert!(text.contains("tests/auth/session.test.ts"));
        assert!(text.contains("pnpm vitest run tests/auth/session.test.ts"));
        assert!(text.contains("safe test file from inventory"));

        std::env::set_current_dir(cwd).unwrap();
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn resource_read_returns_safe_file_slice_and_symbol_results() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo.repo).unwrap();

        let file = handle_line(
            r#"{"jsonrpc":"2.0","id":12,"method":"resources/read","params":{"uri":"ctxpack://file/src/auth/session.ts?lines=1-2"}}"#,
        )
        .unwrap();
        let symbol = handle_line(
            r#"{"jsonrpc":"2.0","id":13,"method":"resources/read","params":{"uri":"ctxpack://symbol/requireSession"}}"#,
        )
        .unwrap();

        let file_text = file["result"]["contents"][0]["text"].as_str().unwrap();
        assert!(file_text.contains("1: import { parseCookie } from './cookies';"));
        assert!(file_text.contains("2: export function requireSession"));
        assert!(!file_text.contains("4: }"));
        assert!(symbol["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("src/auth/session.ts"));

        std::env::set_current_dir(cwd).unwrap();
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prompts_get_returns_agent_workflow_instruction() {
        let response = handle_line(
            r#"{"jsonrpc":"2.0","id":14,"method":"prompts/get","params":{"name":"bugfix","arguments":{"task":"fix auth redirect"}}}"#,
        )
        .unwrap();
        let text = response["result"]["messages"][0]["content"]["text"]
            .as_str()
            .unwrap();

        assert!(text.contains("Task: fix auth redirect"));
        assert!(text.contains("ctxpack.prepare_task"));
        assert!(text.contains("host agent's native tools"));
    }

    #[test]
    fn search_call_returns_file_and_symbol_matches() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{{"name":"search","arguments":{{"query":"requireSession","repo":"{}","limit":2}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["files"][0]["path"],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["symbols"][0]["symbol"]["name"],
            "requireSession"
        );
        assert_eq!(
            response["result"]["structuredContent"]["privacyStatus"]["sourceTextReturned"],
            false
        );
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("src/auth/session.ts"));
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn search_call_filters_to_symbol_kind() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":25,"method":"tools/call","params":{{"name":"search","arguments":{{"query":"requireSession","repo":"{}","limit":2,"kinds":["symbol"]}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["files"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            response["result"]["structuredContent"]["symbols"][0]["symbol"]["path"],
            "src/auth/session.ts"
        );
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_call_returns_tests_co_change_hints_and_dependency_edges() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{{"name":"related","arguments":{{"path":"src/auth/session.ts","repo":"{}","limit":3}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["relatedTests"][0]["path"],
            "tests/auth/session.test.ts"
        );
        let co_change_paths = response["result"]["structuredContent"]["coChangeHints"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|hint| hint["path"].as_str())
            .collect::<Vec<_>>();
        assert!(co_change_paths.contains(&"tests/auth/session.test.ts"));
        assert_eq!(
            response["result"]["structuredContent"]["dependencyEdges"][0]["sourcePath"],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["dependencyEdges"][0]["targetPath"],
            "src/auth/cookies.ts"
        );
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_call_accepts_symbol_anchor() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":22,"method":"tools/call","params":{{"name":"related","arguments":{{"symbol":"requireSession","repo":"{}","limit":3}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["resolvedPaths"][0],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["symbolMatches"][0]["symbol"]["name"],
            "requireSession"
        );
        assert_eq!(
            response["result"]["structuredContent"]["relatedTests"][0]["path"],
            "tests/auth/session.test.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["dependencyEdges"][0]["sourcePath"],
            "src/auth/session.ts"
        );
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_call_requires_path_or_symbol() {
        let response = handle_line(
            r#"{"jsonrpc":"2.0","id":23,"method":"tools/call","params":{"name":"related","arguments":{"limit":3}}}"#,
        )
        .unwrap();

        assert_eq!(response["error"]["code"], -32602);
        assert!(response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("path or symbol"));
    }

    #[test]
    fn related_call_degrades_when_git_history_is_unavailable() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":24,"method":"tools/call","params":{{"name":"related","arguments":{{"symbol":"requireSession","repo":"{}","limit":3}}}}}}"#,
            repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["resolvedPaths"][0],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["relatedTests"][0]["path"],
            "tests/auth/session.test.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["coChangeHints"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert!(response["result"]["structuredContent"]["warnings"][0]
            .as_str()
            .unwrap()
            .contains("co-change hints were unavailable"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_tests_call_returns_targeted_command() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{{"name":"related_tests","arguments":{{"paths":["src/auth/session.ts"],"repo":"{}"}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"][0]["path"],
            "tests/auth/session.test.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"][0]["command"],
            "pnpm test tests/auth/session.test.ts"
        );
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn current_diff_call_returns_changed_paths_without_source_content() {
        let repo = fixture_repo();
        fs::write(
            repo.repo.join("src/auth/session.ts"),
            "export function requireSession() {\n  return false;\n}\n",
        )
        .unwrap();
        fs::write(repo.repo.join("notes.md"), "scratch\n").unwrap();
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{{"name":"current_diff","arguments":{{"repo":"{}","includeUntracked":true}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["unstaged"][0],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["untracked"][0],
            "notes.md"
        );
        assert!(!response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("return false"));
        assert_eq!(
            response["result"]["structuredContent"]["privacyStatus"]["sourceTextReturned"],
            false
        );
    }

    #[test]
    fn current_diff_filters_sensitive_and_generated_paths() {
        let repo = fixture_repo();
        fs::create_dir_all(repo.repo.join("dist")).unwrap();
        fs::write(repo.repo.join("src/auth/session.ts"), "safe change\n").unwrap();
        fs::write(repo.repo.join("dist/generated.js"), "generated change\n").unwrap();
        fs::write(repo.repo.join(".env"), "TOKEN=secret\n").unwrap();
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":21,"method":"tools/call","params":{{"name":"current_diff","arguments":{{"repo":"{}","includeUntracked":true}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();
        let text = response["result"]["content"][0]["text"].as_str().unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["unstaged"][0],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["excluded"]["untracked"],
            2
        );
        assert!(!text.contains("dist/generated.js"));
        assert!(!text.contains(".env"));
        assert!(!text.contains("TOKEN=secret"));
    }

    #[test]
    fn unknown_method_returns_method_not_found() {
        let response = handle_line(
            r#"{"jsonrpc":"2.0","id":4,"method":"sampling/createMessage","params":{}}"#,
        )
        .unwrap();

        assert_eq!(response["error"]["code"], -32601);
    }

    #[test]
    fn get_pack_call_returns_markdown_and_structured_pack() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{{"name":"get_pack","arguments":{{"task":"fix requireSession bug","repo":"{}","mode":"bug_fix","budget":"brief","format":"markdown"}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();
        let text = response["result"]["content"][0]["text"].as_str().unwrap();

        assert_eq!(response["result"]["structuredContent"]["budget"], "brief");
        assert_eq!(
            response["result"]["structuredContent"]["privacyStatus"]["localOnly"],
            true
        );
        assert!(text.contains("# Context Pack"));
        assert!(text.contains("src/auth/session.ts"));
        assert!(text.contains("tests/auth/session.test.ts"));
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn get_pack_call_uses_path_anchor_in_pack() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":16,"method":"tools/call","params":{{"name":"get_pack","arguments":{{"task":"explain unrelated area","repo":"{}","mode":"explain","budget":"brief","format":"markdown","paths":["src/auth/session.ts"]}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();
        let text = response["result"]["content"][0]["text"].as_str().unwrap();

        assert!(text.contains("explicit path anchor from active context"));
        assert!(text.contains("src/auth/session.ts"));
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn run_server_writes_one_response_per_request_line() {
        let input = br#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
"#;
        let mut output = Vec::new();

        run_server(&input[..], &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert_eq!(output.lines().count(), 1);
        assert!(output.contains("\"prepare_task\""));
    }

    fn fixture_repo() -> FixtureRepo {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(
            repo.join("src/auth/cookies.ts"),
            "export function parseCookie() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "import { parseCookie } from './cookies';\nexport function requireSession() {\n  return parseCookie();\n}\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add session"]);

        FixtureRepo {
            _temp: temp,
            repo,
            home,
        }
    }

    fn run_git(repo: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
