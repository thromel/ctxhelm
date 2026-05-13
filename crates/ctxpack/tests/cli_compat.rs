mod common;

use assert_cmd::Command;
use common::{fixture_repo, json_stdout, CTXPACK_HOME_ENV};
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

#[test]
fn workspace_packages_have_release_identity() {
    let repo_root = workspace_root();
    let output = StdCommand::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let metadata: Value = serde_json::from_slice(&output.stdout).unwrap();
    let packages = metadata["packages"].as_array().unwrap();
    let expected = [
        "ctxpack",
        "ctxpack-core",
        "ctxpack-index",
        "ctxpack-compiler",
        "ctxpack-mcp",
    ];
    for name in expected {
        let package = packages
            .iter()
            .find(|package| package["name"] == name)
            .unwrap_or_else(|| panic!("missing package metadata for {name}"));
        assert_eq!(package["version"], "1.1.0", "{name} version");
        assert_eq!(package["license"], "MIT", "{name} license");
        assert!(
            package["repository"].as_str().is_some_and(|value| !value.is_empty()),
            "{name} repository metadata missing"
        );
        assert!(
            package["description"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "{name} description metadata missing"
        );
        assert!(
            package["rust_version"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "{name} rust-version metadata missing"
        );
    }

    let license = fs::read_to_string(repo_root.join("LICENSE")).unwrap();
    assert!(license.contains("MIT License"));
    assert!(license.contains("Permission is hereby granted"));
}

#[test]
fn help_lists_core_commands() {
    Command::cargo_bin("ctxpack")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("index"))
        .stdout(contains("prepare-task"))
        .stdout(contains("get-pack"))
        .stdout(contains("search"))
        .stdout(contains("related-tests"))
        .stdout(contains("dependencies"))
        .stdout(contains("setup-check"))
        .stdout(contains("eval"))
        .stdout(contains("serve-mcp"));
}

#[test]
fn version_reports_release_identity() {
    Command::cargo_bin("ctxpack")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("ctxpack 1.1.0"));
}

#[test]
fn init_reports_file_actions_and_next_steps() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .args(["init", "--repo"])
        .arg(&fixture.repo)
        .assert()
        .success()
        .stdout(contains("Initialized ctxpack in"))
        .stdout(contains("created:"))
        .stdout(contains("skipped:"))
        .stdout(contains(".cursor/rules/ctxpack.mdc"))
        .stdout(contains("Next steps"))
        .stdout(contains("ctxpack --version"))
        .stdout(contains("ctxpack --help"))
        .stdout(contains("ctxpack setup-check --repo"))
        .stdout(contains("prepare_task"))
        .stdout(contains("get_pack"));

    Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .args(["init", "--repo"])
        .arg(&fixture.repo)
        .assert()
        .success()
        .stdout(contains("unchanged:"))
        .stdout(contains("skipped:"));
}

#[test]
fn init_with_adapters_reports_repo_local_outputs_only() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .args(["init", "--repo"])
        .arg(&fixture.repo)
        .args(["--cursor", "--claude", "--opencode"])
        .assert()
        .success()
        .stdout(contains(".cursor/rules/ctxpack.mdc"))
        .stdout(contains(".claude/commands/ctxpack-bugfix.md"))
        .stdout(contains(".ctxpack/adapters/claude-mcp.json"))
        .stdout(contains(".ctxpack/adapters/opencode.jsonc.snippet"))
        .stdout(contains("does not mutate global agent config"))
        .stdout(contains("Copy/paste"))
        .stdout(predicates::str::contains("mutate global Codex, Claude, Cursor, or OpenCode config").not());
}

#[test]
fn setup_check_reports_generated_artifacts() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .args(["init", "--repo"])
        .arg(&fixture.repo)
        .args(["--cursor", "--claude", "--opencode"])
        .assert()
        .success();

    Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .args(["setup-check", "--repo"])
        .arg(&fixture.repo)
        .args(["--cursor", "--claude", "--opencode"])
        .assert()
        .success()
        .stdout(contains("Setup check for"))
        .stdout(contains("pass: AGENTS.md"))
        .stdout(contains("pass: .cursor/rules/ctxpack.mdc"))
        .stdout(contains("warn: ctxpack binary path guidance"))
        .stdout(contains("ctxpack --version"))
        .stdout(contains("which ctxpack"))
        .stdout(contains("does not mutate global agent config"));
}

#[test]
fn setup_check_fails_when_expected_adapter_file_is_missing() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .args(["init", "--repo"])
        .arg(&fixture.repo)
        .assert()
        .success();

    Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .args(["setup-check", "--repo"])
        .arg(&fixture.repo)
        .arg("--cursor")
        .assert()
        .failure()
        .stdout(contains("fail: .cursor/rules/ctxpack.mdc"));
}

#[test]
fn setup_check_help_documents_read_only_validation() {
    Command::cargo_bin("ctxpack")
        .unwrap()
        .args(["setup-check", "--help"])
        .assert()
        .success()
        .stdout(contains("read-only"))
        .stdout(contains("generated setup artifacts"))
        .stdout(contains("--cursor"))
        .stdout(contains("--claude"))
        .stdout(contains("--opencode"));
}

#[test]
fn read_command_help_lists_no_trace_control() {
    Command::cargo_bin("ctxpack")
        .unwrap()
        .args(["prepare-task", "--help"])
        .assert()
        .success()
        .stdout(contains("--no-trace"));

    Command::cargo_bin("ctxpack")
        .unwrap()
        .args(["get-pack", "--help"])
        .assert()
        .success()
        .stdout(contains("--no-trace"));
}

#[test]
fn index_writes_inventory_under_command_home() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .args(["index", "--repo"])
        .arg(&fixture.repo)
        .assert()
        .success()
        .stdout(contains("Indexed"))
        .stdout(contains(fixture.repo.display().to_string()));

    let inventory_path = inventory_path_under(&fixture.home);
    assert!(
        inventory_path.starts_with(&fixture.home),
        "inventory was not written under CTXPACK_HOME: {}",
        inventory_path.display()
    );
    let inventory: Value = serde_json::from_slice(&fs::read(&inventory_path).unwrap()).unwrap();
    let paths = inventory["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| file["path"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert!(paths.contains(&"src/auth/session.ts"));
    assert!(paths.contains(&"tests/auth/session.test.ts"));
    assert!(!paths.contains(&".env"));
    assert!(!paths.contains(&"dist/generated.min.js"));
}

#[test]
fn prepare_task_outputs_context_plan_shape() {
    let fixture = fixture_repo();

    let value = json_stdout(
        Command::cargo_bin("ctxpack")
            .unwrap()
            .env(CTXPACK_HOME_ENV, &fixture.home)
            .args(["prepare-task", "fix requireSession auth bug", "--repo"])
            .arg(&fixture.repo)
            .args([
                "--mode",
                "bug-fix",
                "--path",
                "src/auth/session.ts",
                "--target-agent",
                "codex",
            ])
            .assert(),
    );

    assert_object_has_keys(
        &value,
        &[
            "taskId",
            "taskType",
            "confidence",
            "targetFiles",
            "relatedTests",
            "recommendedCommands",
            "packOptions",
            "missingInfoQuestions",
            "riskFlags",
            "diagnostics",
            "retrievalCandidates",
            "privacyStatus",
        ],
    );
    assert_eq!(value["taskType"], "bug_fix");
    assert_eq!(value["privacyStatus"]["localOnly"], true);
    assert!(value["diagnostics"].is_array());
    assert_path_present(&value["targetFiles"], "src/auth/session.ts");
    assert_path_present(&value["relatedTests"], "tests/auth/session.test.ts");
    let target = value["targetFiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|target| target["path"] == "src/auth/session.ts")
        .unwrap();
    assert!(target["attribution"]
        .as_array()
        .unwrap()
        .iter()
        .any(|evidence| evidence["reasonCode"] == "explicit_path_anchor"
            || evidence["reasonCode"] == "lexical_match"));
    let related_test = value["relatedTests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|test| test["path"] == "tests/auth/session.test.ts")
        .unwrap();
    assert!(related_test["attribution"]
        .as_array()
        .unwrap()
        .iter()
        .any(|evidence| evidence["reasonCode"].is_string() && evidence["signal"].is_string()));
    assert!(value["retrievalCandidates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|candidate| candidate["path"] == "src/auth/session.ts"
            && candidate["kind"] == "file"
            && candidate["signalScores"].is_array()
            && candidate["evidence"].is_array()));
    assert!(value["recommendedCommands"].is_array());
    assert!(value["packOptions"].as_array().unwrap().len() >= 3);
    assert!(value.get("target_files").is_none());
}

#[test]
fn prepare_task_reports_stale_inventory_rebuild_diagnostics() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .args(["index", "--repo"])
        .arg(&fixture.repo)
        .assert()
        .success();
    fs::write(
        fixture.repo.join("src/auth/refreshed.ts"),
        "export const refreshed = true;\n",
    )
    .unwrap();

    let value = json_stdout(
        Command::cargo_bin("ctxpack")
            .unwrap()
            .env(CTXPACK_HOME_ENV, &fixture.home)
            .args(["prepare-task", "fix refreshed auth behavior", "--repo"])
            .arg(&fixture.repo)
            .args(["--mode", "bug-fix", "--no-trace"])
            .assert(),
    );

    let codes = diagnostic_codes(&value);
    assert!(codes.contains(&"inventory_stale"));
    assert!(codes.contains(&"inventory_rebuilt"));
}

#[test]
fn prepare_task_returns_context_when_trace_write_fails() {
    let fixture = fixture_repo();
    let blocked_home = fixture.temp.path().join("ctxpack-home-file");
    fs::write(&blocked_home, "not a directory\n").unwrap();

    let value = json_stdout(
        Command::cargo_bin("ctxpack")
            .unwrap()
            .env(CTXPACK_HOME_ENV, &blocked_home)
            .args(["prepare-task", "fix requireSession auth bug", "--repo"])
            .arg(&fixture.repo)
            .args(["--mode", "bug-fix", "--path", "src/auth/session.ts"])
            .assert(),
    );

    assert_path_present(&value["targetFiles"], "src/auth/session.ts");
    let codes = diagnostic_codes(&value);
    assert!(codes.contains(&"cache_write_failed"));
    assert!(codes.contains(&"trace_write_failed"));
}

#[test]
fn get_pack_outputs_json_and_markdown_contracts() {
    let fixture = fixture_repo();

    let json = json_stdout(
        Command::cargo_bin("ctxpack")
            .unwrap()
            .env(CTXPACK_HOME_ENV, &fixture.home)
            .args(["get-pack", "fix requireSession auth bug", "--repo"])
            .arg(&fixture.repo)
            .args([
                "--mode",
                "bug-fix",
                "--budget",
                "brief",
                "--format",
                "json",
                "--path",
                "src/auth/session.ts",
                "--target-agent",
                "codex",
            ])
            .assert(),
    );

    assert_object_has_keys(
        &json,
        &[
            "id",
            "taskId",
            "repoId",
            "taskHash",
            "taskType",
            "targetAgent",
            "budget",
            "sections",
            "tokenEstimate",
            "confidence",
            "warnings",
            "diagnostics",
            "privacyStatus",
        ],
    );
    assert_eq!(json["taskType"], "bug_fix");
    assert_eq!(json["targetAgent"], "codex");
    assert_eq!(json["budget"], "brief");
    assert!(!json["repoId"].as_str().unwrap().is_empty());
    assert!(!json["taskHash"].as_str().unwrap().is_empty());
    assert!(json["diagnostics"].is_array());
    assert!(json["sections"].as_array().unwrap().iter().any(|section| {
        section["kind"] == "target_files"
            && section["content"]
                .as_str()
                .unwrap()
                .contains("src/auth/session.ts")
    }));

    Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .args(["get-pack", "fix requireSession auth bug", "--repo"])
        .arg(&fixture.repo)
        .args([
            "--mode",
            "bug-fix",
            "--budget",
            "brief",
            "--format",
            "markdown",
            "--path",
            "src/auth/session.ts",
            "--target-agent",
            "codex",
        ])
        .assert()
        .success()
        .stdout(contains("# Context Pack"))
        .stdout(contains("Repo ID:"))
        .stdout(contains("Task hash:"))
        .stdout(contains("Target agent:"));
}

#[test]
fn get_pack_returns_context_when_trace_write_fails() {
    let fixture = fixture_repo();
    let blocked_home = fixture.temp.path().join("ctxpack-home-file");
    fs::write(&blocked_home, "not a directory\n").unwrap();

    let value = json_stdout(
        Command::cargo_bin("ctxpack")
            .unwrap()
            .env(CTXPACK_HOME_ENV, &blocked_home)
            .args(["get-pack", "fix requireSession auth bug", "--repo"])
            .arg(&fixture.repo)
            .args([
                "--mode",
                "bug-fix",
                "--format",
                "json",
                "--path",
                "src/auth/session.ts",
            ])
            .assert(),
    );

    assert_eq!(value["budget"], "brief");
    assert!(
        value["sections"].as_array().unwrap().iter().any(|section| {
            section["kind"] == "target_files"
                && section["content"]
                    .as_str()
                    .unwrap()
                    .contains("src/auth/session.ts")
        }),
        "expected target file section in {value:?}"
    );
    let codes = diagnostic_codes(&value);
    assert!(codes.contains(&"cache_write_failed"));
    assert!(codes.contains(&"trace_write_failed"));
}

#[test]
fn search_related_tests_dependencies_and_eval_history_emit_json_shapes() {
    let fixture = fixture_repo();

    let search = json_stdout(
        Command::cargo_bin("ctxpack")
            .unwrap()
            .env(CTXPACK_HOME_ENV, &fixture.home)
            .args(["search", "requireSession", "--repo"])
            .arg(&fixture.repo)
            .args(["--limit", "3"])
            .assert(),
    );
    let first_search = first_array_item(&search);
    assert_object_has_keys(
        first_search,
        &["path", "role", "language", "score", "reason"],
    );
    assert_path_present(&search, "src/auth/session.ts");

    let related_tests = json_stdout(
        Command::cargo_bin("ctxpack")
            .unwrap()
            .env(CTXPACK_HOME_ENV, &fixture.home)
            .args(["related-tests", "src/auth/session.ts", "--repo"])
            .arg(&fixture.repo)
            .assert(),
    );
    let first_test = first_array_item(&related_tests);
    assert_object_has_keys(first_test, &["path", "command", "confidence", "reason"]);
    assert_eq!(first_test["path"], "tests/auth/session.test.ts");

    let dependencies = json_stdout(
        Command::cargo_bin("ctxpack")
            .unwrap()
            .env(CTXPACK_HOME_ENV, &fixture.home)
            .args(["dependencies", "src/auth/session.ts", "--repo"])
            .arg(&fixture.repo)
            .assert(),
    );
    let first_edge = first_array_item(&dependencies);
    assert_object_has_keys(
        first_edge,
        &["sourcePath", "targetPath", "kind", "confidence", "reason"],
    );
    assert_eq!(first_edge["sourcePath"], "src/auth/session.ts");
    assert_eq!(first_edge["targetPath"], "src/auth/token.ts");
    assert_eq!(first_edge["kind"], "imports");

    let history = json_stdout(
        Command::cargo_bin("ctxpack")
            .unwrap()
            .env(CTXPACK_HOME_ENV, &fixture.home)
            .args(["eval", "history", "--repo"])
            .arg(&fixture.repo)
            .args(["--format", "json", "--limit", "1"])
            .assert(),
    );
    assert_object_has_keys(
        &history,
        &[
            "repoId",
            "evaluatedCommits",
            "base",
            "head",
            "lowInformationCommitCount",
            "fileRecallAt5",
            "fileRecallAt10",
            "lexicalBaselineRecallAt5",
            "lexicalBaselineRecallAt10",
            "ctxpackLiftAt5",
            "ctxpackLiftAt10",
            "sourceRecallAt5",
            "sourceRecallAt10",
            "testRecallAt5",
            "testRecallAt10",
            "testRecommendationRate",
            "averageRecommendedContextFiles",
            "topMissingFiles",
            "commits",
            "evalRangeId",
            "effectiveFilters",
            "refs",
            "rankingComparison",
            "signalAblations",
            "retrievalGapSummaries",
            "privacyStatus",
        ],
    );
    assert_eq!(history["privacyStatus"]["localOnly"], true);
    assert_eq!(history["effectiveFilters"]["rankingBudget"], 10);
    assert_eq!(history["rankingComparison"]["k"], 10);
    assert!(history["signalAblations"].is_array());
    assert!(history["retrievalGapSummaries"].is_array());
    assert_no_source_or_prompt_text(&history);
    if let Some(commit) = history["commits"].as_array().unwrap().first() {
        assert_object_has_keys(
            commit,
            &[
                "sha",
                "taskHash",
                "taskType",
                "targetAgent",
                "safeChangedFiles",
                "excludedChangedFileCount",
                "recommendedFiles",
                "recommendedTests",
                "recommendedContextFiles",
                "recommendedCommands",
                "lexicalBaselineFiles",
                "fileHitsAt5",
                "fileHitsAt10",
                "lexicalBaselineHitsAt5",
                "lexicalBaselineHitsAt10",
                "missingFilesAt10",
                "sourceFilesChanged",
                "sourceHitsAt5",
                "sourceHitsAt10",
                "testFilesChanged",
                "testHitsAt5",
                "testHitsAt10",
                "lowInformationTask",
                "confidence",
                "sourceTextLogged",
            ],
        );
        assert_eq!(commit["sourceTextLogged"], false);
    }

    Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .args(["eval", "history", "--repo"])
        .arg(&fixture.repo)
        .args(["--format", "markdown", "--limit", "1", "--budget", "10"])
        .assert()
        .success()
        .stdout(contains("Eval range ID:"))
        .stdout(contains("Ranking budget K: `10`"))
        .stdout(contains("Recall@K:"))
        .stdout(contains("## Signal Ablations"))
        .stdout(contains("## Grouped Retrieval Failures"))
        .stdout(contains("Source text logged: `false`"));
}

#[test]
fn serve_mcp_speaks_json_rpc_over_stdio() {
    let fixture = fixture_repo();
    let input = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
"#;

    let assert = Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .current_dir(&fixture.repo)
        .arg("serve-mcp")
        .write_stdin(input.as_slice())
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let lines = stdout.lines().collect::<Vec<_>>();

    assert_eq!(lines.len(), 2);
    let initialize: Value = serde_json::from_str(lines[0]).unwrap();
    let tools: Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(initialize["id"], 1);
    assert_eq!(initialize["result"]["serverInfo"]["name"], "ctxpack");
    assert_eq!(tools["id"], 2);
    let tool_names = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(tool_names.contains(&"prepare_task"));
    assert!(tool_names.contains(&"get_pack"));
    assert!(tool_names.contains(&"related_tests"));
}

#[test]
fn mcp_protocol_uses_explicit_repo_from_wrong_cwd() {
    let fixture = fixture_repo();
    let server_cwd = tempfile::tempdir().unwrap();
    fs::write(
        fixture.repo.join("src/auth/session.ts"),
        r#"import { issueToken } from "./token";

export function requireSession(user?: { id: string }) {
  if (!user) {
    throw new Error("auth required");
  }
  issueToken(user.id);
  return user.id;
}

export function refreshSession() {
  return "refresh";
}
"#,
    )
    .unwrap();
    fs::write(
        fixture.repo.join("tests/auth/session_refresh.test.ts"),
        r#"import { refreshSession } from "../../src/auth/session";

test("refreshSession returns refresh", () => {
  expect(refreshSession()).toBe("refresh");
});
"#,
    )
    .unwrap();

    let repo = fixture.repo.display().to_string();
    let input = [
        rpc_request(1, "initialize", json!({})),
        rpc_request(
            2,
            "tools/call",
            json!({
                "name": "prepare_task",
                "arguments": {
                    "task": "fix requireSession auth bug",
                    "repo": repo,
                    "mode": "bug_fix",
                    "paths": ["src/auth/session.ts"],
                    "targetAgent": "codex",
                    "recordTrace": false
                }
            }),
        ),
        rpc_request(
            3,
            "tools/call",
            json!({
                "name": "get_pack",
                "arguments": {
                    "task": "fix requireSession auth bug",
                    "repo": repo,
                    "mode": "bug_fix",
                    "budget": "brief",
                    "format": "json",
                    "paths": ["src/auth/session.ts"],
                    "targetAgent": "codex",
                    "recordTrace": false
                }
            }),
        ),
        rpc_request(
            4,
            "tools/call",
            json!({
                "name": "search",
                "arguments": {
                    "query": "requireSession",
                    "repo": repo,
                    "limit": 5
                }
            }),
        ),
        rpc_request(
            5,
            "tools/call",
            json!({
                "name": "related",
                "arguments": {
                    "path": "src/auth/session.ts",
                    "repo": repo,
                    "include": ["tests", "dependencies"],
                    "limit": 5
                }
            }),
        ),
        rpc_request(
            6,
            "tools/call",
            json!({
                "name": "related_tests",
                "arguments": {
                    "paths": ["src/auth/session.ts"],
                    "repo": repo
                }
            }),
        ),
        rpc_request(
            7,
            "tools/call",
            json!({
                "name": "current_diff",
                "arguments": {
                    "repo": repo,
                    "includeUntracked": true
                }
            }),
        ),
    ]
    .join("\n")
        + "\n";

    let assert = Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .current_dir(server_cwd.path())
        .arg("serve-mcp")
        .write_stdin(input.as_bytes())
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let responses = stdout
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(responses.len(), 7);
    for response in &responses {
        assert!(
            response.get("error").is_none(),
            "unexpected JSON-RPC error: {response}"
        );
    }

    let plan = structured_content(&responses[1]);
    assert_path_present(&plan["targetFiles"], "src/auth/session.ts");

    let pack = structured_content(&responses[2]);
    assert!(!pack["repoId"].as_str().unwrap().is_empty());
    assert!(!pack["sections"].as_array().unwrap().is_empty());

    let search = structured_content(&responses[3]);
    assert_path_present(&search["files"], "src/auth/session.ts");

    let related = structured_content(&responses[4]);
    assert_path_present(&related["relatedTests"], "tests/auth/session.test.ts");
    assert!(
        related["dependencyEdges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| edge["sourcePath"] == "src/auth/session.ts"
                && edge["targetPath"] == "src/auth/token.ts"),
        "missing fixture dependency edge in {related:?}"
    );

    let related_tests = structured_content(&responses[5]);
    assert_path_present(related_tests, "tests/auth/session.test.ts");

    let current_diff = structured_content(&responses[6]);
    assert_string_present(&current_diff["unstaged"], "src/auth/session.ts");
    assert_string_present(
        &current_diff["untracked"],
        "tests/auth/session_refresh.test.ts",
    );
}

#[test]
fn pack_resource_restart_returns_session_scoped_diagnostic() {
    let fixture = fixture_repo();
    let server_cwd = tempfile::tempdir().unwrap();
    let repo = fixture.repo.display().to_string();
    let prepare_input = [
        rpc_request(1, "initialize", json!({})),
        rpc_request(
            2,
            "tools/call",
            json!({
                "name": "prepare_task",
                "arguments": {
                    "task": "fix requireSession auth bug",
                    "repo": repo,
                    "mode": "bug_fix",
                    "paths": ["src/auth/session.ts"],
                    "targetAgent": "codex",
                    "recordTrace": false
                }
            }),
        ),
    ]
    .join("\n")
        + "\n";

    let first = Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .current_dir(server_cwd.path())
        .arg("serve-mcp")
        .write_stdin(prepare_input.as_bytes())
        .assert()
        .success();
    let first_stdout = String::from_utf8(first.get_output().stdout.clone()).unwrap();
    let first_responses = first_stdout
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(first_responses.len(), 2);
    let pack_uri = first_responses[1]["result"]["structuredContent"]["packOptions"][0]
        ["resourceUri"]
        .as_str()
        .unwrap()
        .to_string();

    let read_input = rpc_request(
        3,
        "resources/read",
        json!({
            "uri": pack_uri
        }),
    ) + "\n";
    let second = Command::cargo_bin("ctxpack")
        .unwrap()
        .env(CTXPACK_HOME_ENV, &fixture.home)
        .current_dir(server_cwd.path())
        .arg("serve-mcp")
        .write_stdin(read_input.as_bytes())
        .assert()
        .success();
    let second_stdout = String::from_utf8(second.get_output().stdout.clone()).unwrap();
    let response: Value = serde_json::from_str(second_stdout.lines().next().unwrap()).unwrap();
    assert_eq!(response["error"]["code"], -32602);
    let message = response["error"]["message"].as_str().unwrap();
    assert!(message.contains("session-scoped"));
    assert!(message.contains("same MCP server process"));
    assert!(message.contains("call prepare_task first"));
}

#[test]
fn real_client_smoke_scripts_have_contract_guards() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    for script in ["scripts/smoke-codex-mcp.sh", "scripts/smoke-claude-mcp.sh"] {
        let script_path = workspace_root.join(script);
        let content = fs::read_to_string(&script_path).unwrap_or_else(|error| {
            panic!("failed to read {script}: {error}");
        });

        let syntax = StdCommand::new("bash")
            .arg("-n")
            .arg(&script_path)
            .status()
            .unwrap_or_else(|error| panic!("failed to run bash -n {script}: {error}"));
        assert!(syntax.success(), "{script} failed bash -n");

        let protocol_index = content
            .find("scripts/smoke-mcp-protocol.sh")
            .unwrap_or_else(|| panic!("{script} does not run the protocol hard gate"));
        let client_index = if script.contains("codex") {
            assert!(
                content.contains("--dangerously-bypass-approvals-and-sandbox"),
                "{script} must prevent non-interactive MCP calls from being auto-cancelled"
            );
            content
                .find("codex exec")
                .unwrap_or_else(|| panic!("{script} does not attempt codex exec"))
        } else {
            assert!(
                !content.contains("--bare"),
                "{script} must use the normal Claude Code auth path for real-client smoke"
            );
            content
                .find("claude -p")
                .unwrap_or_else(|| panic!("{script} does not attempt claude -p"))
        };
        assert!(
            protocol_index < client_index,
            "{script} must run scripts/smoke-mcp-protocol.sh before client invocation"
        );

        for needle in [
            "prepare_task",
            "get_pack",
            "repo",
            "CTXPACK_BIN",
            "CTXPACK_REQUIRE_REAL_CLIENT",
            "CTXPACK_SKIP_REAL_CLIENT",
            "CTXPACK_REAL_CLIENT_EVIDENCE_DIR",
            "clientVersion",
            "ctxpackVersion",
            "prepareTask",
            "getPack",
            "required",
            "--version",
            "serve-mcp",
            "request_log",
            "skipped",
            "passed",
            "failed",
        ] {
            assert!(content.contains(needle), "{script} missing {needle}");
        }
        assert!(
            content.contains("CTXPACK_BIN=\"$ctxpack_bin\""),
            "{script} must propagate the selected binary into child smoke commands"
        );
        assert!(
            content.contains("\"$ctxpack_bin\" --version"),
            "{script} must capture ctxpack version from the selected binary"
        );
        assert!(
            content.contains("\"$ctxpack_bin\" serve-mcp") || content.contains("'tee -a \"$CTXPACK_REAL_CLIENT_REQUEST_LOG\" | \"$ctxpack_bin\" serve-mcp'"),
            "{script} must launch the MCP server through the selected binary"
        );
        assert!(
            content.contains("CTXPACK_REAL_CLIENT_EVIDENCE_DIR"),
            "{script} must support stable evidence output"
        );
        assert!(
            content.contains("write_evidence"),
            "{script} must emit machine-checkable evidence on success"
        );
        assert!(
            content.contains("CTXPACK_RUN_REAL_CLIENT")
                || content.contains("CTXPACK_REQUIRE_REAL_CLIENT"),
            "{script} must keep real-client execution env-gated"
        );
    }
}

#[test]
fn mcp_protocol_smoke_script_supports_selected_binary() {
    let script_path = workspace_root().join("scripts/smoke-mcp-protocol.sh");
    let content = fs::read_to_string(&script_path).unwrap();

    let syntax = StdCommand::new("bash")
        .arg("-n")
        .arg(&script_path)
        .status()
        .unwrap();
    assert!(syntax.success(), "scripts/smoke-mcp-protocol.sh failed bash -n");

    assert!(content.contains("CTXPACK_BIN"));
    assert!(content.contains("serve-mcp"));
    assert!(
        content.contains("cargo") && content.contains("run"),
        "development cargo fallback must remain"
    );
    assert!(
        content.find("CTXPACK_BIN").unwrap() < content.find("serve-mcp").unwrap(),
        "selected binary handling must be established before serve-mcp launch"
    );
    assert!(
        content.contains("diff_repo=\"$(mktemp -d)\""),
        "current_diff proof should use an isolated temporary repo"
    );
    assert!(
        content.contains("git -C \"$diff_repo\" init -q"),
        "temporary current_diff repo must be initialized as a git repo"
    );
    assert!(
        !content.contains("smoke_diff_file=\"$CTXPACK_SMOKE_REPO/$smoke_diff_path\""),
        "MCP smoke must not create current_diff proof files inside the target repo"
    );
    assert!(content.contains("prepare_task"));
    assert!(content.contains("get_pack"));
    assert!(content.contains("\"repo\""));
}

#[test]
fn first_pack_smoke_script_contract_and_execution() {
    let script_path = workspace_root().join("scripts/smoke-first-pack.sh");
    let content = fs::read_to_string(&script_path).unwrap();

    let syntax = StdCommand::new("bash")
        .arg("-n")
        .arg(&script_path)
        .status()
        .unwrap();
    assert!(syntax.success(), "scripts/smoke-first-pack.sh failed bash -n");

    for needle in [
        "CTXPACK_BIN",
        "command -v ctxpack",
        "CTXPACK_HOME",
        "git",
        "init --repo",
        "setup-check --repo",
        "scripts/smoke-mcp-protocol.sh",
        "prepare-task",
        "get-pack",
        "targetFiles",
        "packOptions",
        "repoId",
        "sections",
    ] {
        assert!(content.contains(needle), "smoke-first-pack missing {needle}");
    }

    let output = StdCommand::new("bash")
        .arg(&script_path)
        .env("CTXPACK_BIN", env!("CARGO_BIN_EXE_ctxpack"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "smoke-first-pack failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ctxpack first-pack smoke ok"));
    assert!(stdout.contains("binary="));
    assert!(stdout.contains("repo="));
}

fn inventory_path_under(home: &Path) -> PathBuf {
    let repos_dir = home.join("repos");
    fs::read_dir(&repos_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path().join("inventory.json"))
        .find(|path| path.exists())
        .unwrap()
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn rpc_request(id: u64, method: &str, params: Value) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    })
    .to_string()
}

fn structured_content(response: &Value) -> &Value {
    &response["result"]["structuredContent"]
}

fn assert_object_has_keys(value: &Value, expected: &[&str]) {
    let object = value.as_object().unwrap();
    for key in expected {
        assert!(object.contains_key(*key), "missing public field {key}");
    }
}

fn assert_path_present(value: &Value, expected_path: &str) {
    let values = value.as_array().unwrap();
    assert!(
        values
            .iter()
            .any(|item| item["path"].as_str() == Some(expected_path)),
        "missing expected path {expected_path} in {values:?}"
    );
}

fn assert_string_present(value: &Value, expected: &str) {
    let values = value.as_array().unwrap();
    assert!(
        values.iter().any(|item| item.as_str() == Some(expected)),
        "missing expected string {expected} in {values:?}"
    );
}

fn first_array_item(value: &Value) -> &Value {
    value.as_array().unwrap().first().unwrap()
}

fn diagnostic_codes(value: &Value) -> Vec<&str> {
    value["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|diagnostic| diagnostic["code"].as_str())
        .collect()
}

fn assert_no_source_or_prompt_text(value: &Value) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                assert!(
                    !matches!(
                        key.as_str(),
                        "sourceText"
                            | "source_text"
                            | "source"
                            | "snippet"
                            | "prompt"
                            | "task"
                            | "taskText"
                            | "commitSubject"
                    ),
                    "unexpected source or prompt text field {key}"
                );
                assert_no_source_or_prompt_text(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                assert_no_source_or_prompt_text(item);
            }
        }
        _ => {}
    }
}
