---
phase: 05-release-identity-binary-packaging
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - crates/ctxhelm/Cargo.toml
  - crates/ctxhelm-core/Cargo.toml
  - crates/ctxhelm-index/Cargo.toml
  - crates/ctxhelm-compiler/Cargo.toml
  - crates/ctxhelm-mcp/Cargo.toml
  - crates/ctxhelm/src/main.rs
  - crates/ctxhelm/tests/cli_compat.rs
  - LICENSE
autonomous: true
requirements: [PKG-01, PKG-03]
must_haves:
  truths:
    - "User can run an installed or built `ctxhelm --version` and see the v1.1 release version."
    - "Maintainer can inspect Cargo metadata and see one consistent package version for the workspace crates."
    - "Release metadata identifies repository, license, README, description, and Rust version without changing CLI/MCP JSON contracts."
  artifacts:
    - path: "Cargo.toml"
      provides: "Workspace release metadata and shared v1.1 version identity"
    - path: "crates/ctxhelm/src/main.rs"
      provides: "Clap package version wiring for `ctxhelm --version`"
    - path: "LICENSE"
      provides: "Root license file referenced by package metadata and release artifacts"
    - path: "crates/ctxhelm/tests/cli_compat.rs"
      provides: "Binary compatibility coverage for version/help output"
  key_links:
    - from: "crates/ctxhelm/src/main.rs"
      to: "crates/ctxhelm/Cargo.toml"
      via: "Clap command metadata uses Cargo package version"
      pattern: "version"
    - from: "Cargo.toml"
      to: "crates/*/Cargo.toml"
      via: "workspace.package metadata inheritance or explicit matching metadata"
      pattern: "version.*1\\.1\\.0"
---

<objective>
Establish release identity for ctxhelm v1.1.

Purpose: Packaging cannot be credible until the binary, workspace metadata, license, and version diagnostic all agree.
Output: Consistent v1.1 package metadata, root license artifact, and binary-level version tests.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/REQUIREMENTS.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/research/SUMMARY.md
@.planning/research/STACK.md
@.planning/research/PITFALLS.md
@.planning/phases/05-release-identity-binary-packaging/05-CONTEXT.md
@.planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-01-SUMMARY.md
@Cargo.toml
@crates/ctxhelm/Cargo.toml
@crates/ctxhelm/src/main.rs
@crates/ctxhelm/tests/cli_compat.rs

<decision_trace>
- D-01: Use a GitHub Releases-style binary archive path first; do not add crates.io/Homebrew/self-update/signing scope in this phase.
- D-02: Keep workspace crate versions, release tag references, README, license metadata, and binary output consistent for v1.1.
- D-03: Add credible release metadata but avoid changing existing plan/pack/MCP JSON contracts.
</decision_trace>

<interfaces>
Current CLI shape in `crates/ctxhelm/src/main.rs`:
```rust
#[derive(Debug, Parser)]
#[command(name = "ctxhelm")]
#[command(about = "Agent-native context packs for coding agents")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
```

Current workspace metadata in `Cargo.toml`:
```toml
[workspace.package]
edition = "2021"
license = "MIT"
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Lock workspace release metadata</name>
  <files>Cargo.toml, crates/ctxhelm/Cargo.toml, crates/ctxhelm-core/Cargo.toml, crates/ctxhelm-index/Cargo.toml, crates/ctxhelm-compiler/Cargo.toml, crates/ctxhelm-mcp/Cargo.toml, LICENSE</files>
  <behavior>
    - Test 1: `cargo metadata --no-deps --format-version 1` reports `1.1.0` for `ctxhelm`, `ctxhelm-core`, `ctxhelm-index`, `ctxhelm-compiler`, and `ctxhelm-mcp`.
    - Test 2: Cargo metadata for every workspace package has license `MIT`, repository, readme, description, and rust-version populated or inherited.
    - Test 3: A root `LICENSE` file exists and matches the MIT license declared in package metadata.
  </behavior>
  <action>Update package metadata per D-02 and PKG-03. Set the v1.1 release identity to `1.1.0` across all workspace packages using workspace inheritance where Cargo accepts it and explicit per-crate fields where nested readme paths require it. Add repository URL metadata for the public ctxhelm repository, a release-appropriate description, and an honest `rust-version` based on the supported local baseline from research. Create a root `LICENSE` file for MIT. Do not publish to crates.io, add package-manager config, or change dependency versions unless Cargo metadata requires a syntax correction.</action>
  <verify>
    <automated>cargo metadata --no-deps --format-version 1 >/tmp/ctxhelm-metadata.json && python3 - <<'PY'
import json
from pathlib import Path
data=json.loads(Path('/tmp/ctxhelm-metadata.json').read_text())
pkgs={p['name']:p for p in data['packages'] if p['name'].startswith('ctxhelm')}
expected={'ctxhelm','ctxhelm-core','ctxhelm-index','ctxhelm-compiler','ctxhelm-mcp'}
assert expected <= set(pkgs), pkgs.keys()
for name in expected:
    p=pkgs[name]
    assert p['version']=='1.1.0', (name,p['version'])
    assert p.get('license')=='MIT', (name,p.get('license'))
    assert p.get('repository'), (name,'repository')
    assert p.get('description'), (name,'description')
    assert p.get('rust_version'), (name,'rust_version')
assert Path('LICENSE').exists()
PY</automated>
  </verify>
  <done>All ctxhelm workspace packages carry consistent v1.1 release metadata and a root MIT license exists.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add binary version diagnostic coverage</name>
  <files>crates/ctxhelm/src/main.rs, crates/ctxhelm/tests/cli_compat.rs</files>
  <behavior>
    - Test 1: `ctxhelm --version` exits 0 and prints `ctxhelm 1.1.0`.
    - Test 2: `ctxhelm --help` still lists existing core commands including `init`, `prepare-task`, `get-pack`, `eval`, and `serve-mcp`.
    - Test 3: `ctxhelm prepare-task --help` and MCP-facing command behavior remain unchanged except for top-level version support.
  </behavior>
  <action>Add Clap version wiring to the top-level `Cli` command so `ctxhelm --version` uses the package version from `crates/ctxhelm/Cargo.toml`. Extend `crates/ctxhelm/tests/cli_compat.rs` with a focused binary test for `--version` and keep the existing help/core-command assertions intact. Do not alter subcommand names, JSON output fields, MCP tool names, or retrieval behavior.</action>
  <verify>
    <automated>cargo test -p ctxhelm --test cli_compat version -- --nocapture && cargo run -p ctxhelm -- --version && cargo run -p ctxhelm -- --help</automated>
  </verify>
  <done>`ctxhelm --version` is a working support diagnostic and help output remains compatible.</done>
</task>

</tasks>

<verification>
- `cargo metadata --no-deps --format-version 1`
- `cargo test -p ctxhelm --test cli_compat version -- --nocapture`
- `cargo run -p ctxhelm -- --version`
- `cargo run -p ctxhelm -- --help`
- `cargo test --workspace`
</verification>

<success_criteria>
Plan 01 is complete when ctxhelm has a consistent v1.1 identity across Cargo metadata, license files, and binary diagnostics without changing runtime context-selection behavior.
</success_criteria>

<output>
After completion, create `.planning/phases/05-release-identity-binary-packaging/05-release-identity-binary-packaging-01-SUMMARY.md`
</output>
