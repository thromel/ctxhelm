# Testing Patterns

**Analysis Date:** 2026-05-13

## Test Framework

**Runner:**
- Rust built-in test harness via `cargo test`.
- Workspace members are defined in `Cargo.toml`: `crates/ctxpack`, `crates/ctxpack-core`, `crates/ctxpack-index`, `crates/ctxpack-compiler`, and `crates/ctxpack-mcp`.
- No separate test framework config file is detected.
- `tokio` is a workspace dependency in `Cargo.toml`, but no `#[tokio::test]` tests are detected.

**Assertion Library:**
- Rust standard assertions: `assert!`, `assert_eq!`, `assert_ne!`, `matches!`.
- JSON assertions use `serde_json::Value` indexing and `serde_json::json!`, especially in `crates/ctxpack-core/src/contracts.rs` and `crates/ctxpack-mcp/src/lib.rs`.

**Run Commands:**
```bash
cargo test --workspace                         # Run all tests
cargo test -p ctxpack-index                    # Run one crate's tests
cargo test -p ctxpack-mcp prepare_task -- --nocapture  # Run matching tests with output
cargo run -p ctxpack -- --help                 # Validate CLI surface after CLI changes
```

**Watch Mode:**
- Not detected. Use `cargo watch -x test` only if the developer has `cargo-watch` installed locally; it is not part of this repo.

**Coverage:**
- No coverage command, coverage threshold, or coverage config is detected.

## Test File Organization

**Location:**
- Tests are inline in source files under `#[cfg(test)]` modules.
- No top-level `tests/` directory and no `*.test.*` or `*.spec.*` files are detected.

**Naming:**
- Test module names are `tests` for most files and `writer_tests` for a second focused module in `crates/ctxpack-core/src/init.rs`.
- Test functions use behavior-driven snake_case names:
  - `task_type_serializes_as_snake_case` in `crates/ctxpack-core/src/contracts.rs`
  - `inventory_respects_ignores_and_default_exclusions` in `crates/ctxpack-index/src/lib.rs`
  - `prepare_context_plan_fuses_search_tests_and_history` in `crates/ctxpack-compiler/src/lib.rs`
  - `prepare_task_call_returns_structured_context_plan` in `crates/ctxpack-mcp/src/lib.rs`
  - `historical_eval_report_renders_source_free_metrics` in `crates/ctxpack/src/main.rs`

**Structure:**
```text
crates/ctxpack-core/src/contracts.rs       # Contract serialization tests
crates/ctxpack-core/src/repo.rs            # Repo root discovery tests
crates/ctxpack-core/src/init.rs            # Init adapter and writer tests
crates/ctxpack-index/src/lib.rs            # Inventory, search, symbols, git, diff, test-map tests
crates/ctxpack-compiler/src/lib.rs         # Context-plan, pack, eval, card rendering tests
crates/ctxpack-mcp/src/lib.rs              # JSON-RPC/MCP protocol tests
crates/ctxpack/src/main.rs                 # CLI report rendering tests
```

**Detected Test Count:**
- 102 `#[test]` functions.
- Distribution by file:
  - `crates/ctxpack-mcp/src/lib.rs`: 30
  - `crates/ctxpack-index/src/lib.rs`: 26
  - `crates/ctxpack-core/src/init.rs`: 20
  - `crates/ctxpack-compiler/src/lib.rs`: 17
  - `crates/ctxpack-core/src/contracts.rs`: 4
  - `crates/ctxpack/src/main.rs`: 3
  - `crates/ctxpack-core/src/repo.rs`: 2

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_plan_serializes_with_camel_case_contract_fields() {
        let plan = ContextPlan {
            task_id: Uuid::nil(),
            task_type: TaskType::BugFix,
            confidence: 1.0,
            target_files: vec![TargetFile {
                path: "src/lib.rs".to_string(),
                reason: "public API surface".to_string(),
                line_range: Some(LineRange { start: 1, end: 7 }),
                confidence: 0.5,
            }],
            related_tests: vec![],
            recommended_commands: vec![],
            pack_options: vec![PackOption {
                budget: PackBudget::Brief,
                resource_uri: "ctxpack://packs/brief".to_string(),
            }],
            missing_info_questions: vec![],
            risk_flags: vec![],
            privacy_status: PrivacyStatus::local_only(),
        };

        let value = serde_json::to_value(&plan).unwrap();

        assert_eq!(value["taskType"], "bug_fix");
        assert!(value.get("task_id").is_none());
    }
}
```

This pattern is from `crates/ctxpack-core/src/contracts.rs`.

**Patterns:**
- Build complete fixtures inside each test with `tempfile::tempdir()` and `std::fs`.
- Use real files and real git repositories for behavior involving inventory, history, current diff, and co-change logic.
- Use direct function calls for library behavior: `prepare_context_plan`, `write_inventory`, `lexical_search`, `related_tests`, and `handle_line`.
- Assert both positive behavior and privacy exclusions. Tests in `crates/ctxpack-index/src/lib.rs` assert that `.env`, `private.key`, `dist/app.min.js`, ignored files, and generated fixtures are excluded by default.
- Assert serialized contract field names explicitly. `crates/ctxpack-core/src/contracts.rs`, `crates/ctxpack-core/src/init.rs`, and `crates/ctxpack-mcp/src/lib.rs` check camelCase, snake_case, and source-free fields.
- Use `#[cfg(unix)]` for symlink and permission tests in `crates/ctxpack-core/src/init.rs` and `crates/ctxpack-index/src/lib.rs`.

## Mocking

**Framework:** Not detected

**Patterns:**
```rust
fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[test]
fn prepare_context_plan_fuses_search_tests_and_history() {
    let _guard = env_lock();
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let home = temp.path().join("ctxpack-home");
    fs::create_dir_all(repo.join("src/auth")).unwrap();
    std::env::set_var("CTXPACK_HOME", &home);

    let plan = prepare_context_plan(&repo, "fix requireSession bug", TaskType::BugFix).unwrap();

    assert_eq!(plan.target_files[0].path, "src/auth/session.ts");
    std::env::remove_var("CTXPACK_HOME");
}
```

This environment-lock pattern appears in `crates/ctxpack-compiler/src/lib.rs`, `crates/ctxpack-index/src/lib.rs`, and `crates/ctxpack-mcp/src/lib.rs`.

**What to Mock:**
- Prefer fixture repositories over mocks for filesystem, inventory, git, and MCP behavior.
- Use synthetic JSON-RPC strings rather than a network client for MCP request tests in `crates/ctxpack-mcp/src/lib.rs`.
- Use constructed structs for pure renderer tests in `crates/ctxpack/src/main.rs`.

**What NOT to Mock:**
- Do not mock git behavior when validating co-change, current-diff, or historical-eval behavior. Existing tests create real repositories with `git init`, `git add`, and `git commit`.
- Do not mock filesystem privacy behavior. Existing tests create `.env`, `private.key`, generated directories, ignore files, symlinks, and unreadable files.
- Do not mock JSON serialization for public contracts. Tests serialize with `serde_json` and inspect actual values.

## Fixtures and Factories

**Test Data:**
```rust
struct FixtureRepo {
    _temp: tempfile::TempDir,
    repo: std::path::PathBuf,
    home: std::path::PathBuf,
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
    FixtureRepo { _temp: temp, repo, home }
}
```

This fixture shape is used in `crates/ctxpack-mcp/src/lib.rs`.

**Location:**
- Fixtures live inside each test module, not in shared fixture files.
- Test repositories and test home directories are created under `tempfile::TempDir`.
- Fixture file names intentionally mimic supported target projects: `src/auth/session.ts`, `tests/auth/session.test.ts`, `package.json`, `pnpm-lock.yaml`, `.ctxpackignore`, `.env`, `dist/generated.min.js`, `src/session.ts`, and `src/service.py`.
- Environment-dependent tests set `CTXPACK_HOME` to a temp directory and remove it before finishing.

## Coverage

**Requirements:** None enforced

**View Coverage:**
```bash
# Not configured in the repository.
```

**Coverage Emphasis:**
- Context selection and ranking behavior in `crates/ctxpack-compiler/src/lib.rs`.
- Privacy filtering for sensitive/generated/current-diff paths in `crates/ctxpack-index/src/lib.rs`.
- Generated agent instructions and safe repo-local writes in `crates/ctxpack-core/src/init.rs`.
- MCP tool/resource/prompt contracts in `crates/ctxpack-mcp/src/lib.rs`.
- CLI Markdown report rendering in `crates/ctxpack/src/main.rs`.

## Test Types

**Unit Tests:**
- Pure contract and renderer tests assert serialization and text output directly.
- Examples:
  - `crates/ctxpack-core/src/contracts.rs` tests `TaskType`, `ContextPlan`, `ContextPack`, and `EvalTrace` serialization.
  - `crates/ctxpack/src/main.rs` tests eval checklist, historical eval, and context card report rendering.

**Integration Tests:**
- Integration-style tests are inline and use real temp repositories, git commands, and filesystem state.
- Examples:
  - `crates/ctxpack-index/src/lib.rs` tests inventory, lexical search, symbol extraction, related tests, dependency edges, current diff, eval traces, and historical commit samples.
  - `crates/ctxpack-compiler/src/lib.rs` tests context-plan fusion across search, tests, dependency graph, and git history.
  - `crates/ctxpack-mcp/src/lib.rs` tests JSON-RPC request handling through `handle_line` and `run_server`.

**E2E Tests:**
- No external E2E test harness is detected.
- MCP and CLI-adjacent behavior is covered with in-process tests in `crates/ctxpack-mcp/src/lib.rs` and renderer tests in `crates/ctxpack/src/main.rs`.
- README smoke commands in `README.md` and validation guidance in `AGENTS.md` cover manual end-to-end checks:
```bash
cargo test --workspace
cargo run -p ctxpack -- --help
cargo run -p ctxpack -- serve-mcp
```

## Common Patterns

**Async Testing:**
```rust
// Not detected. Existing tests use synchronous #[test] functions.
#[test]
fn run_server_writes_one_response_per_request_line() {
    let input = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
"#;
    let mut output = Vec::new();
    run_server(&input[..], &mut output).unwrap();
    assert!(String::from_utf8(output).unwrap().contains("\"jsonrpc\""));
}
```

Use synchronous tests unless adding behavior that requires async runtime APIs.

**Error Testing:**
```rust
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
```

This pattern is used in `crates/ctxpack-mcp/src/lib.rs`.

**Filesystem Safety Testing:**
```rust
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
```

This pattern is used in `crates/ctxpack-core/src/init.rs`.

**Privacy Testing:**
- Create sensitive and generated fixture paths directly.
- Assert excluded paths are absent from results.
- Assert source text is not logged or returned where the contract promises metadata-only results.
- Examples live in `crates/ctxpack-index/src/lib.rs`, `crates/ctxpack-compiler/src/lib.rs`, `crates/ctxpack-mcp/src/lib.rs`, and `crates/ctxpack/src/main.rs`.

---

*Testing analysis: 2026-05-13*
