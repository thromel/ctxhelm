#![recursion_limit = "256"]

mod common;

use assert_cmd::Command;
use common::{fixture_repo, json_stdout, run_git, CTXHELM_HOME_ENV};
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
        "ctxhelm",
        "ctxhelm-core",
        "ctxhelm-index",
        "ctxhelm-compiler",
        "ctxhelm-mcp",
    ];
    for name in expected {
        let package = packages
            .iter()
            .find(|package| package["name"] == name)
            .unwrap_or_else(|| panic!("missing package metadata for {name}"));
        assert_eq!(package["version"], "2.4.6", "{name} version");
        assert_eq!(package["license"], "MIT", "{name} license");
        assert!(
            package["repository"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
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
    Command::cargo_bin("ctxhelm")
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
        .stdout(contains("precision"))
        .stdout(contains("setup"))
        .stdout(contains("setup-check"))
        .stdout(contains("doctor"))
        .stdout(contains("eval"))
        .stdout(contains("workspace"))
        .stdout(contains("serve-mcp"));
}

#[test]
fn version_reports_release_identity() {
    Command::cargo_bin("ctxhelm")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("ctxhelm 2.4.6"));
}

#[test]
fn doctor_verifies_binary_manifest_and_local_state_source_free() {
    let fixture = fixture_repo();
    let binary = assert_cmd::cargo::cargo_bin("ctxhelm");
    let manifest_path = fixture.temp.path().join("release-manifest.json");
    fs::write(
        &manifest_path,
        json!({
            "version": "2.4.6",
            "archive": {
                "name": "ctxhelm-v2.4.6-test.tar.gz",
                "sha256": "archive-sha"
            },
            "binary": {
                "name": "ctxhelm",
                "sha256": "binary-sha"
            },
            "auditReport": "ctxhelm-v2.4.6-test.audit.json",
            "privacyStatus": {
                "localOnly": true,
                "sourceTextLogged": false
            }
        })
        .to_string(),
    )
    .unwrap();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["doctor", "--format", "json", "--repo"])
            .arg(&fixture.repo)
            .arg("--binary")
            .arg(&binary)
            .arg("--release-manifest")
            .arg(&manifest_path)
            .assert()
            .success(),
    );

    assert_eq!(value["passed"], true);
    assert_eq!(value["binary"]["version"], "ctxhelm 2.4.6");
    assert_eq!(value["releaseManifest"]["version"], "2.4.6");
    assert_eq!(value["privacyStatus"]["localOnly"], true);
    assert_eq!(value["mutatesGlobalAgentConfig"], false);
    assert!(value["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| check["name"] == "local_state_compatibility" && check["passed"] == true));
    let rendered = serde_json::to_string(&value).unwrap();
    assert!(!rendered.contains("UserDb"));
    assert!(!rendered.contains("fix requireSession"));
}

#[test]
fn cards_fallback_generates_source_free_agent_guide() {
    let fixture = fixture_repo();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["cards", "fallback", "--repo"])
            .arg(&fixture.repo)
            .args(["--target-agent", "claude", "--format", "json"])
            .assert(),
    );
    assert_eq!(value["targetAgent"], "claude-code");
    assert_eq!(value["sourceTextLogged"], false);
    assert!(value["cardCount"].as_u64().unwrap() >= 3);
    assert_no_source_or_prompt_text(&value);

    let guide_path = value["guidePath"].as_str().unwrap();
    let guide = fs::read_to_string(guide_path).unwrap();
    assert!(guide.contains("ctxhelm Disconnected Fallback"));
    assert!(guide.contains("Claude Code"));
    assert!(guide.contains("Source snippets included: `false`"));
    assert!(!guide.contains("auth required"));
    assert!(!guide.contains("token:${userId}"));
}

#[test]
fn init_reports_file_actions_and_next_steps() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["init", "--repo"])
        .arg(&fixture.repo)
        .assert()
        .success()
        .stdout(contains("Initialized ctxhelm in"))
        .stdout(contains("created:"))
        .stdout(contains("skipped:"))
        .stdout(contains(".cursor/rules/ctxhelm.mdc"))
        .stdout(contains("Next steps"))
        .stdout(contains("ctxhelm --version"))
        .stdout(contains("ctxhelm --help"))
        .stdout(contains("ctxhelm setup-check --repo"))
        .stdout(contains("prepare_task"))
        .stdout(contains("get_pack"));

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
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

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["init", "--repo"])
        .arg(&fixture.repo)
        .args(["--cursor", "--claude", "--opencode"])
        .assert()
        .success()
        .stdout(contains(".cursor/rules/ctxhelm.mdc"))
        .stdout(contains(".claude/commands/ctxhelm-bugfix.md"))
        .stdout(contains(".ctxhelm/adapters/claude-mcp.json"))
        .stdout(contains(".ctxhelm/adapters/opencode.jsonc.snippet"))
        .stdout(contains("does not mutate global agent config"))
        .stdout(contains("Copy/paste"))
        .stdout(
            predicates::str::contains("mutate global Codex, Claude, Cursor, or OpenCode config")
                .not(),
        );
}

#[test]
fn setup_claude_writes_project_local_mcp_config() {
    let fixture = fixture_repo();
    fs::write(
        fixture.repo.join(".mcp.json"),
        serde_json::to_string_pretty(&json!({
            "mcpServers": {
                "other": {
                    "command": "other-tool",
                    "args": ["serve"]
                }
            }
        }))
        .unwrap(),
    )
    .unwrap();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["setup", "claude", "--repo"])
        .arg(&fixture.repo)
        .assert()
        .success()
        .stdout(contains("Claude Code setup for"))
        .stdout(contains(".claude/commands/ctxhelm-bugfix.md"))
        .stdout(contains(".ctxhelm/adapters/claude-mcp.json"))
        .stdout(contains(".mcp.json"))
        .stdout(contains("did not mutate global Claude Code config"))
        .stdout(contains("Setup check for"))
        .stdout(contains("Result: passed"));

    let mcp: Value =
        serde_json::from_str(&fs::read_to_string(fixture.repo.join(".mcp.json")).unwrap()).unwrap();
    assert_eq!(mcp["mcpServers"]["ctxhelm"]["args"], json!(["serve-mcp"]));
    let command = mcp["mcpServers"]["ctxhelm"]["command"]
        .as_str()
        .expect("ctxhelm command should be a string");
    assert!(
        Path::new(command).is_absolute(),
        "setup should write an absolute ctxhelm binary path"
    );
    assert_eq!(mcp["mcpServers"]["other"]["command"], "other-tool");
}

#[test]
fn setup_claude_dry_run_does_not_write_files() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["setup", "claude", "--repo"])
        .arg(&fixture.repo)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(contains("Claude Code setup dry run"))
        .stdout(contains("would write project MCP config"))
        .stdout(contains("Current setup status:"))
        .stdout(contains("Dry-run setup status is informational"));

    assert!(!fixture.repo.join(".mcp.json").exists());
    assert!(!fixture
        .repo
        .join(".claude/commands/ctxhelm-bugfix.md")
        .exists());
}

#[test]
fn setup_repo_writes_secure_repo_local_agent_artifacts() {
    let fixture = fixture_repo();
    fs::write(
        fixture.repo.join(".mcp.json"),
        serde_json::to_string_pretty(&json!({
            "mcpServers": {
                "existing": {
                    "command": "existing-tool",
                    "args": ["serve"]
                }
            }
        }))
        .unwrap(),
    )
    .unwrap();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["setup", "repo", "--repo"])
        .arg(&fixture.repo)
        .assert()
        .success()
        .stdout(contains("Repo setup for"))
        .stdout(contains(".cursor/rules/ctxhelm.mdc"))
        .stdout(contains(".claude/commands/ctxhelm-bugfix.md"))
        .stdout(contains(".ctxhelm/adapters/claude-mcp.json"))
        .stdout(contains(".ctxhelm/adapters/opencode.jsonc.snippet"))
        .stdout(contains(".mcp.json"))
        .stdout(contains(
            "did not mutate global Codex, Claude, Cursor, or OpenCode config",
        ))
        .stdout(contains("Setup check for"))
        .stdout(contains("Result: passed"));

    assert!(fixture.repo.join(".cursor/rules/ctxhelm.mdc").is_file());
    assert!(fixture
        .repo
        .join(".claude/commands/ctxhelm-bugfix.md")
        .is_file());
    assert!(fixture
        .repo
        .join(".ctxhelm/adapters/opencode.jsonc.snippet")
        .is_file());

    let mcp: Value =
        serde_json::from_str(&fs::read_to_string(fixture.repo.join(".mcp.json")).unwrap()).unwrap();
    assert_eq!(mcp["mcpServers"]["ctxhelm"]["args"], json!(["serve-mcp"]));
    let command = mcp["mcpServers"]["ctxhelm"]["command"]
        .as_str()
        .expect("ctxhelm command should be a string");
    assert!(
        Path::new(command).is_absolute(),
        "repo setup should write an absolute ctxhelm binary path"
    );
    assert_eq!(mcp["mcpServers"]["existing"]["command"], "existing-tool");
}

#[test]
fn setup_repo_dry_run_does_not_write_files() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["setup", "repo", "--repo"])
        .arg(&fixture.repo)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(contains("Repo setup dry run"))
        .stdout(contains("would write Cursor rule"))
        .stdout(contains("would write project MCP config"))
        .stdout(contains(
            "would not mutate global Codex, Claude, Cursor, or OpenCode config",
        ))
        .stdout(contains("Current setup status:"))
        .stdout(contains("Dry-run setup status is informational"));

    assert!(!fixture.repo.join(".mcp.json").exists());
    assert!(!fixture.repo.join(".cursor/rules/ctxhelm.mdc").exists());
    assert!(!fixture
        .repo
        .join(".claude/commands/ctxhelm-bugfix.md")
        .exists());
    assert!(!fixture
        .repo
        .join(".ctxhelm/adapters/opencode.jsonc.snippet")
        .exists());
}

#[test]
fn setup_repo_json_reports_source_free_security_boundary() {
    let fixture = fixture_repo();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["setup", "repo", "--repo"])
            .arg(&fixture.repo)
            .args(["--format", "json"])
            .assert(),
    );

    assert_eq!(value["schemaVersion"], "ctxhelm-setup-report-v1");
    assert_eq!(value["command"], "setup repo");
    assert_eq!(value["dryRun"], false);
    assert_eq!(value["privacyStatus"]["localOnly"], true);
    assert_eq!(value["privacyStatus"]["sourceTextLogged"], false);
    assert_eq!(value["projectMcp"]["action"], "created");
    assert_eq!(value["projectMcp"]["commandUsesAbsoluteBinary"], true);
    assert_eq!(
        value["projectMcp"]["path"],
        fixture.repo.join(".mcp.json").display().to_string()
    );
    assert_eq!(value["setupCheck"]["passed"], true);
    assert_eq!(
        value["setupCheck"]["schemaVersion"],
        "ctxhelm-setup-check-report-v1"
    );
    assert_eq!(
        value["setupCheck"]["checkedAdapters"],
        json!(["cursor", "claude", "opencode"])
    );
    assert_eq!(value["setupCheck"]["summary"]["failCount"], 0);
    assert_eq!(value["setupCheck"]["recommendedNextAction"], "ready");
    assert!(value["plannedFiles"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path == &json!(fixture.repo.join(".cursor/rules/ctxhelm.mdc"))));
    assert!(value["unsupportedActions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|action| action == "global agent config mutation"));
    assert_no_source_or_prompt_text(&value);
}

#[test]
fn setup_claude_json_dry_run_reports_planned_without_writes() {
    let fixture = fixture_repo();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["setup", "claude", "--repo"])
            .arg(&fixture.repo)
            .args(["--dry-run", "--format", "json"])
            .assert(),
    );

    assert_eq!(value["schemaVersion"], "ctxhelm-setup-report-v1");
    assert_eq!(value["command"], "setup claude");
    assert_eq!(value["dryRun"], true);
    assert_eq!(value["projectMcp"]["action"], "planned");
    assert_eq!(value["projectMcp"]["commandUsesAbsoluteBinary"], true);
    assert!(value["initReport"].is_null());
    assert_eq!(value["setupCheck"]["passed"], false);
    assert_eq!(
        value["setupCheck"]["schemaVersion"],
        "ctxhelm-setup-check-report-v1"
    );
    assert_eq!(value["setupCheck"]["checkedAdapters"], json!(["claude"]));
    assert!(
        value["setupCheck"]["summary"]["failCount"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert_eq!(
        value["setupCheck"]["recommendedNextAction"],
        "review_setup_failures"
    );
    assert!(value["setupCheck"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(
            |item| item["name"] == ".claude/commands/ctxhelm-bugfix.md" && item["status"] == "fail"
        ));
    assert!(value["plannedFiles"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path == &json!(fixture.repo.join(".claude/commands/ctxhelm-bugfix.md"))));
    assert!(!fixture.repo.join(".mcp.json").exists());
    assert!(!fixture
        .repo
        .join(".claude/commands/ctxhelm-bugfix.md")
        .exists());
    assert_no_source_or_prompt_text(&value);
}

#[test]
fn workspace_init_and_status_are_source_free_for_multiple_repos() {
    let first = fixture_repo();
    let second = fixture_repo();
    let sentinel = "CTXHELM_WORKSPACE_CLI_SOURCE_SENTINEL";
    fs::write(second.repo.join("src/auth/workspace-secret.ts"), sentinel).unwrap();
    run_git(&second.repo, &["add", "."]);
    run_git(&second.repo, &["commit", "-m", "add workspace sentinel"]);

    let init = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &first.home)
            .args(["workspace", "init", "--repo"])
            .arg(&first.repo)
            .args(["--member"])
            .arg(&second.repo)
            .args(["--label", "primary", "--format", "json"])
            .assert(),
    );
    assert_eq!(init["sourceTextLogged"], false);
    assert_eq!(init["repoCount"], 2);
    assert!(init["repos"].as_array().unwrap().len() == 2);
    assert_no_source_or_prompt_text(&init);
    assert!(!serde_json::to_string(&init).unwrap().contains(sentinel));

    let manifest = first.repo.join(".ctxhelm/workspace.json");
    assert!(manifest.exists());
    let manifest_json: Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    assert_eq!(manifest_json["schemaVersion"], 1);
    assert_eq!(manifest_json["repos"].as_array().unwrap().len(), 2);

    let status = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &first.home)
            .args(["workspace", "status", "--repo"])
            .arg(&first.repo)
            .args(["--format", "json"])
            .assert(),
    );
    assert_object_has_keys(
        &status,
        &[
            "schemaVersion",
            "workspaceRoot",
            "manifestPath",
            "repoCount",
            "availableRepoCount",
            "fileCount",
            "generatedCount",
            "sensitiveCount",
            "sourceTextLogged",
            "privacyStatus",
            "repos",
            "diagnostics",
        ],
    );
    assert_eq!(status["availableRepoCount"], 2);
    assert_eq!(status["sourceTextLogged"], false);
    assert_eq!(status["privacyStatus"]["localOnly"], true);
    assert!(status["repos"].as_array().unwrap().iter().all(|repo| {
        repo["state"] == "available" && repo["privacyStatus"]["sourceTextLogged"] == false
    }));
    assert!(!serde_json::to_string(&status).unwrap().contains(sentinel));
    assert_no_source_or_prompt_text(&status);

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &first.home)
        .args(["workspace", "status", "--repo"])
        .arg(&first.repo)
        .assert()
        .success()
        .stdout(contains("# ctxhelm Workspace Status"))
        .stdout(contains("Source text logged: `false`"))
        .stdout(contains("primary"));
}

#[test]
fn workspace_prepare_task_routes_to_likely_repo_source_free() {
    let first = fixture_repo();
    let second = fixture_repo();
    fs::write(
        second.repo.join("src/auth/workspace-login.ts"),
        "export function workspaceLoginRedirect() { return true; }\n",
    )
    .unwrap();
    run_git(&second.repo, &["add", "."]);
    run_git(&second.repo, &["commit", "-m", "add workspace login"]);

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &first.home)
        .args(["workspace", "init", "--repo"])
        .arg(&first.repo)
        .args(["--member"])
        .arg(&second.repo)
        .assert()
        .success();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &first.home)
            .args([
                "workspace",
                "prepare-task",
                "fix workspace login redirect",
                "--repo",
            ])
            .arg(&first.repo)
            .args(["--mode", "bug-fix", "--format", "json"])
            .assert(),
    );

    assert_object_has_keys(
        &value,
        &[
            "taskId",
            "taskType",
            "confidence",
            "workspaceRoot",
            "manifestPath",
            "selectedRepoCount",
            "sourceTextLogged",
            "privacyStatus",
            "repoPlans",
            "diagnostics",
        ],
    );
    assert_eq!(value["taskType"], "bug_fix");
    assert_eq!(value["sourceTextLogged"], false);
    assert_eq!(value["privacyStatus"]["localOnly"], true);
    assert!(value["selectedRepoCount"].as_u64().unwrap() >= 1);
    assert!(value["repoPlans"].as_array().unwrap().iter().any(|repo| {
        repo["contextPlan"]["targetFiles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|target| target["path"] == "src/auth/workspace-login.ts")
    }));
    assert!(!serde_json::to_string(&value)
        .unwrap()
        .contains("export function workspaceLoginRedirect"));
    assert_no_source_or_prompt_text(&value);
}

#[test]
fn workspace_get_pack_returns_repo_boundary_aware_pack() {
    let first = fixture_repo();
    let second = fixture_repo();
    fs::write(
        second.repo.join("src/auth/workspace-login.ts"),
        "export function workspaceLoginRedirect() { return true; }\n",
    )
    .unwrap();
    run_git(&second.repo, &["add", "."]);
    run_git(&second.repo, &["commit", "-m", "add workspace login"]);

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &first.home)
        .args(["workspace", "init", "--repo"])
        .arg(&first.repo)
        .args(["--member"])
        .arg(&second.repo)
        .assert()
        .success();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &first.home)
            .args([
                "workspace",
                "get-pack",
                "fix workspace login redirect",
                "--repo",
            ])
            .arg(&first.repo)
            .args([
                "--mode",
                "bug-fix",
                "--budget",
                "brief",
                "--target-agent",
                "codex",
                "--format",
                "json",
            ])
            .assert(),
    );

    assert_object_has_keys(
        &value,
        &[
            "id",
            "taskId",
            "taskType",
            "targetAgent",
            "budget",
            "confidence",
            "tokenEstimate",
            "workspaceRoot",
            "manifestPath",
            "selectedRepoCount",
            "sourceTextLogged",
            "privacyStatus",
            "warnings",
            "repoPacks",
            "diagnostics",
        ],
    );
    assert_eq!(value["targetAgent"], "codex");
    assert_eq!(value["budget"], "brief");
    assert_eq!(value["sourceTextLogged"], false);
    assert!(value["repoPacks"].as_array().unwrap().iter().any(|repo| {
        repo["contextPack"]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .any(|section| {
                section["kind"] == "target_snippets"
                    && section["content"]
                        .as_str()
                        .unwrap()
                        .contains("workspaceLoginRedirect")
            })
    }));
    assert!(value["repoPacks"].as_array().unwrap().iter().all(|repo| {
        repo["repoId"].is_string()
            && repo["label"].is_string()
            && repo["pathLabel"].is_string()
            && repo["contextPack"]["repoId"].is_string()
    }));
}

#[test]
fn workspace_shared_artifacts_and_team_policy_are_source_free() {
    let fixture = fixture_repo();
    fs::create_dir_all(fixture.repo.join(".ctxhelm/cards")).unwrap();
    fs::write(
        fixture.repo.join(".ctxhelm/cards/testing.md"),
        "source-free testing summary\n",
    )
    .unwrap();
    fs::write(
        fixture.repo.join(".ctxhelm/feedback-summary.json"),
        r#"{"sourceTextLogged":false,"eventCount":0}"#,
    )
    .unwrap();

    let policy = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["workspace", "policy", "init", "--repo"])
            .arg(&fixture.repo)
            .args(["--format", "json"])
            .assert(),
    );
    assert_eq!(policy["policy"]["allowCloudEmbeddings"], false);
    assert_eq!(policy["policy"]["allowCloudReranking"], false);
    assert_eq!(policy["sourceTextLogged"], false);
    assert!(fixture.repo.join(".ctxhelm/team-policy.json").exists());
    assert_no_source_or_prompt_text(&policy);

    let manifest = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["workspace", "artifacts", "export", "--repo"])
            .arg(&fixture.repo)
            .args(["--format", "json"])
            .assert(),
    );
    assert_eq!(manifest["sourceTextLogged"], false);
    assert!(manifest["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|artifact| {
            artifact["kind"] == "context_cards" && artifact["status"] == "present"
        }));
    assert_no_source_or_prompt_text(&manifest);

    let manifest_path = fixture.repo.join(".ctxhelm/shared-artifacts.json");
    let inspect = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["workspace", "artifacts", "inspect"])
            .arg(&manifest_path)
            .args(["--format", "json"])
            .assert(),
    );
    assert_eq!(inspect["compatible"], true);
    assert_eq!(inspect["sourceTextLogged"], false);

    let import = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["workspace", "artifacts", "import"])
            .arg(&manifest_path)
            .args(["--repo"])
            .arg(&fixture.repo)
            .args(["--format", "json"])
            .assert(),
    );
    assert_eq!(import["compatible"], true);
    assert!(fixture
        .repo
        .join(".ctxhelm/imported-shared-artifacts.json")
        .exists());
}

#[test]
fn setup_check_reports_generated_artifacts() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["init", "--repo"])
        .arg(&fixture.repo)
        .args(["--cursor", "--claude", "--opencode"])
        .assert()
        .success();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["setup-check", "--repo"])
        .arg(&fixture.repo)
        .args(["--cursor", "--claude", "--opencode"])
        .assert()
        .success()
        .stdout(contains("Setup check for"))
        .stdout(contains("pass: AGENTS.md"))
        .stdout(contains("pass: .cursor/rules/ctxhelm.mdc"))
        .stdout(contains("warn: ctxhelm binary path guidance"))
        .stdout(contains("ctxhelm --version"))
        .stdout(contains("which ctxhelm"))
        .stdout(contains("does not mutate global agent config"));
}

#[test]
fn setup_check_json_reports_source_free_project_mcp_readiness() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["setup", "claude", "--repo"])
        .arg(&fixture.repo)
        .assert()
        .success();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["setup-check", "--repo"])
            .arg(&fixture.repo)
            .args(["--claude", "--format", "json"])
            .assert(),
    );

    assert_eq!(value["schemaVersion"], "ctxhelm-setup-check-report-v1");
    assert_eq!(value["passed"], true);
    assert_eq!(value["checkedAdapters"], json!(["claude"]));
    assert_eq!(value["summary"]["failCount"], 0);
    assert_eq!(value["recommendedNextAction"], "ready");
    assert_eq!(value["repoRoot"], fixture.repo.display().to_string());
    assert!(value["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["name"] == ".mcp.json" && item["status"] == "pass"));
    assert_no_source_or_prompt_text(&value);
}

#[test]
fn setup_check_all_json_reports_all_supported_agent_artifacts() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["setup", "repo", "--repo"])
        .arg(&fixture.repo)
        .assert()
        .success();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["setup-check", "--repo"])
            .arg(&fixture.repo)
            .args(["--all", "--format", "json"])
            .assert(),
    );

    assert_eq!(value["schemaVersion"], "ctxhelm-setup-check-report-v1");
    assert_eq!(value["passed"], true);
    assert_eq!(
        value["checkedAdapters"],
        json!(["cursor", "claude", "opencode"])
    );
    assert_eq!(value["summary"]["passCount"], 7);
    assert_eq!(value["summary"]["warnCount"], 1);
    assert_eq!(value["summary"]["failCount"], 0);
    assert_eq!(value["recommendedNextAction"], "ready");
    let items = value["items"].as_array().unwrap();
    for name in [
        "AGENTS.md",
        ".ctxhelm/ctxhelm.toml",
        ".cursor/rules/ctxhelm.mdc",
        ".claude/commands/ctxhelm-bugfix.md",
        ".ctxhelm/adapters/claude-mcp.json",
        ".ctxhelm/adapters/opencode.jsonc.snippet",
        ".mcp.json",
    ] {
        assert!(
            items
                .iter()
                .any(|item| item["name"] == name && item["status"] == "pass"),
            "missing passing setup-check item for {name}: {items:?}"
        );
    }
    assert_no_source_or_prompt_text(&value);
}

#[test]
fn setup_check_json_failure_preserves_parseable_report_and_exit_status() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["init", "--repo"])
        .arg(&fixture.repo)
        .assert()
        .success();

    let assert = Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["setup-check", "--repo"])
        .arg(&fixture.repo)
        .args(["--cursor", "--format", "json"])
        .assert()
        .failure();
    let value: Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();

    assert_eq!(value["schemaVersion"], "ctxhelm-setup-check-report-v1");
    assert_eq!(value["passed"], false);
    assert_eq!(value["checkedAdapters"], json!(["cursor"]));
    assert!(value["summary"]["failCount"].as_u64().unwrap() > 0);
    assert_eq!(value["recommendedNextAction"], "review_setup_failures");
    assert!(value["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| { item["name"] == ".cursor/rules/ctxhelm.mdc" && item["status"] == "fail" }));
    assert_no_source_or_prompt_text(&value);
}

#[test]
fn setup_check_fails_when_expected_adapter_file_is_missing() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["init", "--repo"])
        .arg(&fixture.repo)
        .assert()
        .success();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["setup-check", "--repo"])
        .arg(&fixture.repo)
        .arg("--cursor")
        .assert()
        .failure()
        .stdout(contains("fail: .cursor/rules/ctxhelm.mdc"));
}

#[test]
fn setup_check_help_documents_read_only_validation() {
    Command::cargo_bin("ctxhelm")
        .unwrap()
        .args(["setup-check", "--help"])
        .assert()
        .success()
        .stdout(contains("read-only"))
        .stdout(contains("generated setup artifacts"))
        .stdout(contains("--all"))
        .stdout(contains("--cursor"))
        .stdout(contains("--claude"))
        .stdout(contains("--opencode"))
        .stdout(contains("--format"));
}

#[test]
fn read_command_help_lists_no_trace_control() {
    Command::cargo_bin("ctxhelm")
        .unwrap()
        .args(["prepare-task", "--help"])
        .assert()
        .success()
        .stdout(contains("--no-trace"))
        .stdout(contains("--semantic"));

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .args(["get-pack", "--help"])
        .assert()
        .success()
        .stdout(contains("--no-trace"))
        .stdout(contains("--semantic"));
}

#[test]
fn index_writes_inventory_under_command_home() {
    let fixture = fixture_repo();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["index", "--repo"])
        .arg(&fixture.repo)
        .assert()
        .success()
        .stdout(contains("Indexed"))
        .stdout(contains(fixture.repo.display().to_string()));

    let inventory_path = inventory_path_under(&fixture.home);
    assert!(
        inventory_path.starts_with(&fixture.home),
        "inventory was not written under CTXHELM_HOME: {}",
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
fn index_semantic_persists_source_free_vector_metadata() {
    let fixture = fixture_repo();
    let store = fixture.home.join("semantic.sqlite3");

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["index", "--repo"])
        .arg(&fixture.repo)
        .args(["--semantic", "--store-path"])
        .arg(&store)
        .assert()
        .success()
        .stdout(contains("Semantic storage sync"))
        .stdout(contains("semantic vector records"));

    let bytes = fs::read(&store).unwrap();
    let database_text = String::from_utf8_lossy(&bytes);
    assert!(!database_text.contains("auth required"));
    assert!(!database_text.contains("do-not-index"));
}

#[test]
fn local_semantic_provider_selection_is_source_free_and_policy_visible() {
    let fixture = fixture_repo();

    let status = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["semantic", "status", "--repo"])
            .arg(&fixture.repo)
            .args([
                "--semantic-provider",
                "local_fastembed",
                "--query",
                "fix requireSession auth bug",
                "--format",
                "json",
            ])
            .assert(),
    );

    assert_eq!(status["providerKind"], "local_fastembed");
    assert_eq!(status["modelId"], "AllMiniLML6V2Q");
    assert_eq!(status["providerRole"], "production_local");
    assert_eq!(status["qualityBackend"], true);
    assert_eq!(status["localOnly"], true);
    assert_eq!(status["privacyStatus"]["remoteEmbeddingsUsed"], false);
    assert!(status["providerPolicy"]["decisions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|decision| decision["provider"] == "local_fastembed"
            && (decision["status"] == "allowed" || decision["status"] == "unavailable")));
    assert_eq!(status["sourceTextLogged"], false);
    assert_no_source_or_prompt_text(&status);

    let plan = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["prepare-task", "fix requireSession auth bug", "--repo"])
            .arg(&fixture.repo)
            .args([
                "--semantic",
                "--semantic-provider",
                "local_fastembed",
                "--no-trace",
            ])
            .assert(),
    );

    assert!(plan["providerPolicy"]["decisions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|decision| decision["provider"] == "local_fastembed"
            && (decision["status"] == "allowed" || decision["status"] == "unavailable")));
    assert_no_source_or_prompt_text(&plan);
}

#[test]
fn prepare_task_outputs_context_plan_shape() {
    let fixture = fixture_repo();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
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

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
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
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
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
    let blocked_home = fixture.temp.path().join("ctxhelm-home-file");
    fs::write(&blocked_home, "not a directory\n").unwrap();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &blocked_home)
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
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
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

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
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
    let blocked_home = fixture.temp.path().join("ctxhelm-home-file");
    fs::write(&blocked_home, "not a directory\n").unwrap();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &blocked_home)
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
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
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

    let semantic = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["search", "auth required", "--repo"])
            .arg(&fixture.repo)
            .args(["--limit", "3", "--semantic"])
            .assert(),
    );
    let first_semantic = first_array_item(&semantic);
    assert_object_has_keys(
        first_semantic,
        &["path", "role", "language", "score", "reason", "provider"],
    );
    assert_path_present(&semantic, "src/auth/session.ts");
    assert_no_source_or_prompt_text(&semantic);

    let related_tests = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["related-tests", "src/auth/session.ts", "--repo"])
            .arg(&fixture.repo)
            .assert(),
    );
    let first_test = first_array_item(&related_tests);
    assert_object_has_keys(first_test, &["path", "command", "confidence", "reason"]);
    assert_eq!(first_test["path"], "tests/auth/session.test.ts");

    let dependencies = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
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

    let discovered_precision = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["precision", "discover", "--repo"])
            .arg(&fixture.repo)
            .args(["--format", "json", "--limit", "20"])
            .assert(),
    );
    assert_eq!(
        discovered_precision["provider"],
        "local_tree_sitter_reference_scan"
    );
    assert!(discovered_precision["discoveredEdges"].as_u64().unwrap() > 0);
    assert_no_source_or_prompt_text(&discovered_precision);

    let precision_input = fixture.home.join("precision.json");
    fs::write(
        &precision_input,
        r#"{
  "schemaVersion": 1,
  "provider": "scip-json-fixture",
  "edges": [
    {
      "sourcePath": "src/auth/session.ts",
      "targetPath": "src/auth/token.ts",
      "edgeType": "calls",
      "symbol": "issueToken",
      "confidence": 0.99,
      "reason": "local precision fixture"
    },
    {
      "sourcePath": ".env",
      "targetPath": "src/auth/token.ts",
      "edgeType": "calls",
      "reason": "do-not-index"
    }
  ]
}
"#,
    )
    .unwrap();
    let precision = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["precision", "import", "--repo"])
            .arg(&fixture.repo)
            .args(["--input"])
            .arg(&precision_input)
            .args(["--format", "json"])
            .assert(),
    );
    assert_eq!(precision["provider"], "scip-json-fixture");
    assert_eq!(precision["acceptedEdges"], 1);
    assert_eq!(precision["rejectedEdges"], 1);
    assert_no_source_or_prompt_text(&precision);

    let precision_dependencies = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["dependencies", "src/auth/session.ts", "--repo"])
            .arg(&fixture.repo)
            .args(["--limit", "10"])
            .assert(),
    );
    assert!(precision_dependencies
        .as_array()
        .unwrap()
        .iter()
        .any(
            |edge| edge["kind"] == "precision:calls" && edge["reason"] == "local precision fixture"
        ));
    assert_no_source_or_prompt_text(&precision_dependencies);

    let history = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
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
            "ctxhelmLiftAt5",
            "ctxhelmLiftAt10",
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
            "graphEdgeAblations",
            "tokenRoi",
            "retrievalGapSummaries",
            "graphEdgeProfiles",
            "runtime",
            "privacyStatus",
        ],
    );
    assert_eq!(history["privacyStatus"]["localOnly"], true);
    assert_eq!(history["effectiveFilters"]["rankingBudget"], 10);
    assert_eq!(history["rankingComparison"]["k"], 10);
    assert_eq!(
        history["rankingComparison"]["noContextBaseline"]["recallAtK"],
        0.0
    );
    assert!(
        history["rankingComparison"]["recallLiftVsNoContextAtK"]
            .as_f64()
            .unwrap()
            >= 0.0
    );
    assert!(history["signalAblations"].is_array());
    assert!(history["graphEdgeAblations"].is_array());
    assert!(history["tokenRoi"].is_array());
    assert_eq!(history["tokenRoi"][0]["budget"], "brief");
    assert!(history["retrievalGapSummaries"].is_array());
    assert!(history["graphEdgeProfiles"].is_array());
    assert!(history["runtime"]["totalMillis"].as_u64().is_some());
    assert!(history["runtime"]["commitMillis"].as_u64().is_some());
    assert!(history["runtime"]["overheadMillis"].as_u64().is_some());
    assert!(history["runtime"]["averageCommitMillis"].as_f64().is_some());
    assert!(history["runtime"]["slowCommits"].is_array());
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
                "graphEdgeProfiles",
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
                "elapsedMillis",
                "sourceTextLogged",
            ],
        );
        assert!(commit["elapsedMillis"].as_u64().is_some());
        assert_eq!(commit["sourceTextLogged"], false);
    }

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args(["eval", "history", "--repo"])
        .arg(&fixture.repo)
        .args(["--format", "markdown", "--limit", "1", "--budget", "10"])
        .assert()
        .success()
        .stdout(contains("Eval range ID:"))
        .stdout(contains("Ranking budget K: `10`"))
        .stdout(contains("Recall@K:"))
        .stdout(contains("## Signal Ablations"))
        .stdout(contains("## Graph Edge Ablations"))
        .stdout(contains("## Graph Edge Profiles"))
        .stdout(contains("## Runtime Diagnostics"))
        .stdout(contains("## Grouped Retrieval Failures"))
        .stdout(contains("Source text logged: `false`"));
}

#[test]
fn eval_feedback_records_lists_and_summarizes_source_free_events() {
    let fixture = fixture_repo();
    let record = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args([
                "eval",
                "feedback",
                "record",
                "--task-hash",
                "task-hash-1",
                "--mode",
                "bug-fix",
                "--target-agent",
                "codex",
                "--budget",
                "brief",
                "--outcome",
                "passed",
                "--recommended-file",
                "src/auth/session.ts",
                "--recommended-test",
                "tests/auth/session.test.ts",
                "--recommended-command",
                "pnpm test tests/auth/session.test.ts",
                "--read-file",
                "src/auth/session.ts",
                "--edited-file",
                "src/auth/session.ts",
                "--tested-file",
                "tests/auth/session.test.ts",
                "--tested-command",
                "pnpm test tests/auth/session.test.ts",
                "--tag",
                "accepted_fix",
                "--format",
                "json",
                "--repo",
            ])
            .arg(&fixture.repo)
            .assert(),
    );

    assert_eq!(record["event"]["taskHash"], "task-hash-1");
    assert_eq!(record["event"]["outcome"], "passed");
    assert_eq!(record["event"]["sourceTextLogged"], false);
    assert_eq!(record["status"]["status"], "written");
    assert_no_source_or_prompt_text(&record);

    let events = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["eval", "feedback", "list", "--format", "json", "--repo"])
            .arg(&fixture.repo)
            .assert(),
    );
    assert_eq!(events.as_array().unwrap().len(), 1);
    assert_eq!(events[0]["readFiles"], json!(["src/auth/session.ts"]));
    assert_no_source_or_prompt_text(&events);

    let summary = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["eval", "feedback", "summary", "--format", "json", "--repo"])
            .arg(&fixture.repo)
            .assert(),
    );
    assert_eq!(summary["eventCount"], 1);
    assert_eq!(summary["passedCount"], 1);
    assert_eq!(summary["readFileCount"], 1);
    assert_eq!(summary["sourceTextLogged"], false);
}

#[test]
fn eval_features_exports_and_manages_source_free_rows() {
    let fixture = fixture_repo();
    let export = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args([
                "eval",
                "features",
                "export",
                "fix requireSession auth",
                "--format",
                "json",
                "--repo",
            ])
            .arg(&fixture.repo)
            .assert(),
    );
    let export_id = export["export"]["exportId"].as_str().unwrap().to_string();
    assert_eq!(export["export"]["schemaVersion"], 1);
    assert_eq!(export["export"]["sourceTextLogged"], false);
    assert_eq!(export["export"]["privacyStatus"]["localOnly"], true);
    assert!(export["export"]["rows"].as_array().unwrap().len() > 1);
    assert!(export["storedPath"].as_str().unwrap().ends_with(".json"));
    assert_no_source_or_prompt_text(&export);
    let rendered = serde_json::to_string(&export).unwrap();
    assert!(!rendered.contains("auth required"));
    assert!(!rendered.contains("token:${userId}"));

    let listed = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["eval", "features", "list", "--format", "json", "--repo"])
            .arg(&fixture.repo)
            .assert(),
    );
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(listed[0]["exportId"], export_id);

    let inspected = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args([
                "eval", "features", "inspect", &export_id, "--format", "json", "--repo",
            ])
            .arg(&fixture.repo)
            .assert(),
    );
    assert_eq!(inspected["exportId"], export_id);
    assert_no_source_or_prompt_text(&inspected);

    let comparison = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args([
                "eval",
                "features",
                "compare",
                "--base-export",
                &export_id,
                "--head-export",
                &export_id,
                "--format",
                "json",
                "--repo",
            ])
            .arg(&fixture.repo)
            .assert(),
    );
    assert_eq!(comparison["rowCountDelta"], 0);
    assert_eq!(comparison["sourceTextLogged"], false);

    let deleted = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args([
                "eval", "features", "delete", &export_id, "--yes", "--format", "json", "--repo",
            ])
            .arg(&fixture.repo)
            .assert(),
    );
    assert_eq!(deleted["deleted"], true);
}

#[test]
fn eval_policy_and_outcome_reports_are_source_free() {
    let fixture = fixture_repo();
    record_feedback_event(&fixture, "task-1", "brief", "passed");
    record_feedback_event(&fixture, "task-2", "standard", "blocked");

    let report = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["eval", "policy", "report", "--format", "json", "--repo"])
            .arg(&fixture.repo)
            .assert(),
    );
    assert_eq!(report["eventCount"], 2);
    assert!(report["contextPrecision"].as_f64().unwrap() >= 0.0);
    assert!(report["signalContributions"].is_array());
    assert_no_source_or_prompt_text(&report);

    let export = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args([
                "eval",
                "features",
                "export",
                "fix requireSession auth",
                "--format",
                "json",
                "--repo",
            ])
            .arg(&fixture.repo)
            .assert(),
    );
    assert_eq!(export["export"]["sourceTextLogged"], false);

    let learned = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args([
                "eval",
                "policy",
                "learn",
                "--min-gold-or-selected-rows",
                "0",
                "--format",
                "json",
                "--repo",
            ])
            .arg(&fixture.repo)
            .assert(),
    );
    let learned_id = learned["id"].as_str().unwrap().to_string();
    assert!(learned_id.starts_with("learned-policy-"));
    assert_eq!(learned["profileSchemaVersion"], 2);
    assert_eq!(learned["status"], "candidate");
    assert_eq!(learned["defaultEligible"], true);
    assert!(learned["trainingSources"].as_array().unwrap().len() >= 2);
    assert!(learned["metricSummary"]
        .as_array()
        .unwrap()
        .iter()
        .any(|metric| metric["metric"] == "feature_export_rows"
            && metric["value"].as_f64().unwrap() > 0.0));
    assert!(learned["baselineThresholds"]
        .as_array()
        .unwrap()
        .iter()
        .all(|threshold| threshold["passed"].as_bool().unwrap()));
    assert_no_source_or_prompt_text(&learned);

    let learned_apply = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args([
                "eval",
                "policy",
                "apply",
                &learned_id,
                "--format",
                "json",
                "--repo",
            ])
            .arg(&fixture.repo)
            .assert(),
    );
    assert_eq!(learned_apply["activeProfileId"], learned_id);
    let learned_rollback = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["eval", "policy", "rollback", "--format", "json", "--repo"])
            .arg(&fixture.repo)
            .assert(),
    );
    assert_eq!(learned_rollback["profileId"], learned_id);

    let blocked_learned = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args([
                "eval",
                "policy",
                "learn",
                "--min-context-precision",
                "2.0",
                "--format",
                "json",
                "--repo",
            ])
            .arg(&fixture.repo)
            .assert(),
    );
    let blocked_id = blocked_learned["id"].as_str().unwrap().to_string();
    assert_eq!(blocked_learned["defaultEligible"], false);
    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args([
            "eval",
            "policy",
            "apply",
            &blocked_id,
            "--format",
            "json",
            "--repo",
        ])
        .arg(&fixture.repo)
        .assert()
        .failure()
        .stderr(contains("not eligible"));

    let profile = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["eval", "policy", "tune", "--format", "json", "--repo"])
            .arg(&fixture.repo)
            .assert(),
    );
    let profile_id = profile["id"].as_str().unwrap().to_string();
    assert_eq!(profile["status"], "candidate");
    assert!(profile["weights"].is_array());
    assert!(profile["safetyFloors"].is_array());
    assert_no_source_or_prompt_text(&profile);

    let apply = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args([
                "eval",
                "policy",
                "apply",
                &profile_id,
                "--format",
                "json",
                "--repo",
            ])
            .arg(&fixture.repo)
            .assert(),
    );
    assert_eq!(apply["action"], "apply");
    assert_eq!(apply["activeProfileId"], profile_id);

    let rollback = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["eval", "policy", "rollback", "--format", "json", "--repo"])
            .arg(&fixture.repo)
            .assert(),
    );
    assert_eq!(rollback["action"], "rollback");
    assert!(rollback["activeProfileId"].is_null());

    let outcome = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["eval", "outcome", "compare", "--format", "json", "--repo"])
            .arg(&fixture.repo)
            .assert(),
    );
    assert_eq!(outcome["eventCount"], 2);
    assert!(outcome["budgets"].as_array().unwrap().len() >= 2);
    assert_no_source_or_prompt_text(&outcome);
}

#[test]
fn eval_agent_run_renders_source_free_report() {
    let fixture = fixture_repo();
    let report_path = fixture.temp.path().join("agent-run.json");
    fs::write(
        &report_path,
        json!({
            "schemaVersion": "ctxhelm-agent-run-eval-v1",
            "status": "passed",
            "client": {
                "name": "claude",
                "version": "Claude Code test"
            },
            "ctxhelmVersion": "ctxhelm 2.4.6",
            "repo": {
                "label": "fixture",
                "pathSha256": "repo-hash"
            },
            "task": {
                "taskSha256": "task-hash",
                "rawTaskStored": false
            },
            "clientAvailability": {
                "tinyPromptAvailable": true,
                "tinyPromptStatus": "passed",
                "pairedSuiteAvailable": true,
                "rateLimited": false,
                "clientFailureObserved": false,
                "comparableLaneCount": 1,
                "availabilityBlocker": null
            },
            "lanes": [
                {
                    "lane": "baseline",
                    "status": "passed",
                    "evaluationStatus": "eligible",
                    "evaluationEligible": true,
                    "metrics": {
                        "targetCoverage": 0.5,
                        "targetReadCoverage": 0.5,
                        "targetReadCount": 1,
                        "targetReadPrecision": 0.25,
                        "targetDiscoveredOnlyCount": 0,
                        "missedTargetCount": 1,
                        "readFileCount": 4,
                        "irrelevantReadCount": 2,
                        "irrelevantReadRate": 0.5,
                        "readsPerTargetRead": 4.0,
                        "toolCallCount": 6,
                        "ctxhelmToolCallCount": 0,
                        "forbiddenToolCallCount": 0,
                        "requiredCtxhelmCallCount": 0,
                        "observedRequiredCtxhelmCallCount": 0,
                        "missingRequiredCtxhelmCallCount": 0,
                        "invalidRequiredCtxhelmCallCount": 0,
                        "ctxhelmEvidenceFileCount": 0,
                        "ctxhelmEvidenceTargetHitCount": 0,
                        "ctxhelmEvidenceOnlyTargetCount": 0,
                        "ctxhelmEvidenceMissedTargetCount": 0
                    },
                    "requiredCtxhelmCallSpecs": [],
                    "requiredCtxhelmCalls": [],
                    "observedRequiredCtxhelmCalls": [],
                    "missingRequiredCtxhelmCalls": [],
                    "invalidRequiredCtxhelmCalls": [],
                    "ctxhelmCallCompliance": "not_required",
                    "clientFailureKind": null,
                    "clientApiErrorStatus": null,
                    "rateLimitObserved": false,
                    "readRoleCounts": {
                        "source": 2,
                        "test": 1,
                        "docs": 1
                    },
                    "missedTargetRoleCounts": {
                        "docs": 1
                    },
                    "ctxhelmEvidenceFiles": [],
                    "ctxhelmEvidenceTargetHits": [],
                    "ctxhelmEvidenceOnlyTargets": [],
                    "ctxhelmEvidenceMissedTargets": [],
                    "sourceTextLogged": false,
                    "rawPromptStored": false,
                    "rawTranscriptStored": false,
                    "rawMcpTrafficStored": false
                },
                {
                    "lane": "ctxhelm-brief",
                    "status": "passed",
                    "evaluationStatus": "eligible",
                    "evaluationEligible": true,
                    "metrics": {
                        "targetCoverage": 1.0,
                        "targetReadCoverage": 1.0,
                        "targetReadCount": 2,
                        "targetReadPrecision": 1.0,
                        "targetDiscoveredOnlyCount": 0,
                        "missedTargetCount": 0,
                        "readFileCount": 2,
                        "irrelevantReadCount": 0,
                        "irrelevantReadRate": 0.0,
                        "readsPerTargetRead": 1.0,
                        "toolCallCount": 5,
                        "ctxhelmToolCallCount": 2,
                        "forbiddenToolCallCount": 0,
                        "requiredCtxhelmCallCount": 2,
                        "observedRequiredCtxhelmCallCount": 2,
                        "missingRequiredCtxhelmCallCount": 0,
                        "invalidRequiredCtxhelmCallCount": 0,
                        "ctxhelmEvidenceFileCount": 3,
                        "ctxhelmEvidenceTargetHitCount": 2,
                        "ctxhelmEvidenceOnlyTargetCount": 0,
                        "ctxhelmEvidenceMissedTargetCount": 0
                    },
                    "requiredCtxhelmCallSpecs": [
                        {
                            "name": "prepare_task",
                            "requiresRepo": true,
                            "requiresTask": true
                        },
                        {
                            "name": "get_pack",
                            "requiresRepo": true,
                            "requiresTask": true,
                            "budget": "brief",
                            "format": "json",
                            "recordTrace": false
                        }
                    ],
                    "requiredCtxhelmCalls": ["prepare_task", "get_pack"],
                    "observedRequiredCtxhelmCalls": ["prepare_task", "get_pack"],
                    "missingRequiredCtxhelmCalls": [],
                    "invalidRequiredCtxhelmCalls": [],
                    "ctxhelmCallCompliance": "satisfied",
                    "retry": {
                        "eligible": true,
                        "triggered": true,
                        "selected": true,
                        "evidenceOnlyTargetCountBeforeRetry": 1,
                        "evidenceOnlyTargetCountAfterRetry": 0,
                        "readFileCountBeforeRetry": 3,
                        "readFileCountAfterRetry": 2,
                        "irrelevantReadCountBeforeRetry": 1,
                        "irrelevantReadCountAfterRetry": 0,
                        "targetReadCoverageBeforeRetry": 0.5,
                        "targetReadCoverageAfterRetry": 1.0,
                        "readFileCountDelta": -1,
                        "irrelevantReadCountDelta": -1,
                        "targetReadCoverageDelta": 0.5
                    },
                    "clientFailureKind": null,
                    "clientApiErrorStatus": null,
                    "rateLimitObserved": false,
                    "readRoleCounts": {
                        "source": 1,
                        "docs": 1
                    },
                    "missedTargetRoleCounts": {},
                    "ctxhelmEvidenceFiles": ["src/lib.rs", "docs/auth.md", "tests/auth.test.ts"],
                    "ctxhelmEvidenceTargetHits": ["src/lib.rs", "docs/auth.md"],
                    "ctxhelmEvidenceOnlyTargets": [],
                    "ctxhelmEvidenceMissedTargets": [],
                    "sourceTextLogged": false,
                    "rawPromptStored": false,
                    "rawTranscriptStored": false,
                    "rawMcpTrafficStored": false
                }
            ],
            "comparison": {
                "baselineLane": "baseline",
                "bestLane": "ctxhelm-brief",
                "comparisonEligible": true,
                "baselineEligible": true,
                "comparableCtxhelmLaneCount": 1,
                "targetCoverageDelta": 0.5,
                "targetReadCoverageDelta": 0.5,
                "irrelevantReadDelta": 2,
                "ctxhelmToolCallsObserved": true,
                "forbiddenToolCallsObserved": false,
                "missingRequiredCtxhelmCallsObserved": false,
                "missingRequiredCtxhelmCalls": {},
                "invalidRequiredCtxhelmCallsObserved": false,
                "invalidRequiredCtxhelmCalls": {},
                "clientFailuresObserved": false,
                "rateLimitsObserved": false,
                "ctxhelmEvidenceMissesObserved": false,
                "ctxhelmEvidenceMisses": {},
                "ctxhelmEvidenceOnlyTargetsObserved": false,
                "ctxhelmEvidenceOnlyTargets": {},
                "ctxhelmUnderReadTargetsObserved": false,
                "retryCost": {
                    "retryTriggeredLanes": 1,
                    "retrySelectedLanes": 1,
                    "avgReadFilesBeforeRetry": 3.0,
                    "avgReadFilesAfterRetry": 2.0,
                    "avgIrrelevantReadsBeforeRetry": 1.0,
                    "avgIrrelevantReadsAfterRetry": 0.0,
                    "targetReadCoverageBeforeRetry": 0.5,
                    "targetReadCoverageAfterRetry": 1.0,
                    "evidenceOnlyTargetsBeforeRetry": 1,
                    "evidenceOnlyTargetsAfterRetry": 0
                },
                "readEfficiency": {
                    "analysisAvailable": true,
                    "baselineLane": "baseline",
                    "efficientCtxhelmLane": "ctxhelm-brief",
                    "baselineTargetReadCoverage": 0.5,
                    "efficientTargetReadCoverage": 1.0,
                    "targetReadCoverageDelta": 0.5,
                    "baselineReadFileCount": 4,
                    "efficientReadFileCount": 2,
                    "extraReadFileCount": -2,
                    "baselineIrrelevantReadCount": 2,
                    "efficientIrrelevantReadCount": 0,
                    "extraIrrelevantReadCount": -2,
                    "baselineTargetReadPrecision": 0.25,
                    "efficientTargetReadPrecision": 1.0,
                    "targetReadPrecisionDelta": 0.75,
                    "baselineIrrelevantReadRate": 0.5,
                    "efficientIrrelevantReadRate": 0.0,
                    "irrelevantReadRateDelta": -0.5,
                    "recoveredTargetReadCount": 1,
                    "extraReadsPerRecoveredTarget": -2.0,
                    "extraIrrelevantReadsPerRecoveredTarget": -2.0
                },
                "outcomeClaim": "ctxhelm_improved",
                "recommendedResearchActions": [
                    {
                        "action": "preserve_current_agent_contract",
                        "priority": 3,
                        "reason": "Comparable lanes produced stable source-free outcome evidence."
                    }
                ]
            },
            "privacyStatus": {
                "localOnly": true,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false,
                "sourceTextLogged": false,
                "rawPromptStored": false,
                "rawTranscriptStored": false,
                "rawMcpTrafficStored": false
            }
        })
        .to_string(),
    )
    .unwrap();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .args(["eval", "--help"])
        .assert()
        .success()
        .stdout(contains("agent-run"));

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .args(["eval", "agent-run", "--report"])
        .arg(&report_path)
        .assert()
        .success()
        .stdout(contains("# ctxhelm Agent Run Report"))
        .stdout(contains("ctxhelm-agent-run-eval-v1"))
        .stdout(contains("ctxhelm-brief"))
        .stdout(contains("Tiny prompt available: `true`"))
        .stdout(contains("Paired suite available: `true`"))
        .stdout(contains("Availability blocker: `none`"))
        .stdout(contains("Comparison eligible: `true`"))
        .stdout(contains("Comparable ctxhelm lanes: `1`"))
        .stdout(contains("Missing required ctxhelm calls observed: `false`"))
        .stdout(contains("Invalid required ctxhelm calls observed: `false`"))
        .stdout(contains("Client failures observed: `false`"))
        .stdout(contains("Rate limits observed: `false`"))
        .stdout(contains("ctxhelm evidence misses observed: `false`"))
        .stdout(contains("ctxhelm evidence-only targets observed: `false`"))
        .stdout(contains("Retry cost: triggered `1` selected `1` avg reads before `3.00` after `2.00` avg irrelevant before `1.00` after `0.00` target-read coverage before `0.50` after `1.00` evidence-only targets before `1` after `0`"))
        .stdout(contains("Read efficiency: baseline `baseline` efficient ctxhelm `ctxhelm-brief` target-read coverage `0.50` -> `1.00` read precision `0.25` -> `1.00` irrelevant rate `0.50` -> `0.00` extra reads `-2` extra irrelevant `-2` recovered targets `1` extra reads/recovered `-2.00` extra irrelevant/recovered `-2.00`"))
        .stdout(contains(
            "Recommended R&D actions: `preserve_current_agent_contract(p3)`",
        ))
        .stdout(contains("target coverage `1.00`"))
        .stdout(contains("target read coverage `1.00`"))
        .stdout(contains("read precision `1.00` irrelevant rate `0.00` reads/target `1.00`"))
        .stdout(contains("evaluation `eligible` eligible `true`"))
        .stdout(contains("compliance `satisfied`"))
        .stdout(contains("client failure `none` rate limited `false`"))
        .stdout(contains("missed targets `0`"))
        .stdout(contains("ctxhelm evidence files `3` evidence target hits `2` evidence-only targets `0` evidence misses `0`"))
        .stdout(contains("retry eligible `true` triggered `true` selected `true` evidence-only before `1` after `0` read delta `-1` irrelevant delta `-1` target-read delta `0.50`"))
        .stdout(contains("read roles `docs=1, source=1`"))
        .stdout(contains("ctxhelm calls `2`"))
        .stdout(contains("forbidden calls `0`"));

    let rendered_json = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .args(["eval", "agent-run", "--report"])
            .arg(&report_path)
            .args(["--format", "json"])
            .assert(),
    );
    assert_eq!(rendered_json["task"]["rawTaskStored"], false);
    assert_eq!(rendered_json["privacyStatus"]["sourceTextLogged"], false);
    assert_eq!(rendered_json["privacyStatus"]["rawTranscriptStored"], false);
    assert_eq!(
        rendered_json["comparison"]["outcomeClaim"],
        "ctxhelm_improved"
    );
    assert_eq!(
        rendered_json["comparison"]["retryCost"]["retryTriggeredLanes"],
        1
    );
    assert_eq!(
        rendered_json["clientAvailability"]["pairedSuiteAvailable"],
        true
    );
}

#[test]
fn eval_agent_run_renders_invalid_required_ctxhelm_call_reasons() {
    let fixture = fixture_repo();
    let report_path = fixture.temp.path().join("agent-run-invalid.json");
    fs::write(
        &report_path,
        json!({
            "schemaVersion": "ctxhelm-agent-run-eval-v1",
            "status": "degraded",
            "client": {
                "name": "claude",
                "version": "Claude Code test"
            },
            "repo": {
                "label": "fixture",
                "pathSha256": "repo-hash"
            },
            "lanes": [
                {
                    "lane": "ctxhelm-plan",
                    "status": "passed",
                    "evaluationStatus": "not_comparable",
                    "evaluationEligible": false,
                    "metrics": {
                        "targetCoverage": 0.0,
                        "targetReadCoverage": 0.0,
                        "targetReadCount": 0,
                        "targetDiscoveredOnlyCount": 0,
                        "missedTargetCount": 1,
                        "readFileCount": 0,
                        "irrelevantReadCount": 0,
                        "toolCallCount": 1,
                        "ctxhelmToolCallCount": 1,
                        "forbiddenToolCallCount": 0,
                        "requiredCtxhelmCallCount": 1,
                        "observedRequiredCtxhelmCallCount": 0,
                        "missingRequiredCtxhelmCallCount": 1,
                        "invalidRequiredCtxhelmCallCount": 1,
                        "ctxhelmEvidenceFileCount": 1,
                        "ctxhelmEvidenceTargetHitCount": 0,
                        "ctxhelmEvidenceOnlyTargetCount": 0,
                        "ctxhelmEvidenceMissedTargetCount": 1
                    },
                    "requiredCtxhelmCalls": ["prepare_task"],
                    "observedRequiredCtxhelmCalls": [],
                    "missingRequiredCtxhelmCalls": ["prepare_task"],
                    "invalidRequiredCtxhelmCalls": [
                        {
                            "name": "prepare_task",
                            "reasons": ["repo", "task"],
                            "attemptCount": 1
                        }
                    ],
                    "ctxhelmCallCompliance": "invalid",
                    "clientFailureKind": null,
                    "rateLimitObserved": false,
                    "readRoleCounts": {},
                    "missedTargetRoleCounts": {
                        "source": 1
                    },
                    "ctxhelmEvidenceFiles": ["docs/auth.md"],
                    "ctxhelmEvidenceTargetHits": [],
                    "ctxhelmEvidenceOnlyTargets": [],
                    "ctxhelmEvidenceMissedTargets": ["src/lib.rs"]
                }
            ],
            "comparison": {
                "baselineLane": "baseline",
                "bestLane": "ctxhelm-plan",
                "comparisonEligible": false,
                "baselineEligible": false,
                "comparableCtxhelmLaneCount": 0,
                "targetCoverageDelta": 0.0,
                "targetReadCoverageDelta": 0.0,
                "irrelevantReadDelta": 0,
                "ctxhelmToolCallsObserved": true,
                "forbiddenToolCallsObserved": false,
                "missingRequiredCtxhelmCallsObserved": true,
                "missingRequiredCtxhelmCalls": {
                    "ctxhelm-plan": ["prepare_task"]
                },
                "invalidRequiredCtxhelmCallsObserved": true,
                "invalidRequiredCtxhelmCalls": {
                    "ctxhelm-plan": [
                        {
                            "name": "prepare_task",
                            "reasons": ["repo", "task"],
                            "attemptCount": 1
                        }
                    ]
                },
                "clientFailuresObserved": false,
                "rateLimitsObserved": false,
                "ctxhelmEvidenceMissesObserved": true,
                "ctxhelmEvidenceMisses": {
                    "ctxhelm-plan": ["src/lib.rs"]
                },
                "ctxhelmEvidenceOnlyTargetsObserved": false,
                "ctxhelmEvidenceOnlyTargets": {},
                "ctxhelmUnderReadTargetsObserved": false,
                "outcomeClaim": "insufficient_comparable_lanes",
                "recommendedResearchActions": [
                    {
                        "action": "harden_required_ctxhelm_call_guidance",
                        "priority": 1,
                        "reason": "A ctxhelm-assisted lane did not make all required source-free ctxhelm calls."
                    },
                    {
                        "action": "fix_retrieval_or_query_construction",
                        "priority": 1,
                        "reason": "ctxhelm evidence did not surface at least one expected target."
                    }
                ]
            },
            "privacyStatus": {
                "sourceTextLogged": false
            }
        })
        .to_string(),
    )
    .unwrap();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .args(["eval", "agent-run", "--report"])
        .arg(&report_path)
        .assert()
        .success()
        .stdout(contains("Invalid required ctxhelm calls observed: `true`"))
        .stdout(contains(
            "Invalid required ctxhelm calls: `ctxhelm-plan=prepare_task[repo, task; attempts=1]`",
        ))
        .stdout(contains("compliance `invalid`"))
        .stdout(contains(
            "ctxhelm evidence misses: `ctxhelm-plan=src/lib.rs`",
        ))
        .stdout(contains(
            "Recommended R&D actions: `harden_required_ctxhelm_call_guidance(p1), fix_retrieval_or_query_construction(p1)`",
        ))
        .stdout(contains(
            "invalid required `prepare_task[repo, task; attempts=1]`",
        ));
}

#[test]
fn eval_agent_run_renders_source_free_suite_report() {
    let fixture = fixture_repo();
    let report_path = fixture.temp.path().join("agent-run-suite.json");
    fs::write(
        &report_path,
        json!({
            "schemaVersion": "ctxhelm-agent-run-eval-v1",
            "status": "passed",
            "workflowKind": "paired-agent-context-suite",
            "client": {
                "name": "claude",
                "version": "Claude Code test"
            },
            "ctxhelmVersion": "ctxhelm 2.4.6",
            "repo": {
                "label": "fixture",
                "pathSha256": "repo-hash"
            },
            "suite": {
                "suiteSha256": "suite-hash",
                "rawTasksStored": false,
                "taskCount": 2
            },
            "clientAvailability": {
                "tinyPromptAvailable": true,
                "tinyPromptAvailableCount": 1,
                "pairedSuiteAvailable": false,
                "pairedSuiteAvailableCount": 1,
                "rateLimited": true,
                "clientFailureObserved": true,
                "comparableLaneCount": 1,
                "availabilityBlocker": "rate_limit",
                "availabilityBlockers": ["rate_limit"]
            },
            "tasks": [
                {
                    "taskId": "task-1",
                    "status": "passed",
                    "taskSha256": "task-hash-1",
                    "targetFiles": ["src/lib.rs"],
                    "comparison": {
                        "comparisonEligible": true,
                        "comparableCtxhelmLaneCount": 1,
                        "targetCoverageDelta": 0.5,
                        "targetReadCoverageDelta": 0.5,
                        "irrelevantReadDelta": 1,
                        "ctxhelmToolCallsObserved": true,
                        "forbiddenToolCallsObserved": false,
                        "missingRequiredCtxhelmCallsObserved": false,
                        "invalidRequiredCtxhelmCallsObserved": false,
                        "clientFailuresObserved": false,
                        "rateLimitsObserved": false,
                        "ctxhelmEvidenceMissesObserved": false,
                        "ctxhelmEvidenceOnlyTargetsObserved": true,
                        "ctxhelmUnderReadTargetsObserved": false
                    },
                    "lanes": [],
                    "privacyStatus": {
                        "sourceTextLogged": false
                    }
                }
            ],
            "aggregate": {
                "taskCount": 2,
                "targetCoverageDeltaAverage": 0.25,
                "targetReadCoverageDeltaAverage": 0.25,
                "irrelevantReadDeltaSum": 3,
                "comparisonEligibleCount": 1,
                "comparableCtxhelmLaneCount": 1,
                "ctxhelmToolCallsObserved": true,
                "forbiddenToolCallsObserved": false,
                "missingRequiredCtxhelmCallsObserved": false,
                "invalidRequiredCtxhelmCallsObserved": false,
                "clientFailuresObserved": false,
                "rateLimitsObserved": false,
                "ctxhelmEvidenceMissesObserved": false,
                "ctxhelmEvidenceOnlyTargetsObserved": true,
                "ctxhelmUnderReadTargetsObserved": false,
                "retryCost": {
                    "retryTriggeredLanes": 1,
                    "retrySelectedLanes": 1,
                    "avgReadFilesBeforeRetry": 4.0,
                    "avgReadFilesAfterRetry": 5.0,
                    "avgIrrelevantReadsBeforeRetry": 2.0,
                    "avgIrrelevantReadsAfterRetry": 1.0,
                    "targetReadCoverageBeforeRetry": 0.5,
                    "targetReadCoverageAfterRetry": 0.75,
                    "evidenceOnlyTargetsBeforeRetry": 1,
                    "evidenceOnlyTargetsAfterRetry": 0
                },
                "readEfficiency": {
                    "analysisAvailable": true,
                    "baselineLane": "baseline",
                    "efficientCtxhelmLane": "ctxhelm-brief",
                    "baselineTargetReadCoverage": 0.5,
                    "efficientTargetReadCoverage": 0.75,
                    "targetReadCoverageDelta": 0.25,
                    "baselineReadFileCount": 8,
                    "efficientReadFileCount": 5,
                    "extraReadFileCount": -3,
                    "baselineIrrelevantReadCount": 4,
                    "efficientIrrelevantReadCount": 1,
                    "extraIrrelevantReadCount": -3,
                    "baselineTargetReadPrecision": 0.25,
                    "efficientTargetReadPrecision": 0.6,
                    "targetReadPrecisionDelta": 0.35,
                    "baselineIrrelevantReadRate": 0.5,
                    "efficientIrrelevantReadRate": 0.2,
                    "irrelevantReadRateDelta": -0.3,
                    "recoveredTargetReadCount": 1,
                    "extraReadsPerRecoveredTarget": -3.0,
                    "extraIrrelevantReadsPerRecoveredTarget": -3.0
                },
                "outcomeClaim": "ctxhelm_improved",
                "recommendedResearchActions": [
                    {
                        "action": "improve_agent_consumption_guidance",
                        "priority": 2,
                        "reason": "ctxhelm surfaced expected targets that the agent did not consume with native reads."
                    }
                ],
                "laneSummaries": [
                    {
                        "lane": "baseline",
                        "taskCount": 2,
                        "passedCount": 2,
                        "evaluationEligibleCount": 2,
                        "averageTargetCoverage": 0.5,
                        "averageTargetReadCoverage": 0.5,
                        "targetReadCount": 2,
                        "targetReadPrecision": 0.25,
                        "irrelevantReadRate": 0.5,
                        "readsPerTargetRead": 4.0,
                        "targetDiscoveredOnlyCount": 1,
                        "missedTargetCount": 1,
                        "readFileCount": 8,
                        "irrelevantReadCount": 4,
                        "toolCallCount": 12,
                        "ctxhelmToolCallCount": 0,
                        "forbiddenToolCallCount": 0,
                        "requiredCtxhelmCallCount": 0,
                        "observedRequiredCtxhelmCallCount": 0,
                        "missingRequiredCtxhelmCallCount": 0,
                        "invalidRequiredCtxhelmCallCount": 0,
                        "clientFailureCount": 0,
                        "rateLimitCount": 0,
                        "ctxhelmEvidenceFileCount": 0,
                        "ctxhelmEvidenceTargetHitCount": 0,
                        "ctxhelmEvidenceOnlyTargetCount": 0,
                        "ctxhelmEvidenceMissedTargetCount": 0,
                        "readRoleCounts": {
                            "source": 4,
                            "test": 2,
                            "docs": 2
                        },
                        "missedTargetRoleCounts": {
                            "docs": 1
                        }
                    },
                    {
                        "lane": "ctxhelm-brief",
                        "taskCount": 2,
                        "passedCount": 2,
                        "evaluationEligibleCount": 2,
                        "averageTargetCoverage": 0.75,
                        "averageTargetReadCoverage": 0.75,
                        "targetReadCount": 3,
                        "targetReadPrecision": 0.6,
                        "irrelevantReadRate": 0.2,
                        "readsPerTargetRead": 1.6666666667,
                        "targetDiscoveredOnlyCount": 0,
                        "missedTargetCount": 1,
                        "readFileCount": 5,
                        "irrelevantReadCount": 1,
                        "toolCallCount": 10,
                        "ctxhelmToolCallCount": 4,
                        "forbiddenToolCallCount": 0,
                        "requiredCtxhelmCallCount": 4,
                        "observedRequiredCtxhelmCallCount": 4,
                        "missingRequiredCtxhelmCallCount": 0,
                        "invalidRequiredCtxhelmCallCount": 0,
                        "clientFailureCount": 0,
                        "rateLimitCount": 0,
                        "ctxhelmEvidenceFileCount": 6,
                        "ctxhelmEvidenceTargetHitCount": 4,
                        "ctxhelmEvidenceOnlyTargetCount": 1,
                        "ctxhelmEvidenceMissedTargetCount": 0,
                        "readRoleCounts": {
                            "source": 3,
                            "docs": 2
                        },
                        "missedTargetRoleCounts": {
                            "test": 1
                        }
                    }
                ]
            },
            "privacyStatus": {
                "localOnly": true,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false,
                "sourceTextLogged": false,
                "rawPromptStored": false,
                "rawTranscriptStored": false,
                "rawMcpTrafficStored": false
            }
        })
        .to_string(),
    )
    .unwrap();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .args(["eval", "agent-run", "--report"])
        .arg(&report_path)
        .assert()
        .success()
        .stdout(contains("## Suite Aggregate"))
        .stdout(contains("Tiny prompt available: `true`"))
        .stdout(contains("Paired suite available: `false`"))
        .stdout(contains("Availability blocker: `rate_limit`"))
        .stdout(contains("Availability rate-limited: `true`"))
        .stdout(contains("Comparison eligible tasks: `1`"))
        .stdout(contains("Comparable ctxhelm lanes: `1`"))
        .stdout(contains("Missing required ctxhelm calls observed: `false`"))
        .stdout(contains("Invalid required ctxhelm calls observed: `false`"))
        .stdout(contains("Client failures observed: `false`"))
        .stdout(contains("Rate limits observed: `false`"))
        .stdout(contains("ctxhelm evidence misses observed: `false`"))
        .stdout(contains("ctxhelm evidence-only targets observed: `true`"))
        .stdout(contains(
            "Recommended R&D actions: `improve_agent_consumption_guidance(p2)`",
        ))
        .stdout(contains("Target coverage delta average: `0.25`"))
        .stdout(contains("Target read coverage delta average: `0.25`"))
        .stdout(contains("Irrelevant read delta sum: `3`"))
        .stdout(contains("Retry cost: triggered `1` selected `1` avg reads before `4.00` after `5.00` avg irrelevant before `2.00` after `1.00` target-read coverage before `0.50` after `0.75` evidence-only targets before `1` after `0`"))
        .stdout(contains("Read efficiency: baseline `baseline` efficient ctxhelm `ctxhelm-brief` target-read coverage `0.50` -> `0.75` read precision `0.25` -> `0.60` irrelevant rate `0.50` -> `0.20` extra reads `-3` extra irrelevant `-3` recovered targets `1` extra reads/recovered `-3.00` extra irrelevant/recovered `-3.00`"))
        .stdout(contains("## Suite Lanes"))
        .stdout(contains("ctxhelm-brief"))
        .stdout(contains("eligible `2`"))
        .stdout(contains("avg target read coverage `0.75`"))
        .stdout(contains("read precision `0.60` irrelevant rate `0.20` reads/target `1.67`"))
        .stdout(contains(
            "required ctxhelm calls `4` observed required `4` missing required `0` invalid required `0`",
        ))
        .stdout(contains("client failures `0` rate limits `0`"))
        .stdout(contains("ctxhelm evidence files `6` evidence target hits `4` evidence-only targets `1` evidence misses `0`"))
        .stdout(contains("read roles `docs=2, source=3`"))
        .stdout(contains("forbidden calls `0`"));

    let rendered_json = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .args(["eval", "agent-run", "--report"])
            .arg(&report_path)
            .args(["--format", "json"])
            .assert(),
    );
    assert_eq!(rendered_json["suite"]["rawTasksStored"], false);
    assert_eq!(rendered_json["privacyStatus"]["sourceTextLogged"], false);
    assert_eq!(
        rendered_json["aggregate"]["outcomeClaim"],
        "ctxhelm_improved"
    );
    assert_eq!(
        rendered_json["aggregate"]["retryCost"]["evidenceOnlyTargetsAfterRetry"],
        0
    );
    assert_eq!(
        rendered_json["aggregate"]["readEfficiency"]["efficientCtxhelmLane"],
        "ctxhelm-brief"
    );
    assert_eq!(
        rendered_json["clientAvailability"]["availabilityBlocker"],
        "rate_limit"
    );
}

#[test]
fn inspector_proof_summarizes_agent_run_report_source_free() {
    let fixture = fixture_repo();
    let report_path = fixture.temp.path().join("agent-run-suite.json");
    fs::write(
        &report_path,
        json!({
            "schemaVersion": "ctxhelm-agent-run-eval-v1",
            "status": "passed",
            "workflowKind": "paired-agent-context-suite",
            "client": {
                "name": "codex",
                "version": "Codex test"
            },
            "clientAvailability": {
                "tinyPromptAvailable": true,
                "pairedSuiteAvailable": true,
                "rateLimited": false,
                "clientFailureObserved": false,
                "comparableLaneCount": 1,
                "availabilityBlocker": null
            },
            "aggregate": {
                "taskCount": 2,
                "comparisonEligibleCount": 2,
                "comparableCtxhelmLaneCount": 1,
                "targetCoverageDeltaAverage": 0.25,
                "targetReadCoverageDeltaAverage": 0.25,
                "irrelevantReadDeltaSum": -3,
                "ctxhelmToolCallsObserved": true,
                "forbiddenToolCallsObserved": false,
                "missingRequiredCtxhelmCallsObserved": false,
                "invalidRequiredCtxhelmCallsObserved": false,
                "clientFailuresObserved": false,
                "rateLimitsObserved": false,
                "ctxhelmEvidenceMissesObserved": false,
                "ctxhelmEvidenceOnlyTargetsObserved": true,
                "ctxhelmUnderReadTargetsObserved": false,
                "retryCost": {
                    "retryTriggeredLanes": 1,
                    "retrySelectedLanes": 1,
                    "avgReadFilesBeforeRetry": 4.0,
                    "avgReadFilesAfterRetry": 5.0,
                    "avgIrrelevantReadsBeforeRetry": 2.0,
                    "avgIrrelevantReadsAfterRetry": 1.0,
                    "targetReadCoverageBeforeRetry": 0.5,
                    "targetReadCoverageAfterRetry": 0.75,
                    "evidenceOnlyTargetsBeforeRetry": 1,
                    "evidenceOnlyTargetsAfterRetry": 0
                },
                "readEfficiency": {
                    "analysisAvailable": true,
                    "baselineLane": "baseline",
                    "efficientCtxhelmLane": "ctxhelm-brief",
                    "efficientTargetReadCoverage": 0.75,
                    "efficientTargetReadPrecision": 0.6,
                    "efficientIrrelevantReadCount": 1
                },
                "outcomeClaim": "ctxhelm_improved",
                "laneSummaries": [
                    {
                        "lane": "baseline",
                        "averageTargetReadCoverage": 0.5,
                        "targetReadPrecision": 0.25,
                        "irrelevantReadCount": 4,
                        "ctxhelmEvidenceOnlyTargetCount": 0
                    },
                    {
                        "lane": "ctxhelm-brief",
                        "averageTargetReadCoverage": 0.75,
                        "targetReadPrecision": 0.6,
                        "irrelevantReadCount": 1,
                        "ctxhelmEvidenceOnlyTargetCount": 0
                    }
                ]
            },
            "privacyStatus": {
                "localOnly": true,
                "sourceTextLogged": false,
                "rawPromptStored": false,
                "rawTranscriptStored": false,
                "rawMcpTrafficStored": false,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false
            }
        })
        .to_string(),
    )
    .unwrap();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .args(["inspector", "proof", "--report"])
        .arg(&report_path)
        .assert()
        .success()
        .stdout(contains("# ctxhelm Proof Inspector"))
        .stdout(contains("Report kind: `agent_run_suite`"))
        .stdout(contains("Claim: `ctxhelm_improved`"))
        .stdout(contains("Comparable tasks: `2`"))
        .stdout(contains("Target-read coverage: `0.75`"))
        .stdout(contains("## Client Availability"))
        .stdout(contains("Tiny prompt available: `true`"))
        .stdout(contains("Paired suite available: `true`"))
        .stdout(contains("Availability blocker: `none`"))
        .stdout(contains("Evidence-only targets after retry: `0`"))
        .stdout(contains("Retry cost: triggered `1` selected `1`"))
        .stdout(contains("Forbidden boundary events observed: `false`"))
        .stdout(contains("Source text logged: `false`"))
        .stdout(contains("Summary source-free: `true`"))
        .stdout(contains(
            "Use as source-free outcome evidence, then repeat the suite to check stability and efficiency.",
        ));

    let rendered_json = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .args(["inspector", "proof", "--report"])
            .arg(&report_path)
            .args(["--format", "json"])
            .assert(),
    );
    assert_eq!(rendered_json["schemaVersion"], "ctxhelm-proof-inspector-v1");
    assert_eq!(rendered_json["reportKind"], "agent_run_suite");
    assert_eq!(rendered_json["outcome"]["claim"], "ctxhelm_improved");
    assert_eq!(rendered_json["outcome"]["comparisonEligibleCount"], 2);
    assert_eq!(rendered_json["availability"]["pairedSuiteAvailable"], true);
    assert_eq!(
        rendered_json["evidence"]["evidenceOnlyTargetsAfterRetry"],
        0
    );
    assert_eq!(rendered_json["privacyStatus"]["sourceFreeSummary"], true);
    assert_eq!(
        rendered_json["recommendedNextAction"],
        "Use as source-free outcome evidence, then repeat the suite to check stability and efficiency."
    );
}

#[test]
fn inspector_proof_summarizes_product_proof_report_source_free() {
    let fixture = fixture_repo();
    let report_path = fixture.temp.path().join("product-proof.json");
    fs::write(
        &report_path,
        json!({
            "suiteName": "product-proof-smoke",
            "suiteId": "suite-hash",
            "evaluatedRepositoryCount": 1,
            "evaluatedCommitCount": 3,
            "headlineMetrics": [],
            "releaseGate": {
                "decision": "promote",
                "defaultPromotionAllowed": true,
                "decisionReason": "Promote: source-free proof is clean.",
                "lexicalComparison": {
                    "contextClaim": "beats_all_corpora",
                    "agentEvidenceClaim": "beats_all_corpora",
                    "allFileClaim": "mixed",
                    "averageContextDeltaAt10": 0.20,
                    "averageAgentEvidenceDeltaAt10": 0.30,
                    "averageFileDeltaAt10": 0.10
                },
                "corpusVerdicts": [
                    {
                        "repository": "fixture",
                        "status": "beat",
                        "protectedEvidenceTargetMissRateAt10": 0.0
                    }
                ]
            },
            "privacyStatus": {
                "localOnly": true,
                "sourceTextLogged": false,
                "rawPromptStored": false,
                "rawTranscriptStored": false,
                "rawMcpTrafficStored": false,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false
            }
        })
        .to_string(),
    )
    .unwrap();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .args(["inspector", "proof", "--report"])
        .arg(&report_path)
        .assert()
        .success()
        .stdout(contains("Report kind: `product_proof`"))
        .stdout(contains("Release gate decision: `promote`"))
        .stdout(contains("Default promotion allowed: `true`"))
        .stdout(contains("Context claim: `beats_all_corpora`"))
        .stdout(contains("Max protected target miss-rate@10: `0.00`"))
        .stdout(contains(
            "Use as source-free product-proof evidence, then pair it with real-agent outcome proof before making agent-productivity claims.",
        ))
        .stdout(contains("Source text logged: `false`"));

    let rendered_json = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .args(["inspector", "proof", "--report"])
            .arg(&report_path)
            .args(["--format", "json"])
            .assert(),
    );
    assert_eq!(rendered_json["schemaVersion"], "ctxhelm-proof-inspector-v1");
    assert_eq!(rendered_json["reportKind"], "product_proof");
    assert_eq!(
        rendered_json["productProof"]["releaseGateDecision"],
        "promote"
    );
    assert_eq!(
        rendered_json["productProof"]["maxProtectedTargetMissRateAt10"],
        0.0
    );
    assert_eq!(
        rendered_json["recommendedNextAction"],
        "Use as source-free product-proof evidence, then pair it with real-agent outcome proof before making agent-productivity claims."
    );
}

#[test]
fn inspector_proof_summarizes_multi_report_bundle_source_free() {
    let fixture = fixture_repo();
    let product_report_path = fixture.temp.path().join("product-proof.json");
    fs::write(
        &product_report_path,
        json!({
            "suiteName": "product-proof-smoke",
            "suiteId": "suite-hash",
            "evaluatedRepositoryCount": 1,
            "evaluatedCommitCount": 3,
            "releaseGate": {
                "decision": "promote",
                "defaultPromotionAllowed": true,
                "decisionReason": "Promote: source-free proof is clean.",
                "lexicalComparison": {
                    "contextClaim": "beats_all_corpora",
                    "agentEvidenceClaim": "beats_all_corpora",
                    "allFileClaim": "mixed",
                    "averageContextDeltaAt10": 0.20,
                    "averageAgentEvidenceDeltaAt10": 0.30,
                    "averageFileDeltaAt10": 0.10
                },
                "corpusVerdicts": [
                    {
                        "repository": "fixture",
                        "status": "beat",
                        "protectedEvidenceTargetMissRateAt10": 0.0
                    }
                ]
            },
            "privacyStatus": {
                "localOnly": true,
                "sourceTextLogged": false,
                "rawPromptStored": false,
                "rawTranscriptStored": false,
                "rawMcpTrafficStored": false,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false
            }
        })
        .to_string(),
    )
    .unwrap();
    let agent_report_path = fixture.temp.path().join("agent-run-suite.json");
    fs::write(
        &agent_report_path,
        json!({
            "schemaVersion": "ctxhelm-agent-run-eval-v1",
            "status": "passed",
            "workflowKind": "paired-agent-context-suite",
            "client": {
                "name": "codex",
                "version": "Codex test"
            },
            "aggregate": {
                "taskCount": 1,
                "comparisonEligibleCount": 1,
                "comparableCtxhelmLaneCount": 1,
                "targetCoverageDeltaAverage": 0.25,
                "targetReadCoverageDeltaAverage": 0.25,
                "irrelevantReadDeltaSum": -2,
                "ctxhelmToolCallsObserved": true,
                "forbiddenToolCallsObserved": false,
                "missingRequiredCtxhelmCallsObserved": false,
                "invalidRequiredCtxhelmCallsObserved": false,
                "clientFailuresObserved": false,
                "rateLimitsObserved": false,
                "ctxhelmEvidenceMissesObserved": false,
                "ctxhelmEvidenceOnlyTargetsObserved": true,
                "ctxhelmUnderReadTargetsObserved": false,
                "retryCost": {
                    "retryTriggeredLanes": 1,
                    "retrySelectedLanes": 1,
                    "avgReadFilesBeforeRetry": 3.0,
                    "avgReadFilesAfterRetry": 2.0,
                    "avgIrrelevantReadsBeforeRetry": 1.0,
                    "avgIrrelevantReadsAfterRetry": 0.0,
                    "targetReadCoverageBeforeRetry": 0.5,
                    "targetReadCoverageAfterRetry": 1.0,
                    "evidenceOnlyTargetsBeforeRetry": 1,
                    "evidenceOnlyTargetsAfterRetry": 0
                },
                "readEfficiency": {
                    "analysisAvailable": true,
                    "efficientCtxhelmLane": "ctxhelm-standard",
                    "efficientTargetReadCoverage": 1.0,
                    "efficientTargetReadPrecision": 1.0,
                    "efficientIrrelevantReadCount": 0
                },
                "outcomeClaim": "ctxhelm_improved",
                "laneSummaries": [
                    {
                        "lane": "ctxhelm-standard",
                        "averageTargetReadCoverage": 1.0,
                        "targetReadPrecision": 1.0,
                        "irrelevantReadCount": 0,
                        "ctxhelmEvidenceOnlyTargetCount": 0
                    }
                ]
            },
            "privacyStatus": {
                "localOnly": true,
                "sourceTextLogged": false,
                "rawPromptStored": false,
                "rawTranscriptStored": false,
                "rawMcpTrafficStored": false,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false
            }
        })
        .to_string(),
    )
    .unwrap();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .args(["inspector", "proof", "--report"])
        .arg(&product_report_path)
        .args(["--report"])
        .arg(&agent_report_path)
        .assert()
        .success()
        .stdout(contains("# ctxhelm Proof Bundle Inspector"))
        .stdout(contains(
            "Maturity verdict: `release_and_agent_outcome_evidence_ready`",
        ))
        .stdout(contains("Clean product proofs: `1`"))
        .stdout(contains("Clean agent outcomes: `1`"))
        .stdout(contains("Privacy boundary failed: `false`"))
        .stdout(contains("Read-only boundary failed: `false`"))
        .stdout(contains("Use this as the adoption-facing proof bundle"));

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .args(["inspector", "proof", "--report"])
        .arg(&product_report_path)
        .args(["--report"])
        .arg(&agent_report_path)
        .args(["--require-ready"])
        .assert()
        .success()
        .stdout(contains(
            "Maturity verdict: `release_and_agent_outcome_evidence_ready`",
        ));

    let rendered_json = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .args(["inspector", "proof", "--report"])
            .arg(&product_report_path)
            .args(["--report"])
            .arg(&agent_report_path)
            .args(["--format", "json"])
            .assert(),
    );
    assert_eq!(
        rendered_json["schemaVersion"],
        "ctxhelm-proof-inspector-bundle-v1"
    );
    assert_eq!(
        rendered_json["maturityVerdict"],
        "release_and_agent_outcome_evidence_ready"
    );
    assert_eq!(rendered_json["inventory"]["cleanProductProofCount"], 1);
    assert_eq!(rendered_json["inventory"]["cleanAgentOutcomeCount"], 1);
    assert_eq!(rendered_json["boundary"]["privacyBoundaryFailed"], false);

    let blocked_agent_report_path = fixture.temp.path().join("agent-run-blocked.json");
    fs::write(
        &blocked_agent_report_path,
        json!({
            "schemaVersion": "ctxhelm-agent-run-eval-v1",
            "status": "degraded",
            "workflowKind": "paired-agent-context-suite",
            "client": {
                "name": "claude",
                "version": "Claude Code test"
            },
            "clientAvailability": {
                "tinyPromptAvailable": true,
                "pairedSuiteAvailable": false,
                "rateLimited": true,
                "clientFailureObserved": true,
                "comparableLaneCount": 0,
                "availabilityBlocker": "rate_limit"
            },
            "aggregate": {
                "taskCount": 1,
                "comparisonEligibleCount": 0,
                "comparableCtxhelmLaneCount": 0,
                "forbiddenToolCallsObserved": false,
                "missingRequiredCtxhelmCallsObserved": false,
                "invalidRequiredCtxhelmCallsObserved": false,
                "clientFailuresObserved": true,
                "rateLimitsObserved": true,
                "ctxhelmEvidenceMissesObserved": false,
                "ctxhelmEvidenceOnlyTargetsObserved": false,
                "ctxhelmUnderReadTargetsObserved": false,
                "outcomeClaim": "insufficient_comparable_lanes"
            },
            "privacyStatus": {
                "localOnly": true,
                "sourceTextLogged": false,
                "rawPromptStored": false,
                "rawTranscriptStored": false,
                "rawMcpTrafficStored": false,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false
            }
        })
        .to_string(),
    )
    .unwrap();

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .args(["inspector", "proof", "--report"])
        .arg(&product_report_path)
        .args(["--report"])
        .arg(&blocked_agent_report_path)
        .args(["--require-ready"])
        .assert()
        .failure()
        .stdout(contains(
            "Maturity verdict: `product_proof_ready_agent_outcome_needed`",
        ))
        .stdout(contains("Availability-blocked reports: `1`"))
        .stderr(contains(
            "proof bundle was not release_and_agent_outcome_evidence_ready",
        ));
}

#[test]
fn eval_benchmark_runs_named_suite_source_free() {
    let first = fixture_repo();
    let second = fixture_repo();
    let suite_path = first.temp.path().join("ctxhelm-benchmark.json");
    fs::write(
        &suite_path,
        serde_json::to_string_pretty(&json!({
            "name": "phase-nine-cli-smoke",
            "description": "CLI benchmark smoke",
            "defaults": {
                "limit": 1,
                "rankingBudget": 4,
                "mode": "bug_fix",
                "targetAgent": "codex",
                "roleFilters": ["source", "test"]
            },
            "repositories": [
                {
                    "name": "fixture-a",
                    "path": first.repo
                },
                {
                    "name": "fixture-b",
                    "path": second.repo,
                    "limit": 1,
                    "rankingBudget": 3,
                    "roleFilters": ["source"]
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, first.temp.path().join("ctxhelm-home"))
            .args(["eval", "benchmark", "--config"])
            .arg(&suite_path)
            .args(["--format", "json"])
            .assert(),
    );

    assert_object_has_keys(
        &value,
        &[
            "suiteName",
            "suiteId",
            "repositoryCount",
            "evaluatedRepositoryCount",
            "evaluatedCommitCount",
            "repositories",
            "privacyStatus",
        ],
    );
    assert_eq!(value["suiteName"], "phase-nine-cli-smoke");
    assert_eq!(value["repositoryCount"], 2);
    assert_eq!(value["privacyStatus"]["localOnly"], true);
    assert_eq!(
        value["repositories"][0]["effectiveConfig"]["rankingBudget"],
        4
    );
    assert_eq!(
        value["repositories"][0]["effectiveConfig"]["roleFilters"][0],
        "source"
    );
    assert_eq!(
        value["repositories"][1]["effectiveConfig"]["rankingBudget"],
        3
    );
    assert_eq!(
        value["repositories"][1]["effectiveConfig"]["roleFilters"],
        json!(["source"])
    );
    assert!(value["repositories"][0]["report"]["tokenRoi"].is_array());
    assert_eq!(
        value["repositories"][0]["report"]["rankingComparison"]["noContextBaseline"]["recallAtK"],
        0.0
    );
    assert_no_source_or_prompt_text(&value);

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(
            CTXHELM_HOME_ENV,
            first.temp.path().join("ctxhelm-home-markdown"),
        )
        .args(["eval", "benchmark", "--config"])
        .arg(&suite_path)
        .assert()
        .success()
        .stdout(contains("# ctxhelm Benchmark Suite"))
        .stdout(contains("Suite: `phase-nine-cli-smoke`"))
        .stdout(contains("Repository Results"))
        .stdout(contains("Role filters: `Source, Test`"))
        .stdout(contains("Token ROI"))
        .stdout(contains("No-context Recall@K"))
        .stdout(contains("ctxhelm Lift@10"));
}

#[test]
fn eval_baselines_reports_paired_variants_source_free() {
    let fixture = fixture_repo();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["eval", "baselines", "--repo"])
            .arg(&fixture.repo)
            .args(["--limit", "1", "--budget", "10", "--format", "json"])
            .assert(),
    );

    assert_eq!(value["evaluatedCommits"], 1);
    assert_eq!(value["k"], 10);
    assert_eq!(value["privacyStatus"]["localOnly"], true);
    assert_eq!(value["sourceTextLogged"], false);
    assert_eq!(value["tokenRoi"].as_array().unwrap().len(), 3);
    assert!(value["signalSaturation"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["signal"] == "related_test"
            && row["averageCandidateFiles"].as_f64().unwrap() >= 0.0));
    assert!(value["rows"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["variant"] == "ctxhelm_default" && row["family"] == "default"));
    assert!(value["rows"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["variant"] == "lexical_baseline"));
    assert!(value["rows"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["variant"] == "no_context"));
    assert!(value["rows"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["variant"] == "graph_only"));
    assert!(value["rows"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["variant"] == "feedback_weighted"
            && row["verdict"] == "insufficient_evidence"));
    assert_no_source_or_prompt_text(&value);

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(
            CTXHELM_HOME_ENV,
            fixture.temp.path().join("ctxhelm-home-baselines-md"),
        )
        .args(["eval", "baselines", "--repo"])
        .arg(&fixture.repo)
        .args(["--limit", "1", "--budget", "10"])
        .assert()
        .success()
        .stdout(contains("# ctxhelm Paired Baseline Analysis"))
        .stdout(contains("Variants"))
        .stdout(contains("Token ROI"))
        .stdout(contains("Signal Saturation"))
        .stdout(contains("lexical_baseline"))
        .stdout(contains("feedback_weighted"));
}

#[test]
fn eval_lexical_compare_reports_source_free_bm25_vs_legacy() {
    let fixture = fixture_repo();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["eval", "lexical", "compare", "--repo"])
            .arg(&fixture.repo)
            .args([
                "--query",
                "requireSession",
                "--limit",
                "5",
                "--format",
                "json",
            ])
            .assert(),
    );

    assert_eq!(value["schemaVersion"], "ctxhelm-lexical-comparison-v1");
    assert_eq!(value["query"]["rawQueryStoredInReport"], false);
    assert_eq!(value["privacyStatus"]["localOnly"], true);
    assert_eq!(value["privacyStatus"]["sourceTextLogged"], false);
    assert_eq!(value["privacyStatus"]["resultReasonsOmitted"], true);
    assert!(value["query"].get("queryHash").is_some());
    assert!(value["query"].get("query").is_none());
    assert_eq!(value["bm25"]["backend"], "tantivy_bm25_fielded_v6");
    assert_eq!(value["legacy"]["backend"], "legacy_heuristic_scanner_v1");
    assert!(value["comparison"]["overlapAtLimit"].as_u64().is_some());
    assert!(value["bm25"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .any(|result| result["path"] == "src/auth/session.ts"));
    assert!(value["legacy"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .any(|result| result["path"] == "src/auth/session.ts"));
    assert!(value["bm25"]["results"][0].get("reason").is_none());

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(
            CTXHELM_HOME_ENV,
            fixture.temp.path().join("ctxhelm-home-lexical-md"),
        )
        .args(["eval", "lexical", "compare", "--repo"])
        .arg(&fixture.repo)
        .args(["--query", "requireSession", "--limit", "5"])
        .assert()
        .success()
        .stdout(contains("# ctxhelm Lexical Comparison"))
        .stdout(contains("BM25 Results"))
        .stdout(contains("Legacy Results"))
        .stdout(contains("Raw query stored in report: `false`"))
        .stdout(contains("Source text logged: `false`"));
}

#[test]
fn eval_lexical_corpus_reports_source_free_backend_metrics() {
    let fixture = fixture_repo();
    fs::write(
        fixture.repo.join("src/auth/session.ts"),
        r#"import { issueToken } from "./token";

export function requireSession(user?: { id: string }) {
  if (!user) {
    throw new Error("auth required");
  }
  issueToken(user.id);
  return `session:${user.id}`;
}
"#,
    )
    .unwrap();
    run_git(&fixture.repo, &["add", "src/auth/session.ts"]);
    run_git(
        &fixture.repo,
        &["commit", "-m", "requireSession regression"],
    );

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(CTXHELM_HOME_ENV, &fixture.home)
            .args(["eval", "lexical", "corpus", "--repo"])
            .arg(&fixture.repo)
            .args(["--limit", "1", "--budget", "5", "--format", "json"])
            .assert(),
    );

    assert_eq!(value["schemaVersion"], "lexical-backend-corpus-v1");
    assert_eq!(value["evaluatedCommits"], 1);
    assert_eq!(value["rankingBudget"], 5);
    assert_eq!(value["privacyStatus"]["localOnly"], true);
    assert_eq!(value["sourceTextLogged"], false);
    assert_eq!(value["bm25"]["backend"], "tantivy_bm25_fielded_v6");
    assert_eq!(value["legacy"]["backend"], "legacy_heuristic_scanner_v1");
    assert!(value["runtime"]["inventoryWarmupMillis"].as_u64().is_some());
    assert!(value["comparison"]["recallDeltaAt10"].as_f64().is_some());
    assert!(value["comparison"]["averageOverlapAtK"].as_f64().is_some());
    assert!(value["rows"][0].get("taskHash").is_some());
    assert!(value["rows"][0].get("task").is_none());
    assert!(value["rows"][0]["bm25Files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path == "src/auth/session.ts"));
    assert!(value["rows"][0]["legacyFiles"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path == "src/auth/session.ts"));

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(
            CTXHELM_HOME_ENV,
            fixture.temp.path().join("ctxhelm-home-lexical-corpus-md"),
        )
        .args(["eval", "lexical", "corpus", "--repo"])
        .arg(&fixture.repo)
        .args(["--limit", "1", "--budget", "5"])
        .assert()
        .success()
        .stdout(contains("# ctxhelm Lexical Backend Corpus Comparison"))
        .stdout(contains("Recall delta@5/10"))
        .stdout(contains("Wins at 10"))
        .stdout(contains("Shared inventory warmup ms"))
        .stdout(contains("Source text logged: `false`"));
}

#[test]
fn eval_compare_reports_source_free_metric_and_gap_deltas() {
    let fixture = fixture_repo();
    let suite_path = fixture.temp.path().join("ctxhelm-benchmark.json");
    fs::write(
        &suite_path,
        serde_json::to_string_pretty(&json!({
            "name": "phase-eleven-cli-smoke",
            "defaults": {
                "limit": 1,
                "rankingBudget": 4,
                "mode": "bug_fix",
                "targetAgent": "codex",
                "lexicalBackendComparison": true
            },
            "repositories": [
                {
                    "name": "fixture-a",
                    "path": fixture.repo,
                    "proofRuntimeCeilingMillis": 15000
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();
    let base = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(
                CTXHELM_HOME_ENV,
                fixture.temp.path().join("ctxhelm-home-compare"),
            )
            .args(["eval", "benchmark", "--config"])
            .arg(&suite_path)
            .args(["--format", "json"])
            .assert(),
    );
    let mut base = base;
    base["repositories"][0]["report"]["fileRecallAt10"] = json!(1.0);
    let mut head = base.clone();
    head["suiteId"] = json!("head-suite");
    head["repositories"][0]["report"]["fileRecallAt10"] = json!(0.0);
    let base_path = fixture.temp.path().join("base-report.json");
    let head_path = fixture.temp.path().join("head-report.json");
    fs::write(&base_path, serde_json::to_string_pretty(&base).unwrap()).unwrap();
    fs::write(&head_path, serde_json::to_string_pretty(&head).unwrap()).unwrap();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .args(["eval", "compare", "--base-report"])
            .arg(&base_path)
            .args(["--head-report"])
            .arg(&head_path)
            .args(["--threshold", "fileRecallAt10=0.01", "--format", "json"])
            .assert(),
    );

    assert_eq!(value["headSuiteId"], "head-suite");
    assert_eq!(value["passed"], false);
    assert_eq!(value["thresholdChecks"][0]["metric"], "fileRecallAt10");
    assert_eq!(value["thresholdChecks"][0]["passed"], false);
    assert!(value["metricDeltas"]
        .as_array()
        .unwrap()
        .iter()
        .any(|delta| delta["metric"] == "tokenRoiBrief"));
    assert!(value["gapFamilyDeltas"].is_array());
    assert_no_source_or_prompt_text(&value);

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .args(["eval", "compare", "--base-report"])
        .arg(&base_path)
        .args(["--head-report"])
        .arg(&head_path)
        .args(["--threshold", "fileRecallAt10=0.01"])
        .assert()
        .success()
        .stdout(contains("# ctxhelm Benchmark Comparison"))
        .stdout(contains("Threshold Checks"))
        .stdout(contains("fileRecallAt10"));
}

#[test]
fn eval_proof_generates_source_free_product_report() {
    let fixture = fixture_repo();
    fs::write(
        fixture.repo.join("src/auth/session.ts"),
        r#"import { issueToken } from "./token";

export function requireSession(user?: { id: string }) {
  if (!user) {
    throw new Error("auth required");
  }
  issueToken(user.id);
  return `session:${user.id}`;
}
"#,
    )
    .unwrap();
    run_git(&fixture.repo, &["add", "src/auth/session.ts"]);
    run_git(
        &fixture.repo,
        &["commit", "-m", "requireSession regression"],
    );
    let suite_path = fixture.temp.path().join("ctxhelm-proof.json");
    fs::write(
        &suite_path,
        serde_json::to_string_pretty(&json!({
            "manifestVersion": "ctxhelm-benchmark-corpus-v2.3",
            "name": "phase-twelve-proof-smoke",
            "corpusId": "ctxhelm-proof-fixture-v23",
            "privacyLabel": "local-fixture-source-free",
            "defaults": {
                "limit": 1,
                "rankingBudget": 4,
                "mode": "bug_fix",
                "targetAgent": "codex",
                "lexicalBackendComparison": true
            },
            "repositories": [
                {
                    "name": "fixture-a",
                    "path": fixture.repo
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let value = json_stdout(
        Command::cargo_bin("ctxhelm")
            .unwrap()
            .env(
                CTXHELM_HOME_ENV,
                fixture.temp.path().join("ctxhelm-home-proof"),
            )
            .args(["eval", "proof", "--config"])
            .arg(&suite_path)
            .args(["--format", "json"])
            .assert(),
    );
    assert_eq!(value["suiteName"], "phase-twelve-proof-smoke");
    assert_eq!(value["privacyStatus"]["localOnly"], true);
    assert_eq!(
        value["v23EvalSummary"]["manifestVersion"],
        "ctxhelm-benchmark-corpus-v2.3"
    );
    assert_eq!(
        value["v23EvalSummary"]["fixedCorpusId"],
        "ctxhelm-proof-fixture-v23"
    );
    assert_eq!(
        value["v23EvalSummary"]["privacyLabel"],
        "local-fixture-source-free"
    );
    assert!(value["v23EvalSummary"]["pairedBaselineVerdicts"].is_array());
    assert_eq!(
        value["v23EvalSummary"]["featureExportPrivacy"]["localOnly"],
        true
    );
    assert_eq!(
        value["v23EvalSummary"]["featureExportPrivacy"]["sourceTextLogged"],
        false
    );
    assert_eq!(
        value["v23EvalSummary"]["featureExportPrivacy"]["sourceFreeLabelsOnly"],
        true
    );
    assert_eq!(
        value["v23EvalSummary"]["learnedPolicyStatus"]["profileSchemaVersion"],
        2
    );
    assert_eq!(
        value["v23EvalSummary"]["learnedPolicyStatus"]["defaultRequiresThresholds"],
        true
    );
    assert_eq!(
        value["v23EvalSummary"]["learnedPolicyStatus"]["silentDefaultAllowed"],
        false
    );
    assert!(value["v23EvalSummary"]["proofBoundary"]
        .as_str()
        .unwrap()
        .contains("world-class claims require repeated lift"));
    assert!(value["headlineMetrics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|metric| metric["label"] == "averageCtxhelmLiftAt10"));
    assert_eq!(value["releaseGate"]["decision"], "promote");
    assert_eq!(value["releaseGate"]["defaultPromotionAllowed"], true);
    assert!(value["releaseGate"]["lexicalComparison"].is_object());
    assert!(value["releaseGate"]["lexicalComparison"]["allFileClaim"].is_string());
    assert!(value["releaseGate"]["lexicalComparison"]["agentEvidenceClaim"].is_string());
    assert!(value["releaseGate"]["lexicalComparison"]["contextClaim"].is_string());
    assert!(
        value["releaseGate"]["lexicalComparison"]["averageAgentEvidenceRecallAt10"].is_number()
    );
    assert!(value["releaseGate"]["lexicalBackendComparison"].is_object());
    assert!(value["releaseGate"]["lexicalBackendComparison"]["averageRecallDeltaAt10"].is_number());
    assert!(value["releaseGate"]["lexicalBackendComparison"]["bm25Claim"].is_string());
    assert!(value["releaseGate"]["corpusVerdicts"].is_array());
    assert!(value["releaseGate"]["corpusVerdicts"][0]["sourceRecallAt10"].is_number());
    assert!(value["releaseGate"]["corpusVerdicts"][0]["lexicalSourceRecallAt10"].is_number());
    assert!(value["releaseGate"]["corpusVerdicts"][0]["sourceDeltaAt10"].is_number());
    assert!(value["limitations"].is_array());
    assert!(value["helpsWhen"].is_array());
    assert!(value["doesNotHelpWhen"].is_array());
    assert!(value["futureWork"].is_array());
    assert!(value["benchmarkReport"]["repositories"].is_array());
    assert!(
        value["benchmarkReport"]["repositories"][0]["lexicalBackendCorpus"]["comparison"]
            ["recallDeltaAt10"]
            .is_number()
    );
    assert_no_source_or_prompt_text(&value);

    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(
            CTXHELM_HOME_ENV,
            fixture.temp.path().join("ctxhelm-home-proof-md"),
        )
        .args(["eval", "proof", "--config"])
        .arg(&suite_path)
        .assert()
        .success()
        .stdout(contains("# ctxhelm Product Proof"))
        .stdout(contains("Fixed-Corpus Eval Summary"))
        .stdout(contains("Paired Baseline Verdicts"))
        .stdout(contains("Proof Boundary"))
        .stdout(contains("Release Gate Decision"))
        .stdout(contains("Lexical Backend Comparison Summary"))
        .stdout(contains("Corpus Verdicts"))
        .stdout(contains("source recall@10"))
        .stdout(contains("lexical source recall@10"))
        .stdout(contains("When It Helps"))
        .stdout(contains("When It Does Not Help"))
        .stdout(contains("Limitations"))
        .stdout(contains("Future Work From Gaps"));
}

#[test]
fn serve_mcp_speaks_json_rpc_over_stdio() {
    let fixture = fixture_repo();
    let input = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
"#;

    let assert = Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
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
    assert_eq!(initialize["result"]["serverInfo"]["name"], "ctxhelm");
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

    let assert = Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
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

    let first = Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
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
    let second = Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
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
            assert!(
                content.contains("CODEX_HOME"),
                "{script} must isolate older Codex CLI config when --ignore-user-config is unavailable"
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
            "CTXHELM_BIN",
            "CTXHELM_REQUIRE_REAL_CLIENT",
            "CTXHELM_SKIP_REAL_CLIENT",
            "CTXHELM_REAL_CLIENT_EVIDENCE_DIR",
            "clientVersion",
            "ctxhelmVersion",
            "deterministicProtocol",
            "deterministicContextAreaResourceRead",
            "prepareTask",
            "getPack",
            "status",
            "skipReason",
            "required",
            "requestEvidenceSchemaVersion",
            "ctxhelm-real-client-evidence-v2",
            "serverSideRequestLog",
            "requestLogSha256",
            "requestLogLineCount",
            "methodCounts",
            "initializeRequested",
            "toolsListRequested",
            "explicitRepoToolCallCount",
            "observedToolCalls",
            "requestSummaryFile",
            "hashlib.sha256",
            "--version",
            "serve-mcp",
            "request_log",
            "skipped",
            "passed",
            "failed",
        ] {
            assert!(content.contains(needle), "{script} missing {needle}");
        }
        if script.contains("codex") {
            for needle in [
                "clientExitStatus",
                "clientFailureKind",
                "stderrLineCount",
                "stderrSha256",
                "stream_disconnected",
                "auth_or_model_refusal",
                "rate_limited",
                "tool_call_missing",
            ] {
                assert!(content.contains(needle), "{script} missing {needle}");
            }
        }
        assert!(
            content.contains("CTXHELM_BIN=\"$ctxhelm_bin\""),
            "{script} must propagate the selected binary into child smoke commands"
        );
        assert!(
            content.contains("\"$ctxhelm_bin\" --version"),
            "{script} must capture ctxhelm version from the selected binary"
        );
        assert!(
            content.contains("\"$ctxhelm_bin\" serve-mcp")
                || content.contains(
                    "'tee -a \"$CTXHELM_REAL_CLIENT_REQUEST_LOG\" | \"$ctxhelm_bin\" serve-mcp'"
                ),
            "{script} must launch the MCP server through the selected binary"
        );
        assert!(
            content.contains("CTXHELM_REAL_CLIENT_EVIDENCE_DIR"),
            "{script} must support stable evidence output"
        );
        assert!(
            content.contains("write_evidence"),
            "{script} must emit machine-checkable evidence on success"
        );
        assert!(
            content.contains("write_skip_evidence"),
            "{script} must emit machine-checkable evidence when optional real-client proof skips"
        );
        assert!(
            content.contains("CTXHELM_RUN_REAL_CLIENT")
                || content.contains("CTXHELM_REQUIRE_REAL_CLIENT"),
            "{script} must keep real-client execution env-gated"
        );
    }

    for script in [
        "scripts/smoke-cursor-mcp.sh",
        "scripts/smoke-opencode-mcp.sh",
    ] {
        let script_path = workspace_root.join(script);
        let content = fs::read_to_string(&script_path)
            .unwrap_or_else(|error| panic!("failed to read {script}: {error}"));
        let syntax = StdCommand::new("bash")
            .arg("-n")
            .arg(&script_path)
            .status()
            .unwrap_or_else(|error| panic!("failed to run bash -n {script}: {error}"));
        assert!(syntax.success(), "{script} failed bash -n");
        for needle in [
            "scripts/smoke-mcp-protocol.sh",
            "setup-check",
            "CTXHELM_BIN",
            "CTXHELM_REAL_CLIENT_EVIDENCE_DIR",
            "clientVersion",
            "ctxhelmVersion",
            "deterministicProtocol",
            "deterministicContextAreaResourceRead",
            "realClientToolCalls",
            "proofBoundary",
            "not_installed",
            "passed",
        ] {
            assert!(content.contains(needle), "{script} missing {needle}");
        }
        assert!(
            content.contains("no machine-checkable"),
            "{script} must not overclaim real-client tool-call proof"
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
    assert!(
        syntax.success(),
        "scripts/smoke-mcp-protocol.sh failed bash -n"
    );

    assert!(content.contains("CTXHELM_BIN"));
    assert!(content.contains("serve-mcp"));
    assert!(
        content.contains("cargo") && content.contains("run"),
        "development cargo fallback must remain"
    );
    assert!(
        content.find("CTXHELM_BIN").unwrap() < content.find("serve-mcp").unwrap(),
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
        !content.contains("smoke_diff_file=\"$CTXHELM_SMOKE_REPO/$smoke_diff_path\""),
        "MCP smoke must not create current_diff proof files inside the target repo"
    );
    assert!(content.contains("prepare_task"));
    assert!(content.contains("get_pack"));
    assert!(content.contains("\"repo\""));
    assert!(content.contains("ctxhelm://repo/context-areas"));
    assert!(content.contains("ctxhelm://repo/context-area/"));
    assert!(content.contains("nextReadBatches"));
    assert!(
        content.contains("CTXHELM_REQUIRE_RESOURCE_SCOPE"),
        "MCP smoke should keep post-release resourceScope assertions explicit"
    );
    assert!(
        content.contains("require_resource_scope"),
        "MCP smoke should allow release-compatible protocol expectations only when explicitly requested"
    );
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
    assert!(
        syntax.success(),
        "scripts/smoke-first-pack.sh failed bash -n"
    );

    for needle in [
        "CTXHELM_BIN",
        "command -v ctxhelm",
        "CTXHELM_HOME",
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
        assert!(
            content.contains(needle),
            "smoke-first-pack missing {needle}"
        );
    }

    let output = StdCommand::new("bash")
        .arg(&script_path)
        .env("CTXHELM_BIN", env!("CARGO_BIN_EXE_ctxhelm"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "smoke-first-pack failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ctxhelm first-pack smoke ok"));
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

fn record_feedback_event(
    fixture: &common::FixtureRepo,
    task_hash: &str,
    budget: &str,
    outcome: &str,
) {
    Command::cargo_bin("ctxhelm")
        .unwrap()
        .env(CTXHELM_HOME_ENV, &fixture.home)
        .args([
            "eval",
            "feedback",
            "record",
            "--task-hash",
            task_hash,
            "--mode",
            "bug-fix",
            "--target-agent",
            "codex",
            "--budget",
            budget,
            "--outcome",
            outcome,
            "--recommended-file",
            "src/auth/session.ts",
            "--recommended-test",
            "tests/auth/session.test.ts",
            "--read-file",
            "src/auth/session.ts",
            "--edited-file",
            "src/auth/session.ts",
            "--tested-file",
            "tests/auth/session.test.ts",
            "--tested-command",
            "pnpm test tests/auth/session.test.ts",
            "--format",
            "json",
            "--repo",
        ])
        .arg(&fixture.repo)
        .assert()
        .success();
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
    assert_no_source_or_prompt_text_at(value, None);
}

fn assert_no_source_or_prompt_text_at(value: &Value, parent_key: Option<&str>) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let role_count_label = matches!(
                    parent_key,
                    Some(
                        "roleCounts"
                            | "selectedRoleCounts"
                            | "role_counts"
                            | "selected_role_counts"
                            | "candidateRecoverableRoleCounts"
                            | "candidate_recoverable_role_counts"
                            | "noCandidateRoleCounts"
                            | "no_candidate_role_counts"
                    )
                );
                assert!(
                    role_count_label
                        || !matches!(
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
                assert_no_source_or_prompt_text_at(child, Some(key));
            }
        }
        Value::Array(items) => {
            for item in items {
                assert_no_source_or_prompt_text_at(item, parent_key);
            }
        }
        _ => {}
    }
}
