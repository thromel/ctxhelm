---
phase: 02-trust-layer-operational-diagnostics
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/ctxhelm-core/src/contracts.rs
  - crates/ctxhelm-index/src/lib.rs
  - crates/ctxhelm-index/src/inventory.rs
  - crates/ctxhelm-index/src/policy.rs
autonomous: true
requirements: [SAFE-03, DIAG-01, DIAG-02]
must_haves:
  truths:
    - "Public contracts can carry source-free diagnostics without removing existing riskFlags compatibility."
    - "Sensitive, generated, binary, oversized, unreadable, and non-UTF-8 source-read reasons have typed reason codes."
    - "Privacy classification lives behind one tested index policy module instead of scattered path helpers."
  artifacts:
    - path: "crates/ctxhelm-core/src/contracts.rs"
      provides: "Diagnostic, diagnostic severity, cache/trace status, and additive diagnostics fields"
      contains: "diagnostics"
    - path: "crates/ctxhelm-index/src/policy.rs"
      provides: "Central source policy and source-read classification helpers"
      contains: "read_safe_source"
    - path: "crates/ctxhelm-index/src/lib.rs"
      provides: "Crate-root re-exports for policy contracts needed by downstream plans"
      contains: "pub use policy"
  key_links:
    - from: "crates/ctxhelm-index/src/inventory.rs"
      to: "crates/ctxhelm-index/src/policy.rs"
      via: "classification delegation"
      pattern: "classify_path"
    - from: "crates/ctxhelm-core/src/contracts.rs"
      to: "existing ContextPlan and ContextPack JSON"
      via: "additive camelCase fields with serde defaults"
      pattern: "diagnostics"
---

<objective>
Create the Phase 2 trust-layer contracts and central policy entry point before broad read-path wiring.

Purpose: SAFE-03, DIAG-01, and DIAG-02 require typed, source-free diagnostics and a single privacy/source-read policy before freshness, pack, CLI, and MCP code can use them consistently.
Output: Additive core diagnostics contracts plus a tested `ctxhelm-index` policy module that centralizes privacy and source-read reason codes without changing retrieval ranking.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/REQUIREMENTS.md
@.planning/STATE.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-CONTEXT.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-RESEARCH.md
@.planning/phases/01-compatibility-guardrails-module-boundaries/01-compatibility-guardrails-module-boundaries-02-SUMMARY.md
@.planning/phases/01-compatibility-guardrails-module-boundaries/01-compatibility-guardrails-module-boundaries-03-SUMMARY.md
@.planning/codebase/ARCHITECTURE.md
@.planning/codebase/TESTING.md
@.planning/codebase/CONVENTIONS.md
@AGENTS.md

<interfaces>
From `crates/ctxhelm-core/src/contracts.rs`:
```rust
#[serde(rename_all = "camelCase")]
pub struct RiskFlag { pub code: String, pub message: String }

#[serde(rename_all = "camelCase")]
pub struct ContextPlan {
    pub risk_flags: Vec<RiskFlag>,
    pub privacy_status: PrivacyStatus,
}

#[serde(rename_all = "camelCase")]
pub struct ContextPack {
    pub warnings: Vec<String>,
    pub privacy_status: PrivacyStatus,
}
```

From `crates/ctxhelm-index/src/inventory.rs`:
```rust
#[serde(rename_all = "camelCase")]
pub struct FileInventoryEntry {
    pub path: String,
    pub role: FileRole,
    pub hash: String,
    pub size_bytes: u64,
    pub generated: bool,
    pub ignored: bool,
}

pub(crate) fn classify_path(path: &str) -> FileRole;
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add additive diagnostics contracts</name>
  <files>crates/ctxhelm-core/src/contracts.rs</files>
  <read_first>
    - `crates/ctxhelm-core/src/contracts.rs`
    - `.planning/phases/02-trust-layer-operational-diagnostics/02-RESEARCH.md` diagnostics contract section
  </read_first>
  <behavior>
    - SAFE-03/DIAG-01/DIAG-02: `ContextPlan` and `ContextPack` serialize existing fields exactly as before plus a camelCase `diagnostics` array.
    - Diagnostics are source-free: fields may include code, severity, message, paths, and count, but no snippet or prompt text.
    - Backward compatibility is preserved through `#[serde(default)]` on new fields and by keeping `riskFlags` unchanged.
  </behavior>
  <action>
    Add small typed contracts in `contracts.rs`: `Diagnostic`, `DiagnosticSeverity`, and minimal cache/trace status contracts if needed by later plans. Add `diagnostics: Vec<Diagnostic>` to `ContextPlan` and `ContextPack` with serde defaults. Keep `RiskFlag` intact and do not rename or remove any existing public field. Add public JSON shape tests that assert `diagnostics` exists, `riskFlags` still exists, snake_case names are absent, and serialized diagnostics contain no source or prompt text.
  </action>
  <verify>
    <automated>cargo test -p ctxhelm-core public_json_shape -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - `ContextPlan` and `ContextPack` JSON contain `diagnostics` and existing `riskFlags`/`warnings` fields.
    - New diagnostic severity values serialize predictably using existing serde conventions.
    - Existing Phase 1 core contract tests remain green.
  </acceptance_criteria>
  <done>Core contracts carry source-free diagnostics additively and public JSON compatibility tests prove no field replacement occurred.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Centralize privacy and source-read policy</name>
  <files>crates/ctxhelm-index/src/lib.rs, crates/ctxhelm-index/src/inventory.rs, crates/ctxhelm-index/src/policy.rs</files>
  <read_first>
    - `crates/ctxhelm-index/src/lib.rs`
    - `crates/ctxhelm-index/src/inventory.rs`
    - `.planning/codebase/CONCERNS.md` sensitive-file and read-failure sections
  </read_first>
  <behavior>
    - SAFE-03: common credentials and generated families classify through one policy path: `.npmrc`, `.yarnrc.yml`, `.pypirc`, `.netrc`, `.ssh/id_rsa`, `.ssh/id_ed25519`, `serviceAccountKey.json`, `firebase-adminsdk*.json`, `.aws/credentials`, terraform state, `node_modules`, `target`, `dist`, `build`, `coverage`, `vendor`, minified assets, and lockfiles.
    - DIAG-01: policy skip outcomes produce stable diagnostic codes such as `source_policy_excluded`, `source_binary`, `source_non_utf8`, `source_oversized`, and `source_unreadable`.
    - DIAG-02: policy result types are typed Rust values, not CLI/MCP formatting strings.
  </behavior>
  <action>
    Create `crates/ctxhelm-index/src/policy.rs` with a central source policy API. Move or delegate path classification from `inventory.rs` into this module while preserving existing `classify_path` behavior for current tests. Add `SourceRead`, `SourceReadStatus`/reason enum, and `read_safe_source(repo_root, inventory, path, max_bytes)` returning either safe UTF-8 text plus diagnostics or a source-free diagnostic skip. Re-export only the types/functions needed by downstream crates from `lib.rs`; keep helper internals crate-visible. Do not introduce cloud, embeddings, reranking, or retrieval scoring changes.
  </action>
  <verify>
    <automated>cargo test -p ctxhelm-index policy -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - Table-driven policy tests cover credential/auth, SSH key, cloud credential JSON, generated/vendor, binary, non-UTF-8, oversized, and unreadable cases.
    - Existing inventory role tests still pass through the centralized policy.
    - New public re-exports are minimal and support later compiler/MCP use.
  </acceptance_criteria>
  <done>Privacy and source-read decisions are centralized in `policy.rs`, tested, and available to later freshness/read-path plans.</done>
</task>

</tasks>

<verification>
Run focused contract/policy tests and then the affected crates:
```bash
cargo test -p ctxhelm-core public_json_shape -- --nocapture
cargo test -p ctxhelm-index policy -- --nocapture
cargo test -p ctxhelm-core -p ctxhelm-index
```
</verification>

<success_criteria>
SAFE-03, DIAG-01, and DIAG-02 have foundational typed contracts and policy APIs. Existing public JSON compatibility is additive, and no Phase 3 retrieval ranking/eval lift work is introduced.
</success_criteria>

<output>
After completion, create `.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-01-SUMMARY.md`.
</output>
