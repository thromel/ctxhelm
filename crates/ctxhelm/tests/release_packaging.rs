use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn python3_command() -> Command {
    let mut command = Command::new("python3");
    command.env("PYTHONDONTWRITEBYTECODE", "1");
    command.env(
        "PYTHONPYCACHEPREFIX",
        std::env::temp_dir().join("ctxhelm-release-packaging-pycache"),
    );
    command
}

#[test]
fn release_package_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/release-package.sh");
    assert!(script.exists(), "release package script is missing");

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    assert!(script_text.contains("cargo build -p ctxhelm --release --locked --target"));
    assert!(script_text.contains("CARGO_TARGET_DIR"));
    assert!(script_text.contains("CARGO_BUILD_TARGET_DIR"));
    assert!(script_text.contains("CTXHELM_BUILD_TARGET"));
    assert!(script_text.contains("CTXHELM_DIST_DIR"));
    assert!(script_text.contains("BUILT_BINARY"));
    assert!(script_text.contains("dist"));
    assert!(script_text.contains("CTXHELM_ALLOW_DIRTY"));
    assert!(
        script_text.contains("git diff --quiet") || script_text.contains("git status --porcelain"),
        "script must check for a clean checkout by default"
    );
    assert!(
        script_text.contains("ctxhelm-v${VERSION}-${TARGET_LABEL}.tar.gz")
            || script_text.contains("ctxhelm-v${version}-${target}.tar.gz"),
        "script must create versioned release archives"
    );
    assert!(script_text.contains("tar -czf"));
    assert!(script_text.contains("README.md"));
    assert!(script_text.contains("LICENSE"));
    assert!(script_text.contains("VERSION"));
    assert!(script_text.contains("manifest.json"));
    assert!(script_text.contains("audit.json"));
    assert!(script_text.contains("ARCHIVE_SHA256"));
    assert!(script_text.contains("BINARY_SHA256"));
    assert!(script_text.contains("privacyStatus"));
    assert!(script_text.contains("unsupportedActions"));
    assert!(script_text.contains("sha256sums.txt"));
    assert!(
        script_text.contains("shasum -a 256") || script_text.contains("sha256sum"),
        "script must write SHA-256 checksums"
    );
    assert!(script_text.contains("--version"));
    assert!(script_text.contains("--help"));

    let gitignore = fs::read_to_string(repo_root.join(".gitignore")).unwrap();
    assert!(
        gitignore.lines().any(|line| line.trim() == "/dist/"),
        ".gitignore must ignore /dist/"
    );
}

#[test]
fn release_artifact_audit_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/audit-release-artifact.sh");
    assert!(script.exists(), "release artifact audit script is missing");

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        ".ctxhelm",
        "traces.jsonl",
        "request",
        ".env",
        "token",
        "target/",
        ".git/",
        "/Users/",
        "tar -tf",
        "CTXHELM_AUDIT_REPORT",
        "privacyStatus",
        "sourceFree",
    ] {
        assert!(
            script_text.contains(required),
            "audit script must mention forbidden pattern {required}"
        );
    }
}

#[test]
fn release_artifact_audit_rejects_local_state_archive() {
    let archive = archive_with_entries(&[
        (
            "ctxhelm-v2.4.6-test/ctxhelm",
            "#!/usr/bin/env bash\nexit 0\n",
        ),
        (
            "ctxhelm-v2.4.6-test/.ctxhelm/repos/repo/traces.jsonl",
            "{\"sourceTextLogged\":false}\n",
        ),
    ]);

    let output = Command::new(workspace_root().join("scripts/audit-release-artifact.sh"))
        .arg(&archive)
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "audit unexpectedly passed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn release_artifact_audit_accepts_minimal_release_archive() {
    let archive = archive_with_entries(&[
        (
            "ctxhelm-v2.4.6-test/ctxhelm",
            "#!/usr/bin/env bash\nexit 0\n",
        ),
        ("ctxhelm-v2.4.6-test/README.md", "ctxhelm release\n"),
        ("ctxhelm-v2.4.6-test/LICENSE", "MIT License\n"),
        ("ctxhelm-v2.4.6-test/VERSION", "ctxhelm 2.4.6\n"),
    ]);

    let output = Command::new(workspace_root().join("scripts/audit-release-artifact.sh"))
        .arg(&archive)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "audit failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_artifact_audit_writes_source_free_report() {
    let archive = archive_with_entries(&[
        (
            "ctxhelm-v2.4.6-test/ctxhelm",
            "#!/usr/bin/env bash\nexit 0\n",
        ),
        ("ctxhelm-v2.4.6-test/README.md", "ctxhelm release\n"),
        ("ctxhelm-v2.4.6-test/LICENSE", "MIT License\n"),
        ("ctxhelm-v2.4.6-test/VERSION", "ctxhelm 2.4.6\n"),
    ]);
    let report_dir = TempDir::new().unwrap();
    let report_path = report_dir.path().join("audit.json");

    let output = Command::new(workspace_root().join("scripts/audit-release-artifact.sh"))
        .env("CTXHELM_AUDIT_REPORT", &report_path)
        .arg(&archive)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "audit failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).unwrap()).unwrap();
    assert_eq!(report["status"], "passed");
    assert_eq!(report["sourceFree"], true);
    assert_eq!(report["privacyStatus"]["localOnly"], true);
    let report_text = fs::read_to_string(report_path).unwrap();
    assert!(!report_text.contains("/Users/"));
    assert!(!report_text.contains(".ctxhelm/repos"));
}

#[test]
fn release_artifact_audit_is_called_by_package_script() {
    let script = fs::read_to_string(workspace_root().join("scripts/release-package.sh")).unwrap();
    let audit_pos = script
        .find("audit-release-artifact.sh")
        .expect("release package script must call the artifact audit");
    let success_pos = script
        .find("created ${ARCHIVE_PATH}")
        .expect("release package script must report created archive");
    assert!(
        audit_pos < success_pos,
        "artifact audit must run before success output"
    );
}

#[test]
fn release_gate_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/release-gate.sh");
    assert!(script.exists(), "release gate script is missing");

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "cargo test --workspace",
        "scripts/check-release-docs.sh",
        "scripts/release-package.sh",
        "scripts/verify-release-archive.sh",
        "scripts/check-agent-run-proof.py",
        "scripts/smoke-first-pack.sh",
        "scripts/smoke-storage.sh",
        "scripts/smoke-memory.sh",
        "scripts/smoke-memory-reuse.sh",
        "scripts/smoke-memory-history-lift.sh",
        "scripts/smoke-memory-parent-snapshot-lift.sh",
        "scripts/smoke-memory-benchmark-lift.sh",
        "scripts/smoke-feedback.sh",
        "scripts/smoke-governor.sh",
        "scripts/smoke-shared-artifacts.sh",
        "scripts/smoke-inspector.sh",
        "scripts/smoke-retrieval-health.sh",
        "scripts/smoke-graph.sh",
        "scripts/smoke-policy-embedding.sh",
        "scripts/smoke-agent-preview.sh",
        "scripts/smoke-agent-native-fallback.sh",
        "scripts/smoke-demo-artifacts.sh",
        "scripts/smoke-distribution-metadata.sh",
        "scripts/smoke-release-governance.sh",
        "scripts/smoke-semantic.sh",
        "scripts/smoke-precision.sh",
        "scripts/smoke-v23-eval.sh",
        "scripts/check-product-proof.py",
        "scripts/prepare-proof-fixtures.sh",
        "scripts/smoke-mcp-protocol.sh",
        "scripts/smoke-codex-mcp.sh",
        "scripts/smoke-claude-mcp.sh",
        "scripts/smoke-cursor-real-client.sh",
        "scripts/smoke-opencode-real-client.sh",
        "scripts/e2e-claude-workflow.sh",
        "CTXHELM_BIN",
        "CTXHELM_SKIP_REAL_CLIENT",
        "CTXHELM_REQUIRE_REAL_CLIENT",
        "CTXHELM_RUN_CURSOR_REAL_CLIENT",
        "CTXHELM_REQUIRE_CURSOR_REAL_CLIENT",
        "CTXHELM_RUN_OPENCODE_REAL_CLIENT",
        "CTXHELM_REQUIRE_OPENCODE_REAL_CLIENT",
        "CTXHELM_RUN_CLAUDE_WORKFLOW_EVAL",
        "CTXHELM_REQUIRE_CLAUDE_WORKFLOW_EVAL",
        "CTXHELM_CLEAN_FIXTURE_CONFIG",
        "CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF",
        "CTXHELM_SKIP_CLEAN_FIXTURE_PROOF",
        "CTXHELM_REAL_CLIENT_EVIDENCE_DIR",
        "CTXHELM_PROOF_DIR",
        "CTXHELM_BENCHMARK_CONFIG",
        "CTXHELM_AGENT_RUN_PROOF_REPORT",
        "CTXHELM_REQUIRE_AGENT_RUN_PROOF",
        "CTXHELM_AGENT_RUN_MIN_TASK_COUNT",
        "CTXHELM_AGENT_RUN_MIN_COMPARISON_ELIGIBLE",
        "CTXHELM_AGENT_RUN_MIN_COMPARABLE_CTXHELM_LANES",
        "CTXHELM_AGENT_RUN_MIN_TARGET_READ_COVERAGE",
        "CTXHELM_AGENT_RUN_MAX_EXTRA_READ_DELTA",
        "CTXHELM_AGENT_RUN_MIN_IRRELEVANT_READ_DELTA",
        "CTXHELM_REQUIRE_PROOF_INSPECTOR_READY",
        "phase183-clean-fixture-refresh-config.json",
        "eval proof",
        "inspector proof",
        "check-product-proof.py",
        "check-agent-run-proof.py",
        "release proof bundle",
        "release-proof-summary.json",
        "proof-inspector-readiness-bundle.json",
        "binaryIdentity",
        "optionalProofs",
        "cleanColdFixtureProductProof",
        "cleanColdFixtureRequired",
        "agentRunOutcomeProof",
        "agentRunOutcomeProofRequired",
        "agentRunOutcomeProofReport",
        "proofInspectorReadinessBundle",
        "proofInspectorReadinessRequired",
        "proofInspectorReadinessReport",
        "agent-run-outcome-proof.json",
        "--require-ready",
        "--expected-ctxhelm-version",
        "--expected-client-name",
        "--expected-client-version",
        "CTXHELM_AGENT_RUN_EXPECTED_CLIENT_VERSION",
        "--current-runner-script",
        "--current-suite",
        "2026-06-06-phase251-codex-rd-suite.json",
        "--format json",
        "--output",
        "stale clean proof fixtures",
        "rev-parse",
        "cat-file",
        "claudeWorkflowEval",
        "resourceBackedGapSummaryContract",
        "archive_sha256",
        "--version",
        "--help",
        "tar -xzf",
        "release gate passed",
    ] {
        assert!(
            script_text.contains(required),
            "release gate missing {required}"
        );
    }

    assert!(
        script_text.contains("CTXHELM_BIN=\"$ctxhelm_bin\" bash \"$smoke_first_pack_script\""),
        "release gate must pass selected binary into first-pack smoke"
    );
    assert!(
        !script_text.contains("CTXHELM_ALLOW_DIRTY=\"${CTXHELM_ALLOW_DIRTY:-1}\""),
        "release gate must not bypass release-package clean-checkout enforcement by default"
    );
    assert!(
        script_text.contains("CTXHELM_DIST_DIR=\"$dist_dir\" bash \"$release_package_script\""),
        "release gate should let release-package enforce clean-checkout semantics unless CTXHELM_ALLOW_DIRTY is explicitly inherited"
    );
    assert!(
        script_text.contains("CTXHELM_DIST_DIR=\"$dist_dir\"")
            && script_text.contains("bash \"$smoke_distribution_metadata_script\""),
        "release gate must pass packaged archive directory into distribution metadata smoke"
    );
    assert!(
        script_text.contains(
            "CTXHELM_DISTRIBUTION_METADATA_OUT=\"$work_dir/distribution-metadata-smoke.json\""
        ),
        "release gate must keep distribution metadata smoke output in the temp proof workspace by default"
    );
    assert!(
        script_text.contains("post-smoke worktree cleanliness")
            && script_text.contains("git -C \"$repo_root\" status --porcelain")
            && script_text.contains("final worktree cleanliness")
            && script_text.contains("check_worktree_clean \"release gate\"")
            && script_text.contains("CTXHELM_SKIP_WORKTREE_CLEAN_CHECK"),
        "release gate must fail when smokes leave the worktree dirty"
    );
    assert!(
        script_text.find("scripts/smoke-first-pack.sh").unwrap()
            < script_text.find("scripts/smoke-mcp-protocol.sh").unwrap(),
        "first-pack smoke should run before direct MCP protocol smoke"
    );
    assert!(
        script_text.find("scripts/smoke-mcp-protocol.sh").unwrap()
            < script_text.find("scripts/smoke-codex-mcp.sh").unwrap(),
        "optional real-client hooks should run after deterministic MCP proof"
    );

    for forbidden in [
        "git tag",
        "git push",
        "gh release",
        "cargo publish",
        "crates.io publish",
        "brew tap",
        "notarytool",
        "xcrun altool",
        "upload-artifact",
    ] {
        assert!(
            !script_text.contains(forbidden),
            "release gate must not contain publishing behavior: {forbidden}"
        );
    }
}

#[test]
fn prepare_benchmark_corpus_script_contract_and_dirty_guard() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/prepare-benchmark-corpus.sh");
    assert!(script.exists(), "benchmark corpus prep script is missing");

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "--refresh",
        "--min-commits",
        "sourceTextLogged",
        "gitHistoryUsable",
        "objectContentUsable",
        "dirtyCount",
        "commit subjects",
        "terminal logs",
        "fsck --no-progress",
        "git clone",
        "checkout --detach",
        "reset --hard",
        "clean -fdx",
        "benchmark corpus worktree is dirty",
    ] {
        assert!(
            script_text.contains(required),
            "benchmark corpus script contract missing {required}"
        );
    }

    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let worktree = temp.path().join("worktree");
    let ready_report = temp.path().join("ready.json");
    let dirty_report = temp.path().join("dirty.json");
    let refresh_report = temp.path().join("refresh.json");
    fs::create_dir_all(source.join("src")).unwrap();
    run_git(&source, &["init"]);
    run_git(&source, &["config", "user.email", "ctxhelm@example.com"]);
    run_git(&source, &["config", "user.name", "ctxhelm"]);
    fs::write(
        source.join("src/lib.rs"),
        "pub fn sentinel() -> &'static str { \"CTXHELM_SOURCE_SENTINEL\" }\n",
    )
    .unwrap();
    run_git(&source, &["add", "."]);
    run_git(&source, &["commit", "-m", "seed source sentinel"]);
    fs::write(
        source.join("src/lib.rs"),
        "pub fn sentinel() -> &'static str { \"CTXHELM_SOURCE_SENTINEL_2\" }\n",
    )
    .unwrap();
    run_git(&source, &["add", "."]);
    run_git(&source, &["commit", "-m", "update source sentinel"]);
    let revision = git_stdout(&source, &["rev-parse", "HEAD"]);

    let ready = Command::new("bash")
        .arg(&script)
        .args(["--name", "fixture"])
        .arg("--source")
        .arg(&source)
        .args(["--revision", revision.trim()])
        .arg("--worktree")
        .arg(&worktree)
        .args(["--min-commits", "2"])
        .arg("--output")
        .arg(&ready_report)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        ready.status.success(),
        "prepare benchmark corpus failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&ready.stdout),
        String::from_utf8_lossy(&ready.stderr)
    );
    let ready_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&ready_report).unwrap()).unwrap();
    assert_eq!(ready_json["status"], "ready");
    assert_eq!(ready_json["privacyStatus"]["localOnly"], true);
    assert_eq!(ready_json["privacyStatus"]["sourceTextLogged"], false);
    assert_eq!(ready_json["checks"]["gitHistoryUsable"], true);
    assert_eq!(ready_json["checks"]["objectContentUsable"], true);
    assert_eq!(ready_json["checks"]["dirtyCount"], 0);
    assert_eq!(ready_json["checks"]["commitCount"], 2);
    let ready_text = fs::read_to_string(&ready_report).unwrap();
    assert!(!ready_text.contains("CTXHELM_SOURCE_SENTINEL"));
    assert!(!ready_text.contains("seed source sentinel"));

    fs::write(worktree.join("untracked.txt"), "dirty proof fixture\n").unwrap();
    let dirty = Command::new("bash")
        .arg(&script)
        .args(["--name", "fixture"])
        .arg("--source")
        .arg(&source)
        .args(["--revision", revision.trim()])
        .arg("--worktree")
        .arg(&worktree)
        .args(["--min-commits", "2"])
        .arg("--output")
        .arg(&dirty_report)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !dirty.status.success(),
        "dirty benchmark corpus unexpectedly passed"
    );
    let dirty_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&dirty_report).unwrap()).unwrap();
    assert_eq!(dirty_json["status"], "blocked");
    assert_eq!(dirty_json["checks"]["dirtyCount"], 1);
    assert!(dirty_json["failureReason"]
        .as_str()
        .unwrap()
        .contains("dirty"));

    let refreshed = Command::new("bash")
        .arg(&script)
        .args(["--name", "fixture"])
        .arg("--source")
        .arg(&source)
        .args(["--revision", revision.trim()])
        .arg("--worktree")
        .arg(&worktree)
        .args(["--min-commits", "2"])
        .arg("--output")
        .arg(&refresh_report)
        .arg("--refresh")
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        refreshed.status.success(),
        "refreshing dirty benchmark corpus failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&refreshed.stdout),
        String::from_utf8_lossy(&refreshed.stderr)
    );
    let refresh_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&refresh_report).unwrap()).unwrap();
    assert_eq!(refresh_json["status"], "ready");
    assert_eq!(refresh_json["checks"]["refreshRequested"], true);
    assert_eq!(refresh_json["checks"]["dirtyCount"], 0);
    assert!(
        !worktree.join("untracked.txt").exists(),
        "--refresh should clean untracked disposable fixture files"
    );
}

#[test]
fn prepare_proof_fixtures_reports_unavailable_revisions_source_free() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/prepare-proof-fixtures.sh");
    assert!(script.exists(), "proof fixture prep script is missing");

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "sourceTextLogged",
        "requested proof fixture revision is unavailable",
        "revisionAvailable",
        "commit subjects",
        "remote URLs",
        "cat-file -e",
        "CTXHELM_VERISCHEMA_REVISION",
        "CTXHELM_PROOF_FIXTURE_REPORT_DIR",
    ] {
        assert!(
            script_text.contains(required),
            "proof fixture script contract missing {required}"
        );
    }

    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let fixture_root = temp.path().join("fixtures");
    let report_dir = temp.path().join("reports");
    fs::create_dir_all(source.join("src")).unwrap();
    run_git(&source, &["init"]);
    run_git(&source, &["config", "user.email", "ctxhelm@example.com"]);
    run_git(&source, &["config", "user.name", "ctxhelm"]);
    fs::write(
        source.join("src/lib.rs"),
        "pub fn sentinel() -> &'static str { \"CTXHELM_SOURCE_SENTINEL\" }\n",
    )
    .unwrap();
    run_git(&source, &["add", "."]);
    run_git(&source, &["commit", "-m", "seed source sentinel"]);
    let revision = git_stdout(&source, &["rev-parse", "HEAD"]);
    let missing_revision = "0123456789abcdef0123456789abcdef01234567";

    let output = Command::new("bash")
        .arg(&script)
        .env("CTXHELM_PROOF_FIXTURE_ROOT", &fixture_root)
        .env("CTXHELM_PROOF_FIXTURE_REPORT_DIR", &report_dir)
        .env("CTXHELM_REFACTORINGMINER_SOURCE", &source)
        .env("CTXHELM_REFACTORINGMINER_REVISION", revision.trim())
        .env("CTXHELM_CTXHELM_SOURCE", &source)
        .env("CTXHELM_CTXHELM_REVISION", revision.trim())
        .env("CTXHELM_REAGENT_SOURCE", &source)
        .env("CTXHELM_REAGENT_REVISION", revision.trim())
        .env("CTXHELM_VERISCHEMA_SOURCE", &source)
        .env("CTXHELM_VERISCHEMA_REVISION", missing_revision)
        .current_dir(&repo_root)
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "unavailable proof fixture revision unexpectedly passed"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("proof fixture blocked")
            && stderr.contains("VeriSchema")
            && stderr.contains(missing_revision),
        "unexpected stderr: {stderr}"
    );

    let ready_json: serde_json::Value = serde_json::from_slice(
        &fs::read(report_dir.join("RefactoringMiner.fixture-status.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(ready_json["status"], "ready");
    assert_eq!(ready_json["privacyStatus"]["sourceTextLogged"], false);
    assert_eq!(ready_json["checks"]["revisionAvailable"], true);

    let blocked_json: serde_json::Value = serde_json::from_slice(
        &fs::read(report_dir.join("VeriSchema.fixture-status.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(blocked_json["status"], "blocked");
    assert_eq!(blocked_json["checks"]["revisionAvailable"], false);
    assert!(blocked_json["failureReason"]
        .as_str()
        .unwrap()
        .contains("unavailable"));

    let blocked_text =
        fs::read_to_string(report_dir.join("VeriSchema.fixture-status.json")).unwrap();
    assert!(!blocked_text.contains("CTXHELM_SOURCE_SENTINEL"));
    assert!(!blocked_text.contains("seed source sentinel"));
}

#[test]
fn distribution_metadata_smoke_script_contract() {
    let repo_root = workspace_root();
    let smoke = repo_root.join("scripts/smoke-distribution-metadata.sh");
    let renderer = repo_root.join("scripts/render-homebrew-formula.sh");
    assert!(smoke.exists(), "distribution metadata smoke is missing");
    assert!(renderer.exists(), "Homebrew renderer is missing");

    for script in [&smoke, &renderer] {
        let syntax = Command::new("bash")
            .arg("-n")
            .arg(script)
            .current_dir(&repo_root)
            .output()
            .unwrap();
        assert!(
            syntax.status.success(),
            "bash -n failed for {}: {}",
            script.display(),
            String::from_utf8_lossy(&syntax.stderr)
        );
    }

    let smoke_text = fs::read_to_string(&smoke).unwrap();
    for required in [
        "scripts/render-homebrew-formula.sh",
        "CTXHELM_DIST_DIR",
        "CTXHELM_DISTRIBUTION_METADATA_OUT",
        "CTXHELM_UPDATE_DISTRIBUTION_METADATA",
        "$work_dir/distribution-metadata-smoke.json",
        "ctxhelm-v${version}-${target_label}.tar.gz",
        "https://github.com/thromel/ctxhelm/releases/download/v${version}/${archive_name}",
        "cargo package --manifest-path \"$repo_root/crates/$crate/Cargo.toml\" --locked --allow-dirty --list",
        "cargo package --manifest-path \"$repo_root/crates/ctxhelm-core/Cargo.toml\" --locked --allow-dirty --no-verify",
        "homebrewFormulaRender",
        "cratesPackage",
        "packageListCheckedCrates",
        "leafDryRunCheckedCrates",
        "blocked_until_internal_crates_are_published_in_order",
        "publishOrder",
        "sourceFreeBoundaryChecked",
        ".ctxhelm/",
        ".planning/",
        "target/",
        "dist/",
        "traces.jsonl",
        "/Users/",
    ] {
        assert!(
            smoke_text.contains(required),
            "distribution smoke missing {required}"
        );
    }
    for forbidden in [
        "brew tap publish",
        "brew install ctxhelm now",
        "cargo publish --",
        "gh release create ",
    ] {
        assert!(
            !smoke_text.contains(forbidden),
            "distribution smoke must not publish or install: {forbidden}"
        );
    }

    let renderer_text = fs::read_to_string(&renderer).unwrap();
    for required in [
        "CTXHELM_URL",
        "CTXHELM_SHA256",
        "class Ctxhelm < Formula",
        "depends_on arch: :arm64",
        "bin.install",
        "shell_output",
        "https://github[.]com/thromel/ctxhelm/releases/download",
        "package-manager state",
    ] {
        assert!(
            renderer_text.contains(required),
            "Homebrew renderer missing {required}"
        );
    }
    for forbidden in [
        "brew tap publish",
        "brew install ctxhelm now",
        "git push",
        "gh release create",
    ] {
        assert!(
            !renderer_text.contains(forbidden),
            "Homebrew renderer must not mutate package-manager or release state: {forbidden}"
        );
    }
}

#[test]
fn homebrew_tap_verifier_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/verify-homebrew-tap.sh");
    assert!(script.exists(), "Homebrew tap verifier is missing");

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed for {}: {}",
        script.display(),
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "brew tap \"$tap\"",
        "brew audit --strict --new \"$formula\"",
        "brew install \"$tap/$formula\"",
        "brew test \"$tap/$formula\"",
        "depends_on arch: :arm64",
        "ctxhelm-homebrew-tap-proof-v1",
        "sourceTextLogged",
        "global agent config mutation",
    ] {
        assert!(
            script_text.contains(required),
            "Homebrew tap verifier missing {required}"
        );
    }
    for forbidden in ["gh release create", "git push", "cargo publish --"] {
        assert!(
            !script_text.contains(forbidden),
            "Homebrew tap verifier must not publish unrelated release state: {forbidden}"
        );
    }
}

#[test]
fn ci_workflow_contract() {
    let repo_root = workspace_root();
    let workflow = repo_root.join(".github/workflows/ci.yml");
    assert!(workflow.exists(), "CI workflow is missing");

    let workflow_text = fs::read_to_string(&workflow).unwrap();
    for required in [
        "push:",
        "pull_request:",
        "workflow_dispatch:",
        "FORCE_JAVASCRIPT_ACTIONS_TO_NODE24",
        "actions/checkout@v5",
        "actions/cache@v5",
        "cargo-registry-v1",
        "cargo fmt --all -- --check",
        "rustup toolchain install stable --profile minimal --component rustfmt --component clippy",
        "cargo clippy --workspace --all-targets --locked -- -D warnings",
        "cargo test --workspace --locked --no-fail-fast",
        "cargo run -p ctxhelm --locked -- --help",
        "bash scripts/check-release-docs.sh",
        "bash scripts/release-gate.sh",
        "CTXHELM_SKIP_CLEAN_FIXTURE_PROOF",
        "CTXHELM_SKIP_REAL_CLIENT",
    ] {
        assert!(
            workflow_text.contains(required),
            "CI workflow missing {required}"
        );
    }

    assert!(
        !workflow_text.contains("\n            target\n"),
        "CI workflow must not cache target/ build outputs; large target caches can exhaust hosted runner disk"
    );

    assert!(
        !workflow_text.contains("gh release")
            && !workflow_text.contains("git tag")
            && !workflow_text.contains("cargo publish"),
        "CI workflow must not publish artifacts or mutate release state"
    );
}

#[test]
fn inspector_smoke_script_contract() {
    let repo_root = workspace_root();
    let smoke = repo_root.join("scripts/smoke-inspector.sh");
    assert!(smoke.exists(), "inspector smoke is missing");

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&smoke)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed for {}: {}",
        smoke.display(),
        String::from_utf8_lossy(&syntax.stderr)
    );

    let smoke_text = fs::read_to_string(&smoke).unwrap();
    for required in [
        "ctxhelm inspector proof",
        "ctxhelm-proof-inspector-v1",
        "agent_run_suite",
        "ctxhelm_improved",
        "evidenceOnlyTargetsAfterRetry",
        "sourceFreeSummary",
        "product-proof",
        "ctxhelm-proof-inspector-bundle-v1",
        "release_and_agent_outcome_evidence_ready",
        "--require-ready",
        "releaseGateDecision",
        "maxProtectedTargetMissRateAt10",
        "Use as source-free outcome evidence",
        "INSPECTOR_UI_SOURCE_SENTINEL",
    ] {
        assert!(
            smoke_text.contains(required),
            "inspector smoke missing {required}"
        );
    }
}

#[test]
fn release_artifacts_workflow_contract() {
    let repo_root = workspace_root();
    let workflow = repo_root.join(".github/workflows/release-artifacts.yml");
    assert!(workflow.exists(), "release artifacts workflow is missing");

    let workflow_text = fs::read_to_string(&workflow).unwrap();
    for required in [
        "workflow_dispatch:",
        "tags:",
        "permissions:",
        "contents: write",
        "gh release create",
        "gh release upload",
        "startsWith(github.ref, 'refs/tags/')",
        "actions/checkout@v5",
        "actions/cache@v5",
        "cargo-registry-v1",
        "actions/upload-artifact@v6",
        "ubuntu-latest",
        "macos-15-intel",
        "macos-14",
        "x86_64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "rustup toolchain install stable --profile minimal --target",
        "CTXHELM_BUILD_TARGET",
        "CTXHELM_TARGET_LABEL",
        "scripts/release-package.sh",
        "scripts/verify-release-archive.sh",
        "if-no-files-found: error",
        "--clobber",
    ] {
        assert!(
            workflow_text.contains(required),
            "release artifacts workflow missing {required}"
        );
    }

    assert!(
        !workflow_text.contains("\n            target\n"),
        "release artifacts workflow must not cache target/ build outputs; package jobs already produce bounded artifacts"
    );

    for forbidden in ["git tag", "git push", "cargo publish", "brew install"] {
        assert!(
            !workflow_text.contains(forbidden),
            "release artifacts workflow must not create tags or mutate package-manager state: {forbidden}"
        );
    }
}

#[test]
fn public_release_freshness_script_reports_outdated_without_mutation() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/check-public-release-freshness.sh");
    assert!(script.exists(), "release freshness script is missing");

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "gh release view",
        "targetCommitish",
        "currentCommit",
        "releaseTargetCommit",
        "commitsAhead",
        "productStatus",
        "productCommitsAhead",
        "proofOnlyCommitsAhead",
        "ignoredFreshnessPaths",
        "sourceFree",
        "privacyStatus",
        "--require-current",
        "--require-product-current",
        "publishing",
        "tag creation",
        "asset upload",
    ] {
        assert!(
            script_text.contains(required),
            "release freshness script missing {required}"
        );
    }

    let temp = TempDir::new().unwrap();
    let release_json = temp.path().join("release.json");
    let output_json = temp.path().join("freshness.json");
    fs::write(
        &release_json,
        r#"{
  "isDraft": false,
  "isPrerelease": false,
  "publishedAt": "2026-06-01T00:00:00Z",
  "tagName": "v2.4.6",
  "targetCommitish": "release-commit",
  "url": "https://github.com/thromel/ctxhelm/releases/tag/v2.4.6"
}
"#,
    )
    .unwrap();

    let output = Command::new("bash")
        .arg(&script)
        .args(["--tag", "v2.4.6"])
        .args(["--current-commit", "current-commit"])
        .arg("--release-json")
        .arg(&release_json)
        .arg("--output")
        .arg(&output_json)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "freshness script failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&output_json).unwrap()).unwrap();
    assert_eq!(payload["status"], "outdated");
    assert_eq!(payload["releaseTargetCommit"], "release-commit");
    assert_eq!(payload["currentCommit"], "current-commit");
    assert_eq!(payload["sourceFree"], true);
    assert_eq!(payload["privacyStatus"]["sourceTextLogged"], false);

    let required_current = Command::new("bash")
        .arg(&script)
        .args(["--tag", "v2.4.6"])
        .args(["--current-commit", "current-commit"])
        .arg("--release-json")
        .arg(&release_json)
        .arg("--require-current")
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !required_current.status.success(),
        "--require-current should fail when the release is outdated"
    );
}

#[test]
fn public_release_freshness_distinguishes_proof_only_commits() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/check-public-release-freshness.sh");
    let temp = TempDir::new().unwrap();
    let repo = temp.path();

    let init = Command::new("git")
        .arg("init")
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );
    for args in [
        ["config", "user.email", "ctxhelm@example.invalid"],
        ["config", "user.name", "ctxhelm test"],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git config failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(repo.join("src/main.rs"), "fn main() {}\n").unwrap();
    let output = Command::new("git")
        .args(["add", "src/main.rs"])
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(output.status.success());
    let output = Command::new("git")
        .args(["commit", "-m", "product release"])
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "release commit failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let release_commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(release_commit.status.success());
    let release_commit = String::from_utf8(release_commit.stdout)
        .unwrap()
        .trim()
        .to_owned();

    fs::create_dir_all(repo.join(".planning")).unwrap();
    fs::create_dir_all(repo.join(".ctxhelm/e2e")).unwrap();
    fs::write(repo.join(".planning/STATE.md"), "proof state\n").unwrap();
    fs::write(repo.join(".ctxhelm/e2e/proof.json"), "{}\n").unwrap();
    let output = Command::new("git")
        .args(["add", ".planning/STATE.md", ".ctxhelm/e2e/proof.json"])
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(output.status.success());
    let output = Command::new("git")
        .args(["commit", "-m", "record proof"])
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "proof commit failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let current_commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(current_commit.status.success());
    let current_commit = String::from_utf8(current_commit.stdout)
        .unwrap()
        .trim()
        .to_owned();

    let release_json = repo.join("release.json");
    let output_json = repo.join("freshness.json");
    fs::write(
        &release_json,
        format!(
            r#"{{
  "isDraft": false,
  "isPrerelease": false,
  "publishedAt": "2026-06-01T00:00:00Z",
  "tagName": "v2.4.6",
  "targetCommitish": "{release_commit}",
  "url": "https://github.com/thromel/ctxhelm/releases/tag/v2.4.6"
}}
"#
        ),
    )
    .unwrap();

    let output = Command::new("bash")
        .arg(&script)
        .args(["--tag", "v2.4.6"])
        .args(["--current-commit", &current_commit])
        .arg("--release-json")
        .arg(&release_json)
        .arg("--output")
        .arg(&output_json)
        .arg("--require-product-current")
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "proof-only freshness should be product-current: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&output_json).unwrap()).unwrap();
    assert_eq!(payload["status"], "outdated");
    assert_eq!(payload["productStatus"], "current");
    assert_eq!(payload["commitsAhead"], 1);
    assert_eq!(payload["productCommitsAhead"], 0);
    assert_eq!(payload["proofOnlyCommitsAhead"], 1);
    assert_eq!(payload["ignoredFreshnessPaths"][0], ".ctxhelm/e2e/");
    assert_eq!(payload["ignoredFreshnessPaths"][1], ".planning/");

    let required_current = Command::new("bash")
        .arg(&script)
        .args(["--tag", "v2.4.6"])
        .args(["--current-commit", &current_commit])
        .arg("--release-json")
        .arg(&release_json)
        .arg("--require-current")
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        !required_current.status.success(),
        "exact currentness should still fail for proof-only commits"
    );
}

#[test]
fn claude_workflow_eval_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/e2e-claude-workflow.sh");
    assert!(script.exists(), "Claude workflow eval script is missing");

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "scripts/smoke-claude-mcp.sh",
        "CTXHELM_CLAUDE_WORKFLOW_REPORT",
        "CTXHELM_CLAUDE_WORKFLOW_EVIDENCE_DIR",
        "CTXHELM_RUN_REAL_CLIENT",
        "CTXHELM_REQUIRE_REAL_CLIENT",
        "ctxhelm-claude-workflow-eval-v1",
        "claude-code-mcp-context-workflow",
        "explicitRepoToolCallCountAtLeastTwo",
        "prepareTaskToolCall",
        "getPackToolCall",
        "requestLogSha256",
        "rawRequestLogStored",
        "rawPromptStored",
        "sourceTextLogged",
        "userProjectCommandsRun",
        "localOnly",
        "remoteEmbeddingsUsed",
        "remoteRerankingUsed",
    ] {
        assert!(
            script_text.contains(required),
            "Claude workflow eval script must mention {required}"
        );
    }

    for forbidden in [
        "cat \"$request_log\"",
        "cp \"$request_log\"",
        "rawMcpTrafficStored\": True",
    ] {
        assert!(
            !script_text.contains(forbidden),
            "Claude workflow eval must not persist raw request traffic: {forbidden}"
        );
    }
}

#[test]
fn product_proof_checker_accepts_promote_and_rejects_block() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/check-product-proof.py");
    assert!(script.exists(), "product proof checker is missing");

    let temp = TempDir::new().unwrap();
    let promote_path = temp.path().join("promote.json");
    let block_path = temp.path().join("block.json");
    let missing_resource_path = temp.path().join("missing-resource-gap.json");
    let missing_report_path = temp.path().join("missing-repo-report.json");
    let missing_source_path = temp.path().join("missing-source-recall.json");
    let source_regression_path = temp.path().join("source-regression.json");
    let broad_floor_path = temp.path().join("broad-fixed-corpus-floor.json");
    let broad_regression_path = temp.path().join("broad-fixed-corpus-regression.json");
    fs::write(&promote_path, product_proof_json("promote", true, "beat")).unwrap();
    fs::write(&block_path, product_proof_json("block", false, "match")).unwrap();
    fs::write(
        &missing_resource_path,
        product_proof_json_without_gap_resource_uri(),
    )
    .unwrap();
    fs::write(
        &missing_report_path,
        product_proof_json_without_repository_report(),
    )
    .unwrap();
    fs::write(
        &missing_source_path,
        product_proof_json_without_source_recall_fields(),
    )
    .unwrap();
    fs::write(
        &source_regression_path,
        product_proof_json_with_source_recall_regression(),
    )
    .unwrap();
    fs::write(
        &broad_floor_path,
        product_proof_json_with_broad_fixed_corpus(0.18449473),
    )
    .unwrap();
    fs::write(
        &broad_regression_path,
        product_proof_json_with_broad_fixed_corpus(0.17936651),
    )
    .unwrap();

    let promote = python3_command()
        .arg(&script)
        .arg(&promote_path)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        promote.status.success(),
        "promote proof should pass: {}",
        String::from_utf8_lossy(&promote.stderr)
    );

    let block = python3_command()
        .arg(&script)
        .arg(&block_path)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !block.status.success(),
        "block proof should fail release checker"
    );
    let stderr = String::from_utf8_lossy(&block.stderr);
    assert!(
        stderr.contains("releaseGate.decision was not promote")
            || stderr.contains("corpus did not beat lexical"),
        "unexpected checker error: {stderr}"
    );

    let missing_resource = python3_command()
        .arg(&script)
        .arg(&missing_resource_path)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !missing_resource.status.success(),
        "current reachable gaps without resource-backed next reads should fail release checker"
    );
    let stderr = String::from_utf8_lossy(&missing_resource.stderr);
    assert!(
        stderr.contains("lacked context-area resource URI")
            || stderr.contains("lacked next-read paths"),
        "unexpected checker error: {stderr}"
    );

    let missing_report = python3_command()
        .arg(&script)
        .arg(&missing_report_path)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !missing_report.status.success(),
        "benchmark repository without an embedded report should fail release checker"
    );
    let stderr = String::from_utf8_lossy(&missing_report.stderr);
    assert!(
        stderr.contains("embedded benchmark repository report was missing"),
        "unexpected checker error: {stderr}"
    );

    let missing_source = python3_command()
        .arg(&script)
        .arg(&missing_source_path)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !missing_source.status.success(),
        "stale product proof without source-recall fields should fail release checker"
    );
    let stderr = String::from_utf8_lossy(&missing_source.stderr);
    assert!(
        stderr.contains("missing source-recall field sourceRecallAt10"),
        "unexpected checker error: {stderr}"
    );

    let source_regression = python3_command()
        .arg(&script)
        .arg(&source_regression_path)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !source_regression.status.success(),
        "product proof with source recall regression should fail release checker"
    );
    let stderr = String::from_utf8_lossy(&source_regression.stderr);
    assert!(
        stderr.contains("source recall regression") && stderr.contains("sourceDeltaAt10"),
        "unexpected checker error: {stderr}"
    );

    let broad_floor = python3_command()
        .arg(&script)
        .arg(&broad_floor_path)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        broad_floor.status.success(),
        "broad fixed-corpus floor proof should pass: {}",
        String::from_utf8_lossy(&broad_floor.stderr)
    );

    let broad_regression = python3_command()
        .arg(&script)
        .arg(&broad_regression_path)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !broad_regression.status.success(),
        "broad fixed-corpus metric regression should fail release checker"
    );
    let stderr = String::from_utf8_lossy(&broad_regression.stderr);
    assert!(
        stderr.contains("broad fixed corpus metric regressed below floor")
            && stderr.contains("VeriSchema.fileRecallAt10"),
        "unexpected checker error: {stderr}"
    );
}

#[test]
fn agent_run_proof_checker_accepts_phase322_and_rejects_regression() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/check-agent-run-proof.py");
    assert!(script.exists(), "agent-run proof checker is missing");

    let compile = python3_command()
        .arg("-m")
        .arg("py_compile")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "py_compile failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let phase322_report =
        repo_root.join(".ctxhelm/e2e/phase322-agent-run-codex-target-first-breadth-suite.json");
    let codex_runner_script = repo_root.join("scripts/e2e-agent-run-codex.sh");
    let codex_suite = repo_root.join(".planning/e2e/2026-06-06-phase251-codex-rd-suite.json");
    assert!(
        phase322_report.exists(),
        "Phase 322 agent-run proof fixture is missing"
    );
    assert!(
        codex_runner_script.exists(),
        "Codex agent-run runner script is missing"
    );
    assert!(
        codex_suite.exists(),
        "Codex agent-run suite file is missing"
    );

    let proof_args = [
        "--workflow",
        "suite",
        "--require-outcome",
        "ctxhelm_improved",
        "--expected-ctxhelm-version",
        "ctxhelm 2.4.0",
        "--expected-client-name",
        "codex",
        "--expected-client-version",
        "codex-cli 0.137.0",
        "--min-task-count",
        "4",
        "--min-comparison-eligible",
        "4",
        "--min-comparable-ctxhelm-lanes",
        "16",
        "--min-ctxhelm-target-read-coverage",
        "1.0",
        "--max-extra-read-delta",
        "2",
        "--min-irrelevant-read-delta",
        "2",
        "--require-retry-cost",
        "--require-runner-fingerprint",
    ];

    let accepted = python3_command()
        .arg(&script)
        .arg(&phase322_report)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        accepted.status.success(),
        "Phase 322 proof should pass: stdout={} stderr={}",
        String::from_utf8_lossy(&accepted.stdout),
        String::from_utf8_lossy(&accepted.stderr)
    );

    let summary_temp = TempDir::new().unwrap();
    let summary_path = summary_temp.path().join("agent-run-proof-check.json");
    let accepted_json = python3_command()
        .arg(&script)
        .arg(&phase322_report)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&summary_path)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        accepted_json.status.success(),
        "Phase 322 JSON proof check should pass: {}",
        String::from_utf8_lossy(&accepted_json.stderr)
    );
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
    assert_eq!(summary["schemaVersion"], "ctxhelm-agent-run-proof-check-v1");
    assert_eq!(summary["status"], "passed");
    assert_eq!(summary["sourceFree"], true);
    assert_eq!(summary["metrics"]["outcomeClaim"], "ctxhelm_improved");
    assert_eq!(
        summary["metrics"]["targetReadCoverageDeltaAverage"],
        serde_json::Value::Number(serde_json::Number::from_f64(0.3125).unwrap())
    );
    assert_eq!(
        summary["metrics"]["targetCoverageDeltaAverage"],
        serde_json::Value::Number(serde_json::Number::from_f64(0.25).unwrap())
    );
    assert_eq!(summary["metrics"]["commandExecutionDeltaSum"], 9);
    assert_eq!(summary["metrics"]["ctxhelmToolCallsObserved"], true);
    assert!(
        summary["metrics"]
            .get("targetReadCoverageDeltaAvg")
            .is_none(),
        "suite proof summary should use the validated aggregate field name"
    );
    assert!(
        summary["metrics"].get("targetCoverageDeltaAvg").is_none(),
        "suite proof summary should use the validated aggregate field name"
    );
    assert_eq!(summary["identity"]["matchesExpectedCtxhelmVersion"], true);
    assert_eq!(summary["identity"]["matchesExpectedClientName"], true);
    assert_eq!(summary["identity"]["matchesExpectedClientVersion"], true);
    assert_eq!(summary["thresholds"]["minTaskCount"], 4);
    assert_eq!(
        summary["thresholds"]["expectedCtxhelmVersion"],
        "ctxhelm 2.4.0"
    );
    assert_eq!(summary["thresholds"]["expectedClientName"], "codex");
    assert_eq!(
        summary["thresholds"]["expectedClientVersion"],
        "codex-cli 0.137.0"
    );

    let run_summary_path = summary_temp
        .path()
        .join("agent-run-proof-single-run-check.json");
    let phase322_suite: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    let run_task = phase322_suite["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| task["taskId"] == "governor-release-proof")
        .expect("Phase 322 suite should include governor-release-proof task");
    let phase322_run_report = summary_temp
        .path()
        .join("phase322-derived-agent-run-proof.json");
    let derived_run_report = serde_json::json!({
        "schemaVersion": phase322_suite["schemaVersion"],
        "workflowKind": "paired-agent-context-run",
        "status": run_task["status"],
        "ctxhelmVersion": phase322_suite["ctxhelmVersion"],
        "client": phase322_suite["client"],
        "repo": phase322_suite["repo"],
        "privacyStatus": run_task["privacyStatus"],
        "runner": phase322_suite["runner"],
        "targetFiles": run_task["targetFiles"],
        "comparison": run_task["comparison"],
        "lanes": run_task["lanes"],
    });
    fs::write(
        &phase322_run_report,
        serde_json::to_vec_pretty(&derived_run_report).unwrap(),
    )
    .unwrap();
    let accepted_run_json = python3_command()
        .arg(&script)
        .arg(&phase322_run_report)
        .args([
            "--workflow",
            "run",
            "--require-outcome",
            "ctxhelm_improved",
            "--expected-ctxhelm-version",
            "ctxhelm 2.4.0",
            "--expected-client-name",
            "codex",
            "--expected-client-version",
            "codex-cli 0.137.0",
            "--min-comparable-ctxhelm-lanes",
            "4",
            "--min-ctxhelm-target-read-coverage",
            "1.0",
            "--max-extra-read-delta",
            "2",
            "--min-irrelevant-read-delta",
            "0",
            "--require-retry-cost",
            "--require-runner-fingerprint",
        ])
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&run_summary_path)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        accepted_run_json.status.success(),
        "Phase 322 single-run JSON proof check should pass: stdout={} stderr={}",
        String::from_utf8_lossy(&accepted_run_json.stdout),
        String::from_utf8_lossy(&accepted_run_json.stderr)
    );
    let run_summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&run_summary_path).unwrap()).unwrap();
    assert_eq!(run_summary["workflow"], "run");
    assert_eq!(run_summary["metrics"]["commandExecutionDelta"], 2);
    assert_eq!(run_summary["metrics"]["ctxhelmToolCallsObserved"], true);

    assert_eq!(summary["thresholds"]["requireRunnerFingerprint"], true);
    assert_eq!(summary["thresholds"]["requireCurrentRunnerScript"], true);
    assert_eq!(summary["thresholds"]["requireCurrentSuite"], true);
    assert_eq!(summary["privacyStatus"]["localOnly"], true);
    assert_eq!(summary["privacyStatus"]["sourceTextLogged"], false);
    assert_eq!(summary["privacyStatus"]["rawPromptStored"], false);
    assert_eq!(summary["privacyStatus"]["rawTranscriptStored"], false);
    assert_eq!(summary["privacyStatus"]["rawMcpTrafficStored"], false);
    assert_eq!(summary["privacyStatus"]["rawCommandOutputStored"], false);
    assert_eq!(summary["privacyStatus"]["remoteEmbeddingsUsed"], false);
    assert_eq!(summary["privacyStatus"]["remoteRerankingUsed"], false);
    assert_eq!(summary["privacyChecks"]["localOnly"], true);
    assert_eq!(summary["privacyChecks"]["allSourceFreeFieldsFalse"], true);
    assert_eq!(
        summary["privacyChecks"]["strictFalseFields"]["sourceTextLogged"],
        true
    );
    assert_eq!(summary["boundaryStatus"]["clientFailuresObserved"], false);
    assert_eq!(summary["boundaryStatus"]["rateLimitsObserved"], false);
    assert_eq!(
        summary["boundaryStatus"]["forbiddenCommandsObserved"],
        false
    );
    assert_eq!(
        summary["boundaryStatus"]["ctxhelmEvidenceMissesObserved"],
        false
    );
    assert_eq!(
        summary["boundaryStatus"]["ctxhelmEvidenceOnlyTargetsObserved"],
        false
    );
    assert_eq!(
        summary["boundaryStatus"]["ctxhelmUnderReadTargetsObserved"],
        false
    );
    assert_eq!(
        summary["boundaryStatus"]["missingRequiredCtxhelmCallsObserved"],
        false
    );
    assert_eq!(
        summary["boundaryStatus"]["invalidRequiredCtxhelmCallsObserved"],
        false
    );
    assert_eq!(summary["boundaryChecks"]["clientFailuresObserved"], true);
    assert_eq!(summary["boundaryChecks"]["rateLimitsObserved"], true);
    assert_eq!(summary["boundaryChecks"]["forbiddenCommandsObserved"], true);
    assert_eq!(summary["runner"]["matchesCurrentRunnerScript"], true);
    assert_eq!(summary["suite"]["matchesCurrentSuite"], true);
    assert_eq!(
        summary["suiteTaskChecks"]["strictCurrentSuiteTaskChecks"],
        true
    );
    assert_eq!(summary["suiteTaskChecks"]["reportTaskCount"], 4);
    assert_eq!(summary["suiteTaskChecks"]["currentSuiteTaskCount"], 4);
    assert_eq!(summary["suiteTaskChecks"]["matchesCurrentSuiteTasks"], true);
    assert_eq!(summary["suiteConsistency"]["strictSuiteStatusChecks"], true);
    assert_eq!(summary["suiteConsistency"]["suiteTaskCount"], 4);
    assert_eq!(summary["suiteConsistency"]["derivedTaskCount"], 4);
    assert_eq!(
        summary["suiteConsistency"]["derivedComparisonEligibleCount"],
        4
    );
    assert_eq!(
        summary["suiteConsistency"]["derivedBoundaryObserved"],
        false
    );
    assert_eq!(summary["suiteConsistency"]["derivedStatus"], "passed");
    assert_eq!(summary["suiteConsistency"]["matchesDerivedTaskCount"], true);
    assert_eq!(summary["suiteConsistency"]["matchesDerivedStatus"], true);
    assert_eq!(summary["taskLaneChecks"]["strictTaskLaneChecks"], true);
    assert_eq!(summary["taskLaneChecks"]["taskLaneCount"], 20);
    assert_eq!(summary["taskLaneChecks"]["ctxhelmTaskLaneCount"], 16);
    assert_eq!(
        summary["aggregateConsistency"]["strictAggregateConsistencyChecks"],
        true
    );
    assert_eq!(summary["aggregateConsistency"]["derivedTaskCount"], 4);
    assert_eq!(
        summary["aggregateConsistency"]["derivedComparisonEligibleCount"],
        4
    );
    assert_eq!(
        summary["aggregateConsistency"]["derivedComparableCtxhelmLaneCount"],
        16
    );
    assert_eq!(summary["aggregateConsistency"]["derivedLaneNameCount"], 5);
    assert_eq!(summary["aggregateConsistency"]["laneSummaryCount"], 5);
    assert_eq!(
        summary["aggregateConsistency"]["strictLaneSummaryMetricChecks"],
        true
    );
    assert_eq!(
        summary["aggregateConsistency"]["checkedLaneSummaryCount"],
        5
    );
    assert_eq!(
        summary["aggregateConsistency"]["checkedLaneSummaryMetricCount"],
        145
    );
    assert_eq!(
        summary["aggregateConsistency"]["strictRetryCostConsistencyChecks"],
        true
    );
    assert_eq!(
        summary["aggregateConsistency"]["checkedRetryCostMetricCount"],
        10
    );
    assert_eq!(
        summary["aggregateConsistency"]["strictReadEfficiencyConsistencyChecks"],
        true
    );
    assert_eq!(
        summary["aggregateConsistency"]["checkedReadEfficiencyMetricCount"],
        18
    );
    assert_eq!(
        summary["aggregateConsistency"]["strictComparisonAggregateChecks"],
        true
    );
    assert_eq!(
        summary["aggregateConsistency"]["checkedComparisonAggregateMetricCount"],
        6
    );
    assert_eq!(
        summary["aggregateConsistency"]["strictOutcomeRoutingChecks"],
        true
    );
    assert_eq!(
        summary["aggregateConsistency"]["derivedOutcomeClaim"],
        "ctxhelm_improved"
    );
    assert_eq!(
        summary["aggregateConsistency"]["checkedRecommendedResearchActionCount"],
        1
    );
    assert_eq!(
        summary["aggregateConsistency"]["matchesDerivedAggregates"],
        true
    );
    assert!(summary["reportSha256"].as_str().unwrap().len() == 64);

    let temp = TempDir::new().unwrap();
    let mut payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    payload["aggregate"]["ctxhelmEvidenceOnlyTargetsObserved"] = serde_json::Value::Bool(true);
    let rejected_path = temp.path().join("agent-run-regression.json");
    fs::write(
        &rejected_path,
        serde_json::to_string_pretty(&payload).unwrap(),
    )
    .unwrap();

    let rejected = python3_command()
        .arg(&script)
        .arg(&rejected_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !rejected.status.success(),
        "proof with evidence-only target regression should fail"
    );
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        stderr.contains("ctxhelmEvidenceOnlyTargetsObserved"),
        "unexpected checker error: {stderr}"
    );

    let stale_suite_task_count_path = temp.path().join("agent-run-stale-suite-task-count.json");
    let mut stale_suite_task_count_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_suite_task_count_payload["suite"]["taskCount"] =
        serde_json::Value::Number(serde_json::Number::from(5));
    fs::write(
        &stale_suite_task_count_path,
        serde_json::to_string_pretty(&stale_suite_task_count_payload).unwrap(),
    )
    .unwrap();
    let stale_suite_task_count = python3_command()
        .arg(&script)
        .arg(&stale_suite_task_count_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_suite_task_count.status.success(),
        "proof with stale suite task count should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_suite_task_count.stderr);
    assert!(
        stderr.contains("suite.taskCount did not match derived tasks"),
        "unexpected stale-suite-task-count checker error: {stderr}"
    );

    let stale_status_path = temp.path().join("agent-run-stale-status.json");
    let mut stale_status_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_status_payload["status"] = serde_json::Value::String("skipped".to_string());
    fs::write(
        &stale_status_path,
        serde_json::to_string_pretty(&stale_status_payload).unwrap(),
    )
    .unwrap();
    let stale_status = python3_command()
        .arg(&script)
        .arg(&stale_status_path)
        .arg("--require-status")
        .arg("skipped")
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_status.status.success(),
        "proof with stale suite status should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_status.stderr);
    assert!(
        stderr.contains("report.status did not match derived suite status"),
        "unexpected stale-status checker error: {stderr}"
    );

    let stale_aggregate_path = temp.path().join("agent-run-stale-aggregate.json");
    let mut stale_aggregate_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_aggregate_payload["aggregate"]["comparisonEligibleCount"] =
        serde_json::Value::Number(serde_json::Number::from(5));
    fs::write(
        &stale_aggregate_path,
        serde_json::to_string_pretty(&stale_aggregate_payload).unwrap(),
    )
    .unwrap();
    let stale_aggregate = python3_command()
        .arg(&script)
        .arg(&stale_aggregate_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_aggregate.status.success(),
        "proof with stale aggregate comparison count should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_aggregate.stderr);
    assert!(
        stderr.contains("aggregate.comparisonEligibleCount did not match derived task comparisons"),
        "unexpected stale-aggregate checker error: {stderr}"
    );

    let stale_lane_summary_path = temp.path().join("agent-run-stale-lane-summary.json");
    let mut stale_lane_summary_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_lane_summary_payload["aggregate"]["laneSummaries"][0]["lane"] =
        serde_json::Value::String("not-a-derived-lane".to_string());
    fs::write(
        &stale_lane_summary_path,
        serde_json::to_string_pretty(&stale_lane_summary_payload).unwrap(),
    )
    .unwrap();
    let stale_lane_summary = python3_command()
        .arg(&script)
        .arg(&stale_lane_summary_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_lane_summary.status.success(),
        "proof with stale lane summary names should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_lane_summary.stderr);
    assert!(
        stderr.contains("aggregate.laneSummaries lane names did not match derived task lanes"),
        "unexpected stale-lane-summary checker error: {stderr}"
    );

    let stale_lane_summary_metric_path =
        temp.path().join("agent-run-stale-lane-summary-metric.json");
    let mut stale_lane_summary_metric_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_lane_summary_metric_payload["aggregate"]["laneSummaries"][0]["readFileCount"] =
        serde_json::Value::Number(serde_json::Number::from(18));
    fs::write(
        &stale_lane_summary_metric_path,
        serde_json::to_string_pretty(&stale_lane_summary_metric_payload).unwrap(),
    )
    .unwrap();
    let stale_lane_summary_metric = python3_command()
        .arg(&script)
        .arg(&stale_lane_summary_metric_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_lane_summary_metric.status.success(),
        "proof with stale lane summary metrics should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_lane_summary_metric.stderr);
    assert!(
        stderr.contains(
            "aggregate.laneSummaries[baseline].readFileCount did not match derived task lanes"
        ),
        "unexpected stale-lane-summary-metric checker error: {stderr}"
    );

    let stale_retry_cost_path = temp.path().join("agent-run-stale-retry-cost.json");
    let mut stale_retry_cost_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_retry_cost_payload["aggregate"]["retryCost"]["avgReadFilesAfterRetry"] =
        serde_json::Value::Number(serde_json::Number::from_f64(6.8).unwrap());
    fs::write(
        &stale_retry_cost_path,
        serde_json::to_string_pretty(&stale_retry_cost_payload).unwrap(),
    )
    .unwrap();
    let stale_retry_cost = python3_command()
        .arg(&script)
        .arg(&stale_retry_cost_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_retry_cost.status.success(),
        "proof with stale retry-cost metrics should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_retry_cost.stderr);
    assert!(
        stderr.contains(
            "aggregate.retryCost.avgReadFilesAfterRetry did not match derived task retry costs"
        ),
        "unexpected stale-retry-cost checker error: {stderr}"
    );

    let stale_read_efficiency_path = temp.path().join("agent-run-stale-read-efficiency.json");
    let mut stale_read_efficiency_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_read_efficiency_payload["aggregate"]["readEfficiency"]["extraReadFileCount"] =
        serde_json::Value::Number(serde_json::Number::from(5));
    fs::write(
        &stale_read_efficiency_path,
        serde_json::to_string_pretty(&stale_read_efficiency_payload).unwrap(),
    )
    .unwrap();
    let stale_read_efficiency = python3_command()
        .arg(&script)
        .arg(&stale_read_efficiency_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_read_efficiency.status.success(),
        "proof with stale read-efficiency metrics should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_read_efficiency.stderr);
    assert!(
        stderr.contains(
            "aggregate.readEfficiency.extraReadFileCount did not match derived lane summaries"
        ),
        "unexpected stale-read-efficiency checker error: {stderr}"
    );

    let stale_delta_path = temp.path().join("agent-run-stale-aggregate-delta.json");
    let mut stale_delta_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_delta_payload["aggregate"]["targetReadCoverageDeltaAverage"] =
        serde_json::Value::Number(serde_json::Number::from_f64(0.4125).unwrap());
    fs::write(
        &stale_delta_path,
        serde_json::to_string_pretty(&stale_delta_payload).unwrap(),
    )
    .unwrap();
    let stale_delta = python3_command()
        .arg(&script)
        .arg(&stale_delta_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_delta.status.success(),
        "proof with stale aggregate delta metrics should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_delta.stderr);
    assert!(
        stderr.contains(
            "aggregate.targetReadCoverageDeltaAverage did not match derived task comparisons"
        ),
        "unexpected stale-delta checker error: {stderr}"
    );

    let stale_tool_call_path = temp.path().join("agent-run-stale-tool-call-observed.json");
    let mut stale_tool_call_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_tool_call_payload["aggregate"]["ctxhelmToolCallsObserved"] =
        serde_json::Value::Bool(false);
    fs::write(
        &stale_tool_call_path,
        serde_json::to_string_pretty(&stale_tool_call_payload).unwrap(),
    )
    .unwrap();
    let stale_tool_call = python3_command()
        .arg(&script)
        .arg(&stale_tool_call_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_tool_call.status.success(),
        "proof with stale ctxhelm tool-call aggregate should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_tool_call.stderr);
    assert!(
        stderr
            .contains("aggregate.ctxhelmToolCallsObserved did not match derived task comparisons"),
        "unexpected stale-tool-call checker error: {stderr}"
    );

    let stale_outcome_path = temp.path().join("agent-run-stale-outcome.json");
    let mut stale_outcome_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_outcome_payload["aggregate"]["outcomeClaim"] =
        serde_json::Value::String("ctxhelm_matched".to_string());
    fs::write(
        &stale_outcome_path,
        serde_json::to_string_pretty(&stale_outcome_payload).unwrap(),
    )
    .unwrap();
    let stale_outcome = python3_command()
        .arg(&script)
        .arg(&stale_outcome_path)
        .args([
            "--workflow",
            "suite",
            "--expected-ctxhelm-version",
            "ctxhelm 2.4.0",
            "--expected-client-name",
            "codex",
            "--expected-client-version",
            "codex-cli 0.137.0",
            "--min-task-count",
            "4",
            "--min-comparison-eligible",
            "4",
            "--min-comparable-ctxhelm-lanes",
            "16",
            "--min-ctxhelm-target-read-coverage",
            "1.0",
            "--max-extra-read-delta",
            "2",
            "--min-irrelevant-read-delta",
            "2",
            "--require-retry-cost",
            "--require-runner-fingerprint",
        ])
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_outcome.status.success(),
        "proof with stale outcome claim should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_outcome.stderr);
    assert!(
        stderr.contains("aggregate.outcomeClaim did not match derived task comparisons"),
        "unexpected stale-outcome checker error: {stderr}"
    );

    let stale_action_path = temp.path().join("agent-run-stale-research-action.json");
    let mut stale_action_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_action_payload["aggregate"]["recommendedResearchActions"][0]["action"] =
        serde_json::Value::String("preserve_current_agent_contract".to_string());
    fs::write(
        &stale_action_path,
        serde_json::to_string_pretty(&stale_action_payload).unwrap(),
    )
    .unwrap();
    let stale_action = python3_command()
        .arg(&script)
        .arg(&stale_action_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_action.status.success(),
        "proof with stale research action should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_action.stderr);
    assert!(
        stderr.contains("aggregate.recommendedResearchActions"),
        "unexpected stale-research-action checker error: {stderr}"
    );

    let stale_runner_path = temp.path().join("agent-run-stale-runner.json");
    let mut stale_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_payload["runner"]["scriptSha256"] = serde_json::Value::String("0".repeat(64));
    fs::write(
        &stale_runner_path,
        serde_json::to_string_pretty(&stale_payload).unwrap(),
    )
    .unwrap();
    let stale_runner = python3_command()
        .arg(&script)
        .arg(&stale_runner_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_runner.status.success(),
        "proof with stale runner fingerprint should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_runner.stderr);
    assert!(
        stderr.contains("did not match current runner script"),
        "unexpected stale-runner checker error: {stderr}"
    );

    let stale_suite_path = temp.path().join("agent-run-stale-suite.json");
    let mut stale_suite_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_suite_payload["suite"]["suiteSha256"] = serde_json::Value::String("0".repeat(64));
    fs::write(
        &stale_suite_path,
        serde_json::to_string_pretty(&stale_suite_payload).unwrap(),
    )
    .unwrap();
    let stale_suite = python3_command()
        .arg(&script)
        .arg(&stale_suite_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_suite.status.success(),
        "proof with stale suite fingerprint should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_suite.stderr);
    assert!(
        stderr.contains("did not match current suite"),
        "unexpected stale-suite checker error: {stderr}"
    );

    let stale_client_path = temp.path().join("agent-run-stale-client.json");
    let mut stale_client_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_client_payload["client"]["version"] =
        serde_json::Value::String("codex-cli 0.136.0".to_string());
    fs::write(
        &stale_client_path,
        serde_json::to_string_pretty(&stale_client_payload).unwrap(),
    )
    .unwrap();
    let stale_client = python3_command()
        .arg(&script)
        .arg(&stale_client_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_client.status.success(),
        "proof with stale client version should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_client.stderr);
    assert!(
        stderr.contains("client.version"),
        "unexpected stale-client checker error: {stderr}"
    );

    let stale_lane_path = temp.path().join("agent-run-stale-task-lane.json");
    let mut stale_lane_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_lane_payload["tasks"][0]["lanes"][1]["metrics"]["ctxhelmEvidenceOnlyTargetCount"] =
        serde_json::Value::Number(serde_json::Number::from(1));
    fs::write(
        &stale_lane_path,
        serde_json::to_string_pretty(&stale_lane_payload).unwrap(),
    )
    .unwrap();
    let stale_lane = python3_command()
        .arg(&script)
        .arg(&stale_lane_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_lane.status.success(),
        "proof with stale task-lane metrics should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_lane.stderr);
    assert!(
        stderr.contains("tasks[0].lanes[1].metrics.ctxhelmEvidenceOnlyTargetCount"),
        "unexpected stale-lane checker error: {stderr}"
    );

    let stale_task_path = temp.path().join("agent-run-stale-suite-task.json");
    let mut stale_task_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase322_report).unwrap()).unwrap();
    stale_task_payload["tasks"][0]["targetFiles"][0] =
        serde_json::Value::String("scripts/not-the-current-suite-target.sh".to_string());
    fs::write(
        &stale_task_path,
        serde_json::to_string_pretty(&stale_task_payload).unwrap(),
    )
    .unwrap();
    let stale_task = python3_command()
        .arg(&script)
        .arg(&stale_task_path)
        .args(proof_args)
        .arg("--current-runner-script")
        .arg(&codex_runner_script)
        .arg("--current-suite")
        .arg(&codex_suite)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        !stale_task.status.success(),
        "proof with stale suite task targets should fail"
    );
    let stderr = String::from_utf8_lossy(&stale_task.stderr);
    assert!(
        stderr.contains("tasks[0].targetFiles did not match current suite task"),
        "unexpected stale-task checker error: {stderr}"
    );
}

fn product_proof_json(decision: &str, default_promotion_allowed: bool, status: &str) -> String {
    format!(
        r#"{{
  "privacyStatus": {{"localOnly": true}},
  "benchmarkReport": {{
    "privacyStatus": {{"localOnly": true}},
    "repositories": [{{
      "name": "fixture",
      "report": {{
        "contextAreaPressureSummary": {{
          "contextAreaCount": 1,
          "zeroSelectedAreaCount": 0,
          "totalInspectionPressure": 3,
          "sourceLikePressure": 3,
          "validationPressure": 0,
          "docsPressure": 0,
          "sourceTextLogged": false
        }},
        "contextAreaNextReadSummary": {{
          "missedFileCountAt10": 1,
          "nextReadRecoverableCount": 1,
          "agentEvidenceRecoverableCount": 1,
          "topPressureNextReadRecoverableCount": 1,
          "zeroSelectedAreaRecoverableCount": 0,
          "sourceTextLogged": false
        }},
    "candidateCoverageSummary": {{
      "missedFileCountAt10": 1,
      "candidateRecoverableCount": 1,
      "noCandidateCount": 0,
      "candidateRecoverableRoleCounts": {{"source": 1}},
      "candidateRecoverableSignalCounts": {{"dependency": 1}},
      "noCandidateRoleCounts": {{}},
      "topCandidateRecoverableAreas": [
        {{
          "contextArea": "src/auth",
          "missedCount": 1
        }}
      ],
      "sourceTextLogged": false
    }},
        "retrievalGapSummaries": [{{
          "role": "source",
          "signalGap": "ranked_below_budget_dependency",
          "package": "src",
          "pathFamily": "src/*.rs",
          "contextArea": "src",
          "contextAreaResourceUri": "ctxhelm://repo/context-area/src",
          "targetStatus": "currentReachable",
          "recommendationArea": "parserPrecision",
          "missedCount": 1,
          "examplePaths": ["src/lib.rs"],
          "nextReadPaths": ["src/lib.rs"]
        }}]
      }}
    }}]
  }},
  "headlineMetrics": [{{"label": "averageCtxhelmLiftAt10", "value": 0.1}}],
  "v23EvalSummary": {{
    "fixedCorpusId": "fixture",
    "pairedBaselineVerdicts": [],
    "featureExportPrivacy": {{"localOnly": true, "sourceTextLogged": false}},
    "learnedPolicyStatus": {{"defaultRequiresThresholds": true, "silentDefaultAllowed": false}},
    "proofBoundary": "world-class claims require repeated lift"
  }},
  "releaseGate": {{
    "decision": "{decision}",
    "defaultPromotionAllowed": {default_promotion_allowed},
    "corpusVerdicts": [{{
      "repository": "fixture",
      "status": "{status}",
      "lexicalDeltaAt10": 0.1,
      "sourceRecallAt10": 0.9,
      "lexicalSourceRecallAt10": 0.8,
      "sourceDeltaAt10": 0.1,
      "contextVsAllFileDeltaAt10": 0.0,
      "lexicalContextVsAllFileDeltaAt10": 0.0,
      "allFileDivergenceExplained": true
    }}]
  }}
}}"#
    )
}

fn product_proof_json_without_gap_resource_uri() -> String {
    product_proof_json("promote", true, "beat").replace(
        r#"          "contextAreaResourceUri": "ctxhelm://repo/context-area/src",
"#,
        "",
    )
}

fn product_proof_json_without_repository_report() -> String {
    let mut value: serde_json::Value =
        serde_json::from_str(&product_proof_json("promote", true, "beat")).unwrap();
    value["benchmarkReport"]["repositories"][0]["report"] = serde_json::Value::Null;
    serde_json::to_string_pretty(&value).unwrap()
}

fn product_proof_json_without_source_recall_fields() -> String {
    product_proof_json("promote", true, "beat")
        .replace(
            r#"      "sourceRecallAt10": 0.9,
"#,
            "",
        )
        .replace(
            r#"      "lexicalSourceRecallAt10": 0.8,
"#,
            "",
        )
        .replace(
            r#"      "sourceDeltaAt10": 0.1,
"#,
            "",
        )
}

fn product_proof_json_with_source_recall_regression() -> String {
    product_proof_json("promote", true, "beat").replace(
        r#"      "sourceDeltaAt10": 0.1,"#,
        r#"      "sourceDeltaAt10": -0.031,"#,
    )
}

fn product_proof_json_with_broad_fixed_corpus(verischema_file_recall_at_10: f64) -> String {
    format!(
        r#"{{
  "privacyStatus": {{"localOnly": true}},
  "benchmarkReport": {{
    "corpusId": "phase92-area-aware-gap-taxonomy-2026-05-31",
    "privacyStatus": {{"localOnly": true}},
    "repositories": [
      {{
        "name": "RefactoringMiner",
        "report": {{
          "fileRecallAt10": 0.6,
          "sourceRecallAt10": 1.0,
          "testRecallAt10": 1.0,
          "effectiveValidationRecallAt10": 1.0
        }}
      }},
      {{
        "name": "ctxhelm",
        "report": {{
          "fileRecallAt10": 0.47460318,
          "sourceRecallAt10": 0.7166667,
          "broadContextAreaRecall": 1.0
        }}
      }},
      {{
        "name": "ReAgent",
        "report": {{
          "fileRecallAt10": 0.5,
          "sourceRecallAt10": 1.0,
          "testRecallAt10": 1.0,
          "effectiveValidationRecallAt10": 1.0
        }}
      }},
      {{
        "name": "VeriSchema",
        "report": {{
          "fileRecallAt10": {verischema_file_recall_at_10},
          "sourceRecallAt10": 0.31067252,
          "testRecallAt10": 0.7089947,
          "effectiveValidationRecallAt10": 1.0,
          "broadContextAreaRecall": 0.71851856
        }}
      }}
    ]
  }},
  "headlineMetrics": [{{"label": "averageCtxhelmLiftAt10", "value": 0.1}}],
  "v23EvalSummary": {{
    "fixedCorpusId": "phase92-area-aware-gap-taxonomy-2026-05-31",
    "pairedBaselineVerdicts": [],
    "featureExportPrivacy": {{"localOnly": true, "sourceTextLogged": false}},
    "learnedPolicyStatus": {{"defaultRequiresThresholds": true, "silentDefaultAllowed": false}},
    "proofBoundary": "world-class claims require repeated lift"
  }},
  "releaseGate": {{
    "decision": "promote",
    "defaultPromotionAllowed": true,
    "corpusVerdicts": [{{
      "repository": "phase92-area-aware-gap-taxonomy-2026-05-31",
      "status": "beat",
      "lexicalDeltaAt10": 0.1,
      "sourceRecallAt10": 0.9,
      "lexicalSourceRecallAt10": 0.8,
      "sourceDeltaAt10": 0.1,
      "contextVsAllFileDeltaAt10": 0.0,
      "lexicalContextVsAllFileDeltaAt10": 0.0,
      "allFileDivergenceExplained": true
    }}]
  }}
}}"#
    )
}

#[test]
fn release_docs_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/check-release-docs.sh");
    assert!(script.exists(), "release docs check script is missing");

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "README.md",
        "docs/release.md",
        "docs/quickstart.md",
        "docs/agent-setup.md",
        "docs/troubleshooting.md",
        "docs/demo.md",
        "docs/public-project-summary.md",
        "docs/distribution.md",
        "docs/release-governance.md",
        "ctxhelm --version",
        "ctxhelm --help",
        "v2.4.6",
        "sha256sums.txt",
        "Why ctxhelm",
        "Current proof snapshot",
        "zero protected target misses",
        "agent-evidence retrieval channel",
        "Claude Code \\`2.1.163\\`",
        "Codex CLI \\`0.137.0\\`",
        "2026-06-05-phase237-codex-agent-run-outcome.md",
        "ctxhelm init --repo",
        "ctxhelm setup-check --repo",
        "ctxhelm prepare-task",
        "ctxhelm get-pack",
        "ctxhelm governor decide",
        "ctxhelm inspector serve",
        "localhost-only diagnostic shell",
        "PATH",
        "absolute",
        "CTXHELM_HOME",
        "wrong cwd",
        "MCP startup",
        "setup-check",
        "state cleanup",
        "session-scoped",
        "deterministic protocol proof",
        "real-client proof",
        "Cursor",
        "OpenCode",
        "cargo install --git",
        "--tag v2.4.6",
        "--locked",
        "crates.io",
        "Homebrew",
        "self-update",
        "signed installers",
        "cloud telemetry",
        "global agent config",
        "scripts/release-gate.sh",
        "scripts/verify-release-archive.sh",
        "scripts/smoke-first-pack.sh",
        "scripts/smoke-storage.sh",
        "scripts/smoke-shared-artifacts.sh",
        "scripts/smoke-governor.sh",
        "scripts/smoke-inspector.sh",
        "scripts/smoke-retrieval-health.sh",
        "scripts/smoke-graph.sh",
        "scripts/smoke-policy-embedding.sh",
        "scripts/smoke-agent-preview.sh",
        "scripts/smoke-agent-native-fallback.sh",
        "scripts/smoke-demo-artifacts.sh",
        "scripts/smoke-distribution-metadata.sh",
        "scripts/smoke-release-governance.sh",
        "scripts/verify-github-release.sh",
        "scripts/verify-public-archive-install.sh",
        "scripts/smoke-public-real-clients.sh",
        "scripts/e2e-claude-workflow.sh",
        "scripts/smoke-semantic.sh",
        "scripts/smoke-precision.sh",
        "scripts/smoke-v23-eval.sh",
        "scripts/smoke-mcp-protocol.sh",
        "scripts/smoke-codex-mcp.sh",
        "scripts/smoke-claude-mcp.sh",
        "scripts/smoke-cursor-real-client.sh",
        "scripts/smoke-opencode-real-client.sh",
        "CTXHELM_BIN",
        "CTXHELM_REQUIRE_REAL_CLIENT",
        "CTXHELM_SKIP_REAL_CLIENT",
        "CTXHELM_RUN_CURSOR_REAL_CLIENT",
        "CTXHELM_REQUIRE_CURSOR_REAL_CLIENT",
        "CTXHELM_RUN_OPENCODE_REAL_CLIENT",
        "CTXHELM_REQUIRE_OPENCODE_REAL_CLIENT",
        "CTXHELM_REAL_CLIENT_EVIDENCE_DIR",
        "CTXHELM_RUN_CLAUDE_WORKFLOW_EVAL",
        "CTXHELM_REQUIRE_CLAUDE_WORKFLOW_EVAL",
        "CTXHELM_PROOF_DIR",
        "CTXHELM_CLEAN_FIXTURE_CONFIG",
        "CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF",
        "CTXHELM_SKIP_CLEAN_FIXTURE_PROOF",
        "scripts/prepare-proof-fixtures.sh",
        "does not publish",
        "does not create tags",
        "Cursor and OpenCode real-client proof is optional and source-free",
        "opencode-real-client",
        "cursor-real-client",
        "--proof-summary",
        "archive checksum",
        "binary checksum",
        "local archive channel is ready",
        "manifest.json",
        "audit.json",
        "source-free proof bundle",
        "clean extraction verification",
        "not a self-update implementation",
        "signing and notarization gaps",
        "ready/deferred/blocked",
        "rollback",
        "local archive: ready",
        "Homebrew formula: ready",
        "crates.io package: deferred",
        "signed installer: deferred",
        "roleCounts",
        "selectedRoleCounts",
        "resourceScope.kind = safeInventoryArea",
        "taskConditioned = false",
        "countsSource = safeInventory",
        "pathSource = safeInventory",
        "CTXHELM_HOME",
        "crate-wide",
    ] {
        assert!(
            script_text.contains(required),
            "release docs script must check for {required}"
        );
    }
}

#[test]
fn public_real_client_smoke_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/smoke-public-real-clients.sh");
    assert!(
        script.exists(),
        "public real-client smoke script is missing"
    );

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "https://github.com/${repo}/releases/download/${tag}",
        "verify-release-archive.sh",
        "smoke-codex-mcp.sh",
        "smoke-claude-mcp.sh",
        "CTXHELM_BIN",
        "CTXHELM_RUN_REAL_CLIENT",
        "CTXHELM_REQUIRE_REAL_CLIENT",
        "CTXHELM_SKIP_REAL_CLIENT",
        "CTXHELM_REAL_CLIENT_EVIDENCE_DIR",
        "CTXHELM_REQUIRE_RESOURCE_SCOPE",
        "CTXHELM_PUBLIC_SMOKE_REQUIRE_RESOURCE_SCOPE",
        "downloadedPublicAssets",
        "checksumsVerified",
        "archiveVerified",
        "versionPassed",
        "privacyStatus",
        "sourceTextLogged",
        "global agent config mutation",
        "user project test execution",
        "missing_evidence",
    ] {
        assert!(
            script_text.contains(required),
            "public real-client script missing {required}"
        );
    }
}

#[test]
fn cursor_and_opencode_real_client_smoke_script_contracts() {
    let repo_root = workspace_root();
    for (script_name, client_command, run_env, require_env, schema) in [
        (
            "scripts/smoke-cursor-real-client.sh",
            "cursor agent --print",
            "CTXHELM_RUN_CURSOR_REAL_CLIENT",
            "CTXHELM_REQUIRE_CURSOR_REAL_CLIENT",
            "ctxhelm-cursor-real-client-evidence-v1",
        ),
        (
            "scripts/smoke-opencode-real-client.sh",
            "opencode run",
            "CTXHELM_RUN_OPENCODE_REAL_CLIENT",
            "CTXHELM_REQUIRE_OPENCODE_REAL_CLIENT",
            "ctxhelm-opencode-real-client-evidence-v1",
        ),
    ] {
        let script = repo_root.join(script_name);
        assert!(script.exists(), "{script_name} is missing");
        let syntax = Command::new("bash")
            .arg("-n")
            .arg(&script)
            .current_dir(&repo_root)
            .output()
            .unwrap();
        assert!(
            syntax.status.success(),
            "bash -n failed for {script_name}: {}",
            String::from_utf8_lossy(&syntax.stderr)
        );
        let script_text = fs::read_to_string(&script).unwrap();
        for required in [
            client_command,
            run_env,
            require_env,
            "smoke-mcp-protocol.sh",
            "CTXHELM_REAL_CLIENT_REQUEST_LOG",
            "tee -a \"$CTXHELM_REAL_CLIENT_REQUEST_LOG\"",
            "serve-mcp",
            "prepare_task",
            "get_pack",
            "explicitRepoToolCallCount",
            "observedToolCalls",
            "serverSideRequestLog",
            "requestLogSha256",
            "source-free",
            schema,
        ] {
            assert!(
                script_text.contains(required),
                "{script_name} missing {required}"
            );
        }
    }
}

#[test]
fn agent_run_e2e_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/e2e-agent-run.sh");
    assert!(script.exists(), "agent-run e2e script is missing");

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "ctxhelm-agent-run-eval-v1",
        "baseline",
        "ctxhelm-plan",
        "ctxhelm-brief",
        "ctxhelm-standard",
        "ctxhelm-memory",
        "CTXHELM_RUN_REAL_CLIENT",
        "CTXHELM_REQUIRE_REAL_CLIENT",
        "CTXHELM_AGENT_RUN_TIMEOUT_SECONDS",
        "CTXHELM_AGENT_RUN_PREFLIGHT",
        "CTXHELM_AGENT_RUN_PREFLIGHT_TIMEOUT_SECONDS",
        "--suite",
        "paired-agent-context-suite",
        "targetCoverageDeltaAverage",
        "irrelevantReadDeltaSum",
        "clientPreflight",
        "clientPreflightCount",
        "clientPreflightFailureCount",
        "clientPreflightRateLimitCount",
        "mcp__ctxhelm__prepare_task",
        "mcp__ctxhelm__get_pack",
        "prepare_task",
        "get_pack",
        "standard",
        "selected memory",
        "targetCoverage",
        "targetReadCoverage",
        "targetReadCoverageDelta",
        "targetReadCoverageDeltaAverage",
        "targetReadCount",
        "targetDiscoveredOnlyCount",
        "discoveredOnlyTargets",
        "missedTargetCount",
        "missedTargets",
        "evaluationStatus",
        "evaluationEligible",
        "comparisonEligible",
        "comparisonEligibleCount",
        "baselineEligible",
        "comparableCtxhelmLaneCount",
        "requiredCtxhelmCalls",
        "requiredCtxhelmCallSpecs",
        "observedRequiredCtxhelmCalls",
        "missingRequiredCtxhelmCalls",
        "invalidRequiredCtxhelmCalls",
        "ctxhelmCallCompliance",
        "client_unavailable",
        "requiredCtxhelmCallCount",
        "observedRequiredCtxhelmCallCount",
        "missingRequiredCtxhelmCallCount",
        "invalidRequiredCtxhelmCallCount",
        "missingRequiredCtxhelmCallsObserved",
        "invalidRequiredCtxhelmCallsObserved",
        "insufficient_comparable_lanes",
        "requiresRepo",
        "requiresTask",
        "recordTrace",
        "clientFailureKind",
        "clientAvailability",
        "tinyPromptAvailable",
        "tinyPromptStatus",
        "pairedSuiteAvailable",
        "pairedSuiteAvailableCount",
        "rateLimited",
        "clientFailureObserved",
        "comparableLaneCount",
        "availabilityBlocker",
        "availabilityBlockers",
        "no_comparable_lanes",
        "clientApiErrorStatus",
        "clientFailuresObserved",
        "clientFailureCount",
        "rateLimitObserved",
        "rateLimitsObserved",
        "rateLimitCount",
        "ctxhelmEvidenceFiles",
        "ctxhelmEvidenceFileCount",
        "ctxhelmEvidenceTargetHits",
        "ctxhelmEvidenceTargetHitCount",
        "ctxhelmEvidenceOnlyTargets",
        "ctxhelmEvidenceOnlyTargetCount",
        "ctxhelmEvidenceOnlyTargetsObserved",
        "if not evaluation_eligible",
        "ctxhelmEvidenceMissedTargets",
        "ctxhelmEvidenceMissedTargetCount",
        "ctxhelmEvidenceMissesObserved",
        "ctxhelmEvidenceMisses",
        "recommendedResearchActions",
        "collect_real_client_evidence",
        "retry_real_client_when_available",
        "fix_retrieval_or_query_construction",
        "improve_agent_consumption_guidance",
        "harden_required_ctxhelm_call_guidance",
        "rate_limited",
        "api_error",
        "timeout",
        "readRoleCounts",
        "missedTargetRoleCounts",
        "ctxhelmUnderReadTargetsObserved",
        "irrelevantReadCount",
        "ctxhelmToolCallCount",
        "forbiddenToolCallCount",
        "forbiddenToolCalls",
        "forbiddenToolCallsObserved",
        "Bash",
        "Write",
        "Edit",
        "ctxhelm_improved",
        "ctxhelm_matched",
        "rawPromptStored",
        "rawTranscriptStored",
        "rawMcpTrafficStored",
        "sourceTextLogged",
        "unsupportedActions",
        "source edits",
        "user project tests",
        "global agent config mutation",
        "cloud upload",
    ] {
        assert!(
            script_text.contains(required),
            "agent-run e2e script missing {required}"
        );
    }
}

#[test]
fn agent_client_availability_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/e2e-agent-client-availability.sh");
    assert!(
        script.exists(),
        "agent client availability script is missing"
    );

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "ctxhelm-agent-client-availability-v1",
        "agent-client-availability",
        "Claude Code",
        "Codex CLI",
        "smoke-codex-mcp.sh",
        "clientCount",
        "readyClientCount",
        "unavailableClientCount",
        "rateLimitedClientCount",
        "streamDisconnectedClientCount",
        "realAgentOutcomeCurrentlyRunnable",
        "run_real_agent_outcome_matrix",
        "retry_real_client_when_available",
        "upgrade_or_reconfigure_codex_cli",
        "clientFailureKind",
        "clientApiErrorStatus",
        "rateLimitObserved",
        "stream_disconnected",
        "deterministicProtocol",
        "explicitRepoToolCallCount",
        "observedToolCalls",
        "sourceTextLogged",
        "rawPromptStored",
        "rawTranscriptStored",
        "rawMcpTrafficStored",
        "remoteEmbeddingsUsed",
        "remoteRerankingUsed",
        "global agent config mutation",
        "cloud upload",
    ] {
        assert!(
            script_text.contains(required),
            "agent client availability script missing {required}"
        );
    }
}

#[test]
fn codex_agent_run_e2e_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/e2e-agent-run-codex.sh");
    assert!(script.exists(), "codex agent-run script is missing");

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "ctxhelm-agent-run-eval-v1",
        "paired-agent-context-run",
        "paired-agent-context-suite",
        "Codex CLI",
        "command_execution",
        "commandSha256",
        "--suite",
        "--suite-work-dir",
        "CTXHELM_AGENT_RUN_SUITE_WORK_DIR",
        "checkpointEnabled",
        "checkpointDirSha256",
        "reusedCheckpoint",
        "reusedTaskCount",
        "ctxhelm-agent-run-codex-runner-v1",
        "runner_fingerprint_v1",
        "scriptSha256",
        "contractVersion",
        "checkpointValidation",
        "codex agent-run suite eval wrote",
        "targetCoverageDeltaAverage",
        "targetReadCoverageDeltaAverage",
        "irrelevantReadDeltaSum",
        "commandExecutionDeltaSum",
        "readFileDeltaSum",
        "laneSummaries",
        "target-consumption retry",
        "retryAttempted",
        "retrySelected",
        "retryReason",
        "retryCost",
        "readEfficiency",
        "targetReadPrecision",
        "irrelevantReadRate",
        "readsPerTargetRead",
        "extraReadsPerRecoveredTarget",
        "extraIrrelevantReadsPerRecoveredTarget",
        "retryTriggeredLanes",
        "retrySelectedLanes",
        "optimize_agent_read_efficiency",
        "avgReadFilesBeforeRetry",
        "avgReadFilesAfterRetry",
        "avgIrrelevantReadsBeforeRetry",
        "avgIrrelevantReadsAfterRetry",
        "targetReadCoverageBeforeRetry",
        "targetReadCoverageAfterRetry",
        "evidenceOnlyTargetCountBeforeRetry",
        "evidenceOnlyTargetCountAfterRetry",
        "readFileCountDelta",
        "irrelevantReadCountDelta",
        "ctxhelm_evidence_only_targets",
        "\"client\": {\"name\": \"codex\"",
        "rawCommandOutputStored",
        "forbiddenCommandCount",
        "harden_codex_read_only_prompt",
        "prepare_task",
        "get_pack",
        "targetFiles first",
        "consume every targetFiles path as the first shell reads after ctxhelm calls",
        "read those non-source targets before source-code targets",
        "first-class targets",
        "target-first efficiency probe",
        "Use at most 6 shell commands total after ctxhelm calls",
        "Read no more than 6 current files total",
        "Do not batch-read broad context-area, pack-neighbor, or planning/doc lists",
        "Stop immediately after target-backed reads are enough to name the key files",
        "Do not read extra non-target files just to fill the command budget",
        "selectedMemory sourceLinks",
        "memory-efficiency probe",
        "Use at most 6 shell commands total",
        "Memory evidence may prioritize targetFiles, but it must not displace targetFiles",
        "do not read selectedMemory, docs, or planning paths before the returned targetFiles",
        "Consume targetFiles and high-confidence target paths first",
        "Do not keep exploring just to fill the command budget",
        "mcp_servers.ctxhelm.command",
        "CTXHELM_RUN_REAL_CLIENT=1",
        "source edits",
        "user project tests",
        "global agent config mutation",
        "cloud upload",
    ] {
        assert!(
            script_text.contains(required),
            "codex agent-run script missing {required}"
        );
    }
}

#[test]
fn codex_memory_outcome_suite_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/e2e-codex-memory-outcome-suite.sh");
    assert!(
        script.exists(),
        "codex memory outcome suite script is missing"
    );

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "ctxhelm-codex-memory-outcome-suite-v1",
        "cross-repo-codex-memory-outcome-suite",
        "generate-experience",
        "memory",
        "approve",
        "CTXHELM_RUN_REAL_CLIENT",
        "CTXHELM_HOME",
        "rawTaskStored",
        "rawTaskTextStored",
        "rawCommandOutputStored",
        "rawMcpTrafficStored",
        "remoteEmbeddingsUsed",
        "remoteRerankingUsed",
        "memoryTargetReadImprovedPairCount",
        "memoryTargetReadMatchedOrImprovedPairCount",
        "memoryIrrelevantReadImprovedPairCount",
        "expand_memory_outcome_repository_diversity",
        "source edits",
        "cloud upload",
    ] {
        assert!(
            script_text.contains(required),
            "codex memory outcome suite script missing {required}"
        );
    }
}

#[test]
fn memory_reuse_smoke_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/smoke-memory-reuse.sh");
    assert!(script.exists(), "memory reuse smoke script is missing");

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "ctxhelm-memory-reuse-eval-v1",
        "prepare-task",
        "memory generate-experience",
        "memory approve",
        "beforeApproval",
        "afterApproval",
        "memorySignalCount",
        "selectedMemoryCount",
        "targetHit",
        "sourceTextLogged",
        "rawPromptStored",
        "rawTranscriptStored",
        "rawMcpTrafficStored",
        "CTXHELM_MEMORY_REUSE_SOURCE_SENTINEL",
        "storage leaked source sentinel",
        "model invocation",
        "cloud upload",
    ] {
        assert!(
            script_text.contains(required),
            "memory reuse smoke script missing {required}"
        );
    }
}

#[test]
fn memory_history_lift_smoke_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/smoke-memory-history-lift.sh");
    assert!(
        script.exists(),
        "memory history lift smoke script is missing"
    );

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "ctxhelm-memory-history-lift-eval-v1",
        "eval history",
        "memory generate-experience",
        "memory approve",
        "beforeApproval",
        "afterApproval",
        "memoryUniqueTargetHitCount",
        "memorySelectedAt10Count",
        "evaluate_memory_reuse_lift",
        "targetInLexical",
        "targetInCombined",
        "sourceTextLogged",
        "rawPromptStored",
        "rawTranscriptStored",
        "rawMcpTrafficStored",
        "CTXHELM_MEMORY_HISTORY_SOURCE_SENTINEL",
        "storage leaked source sentinel",
        "model invocation",
        "cloud upload",
    ] {
        assert!(
            script_text.contains(required),
            "memory history lift smoke script missing {required}"
        );
    }
}

#[test]
fn memory_parent_snapshot_lift_smoke_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/smoke-memory-parent-snapshot-lift.sh");
    assert!(
        script.exists(),
        "memory parent-snapshot lift smoke script is missing"
    );

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "ctxhelm-memory-parent-snapshot-lift-eval-v1",
        "parent-snapshot-experience-memory-lift",
        "eval history",
        "--base",
        "--head",
        "memory generate-experience",
        "memory approve",
        "memoryUniqueTargetHitCount",
        "evaluate_memory_reuse_lift",
        "targetInLexical",
        "targetInCombined",
        "source and snapshot storage databases",
        "sourceTextLogged",
        "rawPromptStored",
        "rawTaskTextStored",
        "rawTranscriptStored",
        "rawMcpTrafficStored",
        "CTXHELM_MEMORY_PARENT_SNAPSHOT_SOURCE_SENTINEL",
        "history report leaked source sentinel",
        "history report leaked raw task text",
        "storage leaked source sentinel",
        "model invocation",
        "cloud upload",
    ] {
        assert!(
            script_text.contains(required),
            "memory parent-snapshot lift smoke script missing {required}"
        );
    }
}

#[test]
fn memory_benchmark_lift_smoke_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/smoke-memory-benchmark-lift.sh");
    assert!(
        script.exists(),
        "memory benchmark lift smoke script is missing"
    );

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "ctxhelm-memory-benchmark-lift-eval-v1",
        "eval proof",
        "benchmark-product-proof-experience-memory-lift",
        "memory generate-experience",
        "memory approve",
        "memoryUniqueTargetHitCount",
        "memoryUniqueNonTargetCount",
        "evaluate_memory_reuse_lift",
        "targetInLexical",
        "targetInCombined",
        "evaluatedRepositoryCount",
        "evaluatedCommitCount",
        "sourceTextLogged",
        "rawPromptStored",
        "rawTaskTextStored",
        "rawTranscriptStored",
        "rawMcpTrafficStored",
        "CTXHELM_MEMORY_BENCHMARK_SOURCE_SENTINEL_A",
        "CTXHELM_MEMORY_BENCHMARK_SOURCE_SENTINEL_B",
        "product proof leaked source sentinel",
        "product proof leaked raw task text",
        "storage leaked source sentinel",
        "model invocation",
        "cloud upload",
    ] {
        assert!(
            script_text.contains(required),
            "memory benchmark lift smoke script missing {required}"
        );
    }
}

#[test]
fn memory_generalization_measurement_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/measure-memory-generalization.sh");
    assert!(
        script.exists(),
        "memory generalization measurement script is missing"
    );

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "ctxhelm-memory-generalization-measurement-v2",
        "multi-pair-experience-memory-generalization",
        "--repo",
        "--pairs",
        "--scan-commits",
        "--semantic",
        "--semantic-provider",
        "eval",
        "history",
        "prepare-task",
        "memory",
        "generate-experience",
        "approve",
        "memoryUniqueLiftPairs",
        "candidatePairCount",
        "candidateTargetFileCount",
        "evaluatedTargetFileCount",
        "memoryUniqueNonTargetCount",
        "memoryUniqueNonTargetWithCurrentSupportCount",
        "memoryUniqueTargetHitWithoutCurrentSupportCount",
        "memoryUniqueNonTargetWithoutCurrentSupportCount",
        "memoryUniqueTargetHitCurrentSupportSignalCounts",
        "memoryUniqueNonTargetCurrentSupportSignalCounts",
        "memoryTargetHitsWithGraphSupportUpperBound",
        "memoryTargetHitsWithSemanticSupportUpperBound",
        "memoryUniqueTargetsWithGraphOrSemanticSupportUpperBound",
        "memoryTargetHitsWithoutGraphOrSemanticSupportLowerBound",
        "graphEdgeAblationRemovedTargetHitCount",
        "memoryPackChangedPairs",
        "memoryPackTargetGainPairs",
        "memoryPackAddedFileCount",
        "memoryPackRemovedFileCount",
        "memoryPackAddedTargetCount",
        "memoryPackAddedNonTargetCount",
        "memorySignalOnlyNonTargetCount",
        "semanticSelectedTargetPairs",
        "semanticAblationLiftPairs",
        "memoryNeedsCorroboration",
        "signalOnlyMemoryOverlapObserved",
        "unsupportedMemoryPrecisionNeedsWork",
        "supportedMemoryNoiseNeedsReview",
        "supportedMemoryNoiseDominantSignals",
        "weakSupportedMemoryNoiseNeedsTuning",
        "tune_memory_weight_against_supported_signal_pressure",
        "inspect_remaining_strong_signal_memory_overlap",
        "compare_memory_noise_against_current_signal_roles",
        "track_signal_only_memory_overlap",
        "pairDiversityMeasured",
        "largerPairValidationTargetMet",
        "expand_repository_diversity",
        "semanticUsefulForMemoryTasks",
        "precisionNeedsWork",
        "generalizationProven",
        "rawTaskStored",
        "rawTaskTextStored",
        "rawTranscriptStored",
        "rawMcpTrafficStored",
        "remoteEmbeddingsUsed",
        "remoteRerankingUsed",
        "raw prompt storage",
        "source edits",
    ] {
        assert!(
            script_text.contains(required),
            "memory generalization measurement script missing {required}"
        );
    }
}

#[test]
fn memory_generalization_suite_measurement_script_contract() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/measure-memory-generalization-suite.sh");
    assert!(
        script.exists(),
        "memory generalization suite measurement script is missing"
    );

    let syntax = Command::new("bash")
        .arg("-n")
        .arg(&script)
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "bash -n failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let script_text = fs::read_to_string(&script).unwrap();
    for required in [
        "ctxhelm-memory-generalization-suite-v2",
        "multi-repo-experience-memory-generalization",
        "measure-memory-generalization.sh",
        "--repo",
        "--pairs",
        "--scan-commits",
        "--semantic",
        "--semantic-provider",
        "memoryUniqueLiftPairs",
        "candidatePairCount",
        "candidateTargetFileCount",
        "evaluatedTargetFileCount",
        "memoryUniqueTargetHitCount",
        "memoryUniqueNonTargetCount",
        "memoryUniqueNonTargetWithCurrentSupportCount",
        "memoryUniqueTargetHitWithoutCurrentSupportCount",
        "memoryUniqueNonTargetWithoutCurrentSupportCount",
        "memoryUniqueTargetHitCurrentSupportSignalCounts",
        "memoryUniqueNonTargetCurrentSupportSignalCounts",
        "memoryTargetHitsWithGraphSupportUpperBound",
        "memoryTargetHitsWithSemanticSupportUpperBound",
        "memoryUniqueTargetsWithGraphOrSemanticSupportUpperBound",
        "memoryTargetHitsWithoutGraphOrSemanticSupportLowerBound",
        "graphEdgeAblationRemovedTargetHitCount",
        "memoryPackChangedPairs",
        "memoryPackTargetGainPairs",
        "memoryPackAddedFileCount",
        "memoryPackRemovedFileCount",
        "memoryPackAddedTargetCount",
        "memoryPackAddedNonTargetCount",
        "memorySignalOnlyNonTargetCount",
        "semanticSelectedTargetPairs",
        "semanticAblationLiftPairs",
        "memoryNeedsCorroboration",
        "signalOnlyMemoryOverlapObserved",
        "unsupportedMemoryPrecisionNeedsWork",
        "supportedMemoryNoiseNeedsReview",
        "supportedMemoryNoiseDominantSignals",
        "weakSupportedMemoryNoiseNeedsTuning",
        "tune_memory_weight_against_supported_signal_pressure",
        "inspect_remaining_strong_signal_memory_overlap",
        "compare_memory_noise_against_current_signal_roles",
        "track_signal_only_memory_overlap",
        "semanticUsefulForMemoryTasks",
        "graphCorroborationMeasured",
        "memoryUniqueNonTargetPerUniqueTarget",
        "memoryLiftRepositoryCount",
        "memoryNonTargetRepositoryCount",
        "unsupportedMemoryNoiseRepositoryCount",
        "strongSupportedMemoryNoiseRepositoryCount",
        "multiRepoMeasured",
        "largerPairCountMeasured",
        "largerPairValidationTargetMet",
        "repositoryDiversityTarget",
        "repositoryDiversityTargetMet",
        "repositoryDiversityNeedsExpansion",
        "expand_repository_diversity",
        "pairDiversityMeasured",
        "generalizationProven",
        "precisionNeedsWork",
        "repoPathStored",
        "rawPromptStored",
        "rawTaskTextStored",
        "rawTranscriptStored",
        "rawMcpTrafficStored",
        "remoteEmbeddingsUsed",
        "remoteRerankingUsed",
        "raw prompt storage",
        "source edits",
    ] {
        assert!(
            script_text.contains(required),
            "memory generalization suite measurement script missing {required}"
        );
    }
}

#[test]
fn release_docs_check_passes() {
    let repo_root = workspace_root();
    let output = Command::new(repo_root.join("scripts/check-release-docs.sh"))
        .current_dir(&repo_root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "release docs check failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn run_git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_stdout(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn archive_with_entries(entries: &[(&str, &str)]) -> PathBuf {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.keep();
    let payload = temp_path.join("payload");
    fs::create_dir_all(&payload).unwrap();
    for (relative_path, contents) in entries {
        let path = payload.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    let archive = temp_path.join("fixture.tar.gz");
    let output = Command::new("tar")
        .args(["-czf"])
        .arg(&archive)
        .arg("-C")
        .arg(&payload)
        .arg(".")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fixture tar failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    archive
}
