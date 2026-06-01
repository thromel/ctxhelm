# ctxhelm Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first agent-native MVP of ctxhelm: a local, read-only context broker that exposes compact task context through AGENTS.md, MCP, and thin agent-native adapters.

**Architecture:** Start with a Rust workspace split into `ctxhelm-core`, `ctxhelm-index`, `ctxhelm-compiler`, `ctxhelm-mcp`, and a `ctxhelm` CLI binary. Keep the default MVP lexical, graph-lite, and local-only; vector retrieval, cloud reranking, SCIP/LSP, and UI remain out of scope.

**Tech Stack:** Rust 2021, Cargo workspace, `clap` for CLI, `serde`/`serde_json` for contracts, `anyhow`/`thiserror` for errors, `tokio` for async MCP runtime, SQLite/Tantivy planned after scaffold, Tree-sitter planned after repo scanning skeleton.

---

## Roadmap Summary

### Milestone 0: Scaffold and Contracts

Purpose: Create the repo shape, compile-ready Rust workspace, shared data contracts, and CLI command skeleton.

Exit criteria:

- `cargo test --workspace` passes.
- `ctxhelm --help` shows the intended commands.
- Shared contracts compile and serialize.
- No indexing, MCP, or adapter generation behavior is implemented beyond stubs with typed outputs.

### Milestone 1: Repo Init and Agent Instruction Adapters

Purpose: Make `ctxhelm init` useful in a real repo without indexing yet.

Exit criteria:

- Detect git repo root.
- Create `.ctxhelm/ctxhelm.toml`.
- Create or patch `AGENTS.md` with a bounded `ctxhelm` section.
- Generate `.cursor/rules/ctxhelm.mdc`, `.claude/commands/ctxhelm-bugfix.md`, and an OpenCode config/instruction snippet only when requested.
- Provide Codex MCP setup instructions without mutating global `~/.codex/config.toml` by default.

### Milestone 2: Safe File Inventory

Purpose: Build trust by indexing only safe, relevant local files.

Exit criteria:

- Respect `.gitignore`, `.ctxhelmignore`, `.cursorignore`, and built-in deny rules.
- Exclude `.env`, keys, certs, dumps, dependencies, build output, generated/minified files.
- Classify file role as source, test, config, schema, docs, generated, sensitive, or unknown.
- Persist inventory to local private state under `~/.ctxhelm/repos/<repo-id>/`.

### Milestone 3: Lexical Index and Search

Purpose: Provide the first real retrieval path for exact identifiers, paths, errors, and docs.

Exit criteria:

- Build Tantivy or equivalent BM25 index over safe files.
- Search paths, symbols when available, docs, tests, and config.
- Return ranked `ContextCandidate` objects with source scores and evidence.
- Keep `ctxhelm search` and MCP `search` output small by default.

### Milestone 4: Test Mapper and Git Co-Change Hints

Purpose: Add the high-value validation context that most code search tools miss.

Exit criteria:

- Detect likely test files from path conventions and imports.
- Infer targeted test commands from package manager/config files.
- Index local git commits and commit-file changes.
- Provide co-change hints without reading remote PR data.

### Milestone 5: Context Compiler

Purpose: Turn retrieval results into task-conditioned context plans and packs.

Exit criteria:

- Implement task classifier for bug fix, feature, refactor, review, test, and explain.
- Implement candidate fusion from anchors, lexical, graph-lite, tests, docs, and history.
- Emit `ContextPlan` from `prepare_task`.
- Emit brief/standard/deep `ContextPack` in Markdown and JSON.
- Include evidence labels, warnings, token estimates, and privacy status.

### Milestone 6: MCP Runtime

Purpose: Make ctxhelm agent-native.

Exit criteria:

- Expose six MCP tools: `prepare_task`, `search`, `related`, `get_pack`, `related_tests`, `current_diff`.
- Expose pack/file/symbol/test-map resources.
- Expose workflow prompts for bugfix, feature, refactor, review diff, write tests, and explain area.
- Verify Codex local stdio MCP path first.

### Milestone 7: Local Evaluation Trace

Purpose: Record enough local metadata to improve retrieval later.

Exit criteria:

- Log task hash, task type, pack id, target agent, budget, recommended files/tests, and created time.
- Do not log source text by default.
- Add `ctxhelm eval traces` to inspect local traces.
- Add a manual dogfood checklist comparing recommended files with agent-read and agent-edited files.

## Proposed File Structure

```text
Cargo.toml
crates/
  ctxhelm/
    Cargo.toml
    src/main.rs
  ctxhelm-core/
    Cargo.toml
    src/lib.rs
    src/contracts.rs
    src/repo.rs
    src/privacy.rs
  ctxhelm-index/
    Cargo.toml
    src/lib.rs
  ctxhelm-compiler/
    Cargo.toml
    src/lib.rs
  ctxhelm-mcp/
    Cargo.toml
    src/lib.rs
docs/
  superpowers/
    specs/2026-05-09-repo-context-packer-product-spec.md
    plans/2026-05-09-repo-context-packer-implementation-roadmap.md
README.md
AGENTS.md
.gitignore
```

File responsibilities:

- `crates/ctxhelm`: user-facing CLI binary and command dispatch.
- `crates/ctxhelm-core`: shared contracts, repo root detection, privacy policy, and inventory primitives.
- `crates/ctxhelm-index`: search/index abstractions and later SQLite/Tantivy implementations.
- `crates/ctxhelm-compiler`: task classifier, candidate fusion, context plan, and pack rendering.
- `crates/ctxhelm-mcp`: MCP server runtime and mapping between MCP requests and compiler APIs.
- `AGENTS.md`: project instructions for future coding agents working on ctxhelm itself.
- `README.md`: human-facing project overview and local development commands.

## Milestone 0 Executable Plan

### Task 1: Create Repo Documentation and Ignore Rules

**Files:**
- Create: `.gitignore`
- Create: `README.md`
- Create: `AGENTS.md`

- [ ] **Step 1: Add git ignore rules**

Create `.gitignore`:

```gitignore
/target/
/.ctxhelm/cache/
/.ctxhelm/index/
*.log
.DS_Store
```

- [ ] **Step 2: Add human README**

Create `README.md`:

````md
# ctxhelm

ctxhelm is a local-first, read-only context broker for coding agents.

The MVP exposes compact task context through:

- `AGENTS.md` for portable static instructions
- MCP tools/resources/prompts for dynamic context
- Thin native adapter files for Codex, Claude Code, Cursor, and OpenCode

## Development

```bash
cargo test --workspace
cargo run -p ctxhelm -- --help
```
````

- [ ] **Step 3: Add project agent instructions**

Create `AGENTS.md`:

```md
# AGENTS.md

## Project Goal

Build ctxhelm as a local-first, read-only context broker for coding agents.

## Working Rules

- Keep the MVP agent-native: AGENTS.md, MCP, and thin native rules are the product surface.
- Do not add autonomous editing behavior to ctxhelm.
- Do not add cloud indexing, cloud embeddings, or cloud reranking by default.
- Prefer small typed contracts over stringly typed command output.
- Add focused tests for behavior that affects context selection, privacy, or generated agent instructions.

## Validation

- Run `cargo test --workspace` before claiming implementation work is complete.
- Run `cargo run -p ctxhelm -- --help` after CLI changes.
```

- [ ] **Step 4: Verify docs have no trailing whitespace**

Run:

```bash
git diff --check
```

Expected: no whitespace errors.

- [ ] **Step 5: Commit**

```bash
git add .gitignore README.md AGENTS.md
git commit -m "docs: add ctxhelm project instructions"
```

### Task 2: Create Core Contract Crate

**Files:**
- Create: `Cargo.toml`
- Create: `crates/ctxhelm-core/Cargo.toml`
- Create: `crates/ctxhelm-core/src/lib.rs`
- Create: `crates/ctxhelm-core/src/contracts.rs`
- Create: `crates/ctxhelm-core/src/privacy.rs`
- Create: `crates/ctxhelm-core/src/repo.rs`

- [ ] **Step 1: Create root workspace manifest with the core crate only**

Create `Cargo.toml`:

```toml
[workspace]
members = [
  "crates/ctxhelm-core",
]
resolver = "2"

[workspace.package]
edition = "2021"
license = "MIT"

[workspace.dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
uuid = { version = "1", features = ["serde"] }
```

- [ ] **Step 2: Create crate manifest**

Create `crates/ctxhelm-core/Cargo.toml`:

```toml
[package]
name = "ctxhelm-core"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
thiserror.workspace = true
uuid.workspace = true

[dev-dependencies]
serde_json.workspace = true
```

- [ ] **Step 3: Create public module surface**

Create `crates/ctxhelm-core/src/lib.rs`:

```rust
pub mod contracts;
pub mod privacy;
pub mod repo;

pub use contracts::*;
pub use privacy::PrivacyStatus;
pub use repo::{FileRole, RepoRoot};
```

- [ ] **Step 4: Define shared contracts**

Create `crates/ctxhelm-core/src/contracts.rs`:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::privacy::PrivacyStatus;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    BugFix,
    Feature,
    Refactor,
    Review,
    Test,
    Explain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TargetFile {
    pub path: String,
    pub reason: String,
    pub line_range: Option<LineRange>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelatedTest {
    pub path: String,
    pub reason: String,
    pub command: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Command {
    pub command: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PackBudget {
    Brief,
    Standard,
    Deep,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackOption {
    pub budget: PackBudget,
    pub resource_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskFlag {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextPlan {
    pub task_id: Uuid,
    pub task_type: TaskType,
    pub confidence: f32,
    pub target_files: Vec<TargetFile>,
    pub related_tests: Vec<RelatedTest>,
    pub recommended_commands: Vec<Command>,
    pub pack_options: Vec<PackOption>,
    pub missing_info_questions: Vec<String>,
    pub risk_flags: Vec<RiskFlag>,
    pub privacy_status: PrivacyStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_type_serializes_as_snake_case() {
        let json = serde_json::to_string(&TaskType::BugFix).unwrap();
        assert_eq!(json, "\"bug_fix\"");
    }
}
```

- [ ] **Step 5: Define privacy status**

Create `crates/ctxhelm-core/src/privacy.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyStatus {
    pub local_only: bool,
    pub remote_embeddings_used: bool,
    pub remote_reranking_used: bool,
    pub redactions_applied: u32,
}

impl PrivacyStatus {
    pub fn local_only() -> Self {
        Self {
            local_only: true,
            remote_embeddings_used: false,
            remote_reranking_used: false,
            redactions_applied: 0,
        }
    }
}
```

- [ ] **Step 6: Define repo primitives**

Create `crates/ctxhelm-core/src/repo.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
```

- [ ] **Step 7: Run core tests**

Run:

```bash
cargo test -p ctxhelm-core
```

Expected: PASS with `task_type_serializes_as_snake_case`.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml crates/ctxhelm-core
git commit -m "feat: add ctxhelm core contracts"
```

### Task 3: Create Initial Library Crates

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ctxhelm-index/Cargo.toml`
- Create: `crates/ctxhelm-index/src/lib.rs`
- Create: `crates/ctxhelm-compiler/Cargo.toml`
- Create: `crates/ctxhelm-compiler/src/lib.rs`
- Create: `crates/ctxhelm-mcp/Cargo.toml`
- Create: `crates/ctxhelm-mcp/src/lib.rs`

- [ ] **Step 1: Add library crates to the workspace**

Modify `Cargo.toml`:

```toml
[workspace]
members = [
  "crates/ctxhelm-core",
  "crates/ctxhelm-index",
  "crates/ctxhelm-compiler",
  "crates/ctxhelm-mcp",
]
resolver = "2"

[workspace.package]
edition = "2021"
license = "MIT"

[workspace.dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
uuid = { version = "1", features = ["serde"] }
```

- [ ] **Step 2: Add index crate**

Create `crates/ctxhelm-index/Cargo.toml`:

```toml
[package]
name = "ctxhelm-index"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
ctxhelm-core = { path = "../ctxhelm-core" }
```

Create `crates/ctxhelm-index/src/lib.rs`:

```rust
pub fn crate_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_is_ready() {
        assert!(crate_ready());
    }
}
```

- [ ] **Step 3: Add compiler crate**

Create `crates/ctxhelm-compiler/Cargo.toml`:

```toml
[package]
name = "ctxhelm-compiler"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
ctxhelm-core = { path = "../ctxhelm-core" }
ctxhelm-index = { path = "../ctxhelm-index" }
uuid = { workspace = true, features = ["v4"] }
```

Create `crates/ctxhelm-compiler/src/lib.rs`:

```rust
use ctxhelm_core::{
    ContextPlan, PackBudget, PackOption, PrivacyStatus, TaskType,
};
use uuid::Uuid;

pub fn empty_plan_for_task(task_type: TaskType) -> ContextPlan {
    let task_id = Uuid::new_v4();
    ContextPlan {
        task_id,
        task_type,
        confidence: 0.0,
        target_files: Vec::new(),
        related_tests: Vec::new(),
        recommended_commands: Vec::new(),
        pack_options: vec![PackOption {
            budget: PackBudget::Brief,
            resource_uri: format!("ctxhelm://pack/{task_id}/brief"),
        }],
        missing_info_questions: Vec::new(),
        risk_flags: Vec::new(),
        privacy_status: PrivacyStatus::local_only(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_plan_includes_brief_pack_option() {
        let plan = empty_plan_for_task(TaskType::Explain);
        assert_eq!(plan.pack_options.len(), 1);
        assert!(plan.pack_options[0].resource_uri.ends_with("/brief"));
    }
}
```

- [ ] **Step 4: Add MCP crate**

Create `crates/ctxhelm-mcp/Cargo.toml`:

```toml
[package]
name = "ctxhelm-mcp"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
ctxhelm-compiler = { path = "../ctxhelm-compiler" }
ctxhelm-core = { path = "../ctxhelm-core" }
```

Create `crates/ctxhelm-mcp/src/lib.rs`:

```rust
pub const MCP_TOOL_NAMES: &[&str] = &[
    "prepare_task",
    "search",
    "related",
    "get_pack",
    "related_tests",
    "current_diff",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_tool_surface_stays_small() {
        assert_eq!(MCP_TOOL_NAMES.len(), 6);
    }
}
```

- [ ] **Step 5: Run library tests**

Run:

```bash
cargo test -p ctxhelm-index -p ctxhelm-compiler -p ctxhelm-mcp
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/ctxhelm-index crates/ctxhelm-compiler crates/ctxhelm-mcp
git commit -m "feat: add ctxhelm library crate skeletons"
```

### Task 4: Create CLI Binary

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ctxhelm/Cargo.toml`
- Create: `crates/ctxhelm/src/main.rs`

- [ ] **Step 1: Add CLI crate to the workspace**

Modify `Cargo.toml`:

```toml
[workspace]
members = [
  "crates/ctxhelm",
  "crates/ctxhelm-core",
  "crates/ctxhelm-index",
  "crates/ctxhelm-compiler",
  "crates/ctxhelm-mcp",
]
resolver = "2"

[workspace.package]
edition = "2021"
license = "MIT"

[workspace.dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
uuid = { version = "1", features = ["serde"] }
```

- [ ] **Step 2: Add CLI crate manifest**

Create `crates/ctxhelm/Cargo.toml`:

```toml
[package]
name = "ctxhelm"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
anyhow.workspace = true
clap.workspace = true
ctxhelm-compiler = { path = "../ctxhelm-compiler" }
ctxhelm-core = { path = "../ctxhelm-core" }
serde_json.workspace = true
```

- [ ] **Step 3: Add CLI command skeleton**

Create `crates/ctxhelm/src/main.rs`:

```rust
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use ctxhelm_compiler::empty_plan_for_task;
use ctxhelm_core::TaskType;

#[derive(Debug, Parser)]
#[command(name = "ctxhelm")]
#[command(about = "Agent-native context packs for coding agents")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
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
        Command::Init => {
            println!("Milestone 0 stub: ctxhelm init has no side effects");
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
```

- [ ] **Step 4: Verify CLI help**

Run:

```bash
cargo run -p ctxhelm -- --help
```

Expected: command succeeds and lists `init`, `index`, `prepare-task`, `search`, and `serve-mcp`.

- [ ] **Step 5: Verify prepare-task JSON**

Run:

```bash
cargo run -p ctxhelm -- prepare-task "explain the auth flow" --mode explain
```

Expected: JSON output includes `"taskType": "explain"`, `"localOnly": true`, and a `ctxhelm://pack/` brief resource URI.

- [ ] **Step 6: Run full workspace tests**

Run:

```bash
cargo test --workspace
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates/ctxhelm
git commit -m "feat: add ctxhelm cli skeleton"
```

## Post-Scaffold Review

- [ ] Confirm the workspace contains all planned crates.
- [ ] Confirm `cargo test --workspace` passes.
- [ ] Confirm `ctxhelm --help` communicates the product surface without implying implemented behavior.
- [ ] Confirm stubs remain read-only and local-only.
- [ ] Confirm follow-up milestones can proceed independently from the scaffold.

## Spec Coverage Check

- Product boundary: covered by read-only stubs, AGENTS.md instructions, and no shell/editor behavior inside ctxhelm.
- MVP repo initialization: scheduled for Milestone 1.
- Safe context engine: scheduled for Milestones 2-4.
- Context compiler: started in Milestone 0 contracts, completed in Milestone 5.
- MCP runtime: crate and tool list started in Milestone 0, runtime completed in Milestone 6.
- Evaluation logging: scheduled for Milestone 7.
- Team repeatability: addressed by repo config in Milestone 1 and committed instruction files.

## Execution Handoff

After this roadmap is approved, artifact 3 should implement Milestone 0 only: the initial repo scaffold. Milestone 0 should be small, compile, and commit cleanly before moving to repo initialization behavior.
