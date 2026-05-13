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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InitAction {
    Created,
    Updated,
    Unchanged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InitFile {
    pub path: PathBuf,
    pub action: InitAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InitReport {
    pub repo_root: PathBuf,
    pub files: Vec<InitFile>,
    pub codex_mcp_setup: String,
}

impl InitReport {
    pub fn new(repo_root: PathBuf) -> Self {
        Self {
            repo_root,
            files: Vec::new(),
            codex_mcp_setup: CODEX_MCP_SETUP.trim().to_string(),
        }
    }
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

pub const AGENTS_SECTION_START: &str = "<!-- ctxpack:start -->";
pub const AGENTS_SECTION_END: &str = "<!-- ctxpack:end -->";

pub const CTXPACK_TOML: &str = r#"version = 1
local_only = true

[adapters]
agents_md = true
cursor_rules = false
claude_commands = false
claude_mcp_snippet = false
opencode_snippet = false
"#;

pub const AGENTS_SECTION: &str = r#"<!-- ctxpack:start -->
## ctxpack

For non-trivial code changes, call the ctxpack MCP tool `prepare_task` before planning edits when MCP is configured. Pass the active repository path as `repo` when the agent knows it. Otherwise, run `ctxpack prepare-task`.

Use ctxpack for:
- likely target files
- related tests
- relevant examples
- architecture constraints
- validation commands

Read actual files with the agent's native tools before editing. Request `get_pack` progressively, starting brief or standard, only when direct file reads or the plan are insufficient.
<!-- ctxpack:end -->
"#;

pub const CURSOR_RULE: &str = r#"---
description: Use ctxpack to gather precise repository context before non-trivial edits
alwaysApply: true
---

For tasks that modify code, investigate bugs, add features, or affect multiple files:

1. Call the ctxpack MCP tool `prepare_task` when available, passing the active repository path as `repo` when known, or run `ctxpack prepare-task`.
2. Prefer returned target files, related tests, examples, and constraints.
3. Read actual files using Cursor's native file tools before editing.
4. Call `get_pack` progressively only when direct file reads or brief context are insufficient.
5. Run targeted validation commands returned by ctxpack when available.
"#;

pub const CLAUDE_BUGFIX_COMMAND: &str = r#"# ctxpack Bugfix

Use this command for non-trivial bug fixes.

1. Call `ctxpack.prepare_task` for the user's task when MCP is available, passing the active repository path as `repo` when known.
2. Read the returned target files with native tools.
3. Request `get_pack` progressively, starting brief or standard, only if direct file reads are insufficient.
4. Make the smallest patch that addresses the bug.
5. Run the related test command returned by ctxpack when available.
6. Summarize the changed behavior and validation result.
"#;

pub const CLAUDE_MCP_SNIPPET: &str = r#"{
  "mcpServers": {
    "ctxpack": {
      "command": "ctxpack",
      "args": ["serve-mcp"]
    }
  }
}
"#;

pub const OPENCODE_SNIPPET: &str = r#"{
  "$schema": "https://opencode.ai/config.json",
  "ctxpackNote": "Registers ctxpack as a local read-only MCP context server. Call prepare_task with the active repository path as `repo`, read files natively, and use get_pack progressively only when needed.",
  "instructions": ["AGENTS.md"],
  "mcp": {
    "ctxpack": {
      "type": "local",
      "command": ["ctxpack", "serve-mcp"]
    }
  }
}
"#;

pub const CODEX_MCP_SETUP: &str = r#"
Codex MCP setup:

Add a local stdio MCP server for ctxpack in your Codex MCP configuration:

  command: ctxpack serve-mcp

Call `prepare_task` first and pass the active repository path as `repo` when known. Read actual files with Codex tools, then call `get_pack` progressively only when direct file reads or brief context are insufficient. This command does not mutate global Codex config automatically.
"#;

pub fn adapter_path(adapter: AgentAdapter) -> &'static str {
    match adapter {
        AgentAdapter::Cursor => ".cursor/rules/ctxpack.mdc",
        AgentAdapter::Claude => ".claude/commands/ctxpack-bugfix.md",
        AgentAdapter::OpenCode => ".ctxpack/adapters/opencode.jsonc.snippet",
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
        files.push((".ctxpack/adapters/claude-mcp.json", CLAUDE_MCP_SNIPPET));
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
        ".ctxpack/ctxpack.toml",
        config_toml(options),
        &mut report,
    )?;

    upsert_agents_section(repo_root, &mut report)?;

    for adapter in &options.adapters {
        for (path, content) in adapter_files(*adapter) {
            write_file(repo_root, path, content.to_string(), &mut report)?;
        }
    }

    Ok(report)
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
    fn adapter_guidance_does_not_claim_prepare_task_is_the_only_tool() {
        assert!(!OPENCODE_SNIPPET.contains("The first implemented tool is prepare_task"));
        assert!(!CODEX_MCP_SETUP.contains("The first implemented tool is `prepare_task`"));
    }

    #[test]
    fn claude_mcp_snippet_stays_small_local_stdio_config() {
        let value = serde_json::from_str::<serde_json::Value>(CLAUDE_MCP_SNIPPET).unwrap();

        assert!(CLAUDE_MCP_SNIPPET.len() <= 300);
        assert_eq!(
            value["mcpServers"]["ctxpack"]["command"].as_str(),
            Some("ctxpack")
        );
        assert_eq!(
            value["mcpServers"]["ctxpack"]["args"][0].as_str(),
            Some("serve-mcp")
        );
        assert!(!CLAUDE_MCP_SNIPPET.contains("claude mcp add"));
        assert!(!CLAUDE_MCP_SNIPPET.contains("global"));
    }

    #[test]
    fn adapter_paths_are_repo_local() {
        assert_eq!(
            adapter_path(AgentAdapter::Cursor),
            ".cursor/rules/ctxpack.mdc"
        );
        assert_eq!(
            adapter_path(AgentAdapter::Claude),
            ".claude/commands/ctxpack-bugfix.md"
        );
        assert_eq!(
            adapter_path(AgentAdapter::OpenCode),
            ".ctxpack/adapters/opencode.jsonc.snippet"
        );
        assert!(adapter_files(AgentAdapter::Claude)
            .iter()
            .any(|(path, _)| *path == ".ctxpack/adapters/claude-mcp.json"));
    }

    #[test]
    fn codex_setup_is_guidance_only() {
        assert!(CODEX_MCP_SETUP.contains("local stdio MCP server"));
        assert!(CODEX_MCP_SETUP.contains("ctxpack serve-mcp"));
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
        assert!(OPENCODE_SNIPPET.contains("ctxpack"));
        assert!(OPENCODE_SNIPPET.contains("serve-mcp"));
    }

    #[test]
    fn agents_section_contains_ctxpack_markers() {
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
            value["mcpServers"]["ctxpack"]["command"].as_str(),
            Some("ctxpack")
        );
        assert_eq!(
            value["mcpServers"]["ctxpack"]["args"][0].as_str(),
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

        assert!(temp.path().join(".ctxpack/ctxpack.toml").exists());
        assert!(temp.path().join("AGENTS.md").exists());
        assert!(temp.path().join(".cursor/rules/ctxpack.mdc").exists());
        assert!(temp
            .path()
            .join(".claude/commands/ctxpack-bugfix.md")
            .exists());
        assert!(temp
            .path()
            .join(".ctxpack/adapters/claude-mcp.json")
            .exists());
        assert!(temp
            .path()
            .join(".ctxpack/adapters/opencode.jsonc.snippet")
            .exists());
        assert!(report
            .files
            .iter()
            .all(|file| file.action == InitAction::Created));
        assert!(report.codex_mcp_setup.contains("ctxpack serve-mcp"));
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
    fn init_rejects_symlinked_ctxpack_dir_without_modifying_target() {
        let temp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let outside_config = outside.path().join("ctxpack.toml");
        std::fs::write(&outside_config, "outside original\n").unwrap();
        std::os::unix::fs::symlink(outside.path(), temp.path().join(".ctxpack")).unwrap();

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
        let outside_rule = outside_rules.join("ctxpack.mdc");
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

        assert!(second
            .files
            .iter()
            .all(|file| file.action == InitAction::Unchanged));
    }

    #[test]
    fn generated_config_reflects_requested_adapters() {
        let temp = tempfile::tempdir().unwrap();
        let options = InitOptions {
            adapters: vec![AgentAdapter::Cursor, AgentAdapter::OpenCode],
        };

        run_init(temp.path(), &options).unwrap();
        let config = std::fs::read_to_string(temp.path().join(".ctxpack/ctxpack.toml")).unwrap();

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
        assert!(serialized.get("repo_root").is_none());
        assert!(serialized.get("codex_mcp_setup").is_none());
    }
}
