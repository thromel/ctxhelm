---
phase: 03-measured-retrieval-lift-eval-gates
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/ctxpack-core/src/contracts.rs
  - crates/ctxpack-compiler/src/planning.rs
  - crates/ctxpack-compiler/src/lib.rs
  - crates/ctxpack/src/main.rs
autonomous: true
requirements: [RETR-01, RETR-03, PARS-01]
must_haves:
  truths:
    - "ContextPlan JSON can carry typed retrieval candidates without removing existing targetFiles or relatedTests fields."
    - "Every target file and related test can carry source-free attribution records."
    - "Typed retrieval candidate contracts explicitly cover file, test, symbol, doc, commit, and config candidates."
    - "Old plan JSON without attribution or retrievalCandidates still deserializes."
  artifacts:
    - path: "crates/ctxpack-core/src/contracts.rs"
      provides: "Additive public candidate and attribution contracts"
    - path: "crates/ctxpack-compiler/src/planning.rs"
      provides: "Default empty attribution/candidate wiring for current plan construction"
    - path: "crates/ctxpack/src/main.rs"
      provides: "Updated CLI renderer fixtures for additive contract fields"
  key_links:
    - from: "crates/ctxpack-core/src/contracts.rs"
      to: "crates/ctxpack-compiler/src/planning.rs"
      via: "TargetFile and RelatedTest constructors include attribution defaults"
      pattern: "attribution: Vec::new\\(\\)"
---

<objective>
Create the public typed contract layer that later ranking and eval plans build on.

Purpose: Phase 3 needs evidence-weighted retrieval without breaking existing CLI, MCP, pack, card, or eval clients.
Output: Additive serde contracts for candidates, signal scores, and source-free attribution.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/REQUIREMENTS.md
@.planning/phases/03-measured-retrieval-lift-eval-gates/03-CONTEXT.md
@.planning/phases/03-measured-retrieval-lift-eval-gates/03-RESEARCH.md
@.planning/phases/01-compatibility-guardrails-module-boundaries/01-compatibility-guardrails-module-boundaries-02-SUMMARY.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-04-SUMMARY.md

<decision_trace>
- D-01: Introduce a typed candidate layer before ContextPlan projection.
- D-02: Preserve public JSON compatibility by adding fields, not replacing targetFiles, relatedTests, riskFlags, diagnostics, or provenance.
- D-03: Keep recommended-file and related-test attribution source-free: paths, roles, signal kinds, scores, edge labels, commit ids/counts, and reason codes are allowed; snippets and prompt text are not.
- D-10: Parser/runtime upgrades are allowed only behind existing contracts and tests.
- Deferred: no cloud/vector retrieval, visual dashboard, or client durability work in this phase.
</decision_trace>

<interfaces>
Existing public contracts from crates/ctxpack-core/src/contracts.rs:
```rust
pub struct TargetFile {
    pub path: String,
    pub reason: String,
    pub line_range: Option<LineRange>,
    pub confidence: f32,
}

pub struct RelatedTest {
    pub path: String,
    pub reason: String,
    pub command: Option<String>,
    pub confidence: f32,
}

pub struct ContextPlan {
    pub target_files: Vec<TargetFile>,
    pub related_tests: Vec<RelatedTest>,
    pub recommended_commands: Vec<Command>,
    pub risk_flags: Vec<RiskFlag>,
    pub diagnostics: Vec<Diagnostic>,
    pub privacy_status: PrivacyStatus,
}
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add source-free retrieval candidate and attribution contracts</name>
  <files>crates/ctxpack-core/src/contracts.rs</files>
  <behavior>
    - Test 1: ContextPlan serializes camelCase additive fields `retrievalCandidates`, `targetFiles[].attribution`, and `relatedTests[].attribution`.
    - Test 2: Old JSON without the new fields deserializes with empty default vectors.
    - Test 3: Serialized attribution does not contain task text, source snippets, symbol signatures, or commit subject fields.
    - Test 4: `RetrievalCandidateKind` serializes every Phase 3 required kind: file, test, symbol, doc, commit, and config.
  </behavior>
  <action>Add public serde contracts per D-01/D-03: `RetrievalCandidateKind`, `RetrievalSignalKind`, `RetrievalSignalScore`, `RetrievalEvidence`, and `RetrievalCandidate`. `RetrievalCandidateKind` must include at least `file`, `test`, `symbol`, `doc`, `commit`, and `config` variants so RETR-01 cannot be satisfied by file-only scoring. Add `#[serde(default)] attribution: Vec<RetrievalEvidence>` to `TargetFile` and `RelatedTest`, and `#[serde(default)] retrieval_candidates: Vec<RetrievalCandidate>` to `ContextPlan`. Use camelCase for structs and snake_case/lowercase enum values consistent with existing contracts. Do not remove or rename existing fields per D-02.</action>
  <verify>
    <automated>cargo test -p ctxpack-core retrieval -- --nocapture</automated>
  </verify>
  <done>Core contract tests prove additive candidate/attribution fields exist for all required candidate kinds, remain source-free, and keep old JSON compatible.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Update constructors and compatibility fixtures for additive fields</name>
  <files>crates/ctxpack-compiler/src/planning.rs, crates/ctxpack-compiler/src/lib.rs, crates/ctxpack/src/main.rs, crates/ctxpack-core/src/contracts.rs</files>
  <behavior>
    - Test 1: Existing compiler and CLI renderer fixtures compile with explicit empty attribution/retrievalCandidates defaults.
    - Test 2: Current public JSON shape tests still assert existing targetFiles, relatedTests, diagnostics, and privacyStatus keys.
  </behavior>
  <action>Update every `TargetFile`, `RelatedTest`, and `ContextPlan` literal touched by compiler/core/CLI tests to populate the new additive fields with empty vectors. Preserve the current behavior: Plan 01 only establishes contracts; it must not change ranking order, pack snippets, MCP tools, or eval behavior. Keep this compatible with future parser-backed adapters behind the typed contracts per D-10.</action>
  <verify>
    <automated>cargo test -p ctxpack-core -p ctxpack-compiler -p ctxpack -- --nocapture</automated>
  </verify>
  <done>Workspace code compiles against the new contracts while existing plan/eval/render behavior remains unchanged.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxpack-core retrieval -- --nocapture`
- `cargo test -p ctxpack-core -p ctxpack-compiler -p ctxpack -- --nocapture`
</verification>

<success_criteria>
Plan 01 is complete when the public contract layer supports typed, source-free retrieval evidence additively and old JSON remains readable.
</success_criteria>

<output>
After completion, create `.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-01-SUMMARY.md`
</output>
