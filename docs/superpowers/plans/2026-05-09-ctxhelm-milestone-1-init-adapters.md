# ctxhelm Milestone 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `ctxhelm init` useful in a real repository without indexing: detect the repo root, create `.ctxhelm/ctxhelm.toml`, safely add a bounded `ctxhelm` section to `AGENTS.md`, optionally generate thin native adapter files, and print Codex MCP setup guidance without mutating global config.

**Architecture:** Keep init behavior in `ctxhelm-core` so the CLI remains thin and future MCP/adapter surfaces can reuse the same writer. The CLI parses flags, discovers the repo root, delegates to core, and prints a concise report. All generated content is deterministic and local to the target repo.

**Tech Stack:** Rust 2021, `clap` for CLI flags, `thiserror` for core errors, `tempfile` for tests, standard library filesystem APIs. No TOML parser is needed yet because Milestone 1 only writes a fixed config file.

---

## Scope

Milestone 1 implements only repo-local initialization.

In scope:

- Detect the nearest ancestor containing `.git`.
- Create `.ctxhelm/ctxhelm.toml`.
- Add or replace a bounded `ctxhelm` section in `AGENTS.md`.
- Generate adapter files only when requested with CLI flags:
  - `--cursor`: `.cursor/rules/ctxhelm.mdc`
  - `--claude`: `.claude/commands/ctxhelm-bugfix.md`
  - `--opencode`: `.ctxhelm/adapters/opencode.jsonc.snippet`
- Always print Codex MCP setup guidance to stdout.

Out of scope:

- Mutating `~/.codex/config.toml`.
- Parsing or merging existing TOML/JSONC config files.
- Running indexing, MCP runtime, tests, shell commands, or editor integrations.
- Cloud, embeddings, reranking, remote PR data, or global state.

## Files

```text
Cargo.toml
crates/ctxhelm-core/Cargo.toml
crates/ctxhelm-core/src/lib.rs
crates/ctxhelm-core/src/repo.rs
crates/ctxhelm-core/src/init.rs
crates/ctxhelm/src/main.rs
README.md
```

## Task 1: Add Init Data Model and Templates

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/ctxhelm-core/Cargo.toml`
- Modify: `crates/ctxhelm-core/src/lib.rs`
- Create: `crates/ctxhelm-core/src/init.rs`

- [ ] **Step 1: Add test/dependency support**

Modify root `Cargo.toml` to add `tempfile` as a workspace dependency while preserving existing dependencies:

```toml
[workspace.dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tempfile = "3"
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
uuid = { version = "1", features = ["serde"] }
```

Modify `crates/ctxhelm-core/Cargo.toml`:

```toml
[dependencies]
serde.workspace = true
uuid.workspace = true

[dev-dependencies]
serde_json.workspace = true
tempfile.workspace = true
```

- [ ] **Step 2: Export the init module**

Modify `crates/ctxhelm-core/src/lib.rs`:

```rust
pub mod contracts;
pub mod init;
pub mod privacy;
pub mod repo;

pub use contracts::*;
pub use init::{AgentAdapter, InitAction, InitOptions, InitReport};
pub use privacy::PrivacyStatus;
pub use repo::{FileRole, RepoRoot};
```

- [ ] **Step 3: Create init templates and report types**

Create `crates/ctxhelm-core/src/init.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum AgentAdapter {
    Cursor,
    Claude,
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

pub const AGENTS_SECTION_START: &str = "<!-- ctxhelm:start -->";
pub const AGENTS_SECTION_END: &str = "<!-- ctxhelm:end -->";

pub const CTXHELM_TOML: &str = r#"version = 1
local_only = true

[adapters]
agents_md = true
cursor_rules = false
claude_commands = false
opencode_snippet = false
"#;

pub const AGENTS_SECTION: &str = r#"<!-- ctxhelm:start -->
## ctxhelm

For non-trivial code changes, call `ctxhelm prepare-task` or the ctxhelm MCP tool before planning edits.

Use ctxhelm for:
- likely target files
- related tests
- relevant examples
- architecture constraints
- validation commands

Read actual files with the agent's native tools before editing. Do not request deep packs unless brief or standard context is insufficient.
<!-- ctxhelm:end -->
"#;

pub const CURSOR_RULE: &str = r#"---
description: Use ctxhelm to gather precise repository context before non-trivial edits
alwaysApply: true
---

For tasks that modify code, investigate bugs, add features, or affect multiple files:

1. Call the ctxhelm MCP tool `prepare_task` when available, or run `ctxhelm prepare-task`.
2. Prefer returned target files, related tests, examples, and constraints.
3. Read actual files using Cursor's native file tools before editing.
4. Do not load broad repository context unless ctxhelm recommends it.
5. Run targeted validation commands returned by ctxhelm when available.
"#;

pub const CLAUDE_BUGFIX_COMMAND: &str = r#"# ctxhelm Bugfix

Use this command for non-trivial bug fixes.

1. Call `ctxhelm.prepare_task` for the user's task when MCP is available.
2. Read the returned target files with native tools.
3. Request a standard pack only if direct file reads are insufficient.
4. Make the smallest patch that addresses the bug.
5. Run the related test command returned by ctxhelm when available.
6. Summarize the changed behavior and validation result.
"#;

pub const OPENCODE_SNIPPET: &str = r#"{
  "$schema": "https://opencode.ai/config.json",
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

Add a local stdio MCP server for ctxhelm in your Codex MCP configuration:

  command: ctxhelm serve-mcp

This command does not mutate global Codex config automatically.
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

pub fn agents_section() -> &'static str {
    AGENTS_SECTION
}
```

- [ ] **Step 4: Add template tests**

Append tests to `init.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_paths_are_repo_local() {
        assert_eq!(adapter_path(AgentAdapter::Cursor), ".cursor/rules/ctxhelm.mdc");
        assert_eq!(
            adapter_path(AgentAdapter::Claude),
            ".claude/commands/ctxhelm-bugfix.md"
        );
        assert_eq!(
            adapter_path(AgentAdapter::OpenCode),
            ".ctxhelm/adapters/opencode.jsonc.snippet"
        );
    }

    #[test]
    fn codex_setup_is_guidance_only() {
        assert!(CODEX_MCP_SETUP.contains("ctxhelm serve-mcp"));
        assert!(CODEX_MCP_SETUP.contains("does not mutate global Codex config"));
    }
}
```

- [ ] **Step 5: Verify and commit**

Run:

```bash
cargo fmt --all --check
cargo test -p ctxhelm-core --locked
```

Expected: PASS. If `--locked` fails because the lockfile needs the new `tempfile` dependency, run `cargo generate-lockfile`, then rerun the commands.

Commit:

```bash
git add Cargo.toml Cargo.lock crates/ctxhelm-core
git commit -m "feat: add init templates"
```

## Task 2: Implement Repo Root Discovery and Init Writer

**Files:**
- Modify: `crates/ctxhelm-core/Cargo.toml`
- Modify: `crates/ctxhelm-core/src/repo.rs`
- Modify: `crates/ctxhelm-core/src/init.rs`

- [ ] **Step 1: Add error dependency for init/repo failures**

Modify `crates/ctxhelm-core/Cargo.toml`:

```toml
[dependencies]
serde.workspace = true
thiserror.workspace = true
uuid.workspace = true

[dev-dependencies]
serde_json.workspace = true
tempfile.workspace = true
```

- [ ] **Step 2: Export `run_init` now that it exists**

Modify `crates/ctxhelm-core/src/lib.rs`:

```rust
pub mod contracts;
pub mod init;
pub mod privacy;
pub mod repo;

pub use contracts::*;
pub use init::{run_init, AgentAdapter, InitAction, InitOptions, InitReport};
pub use privacy::PrivacyStatus;
pub use repo::{FileRole, RepoRoot};
```

- [ ] **Step 3: Implement repo root discovery**

Replace `crates/ctxhelm-core/src/repo.rs` with:

```rust
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileRole {
    Source,
    Test,
    Config,
    Schema,
    Docs,
    Generated,
    Sensitive,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoRoot {
    pub path: PathBuf,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RepoRootError {
    #[error("no git repository found from {start}")]
    NotFound { start: PathBuf },
}

impl RepoRoot {
    pub fn discover_from(start: impl AsRef<Path>) -> Result<Self, RepoRootError> {
        let start = start.as_ref();
        for candidate in start.ancestors() {
            if candidate.join(".git").exists() {
                return Ok(Self {
                    path: candidate.to_path_buf(),
                });
            }
        }
        Err(RepoRootError::NotFound {
            start: start.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_nearest_git_ancestor() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let nested = repo.join("a/b/c");
        std::fs::create_dir_all(repo.join(".git")).unwrap();
        std::fs::create_dir_all(&nested).unwrap();

        let found = RepoRoot::discover_from(&nested).unwrap();
        assert_eq!(found.path, repo);
    }

    #[test]
    fn returns_not_found_when_no_git_ancestor_exists() {
        let temp = tempfile::tempdir().unwrap();
        let err = RepoRoot::discover_from(temp.path()).unwrap_err();
        assert!(matches!(err, RepoRootError::NotFound { .. }));
    }
}
```

- [ ] **Step 4: Implement init file writing**

Append to `crates/ctxhelm-core/src/init.rs`:

```rust
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("failed to read {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to write {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
    #[error("failed to create directory {path}: {source}")]
    CreateDir { path: PathBuf, source: io::Error },
}

pub fn run_init(repo_root: impl AsRef<Path>, options: &InitOptions) -> Result<InitReport, InitError> {
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
        write_file(
            repo_root,
            adapter_path(*adapter),
            adapter_content(*adapter).to_string(),
            &mut report,
        )?;
    }

    Ok(report)
}

fn config_toml(options: &InitOptions) -> String {
    let cursor = options.adapters.contains(&AgentAdapter::Cursor);
    let claude = options.adapters.contains(&AgentAdapter::Claude);
    let opencode = options.adapters.contains(&AgentAdapter::OpenCode);
    format!(
        "version = 1\nlocal_only = true\n\n[adapters]\nagents_md = true\ncursor_rules = {cursor}\nclaude_commands = {claude}\nopencode_snippet = {opencode}\n"
    )
}

fn write_file(
    repo_root: &Path,
    relative_path: &str,
    content: String,
    report: &mut InitReport,
) -> Result<(), InitError> {
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
            })
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
    let path = repo_root.join("AGENTS.md");
    let existed = path.exists();
    let existing = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(source) => {
            return Err(InitError::Read {
                path: path.clone(),
                source,
            })
        }
    };

    let next = if let (Some(start), Some(end)) = (
        existing.find(AGENTS_SECTION_START),
        existing.find(AGENTS_SECTION_END),
    ) {
        let section_end = end + AGENTS_SECTION_END.len();
        format!(
            "{}{}{}",
            &existing[..start],
            AGENTS_SECTION.trim_end(),
            &existing[section_end..]
        )
    } else if existing.trim().is_empty() {
        format!("# AGENTS.md\n\n{}\n", AGENTS_SECTION.trim_end())
    } else {
        format!("{}\n\n{}\n", existing.trim_end(), AGENTS_SECTION.trim_end())
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
```

- [ ] **Step 5: Add init writer tests**

Append tests to `init.rs`:

```rust
#[cfg(test)]
mod writer_tests {
    use super::*;

    #[test]
    fn init_creates_config_agents_and_requested_adapters() {
        let temp = tempfile::tempdir().unwrap();
        let options = InitOptions {
            adapters: vec![AgentAdapter::Cursor, AgentAdapter::Claude, AgentAdapter::OpenCode],
        };

        let report = run_init(temp.path(), &options).unwrap();

        assert!(temp.path().join(".ctxhelm/ctxhelm.toml").exists());
        assert!(temp.path().join("AGENTS.md").exists());
        assert!(temp.path().join(".cursor/rules/ctxhelm.mdc").exists());
        assert!(temp.path().join(".claude/commands/ctxhelm-bugfix.md").exists());
        assert!(temp
            .path()
            .join(".ctxhelm/adapters/opencode.jsonc.snippet")
            .exists());
        assert!(report.codex_mcp_setup.contains("ctxhelm serve-mcp"));
    }

    #[test]
    fn init_replaces_existing_bounded_agents_section() {
        let temp = tempfile::tempdir().unwrap();
        let agents = temp.path().join("AGENTS.md");
        std::fs::write(
            &agents,
            format!(
                "# Existing\n\n{}\nold\n{}\n",
                AGENTS_SECTION_START, AGENTS_SECTION_END
            ),
        )
        .unwrap();

        run_init(temp.path(), &InitOptions::default()).unwrap();
        let content = std::fs::read_to_string(agents).unwrap();

        assert!(content.contains("# Existing"));
        assert!(content.contains("For non-trivial code changes"));
        assert!(!content.contains("\nold\n"));
    }

    #[test]
    fn init_is_idempotent() {
        let temp = tempfile::tempdir().unwrap();
        let options = InitOptions::default();
        run_init(temp.path(), &options).unwrap();
        let second = run_init(temp.path(), &options).unwrap();

        assert!(second
            .files
            .iter()
            .all(|file| file.action == InitAction::Unchanged));
    }
}
```

- [ ] **Step 6: Verify and commit**

Run:

```bash
cargo fmt --all --check
cargo test -p ctxhelm-core --locked
cargo clippy -p ctxhelm-core --all-targets -- -D warnings
```

Expected: PASS.

Commit:

```bash
git add crates/ctxhelm-core
git commit -m "feat: implement repo init writer"
```

## Task 3: Wire `ctxhelm init` CLI

**Files:**
- Modify: `crates/ctxhelm/src/main.rs`
- Modify: `README.md`

- [ ] **Step 1: Add CLI args and init execution**

Update `crates/ctxhelm/src/main.rs` to use `clap::Args`, `PathBuf`, `RepoRoot`, and init APIs. The final file should follow this shape:

```rust
use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use ctxhelm_compiler::empty_plan_for_task;
use ctxhelm_core::{run_init, AgentAdapter, InitAction, InitOptions, InitReport, RepoRoot, TaskType};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "ctxhelm")]
#[command(about = "Agent-native context packs for coding agents")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init(InitArgs),
    Index,
    PrepareTask {
        task: String,
        #[arg(long, value_enum, default_value_t = Mode::Explain)]
        mode: Mode,
    },
    Search {
        query: String,
    },
    ServeMcp,
}

#[derive(Debug, Args)]
struct InitArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long)]
    cursor: bool,
    #[arg(long)]
    claude: bool,
    #[arg(long)]
    opencode: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    BugFix,
    Feature,
    Refactor,
    Review,
    Test,
    Explain,
}

impl From<Mode> for TaskType {
    fn from(value: Mode) -> Self {
        match value {
            Mode::BugFix => TaskType::BugFix,
            Mode::Feature => TaskType::Feature,
            Mode::Refactor => TaskType::Refactor,
            Mode::Review => TaskType::Review,
            Mode::Test => TaskType::Test,
            Mode::Explain => TaskType::Explain,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init(args) => {
            let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            let report = run_init(&repo.path, &init_options(&args))?;
            print_init_report(&report);
        }
        Command::Index => {
            println!("Milestone 0 stub: ctxhelm index has no side effects");
        }
        Command::PrepareTask { task: _, mode } => {
            let plan = empty_plan_for_task(mode.into());
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        Command::Search { query } => {
            println!("Milestone 0 stub: ctxhelm search has no index yet: {query}");
        }
        Command::ServeMcp => {
            println!("Milestone 0 stub: ctxhelm serve-mcp does not start a server yet");
        }
    }
    Ok(())
}

fn init_options(args: &InitArgs) -> InitOptions {
    let mut adapters = Vec::new();
    if args.cursor {
        adapters.push(AgentAdapter::Cursor);
    }
    if args.claude {
        adapters.push(AgentAdapter::Claude);
    }
    if args.opencode {
        adapters.push(AgentAdapter::OpenCode);
    }
    InitOptions { adapters }
}

fn print_init_report(report: &InitReport) {
    println!("Initialized ctxhelm in {}", report.repo_root.display());
    for file in &report.files {
        let action = match file.action {
            InitAction::Created => "created",
            InitAction::Updated => "updated",
            InitAction::Unchanged => "unchanged",
        };
        println!("- {action}: {}", file.path.display());
    }
    println!();
    println!("{}", report.codex_mcp_setup);
}
```

- [ ] **Step 2: Update README with init examples**

Append to `README.md`:

````md
## Initialization

Initialize a repository with the portable AGENTS.md guidance and `.ctxhelm/ctxhelm.toml`:

```bash
cargo run -p ctxhelm -- init --repo /path/to/repo
```

Generate optional native adapter files:

```bash
cargo run -p ctxhelm -- init --repo /path/to/repo --cursor --claude --opencode
```

`ctxhelm init` writes only repo-local files. It prints Codex MCP setup guidance but does not mutate global Codex configuration.
````

- [ ] **Step 3: Verify CLI init in a temporary git repo**

Run:

```bash
tmp="$(mktemp -d)"
git -C "$tmp" init
cargo run -p ctxhelm -- init --repo "$tmp" --cursor --claude --opencode
test -f "$tmp/.ctxhelm/ctxhelm.toml"
test -f "$tmp/AGENTS.md"
test -f "$tmp/.cursor/rules/ctxhelm.mdc"
test -f "$tmp/.claude/commands/ctxhelm-bugfix.md"
test -f "$tmp/.ctxhelm/adapters/opencode.jsonc.snippet"
```

Expected: all commands succeed and stdout includes `Codex MCP setup`.

- [ ] **Step 4: Verify and commit**

Run:

```bash
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

Expected: PASS.

Commit:

```bash
git add crates/ctxhelm/src/main.rs README.md
git commit -m "feat: wire ctxhelm init cli"
```

## Task 4: Milestone 1 Acceptance Pass

**Files:**
- No required file edits unless review finds a defect.

- [ ] **Step 1: Verify default init writes only baseline files**

Run:

```bash
tmp="$(mktemp -d)"
git -C "$tmp" init
cargo run -p ctxhelm -- init --repo "$tmp"
find "$tmp" -path "$tmp/.git" -prune -o -type f -print | sort
```

Expected files:

```text
$tmp/.ctxhelm/ctxhelm.toml
$tmp/AGENTS.md
```

No `.cursor`, `.claude`, or OpenCode snippet should exist unless flags are passed.

- [ ] **Step 2: Verify optional adapter generation**

Run:

```bash
tmp="$(mktemp -d)"
git -C "$tmp" init
cargo run -p ctxhelm -- init --repo "$tmp" --cursor --claude --opencode
test -f "$tmp/.cursor/rules/ctxhelm.mdc"
test -f "$tmp/.claude/commands/ctxhelm-bugfix.md"
test -f "$tmp/.ctxhelm/adapters/opencode.jsonc.snippet"
```

Expected: all tests pass.

- [ ] **Step 3: Verify idempotency**

Run:

```bash
tmp="$(mktemp -d)"
git -C "$tmp" init
cargo run -p ctxhelm -- init --repo "$tmp" --cursor
cargo run -p ctxhelm -- init --repo "$tmp" --cursor
```

Expected: second run reports unchanged for `.ctxhelm/ctxhelm.toml`, `AGENTS.md`, and `.cursor/rules/ctxhelm.mdc`.

- [ ] **Step 4: Verify global Codex config is untouched**

Run:

```bash
before="$(shasum "$HOME/.codex/config.toml" 2>/dev/null || true)"
tmp="$(mktemp -d)"
git -C "$tmp" init
cargo run -p ctxhelm -- init --repo "$tmp"
after="$(shasum "$HOME/.codex/config.toml" 2>/dev/null || true)"
test "$before" = "$after"
```

Expected: PASS.

- [ ] **Step 5: Run full validation**

Run:

```bash
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets -- -D warnings
git status --short
```

Expected: PASS and clean worktree.

If this task required no edits, do not create an empty commit. If it required fixes, commit them with:

```bash
git add <fixed-files>
git commit -m "fix: complete ctxhelm init acceptance"
```

## Post-Milestone Review

- [ ] `ctxhelm init --repo <repo>` detects a git root.
- [ ] Default init writes only `.ctxhelm/ctxhelm.toml` and `AGENTS.md`.
- [ ] Adapter files are generated only with requested flags.
- [ ] Existing bounded `AGENTS.md` section is replaced, not duplicated.
- [ ] Codex guidance is printed and no global config is mutated.
- [ ] Workspace tests and clippy pass with `--locked`.

## Execution Notes

Use an isolated branch/worktree for implementation. If using subagent-driven development, run Task 1 through Task 4 sequentially with spec compliance and code quality reviews after each implementation task.
