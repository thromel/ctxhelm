use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum AgentAdapter {
    Cursor,
    Claude,
    #[serde(rename = "opencode")]
    OpenCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct InitOptions {
    pub adapters: Vec<AgentAdapter>,
}

impl InitOptions {
    pub fn all_adapters() -> Self {
        Self {
            adapters: vec![
                AgentAdapter::Cursor,
                AgentAdapter::Claude,
                AgentAdapter::OpenCode,
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InitAction {
    Created,
    Updated,
    Unchanged,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InitFile {
    pub path: PathBuf,
    pub action: InitAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InitNextStep {
    pub label: String,
    pub command: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InitReport {
    pub repo_root: PathBuf,
    pub files: Vec<InitFile>,
    pub codex_mcp_setup: String,
    pub next_steps: Vec<InitNextStep>,
}

impl InitReport {
    pub fn new(repo_root: PathBuf) -> Self {
        Self {
            repo_root,
            files: Vec::new(),
            codex_mcp_setup: CODEX_MCP_SETUP.trim().to_string(),
            next_steps: init_next_steps(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SetupCheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetupCheckItem {
    pub name: String,
    pub status: SetupCheckStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetupCheckReport {
    #[serde(default = "setup_check_report_schema_version")]
    pub schema_version: String,
    pub repo_root: PathBuf,
    pub items: Vec<SetupCheckItem>,
    pub passed: bool,
}

pub fn setup_check_report_schema_version() -> String {
    "ctxhelm-setup-check-report-v1".to_string()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectMcpAction {
    Planned,
    Created,
    Updated,
    Unchanged,
}

impl ProjectMcpAction {
    pub fn as_str(self) -> &'static str {
        match self {
            ProjectMcpAction::Planned => "planned",
            ProjectMcpAction::Created => "created",
            ProjectMcpAction::Updated => "updated",
            ProjectMcpAction::Unchanged => "unchanged",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetupRunReport {
    pub schema_version: String,
    pub command: String,
    pub repo_root: PathBuf,
    pub dry_run: bool,
    pub planned_files: Vec<PathBuf>,
    pub init_report: Option<InitReport>,
    pub project_mcp: ProjectMcpReport,
    pub setup_check: Option<SetupCheckReport>,
    pub privacy_status: SetupPrivacyStatus,
    pub unsupported_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetupPrivacyStatus {
    pub local_only: bool,
    pub source_text_logged: bool,
    pub raw_prompt_stored: bool,
    pub global_config_mutated: bool,
    pub remote_embeddings_used: bool,
    pub remote_reranking_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMcpReport {
    pub path: PathBuf,
    pub action: ProjectMcpAction,
    pub binary: PathBuf,
    pub command_uses_absolute_binary: bool,
}

#[derive(Debug, Clone)]
pub struct SetupRunReportInput {
    pub command: String,
    pub repo_root: PathBuf,
    pub dry_run: bool,
    pub planned_files: Vec<PathBuf>,
    pub init_report: Option<InitReport>,
    pub project_mcp: ProjectMcpReport,
    pub setup_check: Option<SetupCheckReport>,
}

#[derive(Debug, Error)]
pub enum InitError {
    #[error("failed to read {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to write {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
    #[error("failed to create directory {path}: {source}")]
    CreateDir { path: PathBuf, source: io::Error },
    #[error("unsafe init path {path}: {reason}")]
    UnsafePath { path: PathBuf, reason: String },
}

pub const AGENTS_SECTION_START: &str = "<!-- ctxhelm:start -->";
pub const AGENTS_SECTION_END: &str = "<!-- ctxhelm:end -->";

pub const CTXHELM_TOML: &str = r#"version = 1
local_only = true

[adapters]
agents_md = true
cursor_rules = false
claude_commands = false
claude_mcp_snippet = false
opencode_snippet = false
"#;

pub const AGENTS_SECTION: &str = r#"<!-- ctxhelm:start -->
## ctxhelm

For non-trivial code changes, call the ctxhelm MCP tool `prepare_task` before planning edits when MCP is configured. Pass the active repository path as `repo` when the agent knows it. Otherwise, run `ctxhelm prepare-task --repo <repo>`.

Use ctxhelm for:
- likely target files
- related tests
- relevant examples
- architecture constraints
- validation commands

Start by reading the first up to 5 returned target files with the agent's native tools before editing; discovering a path is not the same as consuming it. If selectedMemory appears, read up to 3 memory source/evidence paths too. Docs, config, schema, and script entries in that initial set are first-class targets, not optional background. Stop after those reads if they answer the task. Request `get_pack` progressively, starting brief or standard, only when native file reads or the plan are insufficient.

Pack resources returned by `prepare_task` are session-scoped MCP resources. After reconnect or server restart, call `get_pack` with the same task and `repo` for durable materialization.
<!-- ctxhelm:end -->
"#;

pub const CURSOR_RULE: &str = r#"---
description: Use ctxhelm to gather precise repository context before non-trivial edits
alwaysApply: true
---

For tasks that modify code, investigate bugs, add features, or affect multiple files:

1. Call the ctxhelm MCP tool `prepare_task` when available, passing the active repository path as `repo` when known, or run `ctxhelm prepare-task --repo <repo>`.
2. Prefer returned target files, related tests, examples, and constraints.
3. Read the first up to 5 returned target files using Cursor's native file tools before editing; discovering a path is not the same as consuming it. If selectedMemory appears, read up to 3 memory source/evidence paths too. Docs, config, schema, and script files in that initial target set are first-class targets, not optional background. Stop after those reads if they answer the task.
4. Call `get_pack` progressively only when direct file reads or brief context are insufficient.
5. Run targeted validation commands returned by ctxhelm when available.

Pack resources returned by `prepare_task` are same-session MCP resources. After reconnect or server restart, call `get_pack` with the same task and `repo`.
"#;

pub const CLAUDE_BUGFIX_COMMAND: &str = r#"# ctxhelm Bugfix

Use this command for non-trivial bug fixes.

1. Call `ctxhelm.prepare_task` for the user's task when MCP is available, passing the active repository path as `repo` when known.
2. Read the first up to 5 returned target files with native tools; discovering a path is not the same as consuming it. If selectedMemory appears, read up to 3 memory source/evidence paths too. Docs, config, schema, and script files in that initial target set are first-class targets, not optional background. Stop after those reads if they answer the task.
3. Request `get_pack` progressively, starting brief or standard, only if direct file reads are insufficient.
4. Make the smallest patch that addresses the bug.
5. Run the related test command returned by ctxhelm when available.
6. Summarize the changed behavior and validation result.

Pack resources returned by `prepare_task` are session-scoped to the same MCP server process. After reconnect or restart, call `get_pack` with the same task and `repo`.
"#;

pub const CLAUDE_MCP_SNIPPET: &str = r#"{
  "mcpServers": {
    "ctxhelm": {
      "command": "ctxhelm",
      "args": ["serve-mcp"]
    }
  }
}
"#;

pub const OPENCODE_SNIPPET: &str = r#"{
  "$schema": "https://opencode.ai/config.json",
  "ctxhelmNote": "Registers ctxhelm as a local read-only MCP context server. Call prepare_task with the active repository path as `repo`. Read first up to 5 target files natively; discovering a path is not the same as consuming it. If selectedMemory appears, read up to 3 memory source/evidence paths too. Treat docs/config/schema/script as first-class targets. Stop if enough; use get_pack only when needed. Pack resources are same-session; after reconnect or restart call get_pack.",
  "instructions": ["AGENTS.md"],
  "mcp": {
    "ctxhelm": {
      "type": "local",
      "command": ["ctxhelm", "serve-mcp"]
    }
  }
}
"#;

pub const CODEX_MCP_SETUP: &str = r#"
Codex MCP setup:

Copy/paste this local stdio MCP server setup yourself; ctxhelm does not apply it automatically:

  codex mcp add ctxhelm -- ctxhelm serve-mcp

Call `prepare_task` first with the active repository path as `repo` when known. Read first up to 5 target files with Codex native tools; discovering a path is not the same as consuming it. If selectedMemory appears, read up to 3 memory source/evidence paths too. Treat docs/config/schema/script as first-class targets. Stop if enough. Call `get_pack` only when native reads or brief context are insufficient. Pack resources are session-scoped; after reconnect or restart, call `get_pack` with the same task and `repo`.

If an agent cannot spawn `ctxhelm`, replace the command with the absolute path from `which ctxhelm`. This command does not mutate global Codex config automatically.
"#;

pub fn adapter_path(adapter: AgentAdapter) -> &'static str {
    match adapter {
        AgentAdapter::Cursor => ".cursor/rules/ctxhelm.mdc",
        AgentAdapter::Claude => ".claude/commands/ctxhelm-bugfix.md",
        AgentAdapter::OpenCode => ".ctxhelm/adapters/opencode.jsonc.snippet",
    }
}

pub fn adapter_content(adapter: AgentAdapter) -> &'static str {
    match adapter {
        AgentAdapter::Cursor => CURSOR_RULE,
        AgentAdapter::Claude => CLAUDE_BUGFIX_COMMAND,
        AgentAdapter::OpenCode => OPENCODE_SNIPPET,
    }
}

pub fn adapter_files(adapter: AgentAdapter) -> Vec<(&'static str, &'static str)> {
    let mut files = vec![(adapter_path(adapter), adapter_content(adapter))];
    if adapter == AgentAdapter::Claude {
        files.push((".ctxhelm/adapters/claude-mcp.json", CLAUDE_MCP_SNIPPET));
    }
    files
}

pub fn agents_section() -> &'static str {
    AGENTS_SECTION
}

pub fn run_init(
    repo_root: impl AsRef<Path>,
    options: &InitOptions,
) -> Result<InitReport, InitError> {
    let repo_root = repo_root.as_ref();
    let mut report = InitReport::new(repo_root.to_path_buf());

    write_file(
        repo_root,
        ".ctxhelm/ctxhelm.toml",
        config_toml(options),
        &mut report,
    )?;

    upsert_agents_section(repo_root, &mut report)?;

    for adapter in &options.adapters {
        for (path, content) in adapter_files(*adapter) {
            write_file(repo_root, path, content.to_string(), &mut report)?;
        }
    }
    record_skipped_adapter_files(repo_root, options, &mut report);

    Ok(report)
}

pub fn run_setup_check(
    repo_root: impl AsRef<Path>,
    options: &InitOptions,
) -> Result<SetupCheckReport, InitError> {
    let repo_root = repo_root.as_ref();
    let mut items = Vec::new();

    check_setup_file(
        repo_root,
        "AGENTS.md",
        SetupFileKind::Guidance(1_400),
        &mut items,
    );
    check_setup_file(
        repo_root,
        ".ctxhelm/ctxhelm.toml",
        SetupFileKind::Config,
        &mut items,
    );

    for adapter in &options.adapters {
        for (path, _) in adapter_files(*adapter) {
            let kind = match path {
                ".ctxhelm/adapters/claude-mcp.json" => SetupFileKind::Json(300),
                ".ctxhelm/adapters/opencode.jsonc.snippet" => SetupFileKind::Json(900),
                ".cursor/rules/ctxhelm.mdc" => SetupFileKind::Guidance(1_400),
                ".claude/commands/ctxhelm-bugfix.md" => SetupFileKind::Guidance(1_400),
                _ => SetupFileKind::Guidance(1_400),
            };
            check_setup_file(repo_root, path, kind, &mut items);
        }
        if *adapter == AgentAdapter::Claude {
            check_project_mcp_config(repo_root, &mut items);
        }
    }

    items.push(SetupCheckItem {
        name: "ctxhelm binary path guidance".to_string(),
        status: SetupCheckStatus::Warn,
        detail: "Run `ctxhelm --version`; if an agent cannot find ctxhelm on PATH, use the absolute path from `which ctxhelm` in that agent's explicit configuration.".to_string(),
    });

    let passed = !items
        .iter()
        .any(|item| item.status == SetupCheckStatus::Fail);

    Ok(SetupCheckReport {
        schema_version: setup_check_report_schema_version(),
        repo_root: repo_root.to_path_buf(),
        items,
        passed,
    })
}

pub fn build_setup_run_report(input: SetupRunReportInput) -> SetupRunReport {
    SetupRunReport {
        schema_version: "ctxhelm-setup-report-v1".to_string(),
        command: input.command,
        repo_root: input.repo_root,
        dry_run: input.dry_run,
        planned_files: input.planned_files,
        init_report: input.init_report,
        project_mcp: input.project_mcp,
        setup_check: input.setup_check,
        privacy_status: SetupPrivacyStatus {
            local_only: true,
            source_text_logged: false,
            raw_prompt_stored: false,
            global_config_mutated: false,
            remote_embeddings_used: false,
            remote_reranking_used: false,
        },
        unsupported_actions: vec![
            "global agent config mutation".to_string(),
            "source edits".to_string(),
            "project dependency install".to_string(),
            "cloud upload".to_string(),
            "model invocation".to_string(),
        ],
    }
}

pub fn project_mcp_report(
    mcp_path: impl AsRef<Path>,
    action: ProjectMcpAction,
    binary: impl AsRef<Path>,
) -> ProjectMcpReport {
    let binary = binary.as_ref().to_path_buf();
    ProjectMcpReport {
        path: mcp_path.as_ref().to_path_buf(),
        action,
        command_uses_absolute_binary: binary.is_absolute(),
        binary,
    }
}

pub fn repo_setup_planned_files(repo_root: impl AsRef<Path>) -> Vec<PathBuf> {
    planned_files(
        repo_root.as_ref(),
        &[
            "AGENTS.md",
            ".ctxhelm/ctxhelm.toml",
            ".cursor/rules/ctxhelm.mdc",
            ".claude/commands/ctxhelm-bugfix.md",
            ".ctxhelm/adapters/claude-mcp.json",
            ".ctxhelm/adapters/opencode.jsonc.snippet",
            ".mcp.json",
        ],
    )
}

pub fn claude_setup_planned_files(repo_root: impl AsRef<Path>) -> Vec<PathBuf> {
    planned_files(
        repo_root.as_ref(),
        &[
            "AGENTS.md",
            ".ctxhelm/ctxhelm.toml",
            ".claude/commands/ctxhelm-bugfix.md",
            ".ctxhelm/adapters/claude-mcp.json",
            ".mcp.json",
        ],
    )
}

fn planned_files(repo_root: &Path, relative_paths: &[&str]) -> Vec<PathBuf> {
    relative_paths
        .iter()
        .map(|path| repo_root.join(path))
        .collect()
}

#[derive(Debug, Clone, Copy)]
enum SetupFileKind {
    Config,
    Guidance(usize),
    Json(usize),
}

fn check_setup_file(
    repo_root: &Path,
    relative_path: &str,
    kind: SetupFileKind,
    items: &mut Vec<SetupCheckItem>,
) {
    if let Err(error) = reject_existing_symlink_components(repo_root, relative_path) {
        items.push(SetupCheckItem {
            name: relative_path.to_string(),
            status: SetupCheckStatus::Fail,
            detail: format!("unsafe generated setup path: {error}"),
        });
        return;
    }

    let path = repo_root.join(relative_path);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            items.push(SetupCheckItem {
                name: relative_path.to_string(),
                status: SetupCheckStatus::Fail,
                detail: "expected generated setup file is missing".to_string(),
            });
            return;
        }
        Err(error) => {
            items.push(SetupCheckItem {
                name: relative_path.to_string(),
                status: SetupCheckStatus::Fail,
                detail: format!("failed to read generated setup file: {error}"),
            });
            return;
        }
    };

    match validate_setup_content(relative_path, &content, kind) {
        Ok(()) => items.push(SetupCheckItem {
            name: relative_path.to_string(),
            status: SetupCheckStatus::Pass,
            detail: "generated setup artifact is present and valid".to_string(),
        }),
        Err(detail) => items.push(SetupCheckItem {
            name: relative_path.to_string(),
            status: SetupCheckStatus::Fail,
            detail,
        }),
    }
}

fn check_project_mcp_config(repo_root: &Path, items: &mut Vec<SetupCheckItem>) {
    const RELATIVE_PATH: &str = ".mcp.json";

    if let Err(error) = reject_existing_symlink_components(repo_root, RELATIVE_PATH) {
        items.push(SetupCheckItem {
            name: RELATIVE_PATH.to_string(),
            status: SetupCheckStatus::Fail,
            detail: format!("unsafe project MCP config path: {error}"),
        });
        return;
    }

    let path = repo_root.join(RELATIVE_PATH);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            items.push(SetupCheckItem {
                name: RELATIVE_PATH.to_string(),
                status: SetupCheckStatus::Warn,
                detail:
                    "project MCP config is not present; run `ctxhelm setup claude` or `ctxhelm setup repo` to write a repo-local ctxhelm server entry."
                        .to_string(),
            });
            return;
        }
        Err(error) => {
            items.push(SetupCheckItem {
                name: RELATIVE_PATH.to_string(),
                status: SetupCheckStatus::Fail,
                detail: format!("failed to read project MCP config: {error}"),
            });
            return;
        }
    };

    match validate_project_mcp_content(&content) {
        Ok(()) => items.push(SetupCheckItem {
            name: RELATIVE_PATH.to_string(),
            status: SetupCheckStatus::Pass,
            detail: "project MCP config registers ctxhelm with an absolute binary path and serve-mcp args."
                .to_string(),
        }),
        Err(detail) => items.push(SetupCheckItem {
            name: RELATIVE_PATH.to_string(),
            status: SetupCheckStatus::Fail,
            detail,
        }),
    }
}

fn validate_project_mcp_content(content: &str) -> Result<(), String> {
    let value = serde_json::from_str::<serde_json::Value>(content)
        .map_err(|error| format!("project MCP config is invalid JSON: {error}"))?;
    let root = value
        .as_object()
        .ok_or_else(|| "project MCP config root must be a JSON object".to_string())?;
    let servers = root
        .get("mcpServers")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "project MCP config is missing object `mcpServers`".to_string())?;
    let ctxhelm = servers
        .get("ctxhelm")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "project MCP config is missing `mcpServers.ctxhelm`".to_string())?;
    let command = ctxhelm
        .get("command")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "`mcpServers.ctxhelm.command` must be a string".to_string())?;
    if !Path::new(command).is_absolute() {
        return Err("`mcpServers.ctxhelm.command` must be an absolute path".to_string());
    }
    let args = ctxhelm
        .get("args")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "`mcpServers.ctxhelm.args` must be an array".to_string())?;
    let args = args
        .iter()
        .map(|arg| {
            arg.as_str()
                .ok_or_else(|| "`mcpServers.ctxhelm.args` must contain only strings".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if args != ["serve-mcp"] {
        return Err("`mcpServers.ctxhelm.args` must be [\"serve-mcp\"]".to_string());
    }
    Ok(())
}

fn validate_setup_content(
    relative_path: &str,
    content: &str,
    kind: SetupFileKind,
) -> Result<(), String> {
    match kind {
        SetupFileKind::Config => {
            for required in ["version = 1", "local_only = true", "[adapters]"] {
                if !content.contains(required) {
                    return Err(format!(
                        "generated config is missing required setting `{required}`"
                    ));
                }
            }
        }
        SetupFileKind::Guidance(byte_limit) => {
            validate_thin_guidance(relative_path, content, byte_limit)?;
        }
        SetupFileKind::Json(byte_limit) => {
            validate_generated_size_and_forbidden_phrases(content, byte_limit)?;
            serde_json::from_str::<serde_json::Value>(content)
                .map_err(|error| format!("generated JSON snippet is invalid: {error}"))?;
        }
    }
    Ok(())
}

fn validate_thin_guidance(
    relative_path: &str,
    content: &str,
    byte_limit: usize,
) -> Result<(), String> {
    validate_generated_size_and_forbidden_phrases(content, byte_limit)?;

    if relative_path == "AGENTS.md" {
        for marker in [AGENTS_SECTION_START, AGENTS_SECTION_END] {
            if !content.contains(marker) {
                return Err(format!("AGENTS.md is missing ctxhelm marker `{marker}`"));
            }
        }
    }

    for required in ["prepare_task", "repo", "native", "get_pack"] {
        if !content.contains(required) {
            return Err(format!(
                "generated guidance is missing required ctxhelm term `{required}`"
            ));
        }
    }

    if !(content.contains("session-scoped") || content.contains("same-session")) {
        return Err(
            "generated guidance is missing MCP pack resource session-scope caveat".to_string(),
        );
    }

    Ok(())
}

fn validate_generated_size_and_forbidden_phrases(
    content: &str,
    byte_limit: usize,
) -> Result<(), String> {
    if content.len() > byte_limit {
        return Err(format!(
            "generated guidance is {} bytes, above {byte_limit}",
            content.len()
        ));
    }

    for phrase in [
        "Repository map",
        "inventory dump",
        "source snippets",
        "paste this repo",
    ] {
        if content.contains(phrase) {
            return Err(format!(
                "generated guidance contains forbidden large-static-context phrase `{phrase}`"
            ));
        }
    }

    Ok(())
}

fn init_next_steps() -> Vec<InitNextStep> {
    vec![
        InitNextStep {
            label: "Verify binary".to_string(),
            command: "ctxhelm --version && ctxhelm --help".to_string(),
            detail: "Confirm the ctxhelm binary is the expected release and exposes the installed CLI surface."
                .to_string(),
        },
        InitNextStep {
            label: "Validate setup artifacts".to_string(),
            command: "ctxhelm setup-check --repo <repo>".to_string(),
            detail: "Read-only check for generated repo-local files, adapter snippets, and path guidance."
                .to_string(),
        },
        InitNextStep {
            label: "Prove MCP protocol".to_string(),
            command: "scripts/smoke-mcp-protocol.sh".to_string(),
            detail: "Run the deterministic stdio MCP smoke before debugging real agent clients."
                .to_string(),
        },
        InitNextStep {
            label: "Configure one agent explicitly".to_string(),
            command: "codex mcp add ctxhelm -- ctxhelm serve-mcp".to_string(),
            detail: "Copy/paste or merge the generated guidance yourself; ctxhelm does not mutate global agent config."
                .to_string(),
        },
        InitNextStep {
            label: "Request first context".to_string(),
            command: "prepare_task, then get_pack when direct file reads are insufficient".to_string(),
            detail: "Pass the active repository path as repo, inspect target files natively, then materialize a brief pack if needed."
                .to_string(),
        },
    ]
}

fn record_skipped_adapter_files(repo_root: &Path, options: &InitOptions, report: &mut InitReport) {
    for adapter in [
        AgentAdapter::Cursor,
        AgentAdapter::Claude,
        AgentAdapter::OpenCode,
    ] {
        if options.adapters.contains(&adapter) {
            continue;
        }
        for (path, _) in adapter_files(adapter) {
            report.files.push(InitFile {
                path: repo_root.join(path),
                action: InitAction::Skipped,
            });
        }
    }
}

fn config_toml(options: &InitOptions) -> String {
    let cursor = options.adapters.contains(&AgentAdapter::Cursor);
    let claude = options.adapters.contains(&AgentAdapter::Claude);
    let opencode = options.adapters.contains(&AgentAdapter::OpenCode);
    format!(
        "version = 1\nlocal_only = true\n\n[adapters]\nagents_md = true\ncursor_rules = {cursor}\nclaude_commands = {claude}\nclaude_mcp_snippet = {claude}\nopencode_snippet = {opencode}\n"
    )
}

fn write_file(
    repo_root: &Path,
    relative_path: &str,
    content: String,
    report: &mut InitReport,
) -> Result<(), InitError> {
    reject_existing_symlink_components(repo_root, relative_path)?;

    let path = repo_root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InitError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let action = match fs::read_to_string(&path) {
        Ok(existing) if existing == content => InitAction::Unchanged,
        Ok(_) => InitAction::Updated,
        Err(error) if error.kind() == io::ErrorKind::NotFound => InitAction::Created,
        Err(source) => {
            return Err(InitError::Read {
                path: path.clone(),
                source,
            });
        }
    };

    if action != InitAction::Unchanged {
        fs::write(&path, content).map_err(|source| InitError::Write {
            path: path.clone(),
            source,
        })?;
    }

    report.files.push(InitFile { path, action });
    Ok(())
}

fn upsert_agents_section(repo_root: &Path, report: &mut InitReport) -> Result<(), InitError> {
    reject_existing_symlink_components(repo_root, "AGENTS.md")?;

    let path = repo_root.join("AGENTS.md");
    let existed = path.exists();
    let existing = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(source) => {
            return Err(InitError::Read {
                path: path.clone(),
                source,
            });
        }
    };

    let next = if let Some((start, section_end)) = find_agents_section_bounds(&existing) {
        format!(
            "{}{}{}",
            &existing[..start],
            AGENTS_SECTION.trim_end(),
            &existing[section_end..]
        )
    } else if existing.trim().is_empty() {
        format!("# AGENTS.md\n\n{}\n", AGENTS_SECTION.trim_end())
    } else {
        let mut next = existing.clone();
        if !next.ends_with('\n') {
            next.push('\n');
        }
        next.push('\n');
        next.push_str(AGENTS_SECTION.trim_end());
        next.push('\n');
        next
    };

    let action = if existing == next {
        InitAction::Unchanged
    } else if existed {
        InitAction::Updated
    } else {
        InitAction::Created
    };

    if action != InitAction::Unchanged {
        fs::write(&path, next).map_err(|source| InitError::Write {
            path: path.clone(),
            source,
        })?;
    }

    report.files.push(InitFile { path, action });
    Ok(())
}

fn reject_existing_symlink_components(
    repo_root: &Path,
    relative_path: &str,
) -> Result<(), InitError> {
    let mut path = repo_root.to_path_buf();

    for component in Path::new(relative_path).components() {
        let Component::Normal(segment) = component else {
            return Err(InitError::UnsafePath {
                path: repo_root.join(relative_path),
                reason: "generated init path must stay relative to the repo root".to_string(),
            });
        };

        path.push(segment);

        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(InitError::UnsafePath {
                    path,
                    reason: "path component is a symlink".to_string(),
                });
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(InitError::Read {
                    path: path.clone(),
                    source,
                });
            }
        }
    }

    Ok(())
}

fn find_agents_section_bounds(existing: &str) -> Option<(usize, usize)> {
    let mut search_from = 0;

    while let Some(relative_start) = existing[search_from..].find(AGENTS_SECTION_START) {
        let start = search_from + relative_start;
        let after_start = start + AGENTS_SECTION_START.len();
        let next_start = existing[after_start..]
            .find(AGENTS_SECTION_START)
            .map(|index| after_start + index);
        let next_end = existing[after_start..]
            .find(AGENTS_SECTION_END)
            .map(|index| after_start + index);

        match (next_start, next_end) {
            (Some(next_start), Some(next_end)) if next_start < next_end => {
                search_from = next_start;
            }
            (_, Some(next_end)) => {
                return Some((start, next_end + AGENTS_SECTION_END.len()));
            }
            (Some(next_start), None) => {
                search_from = next_start;
            }
            (None, None) => return None,
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generated_guidance_artifacts() -> [(&'static str, &'static str, usize); 5] {
        [
            ("AGENTS", AGENTS_SECTION, 1_400),
            ("Cursor", CURSOR_RULE, 1_400),
            ("Claude command", CLAUDE_BUGFIX_COMMAND, 1_400),
            ("OpenCode snippet", OPENCODE_SNIPPET, 900),
            ("Codex setup", CODEX_MCP_SETUP, 900),
        ]
    }

    #[test]
    fn adapter_guidance_stays_thin_and_dynamic() {
        let forbidden = [
            "Repository map:",
            "Full context pack",
            "inventory dump",
            "paste this repo",
            "source snippets",
        ];

        for (name, content, byte_limit) in generated_guidance_artifacts() {
            assert!(
                content.len() <= byte_limit,
                "{name} guidance is {} bytes, above {byte_limit}",
                content.len()
            );
            assert!(
                content.contains("prepare_task"),
                "{name} guidance must point agents to dynamic prepare_task calls"
            );
            assert!(
                content.contains("active repository path as `repo`"),
                "{name} guidance must tell agents to pass explicit repo when known"
            );

            for phrase in forbidden {
                assert!(
                    !content.contains(phrase),
                    "{name} guidance must not include static context dump phrase {phrase:?}"
                );
            }
        }
    }

    #[test]
    fn adapter_guidance_points_to_progressive_packs() {
        for (name, content, _) in generated_guidance_artifacts() {
            assert!(
                content.contains("prepare_task"),
                "{name} guidance must call prepare_task first"
            );
            assert!(
                content.contains("get_pack") || content.contains("progressive"),
                "{name} guidance must mention get_pack or progressive pack loading"
            );
        }
    }

    #[test]
    fn adapter_guidance_explains_progressive_native_reads_and_session_scope() {
        for (name, content, _) in generated_guidance_artifacts() {
            assert!(
                content.contains("prepare_task"),
                "{name} guidance must tell agents to call prepare_task"
            );
            assert!(
                content.contains("repo"),
                "{name} guidance must include explicit repo usage"
            );
            assert!(
                content.contains("native"),
                "{name} guidance must direct agents to read files with native tools"
            );
            assert!(
                content.contains("discovering a path is not the same as consuming it"),
                "{name} guidance must distinguish discovered paths from consumed files"
            );
            assert!(
                content.contains("first-class targets"),
                "{name} guidance must treat docs/config/schema/script targets as first-class targets"
            );
            assert!(
                content.contains("selectedMemory"),
                "{name} guidance must direct native reads for selected memory evidence"
            );
            assert!(
                content.contains("get_pack"),
                "{name} guidance must direct progressive get_pack materialization"
            );
            assert!(
                content.contains("session-scoped") || content.contains("same-session"),
                "{name} guidance must explain MCP pack resource session scope"
            );
            assert!(
                content.contains("reconnect") || content.contains("restart"),
                "{name} guidance must point reconnects or restarts to get_pack"
            );
        }
    }

    #[test]
    fn adapter_guidance_does_not_claim_prepare_task_is_the_only_tool() {
        assert!(!OPENCODE_SNIPPET.contains("The first implemented tool is prepare_task"));
        assert!(!CODEX_MCP_SETUP.contains("The first implemented tool is `prepare_task`"));
    }

    #[test]
    fn claude_mcp_snippet_stays_small_local_stdio_config() {
        let value = serde_json::from_str::<serde_json::Value>(CLAUDE_MCP_SNIPPET).unwrap();

        assert!(CLAUDE_MCP_SNIPPET.len() <= 300);
        assert_eq!(
            value["mcpServers"]["ctxhelm"]["command"].as_str(),
            Some("ctxhelm")
        );
        assert_eq!(
            value["mcpServers"]["ctxhelm"]["args"][0].as_str(),
            Some("serve-mcp")
        );
        assert!(!CLAUDE_MCP_SNIPPET.contains("claude mcp add"));
        assert!(!CLAUDE_MCP_SNIPPET.contains("global"));
    }

    #[test]
    fn adapter_paths_are_repo_local() {
        assert_eq!(
            adapter_path(AgentAdapter::Cursor),
            ".cursor/rules/ctxhelm.mdc"
        );
        assert_eq!(
            adapter_path(AgentAdapter::Claude),
            ".claude/commands/ctxhelm-bugfix.md"
        );
        assert_eq!(
            adapter_path(AgentAdapter::OpenCode),
            ".ctxhelm/adapters/opencode.jsonc.snippet"
        );
        assert!(adapter_files(AgentAdapter::Claude)
            .iter()
            .any(|(path, _)| *path == ".ctxhelm/adapters/claude-mcp.json"));
    }

    #[test]
    fn codex_setup_is_guidance_only() {
        assert!(CODEX_MCP_SETUP.contains("local stdio MCP server"));
        assert!(CODEX_MCP_SETUP.contains("ctxhelm serve-mcp"));
        assert!(CODEX_MCP_SETUP.contains("codex mcp add ctxhelm -- ctxhelm serve-mcp"));
        assert!(CODEX_MCP_SETUP.contains("does not mutate global Codex config"));
        assert!(CODEX_MCP_SETUP.contains("prepare_task"));
        assert!(CODEX_MCP_SETUP.contains("active repository path as `repo`"));
    }

    #[test]
    fn opencode_adapter_serializes_without_hyphen() {
        let serialized = serde_json::to_string(&AgentAdapter::OpenCode).unwrap();
        assert_eq!(serialized, r#""opencode""#);
    }

    #[test]
    fn opencode_snippet_mentions_mcp_guidance() {
        assert!(OPENCODE_SNIPPET.contains("local read-only MCP context server"));
        assert!(OPENCODE_SNIPPET.contains("prepare_task"));
        assert!(OPENCODE_SNIPPET.contains("ctxhelm"));
        assert!(OPENCODE_SNIPPET.contains("serve-mcp"));
    }

    #[test]
    fn agents_section_contains_ctxhelm_markers() {
        assert!(AGENTS_SECTION.contains("when MCP is configured"));
        assert!(AGENTS_SECTION.contains("prepare_task"));
        assert!(AGENTS_SECTION.contains("active repository path as `repo`"));
        assert!(AGENTS_SECTION.contains(AGENTS_SECTION_START));
        assert!(AGENTS_SECTION.contains(AGENTS_SECTION_END));
    }

    #[test]
    fn native_rules_ask_agents_to_pass_explicit_repo_path() {
        assert!(CURSOR_RULE.contains("active repository path as `repo`"));
        assert!(CLAUDE_BUGFIX_COMMAND.contains("active repository path as `repo`"));
    }

    #[test]
    fn claude_mcp_snippet_is_valid_project_mcp_json() {
        let value = serde_json::from_str::<serde_json::Value>(CLAUDE_MCP_SNIPPET).unwrap();

        assert_eq!(
            value["mcpServers"]["ctxhelm"]["command"].as_str(),
            Some("ctxhelm")
        );
        assert_eq!(
            value["mcpServers"]["ctxhelm"]["args"][0].as_str(),
            Some("serve-mcp")
        );
    }

    #[test]
    fn opencode_snippet_is_valid_json() {
        serde_json::from_str::<serde_json::Value>(OPENCODE_SNIPPET).unwrap();
    }
}

#[cfg(test)]
mod writer_tests {
    use super::*;

    #[test]
    fn init_creates_config_agents_and_requested_adapters() {
        let temp = tempfile::tempdir().unwrap();
        let options = InitOptions {
            adapters: vec![
                AgentAdapter::Cursor,
                AgentAdapter::Claude,
                AgentAdapter::OpenCode,
            ],
        };

        let report = run_init(temp.path(), &options).unwrap();

        assert!(temp.path().join(".ctxhelm/ctxhelm.toml").exists());
        assert!(temp.path().join("AGENTS.md").exists());
        assert!(temp.path().join(".cursor/rules/ctxhelm.mdc").exists());
        assert!(temp
            .path()
            .join(".claude/commands/ctxhelm-bugfix.md")
            .exists());
        assert!(temp
            .path()
            .join(".ctxhelm/adapters/claude-mcp.json")
            .exists());
        assert!(temp
            .path()
            .join(".ctxhelm/adapters/opencode.jsonc.snippet")
            .exists());
        assert!(report
            .files
            .iter()
            .all(|file| file.action == InitAction::Created));
        assert!(report.codex_mcp_setup.contains("ctxhelm serve-mcp"));
    }

    #[test]
    fn init_replaces_existing_bounded_agents_section() {
        let temp = tempfile::tempdir().unwrap();
        let agents = temp.path().join("AGENTS.md");
        std::fs::write(
            &agents,
            format!(
                "# Existing\n\n{}\nold\n{}\n\nKeep this.\n",
                AGENTS_SECTION_START, AGENTS_SECTION_END
            ),
        )
        .unwrap();

        run_init(temp.path(), &InitOptions::default()).unwrap();
        let content = std::fs::read_to_string(agents).unwrap();

        assert!(content.contains("# Existing"));
        assert!(content.contains("Keep this."));
        assert!(content.contains("For non-trivial code changes"));
        assert!(!content.contains("\nold\n"));
    }

    #[test]
    fn init_appends_when_agents_section_has_start_only_marker() {
        let temp = tempfile::tempdir().unwrap();
        let agents = temp.path().join("AGENTS.md");
        std::fs::write(
            &agents,
            format!(
                "# Existing\n\n{}\nKeep this user content.\n",
                AGENTS_SECTION_START
            ),
        )
        .unwrap();

        run_init(temp.path(), &InitOptions::default()).unwrap();
        let content = std::fs::read_to_string(agents).unwrap();

        assert!(content.contains("Keep this user content."));
        assert_eq!(content.matches(AGENTS_SECTION_START).count(), 2);
        assert_eq!(content.matches(AGENTS_SECTION_END).count(), 1);
    }

    #[test]
    fn init_appends_when_agents_section_has_end_only_marker() {
        let temp = tempfile::tempdir().unwrap();
        let agents = temp.path().join("AGENTS.md");
        std::fs::write(
            &agents,
            format!(
                "# Existing\n\n{}\nKeep this user content.\n",
                AGENTS_SECTION_END
            ),
        )
        .unwrap();

        run_init(temp.path(), &InitOptions::default()).unwrap();
        let content = std::fs::read_to_string(agents).unwrap();

        assert!(content.contains("Keep this user content."));
        assert_eq!(content.matches(AGENTS_SECTION_START).count(), 1);
        assert_eq!(content.matches(AGENTS_SECTION_END).count(), 2);
    }

    #[test]
    fn init_appends_when_agents_section_has_end_before_start() {
        let temp = tempfile::tempdir().unwrap();
        let agents = temp.path().join("AGENTS.md");
        std::fs::write(
            &agents,
            format!(
                "# Existing\n\n{}\nKeep this before start.\n{}\nKeep this after start.\n",
                AGENTS_SECTION_END, AGENTS_SECTION_START
            ),
        )
        .unwrap();

        run_init(temp.path(), &InitOptions::default()).unwrap();
        let content = std::fs::read_to_string(agents).unwrap();

        assert!(content.contains("Keep this before start."));
        assert!(content.contains("Keep this after start."));
        assert_eq!(content.matches(AGENTS_SECTION_START).count(), 2);
        assert_eq!(content.matches(AGENTS_SECTION_END).count(), 2);
    }

    #[test]
    fn init_twice_after_malformed_marker_preserves_intervening_user_content() {
        let temp = tempfile::tempdir().unwrap();
        let agents = temp.path().join("AGENTS.md");
        std::fs::write(
            &agents,
            format!(
                "# Existing\n\n{}\nIntervening user content must stay.\n",
                AGENTS_SECTION_START
            ),
        )
        .unwrap();

        run_init(temp.path(), &InitOptions::default()).unwrap();
        run_init(temp.path(), &InitOptions::default()).unwrap();
        let content = std::fs::read_to_string(agents).unwrap();

        assert_eq!(
            content
                .matches("Intervening user content must stay.")
                .count(),
            1
        );
        assert_eq!(content.matches(AGENTS_SECTION_START).count(), 2);
        assert_eq!(content.matches(AGENTS_SECTION_END).count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn init_rejects_symlinked_agents_file_without_modifying_target() {
        let temp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let outside_agents = outside.path().join("AGENTS.md");
        std::fs::write(&outside_agents, "outside original\n").unwrap();
        std::os::unix::fs::symlink(&outside_agents, temp.path().join("AGENTS.md")).unwrap();

        let err = run_init(temp.path(), &InitOptions::default()).unwrap_err();

        assert!(matches!(err, InitError::UnsafePath { .. }));
        assert_eq!(
            std::fs::read_to_string(outside_agents).unwrap(),
            "outside original\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn init_rejects_symlinked_ctxhelm_dir_without_modifying_target() {
        let temp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let outside_config = outside.path().join("ctxhelm.toml");
        std::fs::write(&outside_config, "outside original\n").unwrap();
        std::os::unix::fs::symlink(outside.path(), temp.path().join(".ctxhelm")).unwrap();

        let err = run_init(temp.path(), &InitOptions::default()).unwrap_err();

        assert!(matches!(err, InitError::UnsafePath { .. }));
        assert_eq!(
            std::fs::read_to_string(outside_config).unwrap(),
            "outside original\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn init_rejects_symlinked_cursor_dir_without_modifying_target() {
        let temp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let outside_rules = outside.path().join("rules");
        std::fs::create_dir_all(&outside_rules).unwrap();
        let outside_rule = outside_rules.join("ctxhelm.mdc");
        std::fs::write(&outside_rule, "outside original\n").unwrap();
        std::os::unix::fs::symlink(outside.path(), temp.path().join(".cursor")).unwrap();

        let err = run_init(
            temp.path(),
            &InitOptions {
                adapters: vec![AgentAdapter::Cursor],
            },
        )
        .unwrap_err();

        assert!(matches!(err, InitError::UnsafePath { .. }));
        assert_eq!(
            std::fs::read_to_string(outside_rule).unwrap(),
            "outside original\n"
        );
    }

    #[test]
    fn init_is_idempotent() {
        let temp = tempfile::tempdir().unwrap();
        let options = InitOptions {
            adapters: vec![AgentAdapter::Cursor, AgentAdapter::OpenCode],
        };
        run_init(temp.path(), &options).unwrap();
        let second = run_init(temp.path(), &options).unwrap();

        assert_file_action(&second, ".ctxhelm/ctxhelm.toml", InitAction::Unchanged);
        assert_file_action(&second, "AGENTS.md", InitAction::Unchanged);
        assert_file_action(&second, ".cursor/rules/ctxhelm.mdc", InitAction::Unchanged);
        assert_file_action(
            &second,
            ".ctxhelm/adapters/opencode.jsonc.snippet",
            InitAction::Unchanged,
        );
        assert_file_action(
            &second,
            ".claude/commands/ctxhelm-bugfix.md",
            InitAction::Skipped,
        );
        assert_file_action(
            &second,
            ".ctxhelm/adapters/claude-mcp.json",
            InitAction::Skipped,
        );
    }

    #[test]
    fn generated_config_reflects_requested_adapters() {
        let temp = tempfile::tempdir().unwrap();
        let options = InitOptions {
            adapters: vec![AgentAdapter::Cursor, AgentAdapter::OpenCode],
        };

        run_init(temp.path(), &options).unwrap();
        let config = std::fs::read_to_string(temp.path().join(".ctxhelm/ctxhelm.toml")).unwrap();

        assert!(config.contains("agents_md = true"));
        assert!(config.contains("cursor_rules = true"));
        assert!(config.contains("claude_commands = false"));
        assert!(config.contains("claude_mcp_snippet = false"));
        assert!(config.contains("opencode_snippet = true"));
    }

    #[test]
    fn init_report_serializes_with_camel_case_keys() {
        let temp = tempfile::tempdir().unwrap();
        let report = run_init(temp.path(), &InitOptions::default()).unwrap();
        let serialized = serde_json::to_value(&report).unwrap();

        assert!(serialized.get("repoRoot").is_some());
        assert!(serialized.get("codexMcpSetup").is_some());
        assert!(serialized.get("nextSteps").is_some());
        assert!(serialized.get("repo_root").is_none());
        assert!(serialized.get("codex_mcp_setup").is_none());
    }

    #[test]
    fn init_report_records_skipped_unrequested_adapters() {
        let temp = tempfile::tempdir().unwrap();
        let report = run_init(temp.path(), &InitOptions::default()).unwrap();

        assert_file_action(&report, ".ctxhelm/ctxhelm.toml", InitAction::Created);
        assert_file_action(&report, "AGENTS.md", InitAction::Created);
        assert_file_action(&report, ".cursor/rules/ctxhelm.mdc", InitAction::Skipped);
        assert_file_action(
            &report,
            ".claude/commands/ctxhelm-bugfix.md",
            InitAction::Skipped,
        );
        assert_file_action(
            &report,
            ".ctxhelm/adapters/claude-mcp.json",
            InitAction::Skipped,
        );
        assert_file_action(
            &report,
            ".ctxhelm/adapters/opencode.jsonc.snippet",
            InitAction::Skipped,
        );

        let second = run_init(temp.path(), &InitOptions::default()).unwrap();
        assert_file_action(&second, ".ctxhelm/ctxhelm.toml", InitAction::Unchanged);
        assert_file_action(&second, "AGENTS.md", InitAction::Unchanged);
        assert_file_action(&second, ".cursor/rules/ctxhelm.mdc", InitAction::Skipped);
    }

    #[test]
    fn init_report_records_requested_adapters_as_written() {
        let temp = tempfile::tempdir().unwrap();
        let options = InitOptions {
            adapters: vec![
                AgentAdapter::Cursor,
                AgentAdapter::Claude,
                AgentAdapter::OpenCode,
            ],
        };

        let report = run_init(temp.path(), &options).unwrap();

        assert_file_action(&report, ".cursor/rules/ctxhelm.mdc", InitAction::Created);
        assert_file_action(
            &report,
            ".claude/commands/ctxhelm-bugfix.md",
            InitAction::Created,
        );
        assert_file_action(
            &report,
            ".ctxhelm/adapters/claude-mcp.json",
            InitAction::Created,
        );
        assert_file_action(
            &report,
            ".ctxhelm/adapters/opencode.jsonc.snippet",
            InitAction::Created,
        );
        assert!(!report
            .files
            .iter()
            .any(|file| file.action == InitAction::Skipped));
    }

    #[test]
    fn init_report_next_steps_name_first_pack_ladder() {
        let temp = tempfile::tempdir().unwrap();
        let report = run_init(temp.path(), &InitOptions::default()).unwrap();
        let serialized = serde_json::to_value(&report).unwrap();
        let steps = serialized["nextSteps"].as_array().unwrap();
        let rendered = steps
            .iter()
            .map(|step| step.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("ctxhelm --version"));
        assert!(rendered.contains("ctxhelm --help"));
        assert!(rendered.contains("ctxhelm setup-check --repo"));
        assert!(rendered.contains("prepare_task"));
        assert!(rendered.contains("get_pack"));
    }

    fn assert_file_action(report: &InitReport, relative_path: &str, action: InitAction) {
        let file = report
            .files
            .iter()
            .find(|file| file.path.ends_with(relative_path))
            .unwrap_or_else(|| panic!("missing init report entry for {relative_path}"));
        assert_eq!(file.action, action, "{relative_path}");
    }
}

#[cfg(test)]
mod setup_check_tests {
    use super::*;

    fn all_adapters() -> InitOptions {
        InitOptions {
            adapters: vec![
                AgentAdapter::Cursor,
                AgentAdapter::Claude,
                AgentAdapter::OpenCode,
            ],
        }
    }

    #[test]
    fn setup_check_passes_generated_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        let options = all_adapters();
        run_init(temp.path(), &options).unwrap();

        let report = run_setup_check(temp.path(), &options).unwrap();

        assert!(report.passed);
        assert_eq!(report.schema_version, "ctxhelm-setup-check-report-v1");
        assert_item_status(&report, "AGENTS.md", SetupCheckStatus::Pass);
        assert_item_status(&report, ".ctxhelm/ctxhelm.toml", SetupCheckStatus::Pass);
        assert_item_status(&report, ".cursor/rules/ctxhelm.mdc", SetupCheckStatus::Pass);
        assert_item_status(
            &report,
            ".claude/commands/ctxhelm-bugfix.md",
            SetupCheckStatus::Pass,
        );
        assert_item_status(
            &report,
            ".ctxhelm/adapters/claude-mcp.json",
            SetupCheckStatus::Pass,
        );
        assert_item_status(
            &report,
            ".ctxhelm/adapters/opencode.jsonc.snippet",
            SetupCheckStatus::Pass,
        );
        assert_item_status(&report, ".mcp.json", SetupCheckStatus::Warn);
    }

    #[test]
    fn setup_check_report_serializes_schema_version_and_defaults_old_json() {
        let report = SetupCheckReport {
            schema_version: setup_check_report_schema_version(),
            repo_root: PathBuf::from("/repo"),
            items: Vec::new(),
            passed: true,
        };
        let value = serde_json::to_value(&report).unwrap();
        assert_eq!(value["schemaVersion"], "ctxhelm-setup-check-report-v1");
        assert!(value.get("schema_version").is_none());

        let old_json = serde_json::json!({
            "repoRoot": "/repo",
            "items": [],
            "passed": true
        });
        let old_report: SetupCheckReport = serde_json::from_value(old_json).unwrap();
        assert_eq!(old_report.schema_version, "ctxhelm-setup-check-report-v1");
    }

    #[test]
    fn setup_check_fails_missing_expected_file() {
        let temp = tempfile::tempdir().unwrap();
        let options = InitOptions {
            adapters: vec![AgentAdapter::Cursor],
        };
        run_init(temp.path(), &options).unwrap();
        std::fs::remove_file(temp.path().join(".cursor/rules/ctxhelm.mdc")).unwrap();

        let report = run_setup_check(temp.path(), &options).unwrap();

        assert!(!report.passed);
        assert_item_status(&report, ".cursor/rules/ctxhelm.mdc", SetupCheckStatus::Fail);
    }

    #[test]
    fn setup_check_validates_adapter_json() {
        let temp = tempfile::tempdir().unwrap();
        let options = all_adapters();
        run_init(temp.path(), &options).unwrap();
        std::fs::write(
            temp.path().join(".ctxhelm/adapters/claude-mcp.json"),
            "{not-json",
        )
        .unwrap();

        let report = run_setup_check(temp.path(), &options).unwrap();

        assert!(!report.passed);
        assert_item_status(
            &report,
            ".ctxhelm/adapters/claude-mcp.json",
            SetupCheckStatus::Fail,
        );
    }

    #[test]
    fn setup_check_flags_large_static_context_guidance() {
        let temp = tempfile::tempdir().unwrap();
        let options = InitOptions {
            adapters: vec![AgentAdapter::Cursor],
        };
        run_init(temp.path(), &options).unwrap();
        std::fs::write(
            temp.path().join(".cursor/rules/ctxhelm.mdc"),
            "Repository map:\nsource snippets\n",
        )
        .unwrap();

        let report = run_setup_check(temp.path(), &options).unwrap();

        assert!(!report.passed);
        assert_item_status(&report, ".cursor/rules/ctxhelm.mdc", SetupCheckStatus::Fail);
    }

    #[test]
    fn setup_check_includes_absolute_binary_path_guidance() {
        let temp = tempfile::tempdir().unwrap();
        run_init(temp.path(), &InitOptions::default()).unwrap();

        let report = run_setup_check(temp.path(), &InitOptions::default()).unwrap();
        let rendered = report
            .items
            .iter()
            .map(|item| format!("{} {}", item.name, item.detail))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("ctxhelm --version"));
        assert!(rendered.contains("which ctxhelm"));
        assert!(rendered.contains("absolute path"));
    }

    #[test]
    fn setup_check_passes_project_mcp_config_with_absolute_ctxhelm_command() {
        let temp = tempfile::tempdir().unwrap();
        let options = InitOptions {
            adapters: vec![AgentAdapter::Claude],
        };
        run_init(temp.path(), &options).unwrap();
        std::fs::write(
            temp.path().join(".mcp.json"),
            serde_json::json!({
                "mcpServers": {
                    "ctxhelm": {
                        "command": "/usr/local/bin/ctxhelm",
                        "args": ["serve-mcp"]
                    },
                    "other": {
                        "command": "other-tool"
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        let report = run_setup_check(temp.path(), &options).unwrap();

        assert!(report.passed);
        assert_item_status(&report, ".mcp.json", SetupCheckStatus::Pass);
    }

    #[test]
    fn setup_check_rejects_project_mcp_config_with_relative_ctxhelm_command() {
        let temp = tempfile::tempdir().unwrap();
        let options = InitOptions {
            adapters: vec![AgentAdapter::Claude],
        };
        run_init(temp.path(), &options).unwrap();
        std::fs::write(
            temp.path().join(".mcp.json"),
            serde_json::json!({
                "mcpServers": {
                    "ctxhelm": {
                        "command": "ctxhelm",
                        "args": ["serve-mcp"]
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        let report = run_setup_check(temp.path(), &options).unwrap();

        assert!(!report.passed);
        assert_item_status(&report, ".mcp.json", SetupCheckStatus::Fail);
        assert!(setup_item_detail(&report, ".mcp.json").contains("absolute path"));
    }

    fn assert_item_status(report: &SetupCheckReport, name: &str, status: SetupCheckStatus) {
        let item = report
            .items
            .iter()
            .find(|item| item.name.contains(name))
            .unwrap_or_else(|| panic!("missing setup-check item for {name}"));
        assert_eq!(item.status, status, "{name}: {}", item.detail);
    }

    fn setup_item_detail<'a>(report: &'a SetupCheckReport, name: &str) -> &'a str {
        report
            .items
            .iter()
            .find(|item| item.name.contains(name))
            .map(|item| item.detail.as_str())
            .unwrap_or_else(|| panic!("missing setup-check item for {name}"))
    }
}
