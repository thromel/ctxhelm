use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

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
            "ctxhelm-v1.1.12-test/ctxhelm",
            "#!/usr/bin/env bash\nexit 0\n",
        ),
        (
            "ctxhelm-v1.1.12-test/.ctxhelm/repos/repo/traces.jsonl",
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
            "ctxhelm-v1.1.12-test/ctxhelm",
            "#!/usr/bin/env bash\nexit 0\n",
        ),
        ("ctxhelm-v1.1.12-test/README.md", "ctxhelm release\n"),
        ("ctxhelm-v1.1.12-test/LICENSE", "MIT License\n"),
        ("ctxhelm-v1.1.12-test/VERSION", "ctxhelm 1.1.12\n"),
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
            "ctxhelm-v1.1.12-test/ctxhelm",
            "#!/usr/bin/env bash\nexit 0\n",
        ),
        ("ctxhelm-v1.1.12-test/README.md", "ctxhelm release\n"),
        ("ctxhelm-v1.1.12-test/LICENSE", "MIT License\n"),
        ("ctxhelm-v1.1.12-test/VERSION", "ctxhelm 1.1.12\n"),
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
        "scripts/smoke-first-pack.sh",
        "scripts/smoke-storage.sh",
        "scripts/smoke-memory.sh",
        "scripts/smoke-memory-reuse.sh",
        "scripts/smoke-shared-artifacts.sh",
        "scripts/smoke-inspector.sh",
        "scripts/smoke-retrieval-health.sh",
        "scripts/smoke-graph.sh",
        "scripts/smoke-policy-embedding.sh",
        "scripts/smoke-agent-preview.sh",
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
        "scripts/e2e-claude-workflow.sh",
        "CTXHELM_BIN",
        "CTXHELM_SKIP_REAL_CLIENT",
        "CTXHELM_REQUIRE_REAL_CLIENT",
        "CTXHELM_RUN_CLAUDE_WORKFLOW_EVAL",
        "CTXHELM_REQUIRE_CLAUDE_WORKFLOW_EVAL",
        "CTXHELM_CLEAN_FIXTURE_CONFIG",
        "CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF",
        "CTXHELM_SKIP_CLEAN_FIXTURE_PROOF",
        "CTXHELM_REAL_CLIENT_EVIDENCE_DIR",
        "CTXHELM_PROOF_DIR",
        "CTXHELM_BENCHMARK_CONFIG",
        "phase183-clean-fixture-refresh-config.json",
        "eval proof",
        "check-product-proof.py",
        "release proof bundle",
        "release-proof-summary.json",
        "binaryIdentity",
        "optionalProofs",
        "cleanColdFixtureProductProof",
        "cleanColdFixtureRequired",
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
        script_text.contains(
            "CTXHELM_DIST_DIR=\"$dist_dir\" bash \"$smoke_distribution_metadata_script\""
        ),
        "release gate must pass packaged archive directory into distribution metadata smoke"
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
  "tagName": "v1.1.12",
  "targetCommitish": "release-commit",
  "url": "https://github.com/thromel/ctxhelm/releases/tag/v1.1.12"
}
"#,
    )
    .unwrap();

    let output = Command::new("bash")
        .arg(&script)
        .args(["--tag", "v1.1.12"])
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
        .args(["--tag", "v1.1.12"])
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
  "tagName": "v1.1.12",
  "targetCommitish": "{release_commit}",
  "url": "https://github.com/thromel/ctxhelm/releases/tag/v1.1.12"
}}
"#
        ),
    )
    .unwrap();

    let output = Command::new("bash")
        .arg(&script)
        .args(["--tag", "v1.1.12"])
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
        .args(["--tag", "v1.1.12"])
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

    let promote = Command::new("python3")
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

    let block = Command::new("python3")
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

    let missing_resource = Command::new("python3")
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

    let missing_report = Command::new("python3")
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

    let missing_source = Command::new("python3")
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

    let source_regression = Command::new("python3")
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

    let broad_floor = Command::new("python3")
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

    let broad_regression = Command::new("python3")
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
        "v1.1.12",
        "sha256sums.txt",
        "Why ctxhelm",
        "Current proof snapshot",
        "zero protected target misses",
        "agent-evidence retrieval channel",
        "Claude Code \\`2.1.159\\`",
        "Codex CLI \\`0.44.0\\`",
        "2026-06-01-phase132-claude-workflow-eval.md",
        "ctxhelm init --repo",
        "ctxhelm setup-check --repo",
        "ctxhelm prepare-task",
        "ctxhelm get-pack",
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
        "--tag v1.1.12",
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
        "scripts/smoke-inspector.sh",
        "scripts/smoke-retrieval-health.sh",
        "scripts/smoke-graph.sh",
        "scripts/smoke-policy-embedding.sh",
        "scripts/smoke-agent-preview.sh",
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
        "CTXHELM_BIN",
        "CTXHELM_REQUIRE_REAL_CLIENT",
        "CTXHELM_SKIP_REAL_CLIENT",
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
        "Cursor and OpenCode real-client proof is not claimed",
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
        "CTXHELM_RUN_REAL_CLIENT",
        "CTXHELM_REQUIRE_REAL_CLIENT",
        "CTXHELM_AGENT_RUN_TIMEOUT_SECONDS",
        "--suite",
        "paired-agent-context-suite",
        "targetCoverageDeltaAverage",
        "irrelevantReadDeltaSum",
        "mcp__ctxhelm__prepare_task",
        "mcp__ctxhelm__get_pack",
        "prepare_task",
        "get_pack",
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
