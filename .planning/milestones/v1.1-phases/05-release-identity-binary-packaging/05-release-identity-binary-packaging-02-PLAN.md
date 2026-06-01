---
phase: 05-release-identity-binary-packaging
plan: 02
type: execute
wave: 2
depends_on: [01]
files_modified:
  - scripts/release-package.sh
  - crates/ctxhelm/tests/release_packaging.rs
  - .gitignore
autonomous: true
requirements: [PKG-01, PKG-02]
must_haves:
  truths:
    - "Maintainer can run one local script from a clean checkout to build a release-mode ctxhelm binary with locked dependencies."
    - "The script emits a GitHub Releases-style archive plus SHA-256 checksum files under `dist/`."
    - "The extracted artifact can run `ctxhelm --version` and `ctxhelm --help` from a directory outside the source checkout."
  artifacts:
    - path: "scripts/release-package.sh"
      provides: "Repeatable local release archive and checksum builder"
    - path: "crates/ctxhelm/tests/release_packaging.rs"
      provides: "Script contract tests for packaging behavior and safety defaults"
    - path: ".gitignore"
      provides: "Ignored local release output directory"
  key_links:
    - from: "scripts/release-package.sh"
      to: "target/release/ctxhelm"
      via: "cargo build -p ctxhelm --release --locked"
      pattern: "cargo build.*--release.*--locked"
    - from: "scripts/release-package.sh"
      to: "dist/sha256sums.txt"
      via: "shasum -a 256 or sha256sum over generated archives"
      pattern: "sha256"
---

<objective>
Create the repeatable local binary packaging path.

Purpose: Users need an installable artifact path that proves ctxhelm works without a source checkout.
Output: A release packaging script, checksum output, ignored `dist/` artifacts, and script contract tests.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/REQUIREMENTS.md
@.planning/ROADMAP.md
@.planning/research/STACK.md
@.planning/research/PITFALLS.md
@.planning/phases/05-release-identity-binary-packaging/05-CONTEXT.md
@.planning/phases/05-release-identity-binary-packaging/05-release-identity-binary-packaging-01-SUMMARY.md
@scripts/smoke-mcp-protocol.sh
@Cargo.toml
@.gitignore

<decision_trace>
- D-01: Use GitHub Releases-style archives and SHA-256 checksums as the first binary artifact shape.
- D-04: Build from a clean checkout and smoke the extracted or installed binary rather than only `cargo run`.
- D-07: Use boring shell scripts and standard Cargo tooling before adding dedicated release tooling.
</decision_trace>

<interfaces>
Existing smoke pattern from Phase 4:
```bash
CTXHELM_SMOKE_REPO="$PWD" CTXHELM_SMOKE_TASK="..." bash scripts/smoke-mcp-protocol.sh
```

Packaging script contract for this plan:
```bash
bash scripts/release-package.sh
CTXHELM_DIST_DIR=/tmp/ctxhelm-dist bash scripts/release-package.sh
CTXHELM_ALLOW_DIRTY=1 bash scripts/release-package.sh
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add release package script contract tests</name>
  <files>crates/ctxhelm/tests/release_packaging.rs, scripts/release-package.sh, .gitignore</files>
  <behavior>
    - Test 1: `scripts/release-package.sh` exists, passes `bash -n`, and contains `cargo build -p ctxhelm --release --locked`.
    - Test 2: The script writes release output under configurable `CTXHELM_DIST_DIR` defaulting to `dist/`.
    - Test 3: `.gitignore` ignores `dist/` so generated archives and checksums are not accidentally committed.
    - Test 4: The script supports a dirty-worktree escape hatch named `CTXHELM_ALLOW_DIRTY=1`, but defaults to requiring a clean checkout.
  </behavior>
  <action>Create `crates/ctxhelm/tests/release_packaging.rs` with tests that inspect the shell script contract and `.gitignore` entry before the full packaging behavior exists. Add an initial executable `scripts/release-package.sh` skeleton if needed to make the tests meaningful in Task 2. Keep tests independent of network access and do not require GitHub credentials.</action>
  <verify>
    <automated>cargo test -p ctxhelm --test release_packaging release_package_script_contract -- --nocapture</automated>
  </verify>
  <done>Packaging behavior is guarded by focused script contract tests before the release script is filled in.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement local archive and checksum generation</name>
  <files>scripts/release-package.sh, crates/ctxhelm/tests/release_packaging.rs, .gitignore</files>
  <behavior>
    - Test 1: Running the script builds `ctxhelm` with `cargo build -p ctxhelm --release --locked`.
    - Test 2: The script creates an archive named like `ctxhelm-v1.1.0-{target}.tar.gz` on Unix targets, containing `ctxhelm`, `README.md`, `LICENSE`, and a small `VERSION` or manifest file.
    - Test 3: The script writes SHA-256 checksums for produced archives to `sha256sums.txt` and per-archive `.sha256` files or equivalent.
    - Test 4: The script extracts the archive into a temp directory outside the source checkout and verifies the extracted binary with `--version` and `--help`.
    - Test 5: The script exits non-zero when the checkout is dirty unless `CTXHELM_ALLOW_DIRTY=1` is set.
  </behavior>
  <action>Implement `scripts/release-package.sh` per PKG-01 and PKG-02. Use `set -euo pipefail`; resolve the repository root from the script location; derive version from the `ctxhelm` package metadata; derive a target label from `rustc -vV` host unless `CTXHELM_TARGET_LABEL` is set; build with locked dependencies; stage only explicit release files; create `dist/ctxhelm-v${version}-${target}.tar.gz`; write checksums using `shasum -a 256` on macOS with a `sha256sum` fallback for Linux; smoke the extracted binary from a temp directory outside the repo. Do not publish a GitHub release, call network APIs, mutate agent configs, or bundle `.ctxhelm` local state.</action>
  <verify>
    <automated>cargo test -p ctxhelm --test release_packaging release_package_script_contract -- --nocapture && CTXHELM_ALLOW_DIRTY=1 CTXHELM_DIST_DIR="$(mktemp -d)" bash scripts/release-package.sh</automated>
  </verify>
  <done>A maintainer can produce a local release archive and checksum set, then verify the extracted binary without using `cargo run`.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxhelm --test release_packaging release_package_script_contract -- --nocapture`
- `CTXHELM_ALLOW_DIRTY=1 CTXHELM_DIST_DIR="$(mktemp -d)" bash scripts/release-package.sh`
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
</verification>

<success_criteria>
Plan 02 is complete when the first binary artifact path is repeatable locally, checksum-producing, and validated from an extracted binary outside the source checkout.
</success_criteria>

<output>
After completion, create `.planning/phases/05-release-identity-binary-packaging/05-release-identity-binary-packaging-02-SUMMARY.md`
</output>
