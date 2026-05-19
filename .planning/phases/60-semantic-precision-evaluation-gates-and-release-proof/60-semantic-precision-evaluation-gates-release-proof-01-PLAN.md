---
phase: 60
title: Semantic Precision Evaluation Gates And Release Proof
status: planned
requirements_addressed:
  - GATE-01
  - GATE-02
  - GATE-03
  - GATE-04
depends_on:
  - 57
  - 58
  - 59
---

# Phase 60 Plan: Semantic Precision Evaluation Gates And Release Proof

<objective>
Add fixed-corpus semantic/precision/reranker evaluation gates and release proof so v2.4 can honestly report quality lift, neutral impact, or regressions before promoting runtime defaults.
</objective>

<threat_model>
- A feature is declared useful because it exists rather than because eval evidence supports it.
- Semantic or precision variants regress named cases but aggregate metrics hide the issue.
- Reranker/cloud variants accidentally run without policy permission.
- Release docs claim improvements not present in fixed-corpus reports.
</threat_model>

<must_haves>
- Fixed-corpus variants cover lexical, graph, semantic, precision, full hybrid, and policy-allowed reranker modes.
- Reports include quality, runtime, cache, token, provider, precision, reranker, and privacy status.
- Gate outcomes distinguish promote, hold, and block.
- Named improvements, misses, and regressions are emitted.
- Release docs are generated from measured evidence, not assumptions.
</must_haves>

<tasks>

<task id="60.1" name="Extend benchmark variants for semantic and precision gates">
<read_first>
- `crates/ctxpack-compiler/src/eval.rs`
- `crates/ctxpack-core/src/contracts.rs`
- `docs/benchmarking.md`
- `scripts/smoke-v23-eval.sh`
</read_first>
<action>
Add benchmark variant controls for lexical, lexical+graph, local semantic, precision-enriched semantic, semantic+precision, full hybrid, and policy-allowed reranked variants. Reuse the Phase 58 query trace and Phase 59 policy decisions.
</action>
<verify>
- Add eval tests for variant construction and skipped policy-blocked variants.
- Run `cargo test -p ctxpack-compiler eval`.
</verify>
<acceptance_criteria>
- Variants are deterministic for a fixed corpus manifest.
- Policy-blocked variants are reported as skipped, not silently omitted.
</acceptance_criteria>
</task>

<task id="60.2" name="Add gate metrics and named case diagnostics">
<read_first>
- `crates/ctxpack-compiler/src/eval.rs`
- `crates/ctxpack-core/src/contracts.rs`
</read_first>
<action>
Extend reports with Recall@K, MRR@K where available, Test Recall@K, candidate precision proxy, token efficiency, runtime, cache status, named wins, named regressions, and named misses.
</action>
<verify>
- Add tests using a small fixed manifest with known wins/regressions.
- Run `cargo test -p ctxpack-compiler eval`.
</verify>
<acceptance_criteria>
- Mixed results are visible in named-case diagnostics.
- Reports include enough evidence to explain why a feature did or did not improve.
</acceptance_criteria>
</task>

<task id="60.3" name="Implement promote, hold, and block release gate">
<read_first>
- `crates/ctxpack-compiler/src/eval.rs`
- `crates/ctxpack-core/src/contracts.rs`
- `docs/release.md`
</read_first>
<action>
Implement a release gate decision that can promote, hold, or block based on thresholds, critical regressions, missing reports, unsafe privacy state, and policy violations.
</action>
<verify>
- Add tests for promote, hold, block, missing report, critical regression, and unsafe privacy state.
- Run `cargo test -p ctxpack-compiler gate`.
</verify>
<acceptance_criteria>
- Unsafe privacy or unapproved provider state blocks release proof.
- Neutral/mixed quality keeps features opt-in or held instead of promoted.
- Gate output is machine-readable and human-readable.
</acceptance_criteria>
</task>

<task id="60.4" name="Create v2.4 smoke script and release proof docs">
<read_first>
- `scripts/smoke-v23-eval.sh`
- `docs/benchmarking.md`
- `docs/release.md`
- `docs/semantic.md`
- `docs/precision.md`
</read_first>
<action>
Add or update a smoke script that runs the v2.4 semantic/precision gate on local fixtures. Update docs to describe how to interpret promote, hold, and block outcomes.
</action>
<verify>
- Run the new v2.4 smoke script.
- Run `scripts/smoke-v23-eval.sh` to check compatibility.
</verify>
<acceptance_criteria>
- v2.4 gate can run locally without external services.
- Docs state any held or opt-in behavior plainly.
</acceptance_criteria>
</task>

<task id="60.5" name="Finalize v2.4 planning and release state">
<read_first>
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `docs/release.md`
</read_first>
<action>
After all verification passes, update milestone state, roadmap checkboxes, and release notes to reflect completion of v2.4 semantic/precision production gates.
</action>
<verify>
- Run `cargo test --workspace`.
- Run `cargo run -p ctxpack -- --help`.
- Run `git diff --check`.
</verify>
<acceptance_criteria>
- Phase 60 and v2.4 are marked complete only after all required gates pass.
- Release proof does not claim unmeasured quality improvements.
</acceptance_criteria>
</task>

</tasks>

<verification>
- `cargo test -p ctxpack-compiler eval`
- `cargo test -p ctxpack-compiler gate`
- `scripts/smoke-v23-eval.sh`
- v2.4 semantic/precision gate smoke script
- `cargo test --workspace`
- `cargo run -p ctxpack -- --help`
</verification>

<success_criteria>
- Semantic/precision variants are evaluated against fixed corpora.
- Release gate reports promote, hold, or block.
- Named regressions and misses are visible.
- v2.4 claims are backed by measured reports.
</success_criteria>
