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
    assert!(script_text.contains("cargo build -p ctxpack --release --locked"));
    assert!(script_text.contains("CARGO_TARGET_DIR"));
    assert!(script_text.contains("CARGO_BUILD_TARGET_DIR"));
    assert!(script_text.contains("CTXPACK_DIST_DIR"));
    assert!(script_text.contains("dist"));
    assert!(script_text.contains("CTXPACK_ALLOW_DIRTY"));
    assert!(
        script_text.contains("git diff --quiet") || script_text.contains("git status --porcelain"),
        "script must check for a clean checkout by default"
    );
    assert!(
        script_text.contains("ctxpack-v${VERSION}-${TARGET_LABEL}.tar.gz")
            || script_text.contains("ctxpack-v${version}-${target}.tar.gz"),
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
        ".ctxpack",
        "traces.jsonl",
        "request",
        ".env",
        "token",
        "target/",
        ".git/",
        "/Users/",
        "tar -tf",
        "CTXPACK_AUDIT_REPORT",
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
            "ctxpack-v1.1.0-test/ctxpack",
            "#!/usr/bin/env bash\nexit 0\n",
        ),
        (
            "ctxpack-v1.1.0-test/.ctxpack/repos/repo/traces.jsonl",
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
            "ctxpack-v1.1.0-test/ctxpack",
            "#!/usr/bin/env bash\nexit 0\n",
        ),
        ("ctxpack-v1.1.0-test/README.md", "ctxpack release\n"),
        ("ctxpack-v1.1.0-test/LICENSE", "MIT License\n"),
        ("ctxpack-v1.1.0-test/VERSION", "ctxpack 1.1.0\n"),
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
            "ctxpack-v1.1.0-test/ctxpack",
            "#!/usr/bin/env bash\nexit 0\n",
        ),
        ("ctxpack-v1.1.0-test/README.md", "ctxpack release\n"),
        ("ctxpack-v1.1.0-test/LICENSE", "MIT License\n"),
        ("ctxpack-v1.1.0-test/VERSION", "ctxpack 1.1.0\n"),
    ]);
    let report_dir = TempDir::new().unwrap();
    let report_path = report_dir.path().join("audit.json");

    let output = Command::new(workspace_root().join("scripts/audit-release-artifact.sh"))
        .env("CTXPACK_AUDIT_REPORT", &report_path)
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
    assert!(!report_text.contains(".ctxpack/repos"));
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
        "scripts/smoke-mcp-protocol.sh",
        "scripts/smoke-codex-mcp.sh",
        "scripts/smoke-claude-mcp.sh",
        "CTXPACK_BIN",
        "CTXPACK_SKIP_REAL_CLIENT",
        "CTXPACK_REQUIRE_REAL_CLIENT",
        "CTXPACK_REAL_CLIENT_EVIDENCE_DIR",
        "CTXPACK_PROOF_DIR",
        "CTXPACK_BENCHMARK_CONFIG",
        "eval proof",
        "check-product-proof.py",
        "release proof bundle",
        "release-proof-summary.json",
        "binaryIdentity",
        "optionalProofs",
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
        script_text.contains("CTXPACK_BIN=\"$ctxpack_bin\" bash \"$smoke_first_pack_script\""),
        "release gate must pass selected binary into first-pack smoke"
    );
    assert!(
        !script_text.contains("CTXPACK_ALLOW_DIRTY=\"${CTXPACK_ALLOW_DIRTY:-1}\""),
        "release gate must not bypass release-package clean-checkout enforcement by default"
    );
    assert!(
        script_text.contains("CTXPACK_DIST_DIR=\"$dist_dir\" bash \"$release_package_script\""),
        "release gate should let release-package enforce clean-checkout semantics unless CTXPACK_ALLOW_DIRTY is explicitly inherited"
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
fn product_proof_checker_accepts_promote_and_rejects_block() {
    let repo_root = workspace_root();
    let script = repo_root.join("scripts/check-product-proof.py");
    assert!(script.exists(), "product proof checker is missing");

    let temp = TempDir::new().unwrap();
    let promote_path = temp.path().join("promote.json");
    let block_path = temp.path().join("block.json");
    let missing_resource_path = temp.path().join("missing-resource-gap.json");
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
        "retrievalGapSummaries": [{{
          "role": "source",
          "signalGap": "ranked_below_budget_dependency",
          "package": "src",
          "pathFamily": "src/*.rs",
          "contextArea": "src",
          "contextAreaResourceUri": "ctxpack://repo/context-area/src",
          "targetStatus": "currentReachable",
          "recommendationArea": "parserPrecision",
          "missedCount": 1,
          "examplePaths": ["src/lib.rs"],
          "nextReadPaths": ["src/lib.rs"]
        }}]
      }}
    }}]
  }},
  "headlineMetrics": [{{"label": "averageCtxpackLiftAt10", "value": 0.1}}],
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
        r#"          "contextAreaResourceUri": "ctxpack://repo/context-area/src",
"#,
        "",
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
        "name": "ctxpack",
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
  "headlineMetrics": [{{"label": "averageCtxpackLiftAt10", "value": 0.1}}],
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
        "ctxpack --version",
        "ctxpack --help",
        "v1.1.0",
        "sha256sums.txt",
        "ctxpack init --repo",
        "ctxpack setup-check --repo",
        "ctxpack prepare-task",
        "ctxpack get-pack",
        "PATH",
        "absolute",
        "CTXPACK_HOME",
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
        "--tag v1.1.0",
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
        "scripts/smoke-semantic.sh",
        "scripts/smoke-precision.sh",
        "scripts/smoke-v23-eval.sh",
        "scripts/smoke-mcp-protocol.sh",
        "scripts/smoke-codex-mcp.sh",
        "scripts/smoke-claude-mcp.sh",
        "CTXPACK_BIN",
        "CTXPACK_REQUIRE_REAL_CLIENT",
        "CTXPACK_SKIP_REAL_CLIENT",
        "CTXPACK_REAL_CLIENT_EVIDENCE_DIR",
        "CTXPACK_PROOF_DIR",
        "does not publish",
        "does not create tags",
        "Cursor and OpenCode real-client proof is not claimed",
        "manifest.json",
        "audit.json",
        "source-free proof bundle",
        "clean extraction verification",
        "not a self-update implementation",
        "signing and notarization gaps",
        "ready/deferred/blocked",
        "rollback",
    ] {
        assert!(
            script_text.contains(required),
            "release docs script must check for {required}"
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
