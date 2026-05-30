---
phase: 65
title: v2.5 Product Proof And Release Gate
status: planned
requirements_addressed:
  - PROOF-01
  - PROOF-02
  - PROOF-03
  - PROOF-04
depends_on:
  - 61
  - 62
  - 63
  - 64
---

# Phase 65 Plan: v2.5 Product Proof And Release Gate

<objective>
Finalize a source-free v2.5 product proof and release gate that can promote,
hold, or block the current retrieval default from measured evidence.
</objective>

<threat_model>
- A big RefactoringMiner improvement hides that ctxpack still trails lexical on
  required corpora.
- Optional proof in the release gate becomes a report-generation smoke instead
  of a product-quality blocker.
- Docs imply production readiness while the measured gate says block.
- Full validation passes unit tests but misses the source-free proof path.
</threat_model>

<must_haves>
- Reuse `ctxpack eval proof` and existing benchmark suite configs.
- Product proof reports beat/match/trail per corpus and variant.
- Release gate fails non-promote benchmark proof when a config is supplied.
- Docs state the current retrieval recommendation honestly.
- Full validation passes.
</must_haves>

<tasks>

<task id="65.1" name="Add product-proof release decision">
<action>
Add release decision fields to `ProductProofReport` and render them in JSON and
Markdown.
</action>
<verify>
- Focused compiler tests cover promote and block behavior.
</verify>
<acceptance_criteria>
- Mixed or trailing corpus verdicts block default promotion.
</acceptance_criteria>
</task>

<task id="65.2" name="Wire optional release-gate proof">
<action>
Update `scripts/release-gate.sh` so configured benchmark proof fails when the
product-proof decision is not `promote`.
</action>
<verify>
- Release packaging contract tests cover the new proof fields.
</verify>
<acceptance_criteria>
- `CTXPACK_BENCHMARK_CONFIG` is a quality gate, not only a report smoke.
</acceptance_criteria>
</task>

<task id="65.3" name="Run source-free proof">
<action>
Run the v2.5 two-repo proof and record the promote/hold/block decision.
</action>
<verify>
- `ctxpack eval proof --config .ctxpack/e2e/phase62-default-config.json --format json`
- Inspect `releaseGate`.
</verify>
<acceptance_criteria>
- The decision matches measured corpus verdicts.
</acceptance_criteria>
</task>

<task id="65.4" name="Finalize docs and state">
<action>
Update benchmarking/release docs, Phase 65 summary, roadmap, and state.
</action>
<verify>
- `cargo test --workspace --no-fail-fast`
- `cargo run -p ctxpack -- --help`
- `git diff --check`
</verify>
<acceptance_criteria>
- v2.5 is marked complete only if proof, docs, and validation are coherent.
</acceptance_criteria>
</task>

</tasks>

<verification>
- focused product-proof release-gate tests
- source-free v2.5 proof JSON
- full workspace tests
- CLI help
- diff hygiene
</verification>

<success_criteria>
- Product proof says whether the current default beats, matches, or trails
  lexical per corpus.
- Release gate blocks the current default if the proof is mixed.
- Docs explain what users should run today and why.
</success_criteria>
