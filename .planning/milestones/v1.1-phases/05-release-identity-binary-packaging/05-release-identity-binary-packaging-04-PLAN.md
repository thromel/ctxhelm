---
phase: 05-release-identity-binary-packaging
plan: 04
type: execute
wave: 4
depends_on: [01, 02, 03]
files_modified:
  - README.md
  - docs/release.md
  - scripts/check-release-docs.sh
  - crates/ctxhelm/tests/release_packaging.rs
autonomous: true
requirements: [PKG-01, PKG-03, PKG-04, PKG-05]
must_haves:
  truths:
    - "User can read the normal install path for a prebuilt v1.1 archive, verify checksums, install the binary on PATH, and run `ctxhelm --version` plus `ctxhelm --help` without a source checkout."
    - "User has source-build fallback commands using locked dependencies for tagged git install and local checkout installation."
    - "README, release docs, binary version output, package metadata, tag references, license, and artifact audit instructions all agree on v1.1.0."
    - "Docs explicitly state that v1.1 does not require crates.io, Homebrew, self-update, signed installers, cloud indexing, telemetry, or global agent config mutation."
  artifacts:
    - path: "README.md"
      provides: "Short release install and verification path"
    - path: "docs/release.md"
      provides: "Maintainer packaging, checksum, source-build fallback, and audit reference"
    - path: "scripts/check-release-docs.sh"
      provides: "Docs command/version consistency check"
    - path: "crates/ctxhelm/tests/release_packaging.rs"
      provides: "Release docs contract tests"
  key_links:
    - from: "README.md"
      to: "scripts/release-package.sh"
      via: "documented archive/checksum flow matches script output names"
      pattern: "ctxhelm-v1\\.1\\.0"
    - from: "docs/release.md"
      to: "Cargo.toml"
      via: "documented tag/version identity matches package metadata"
      pattern: "v1\\.1\\.0"
    - from: "scripts/check-release-docs.sh"
      to: "README.md"
      via: "grep-based release docs consistency checks"
      pattern: "cargo install --git.*--tag v1\\.1\\.0.*--locked"
---

<objective>
Document the Phase 5 install, fallback, and release-audit paths.

Purpose: Packaging is only useful if users can install the binary without source and maintainers can reproduce the artifact with a clear fallback story.
Output: README release quickstart, detailed release docs, and automated docs consistency checks.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/REQUIREMENTS.md
@.planning/ROADMAP.md
@.planning/research/SUMMARY.md
@.planning/research/PITFALLS.md
@.planning/phases/05-release-identity-binary-packaging/05-CONTEXT.md
@.planning/phases/05-release-identity-binary-packaging/05-release-identity-binary-packaging-01-SUMMARY.md
@.planning/phases/05-release-identity-binary-packaging/05-release-identity-binary-packaging-02-SUMMARY.md
@.planning/phases/05-release-identity-binary-packaging/05-release-identity-binary-packaging-03-SUMMARY.md
@README.md
@scripts/release-package.sh
@scripts/audit-release-artifact.sh

<decision_trace>
- D-01: Prebuilt GitHub Releases-style archives and checksums are the primary v1.1 install story.
- D-02: `cargo install --git ... --tag ... --locked` and local checkout installation are fallback paths.
- D-03: Version identity across tag, package metadata, docs, README, and binary output must be consistent.
- D-04: Deferred package ecosystems remain out of this phase.
</decision_trace>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add release docs consistency checks</name>
  <files>scripts/check-release-docs.sh, crates/ctxhelm/tests/release_packaging.rs</files>
  <behavior>
    - Test 1: `scripts/check-release-docs.sh` exists, passes `bash -n`, and exits non-zero if README or `docs/release.md` omit `ctxhelm --version`, `ctxhelm --help`, `v1.1.0`, checksum verification, or `cargo install --git ... --tag v1.1.0 ... --locked`.
    - Test 2: The script fails if docs advertise crates.io, Homebrew, self-update, signed installers, cloud telemetry, or global agent config mutation as required v1.1 paths.
    - Test 3: Rust contract tests run the script successfully once Task 2 fills the docs.
  </behavior>
  <action>Create `scripts/check-release-docs.sh` and extend `crates/ctxhelm/tests/release_packaging.rs` to run it. The script should use simple shell/grep checks against `README.md` and `docs/release.md` for required version, install, checksum, fallback, and out-of-scope claims. Keep it strict enough to catch stale version strings but narrow enough not to become a general docs linter.</action>
  <verify>
    <automated>bash -n scripts/check-release-docs.sh && cargo test -p ctxhelm --test release_packaging release_docs -- --nocapture</automated>
  </verify>
  <done>Release docs have an automated consistency gate before implementation is declared complete.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Write install, fallback, and audit documentation</name>
  <files>README.md, docs/release.md, scripts/check-release-docs.sh, crates/ctxhelm/tests/release_packaging.rs</files>
  <behavior>
    - Test 1: README has a short normal-user path: download archive, verify SHA-256 checksum, extract/move `ctxhelm` onto PATH, run `ctxhelm --version`, and run `ctxhelm --help`.
    - Test 2: `docs/release.md` has maintainer steps for clean checkout packaging, generated archives/checksums, artifact audit, and extracted-binary verification.
    - Test 3: Documentation includes fallback commands `cargo install --git <repo-url> --tag v1.1.0 ctxhelm --locked`, `cargo install --path crates/ctxhelm --locked`, and `cargo build -p ctxhelm --release --locked`.
    - Test 4: Documentation states that crates.io, Homebrew, signed installers, self-update, cloud telemetry/indexing, and global agent config mutation are deferred or not required for v1.1.
  </behavior>
  <action>Update README and add `docs/release.md` per PKG-01, PKG-03, PKG-04, and PKG-05. Keep README concise and user-facing; put maintainer packaging details in `docs/release.md`. Match the actual script names and artifact naming from Plans 02-03. Include checksum verification commands for macOS (`shasum -a 256 -c sha256sums.txt`) and Linux (`sha256sum -c sha256sums.txt`) where appropriate. Do not document unsupported package-manager installs as the normal path and do not claim a release was already published unless it exists.</action>
  <verify>
    <automated>bash scripts/check-release-docs.sh && cargo test -p ctxhelm --test release_packaging release_docs -- --nocapture && cargo run -p ctxhelm -- --version && cargo run -p ctxhelm -- --help</automated>
  </verify>
  <done>Users and maintainers have accurate v1.1 release identity, binary install, source fallback, and artifact-audit documentation.</done>
</task>

</tasks>

<verification>
- `bash scripts/check-release-docs.sh`
- `cargo test -p ctxhelm --test release_packaging release_docs -- --nocapture`
- `cargo run -p ctxhelm -- --version`
- `cargo run -p ctxhelm -- --help`
- `cargo test --workspace`
</verification>

<success_criteria>
Plan 04 is complete when Phase 5 has a coherent user and maintainer release story for v1.1.0, including binary install, checksum verification, source-build fallback, and artifact audit documentation.
</success_criteria>

<output>
After completion, create `.planning/phases/05-release-identity-binary-packaging/05-release-identity-binary-packaging-04-SUMMARY.md`
</output>
