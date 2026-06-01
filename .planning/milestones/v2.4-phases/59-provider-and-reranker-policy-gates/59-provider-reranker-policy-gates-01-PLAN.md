---
phase: 59
title: Provider And Reranker Policy Gates
status: planned
requirements_addressed:
  - PROVIDER-01
  - PROVIDER-02
  - PROVIDER-03
  - PROVIDER-04
depends_on:
  - 58
---

# Phase 59 Plan: Provider And Reranker Policy Gates

<objective>
Add explicit provider and reranker policy gates so semantic quality backends can be introduced safely while local-first, source-safe defaults remain enforced.
</objective>

<threat_model>
- Cloud embeddings or cloud rerankers become reachable by default.
- Reranker inputs include raw source snippets without explicit repo policy.
- Provider configuration drift makes reports claim quality backends that are unavailable or disabled.
- Policy warnings are lost before reaching CLI/MCP/eval consumers.
</threat_model>

<must_haves>
- Safe default policy denies cloud and source transfer.
- Provider and reranker decisions are typed, inspectable, and attached to reports.
- Reranker abstraction exists but is disabled unless policy allows it.
- Policy-denied paths produce structured warnings, not silent fallback.
- Tests prove cloud/source transfer remains blocked by default.
</must_haves>

<tasks>

<task id="59.1" name="Add provider and reranker policy contracts">
<read_first>
- `crates/ctxhelm-core/src/contracts.rs`
- `crates/ctxhelm-compiler/src/policy.rs`
- `docs/policy-embedding.md`
</read_first>
<action>
Add contracts for provider policy, data class permissions, reranker policy, provider decisions, and policy reports. Include defaults that allow local metadata/indexing and deny cloud/source transfer.
</action>
<verify>
- Add serialization/default tests.
- Run `cargo test -p ctxhelm-core policy`.
</verify>
<acceptance_criteria>
- Default policy is explicit and source-safe.
- Policy reports can state allowed, denied, unavailable, disabled, and skipped decisions.
</acceptance_criteria>
</task>

<task id="59.2" name="Implement policy loading and enforcement">
<read_first>
- `crates/ctxhelm-compiler/src/policy.rs`
- `crates/ctxhelm-index/src/semantic.rs`
- `crates/ctxhelm-compiler/src/planning.rs`
</read_first>
<action>
Load policy from existing repo config surfaces when available and apply safe defaults when absent. Enforce decisions before semantic provider execution, precision/reranker use, and any provider-dependent eval variant.
</action>
<verify>
- Add tests for absent config, allowed local provider, denied cloud provider, denied source-transfer, and disabled reranker.
- Run `cargo test -p ctxhelm-compiler policy`.
</verify>
<acceptance_criteria>
- No absent config path permits cloud or source transfer.
- Policy decisions appear in prepare/eval report diagnostics.
</acceptance_criteria>
</task>

<task id="59.3" name="Add reranker abstraction behind policy gates">
<read_first>
- `crates/ctxhelm-compiler/src/ranking.rs`
- `crates/ctxhelm-compiler/src/planning.rs`
- `crates/ctxhelm-compiler/src/eval.rs`
</read_first>
<action>
Add a reranker abstraction that can run a deterministic local fixture reranker for tests and report disabled/blocked status otherwise. Reranker inputs must use source-free candidate summaries by default.
</action>
<verify>
- Add tests proving reranker does not run under default policy.
- Add tests proving deterministic local reranker changes ordering only when explicitly enabled.
- Run `cargo test -p ctxhelm-compiler rerank`.
</verify>
<acceptance_criteria>
- Default runtime produces `reranker=disabled` or equivalent.
- Enabled local test reranker is deterministic and source-free.
- Cloud reranker variants are blocked unless explicitly allowed.
</acceptance_criteria>
</task>

<task id="59.4" name="Propagate policy decisions through CLI, MCP-ready reports, and eval">
<read_first>
- `crates/ctxhelm-cli/src/main.rs`
- `crates/ctxhelm-compiler/src/planning.rs`
- `crates/ctxhelm-compiler/src/eval.rs`
- `crates/ctxhelm-mcp/src/*`
</read_first>
<action>
Attach policy decisions and concise warnings to existing CLI JSON, prepare-task reports, context packs, and eval/product proof reports without adding new MCP tools.
</action>
<verify>
- Add integration tests or snapshots for default blocked policy output.
- Run `cargo run -p ctxhelm -- --help`.
- Run relevant smoke scripts that exercise semantic and eval outputs.
</verify>
<acceptance_criteria>
- Users can see why a provider/reranker did or did not run.
- MCP-facing reports remain bounded and structured.
</acceptance_criteria>
</task>

<task id="59.5" name="Document opt-in provider policy">
<read_first>
- `docs/policy-embedding.md`
- `docs/semantic.md`
- `docs/release.md`
- `.planning/ROADMAP.md`
</read_first>
<action>
Document provider/reranker defaults, opt-in policy shape, and privacy implications. Update planning state after verification.
</action>
<verify>
- Run `cargo test --workspace`.
- Run `cargo run -p ctxhelm -- --help`.
- Run `git diff --check`.
</verify>
<acceptance_criteria>
- Docs make clear that quality backends are optional and policy-gated.
- Phase 59 is marked complete only after workspace tests and CLI help pass.
</acceptance_criteria>
</task>

</tasks>

<verification>
- `cargo test -p ctxhelm-core policy`
- `cargo test -p ctxhelm-compiler policy`
- `cargo test -p ctxhelm-compiler rerank`
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
</verification>

<success_criteria>
- Provider/reranker policy exists and is enforced.
- Cloud/source-transfer paths are denied by default.
- Reranker behavior is policy-gated and source-free by default.
- Reports explain provider and reranker decisions.
</success_criteria>
