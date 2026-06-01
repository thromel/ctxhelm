---
phase: 63
title: Reranker And Fusion Promotion
status: complete
requirements_addressed:
  - RANK-01
  - RANK-02
  - RANK-03
  - RANK-04
depends_on:
  - 61
  - 62
---

# Phase 63 Plan: Reranker And Fusion Promotion

<objective>
Implement and evaluate promotion-safe reranker/fusion comparison so ctxhelm can decide whether any local ranking variant should become a stronger default.
</objective>

<threat_model>
- A reranker improves aggregate recall while demoting explicit paths, current diff, exact lexical matches, or strong symbols.
- Fusion variants crowd out source/test evidence with semantic or graph neighbors.
- Reports leak source text or add MCP tool surface.
- Promotion gates miss named regressions or runtime/token-cost blowups.
</threat_model>

<must_haves>
- Use the existing benchmark/eval surfaces.
- Keep all reports source-free.
- Compare baseline, lexical, and configured reranker/fusion variants on the fixed corpus.
- Add explicit protected-evidence checks.
- Block promotion on named regressions and privacy/runtime failures.
</must_haves>

<tasks>

<task id="63.1" name="Inspect current ranking and gate behavior">
<read_first>
- `crates/ctxhelm-compiler/src/ranking.rs`
- `crates/ctxhelm-compiler/src/policy.rs`
- `crates/ctxhelm-compiler/src/eval.rs`
- `docs/benchmarking.md`
</read_first>
<action>
Map existing reranker/fusion knobs, benchmark report fields, and promotion gate behavior before editing.
</action>
<verify>
- Identify the narrowest code path that can expose variant comparison without adding MCP tools.
</verify>
<acceptance_criteria>
- Phase 63 edits build on existing ranking/eval contracts.
</acceptance_criteria>
</task>

<task id="63.2" name="Add protected-evidence promotion checks">
<read_first>
- `crates/ctxhelm-compiler/src/ranking.rs`
- `crates/ctxhelm-compiler/src/eval.rs`
</read_first>
<action>
Ensure benchmark reports and promotion gates can detect when explicit anchors, current diff, exact lexical matches, or high-confidence symbols are demoted by a variant.
</action>
<verify>
- Add focused tests for protected evidence behavior.
</verify>
<acceptance_criteria>
- A variant that violates protected evidence policy fails promotion even with aggregate recall lift.
</acceptance_criteria>
</task>

<task id="63.3" name="Compare reranker/fusion variants on real repos">
<read_first>
- `.planning/e2e/2026-05-22-v25-multirepo-baseline.md`
- `.planning/e2e/2026-05-30-phase62-production-local-embedding-quality.md`
</read_first>
<action>
Run the fixed two-repo corpus with baseline and available reranker/fusion variants. Save large outputs under ignored `.ctxhelm/e2e/` and commit a concise source-free summary.
</action>
<verify>
- Compare Recall@10, MRR@10, precision proxy, test recall, token ROI, runtime, privacy, and named regressions.
</verify>
<acceptance_criteria>
- The phase summary states beat/match/trail and promotion decision for each variant.
</acceptance_criteria>
</task>

<task id="63.4" name="Update docs and phase state">
<read_first>
- `docs/benchmarking.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
</read_first>
<action>
Document the variant comparison workflow, promotion gates, and outcome. Complete Phase 63 only after implementation, E2E, and validation pass.
</action>
<verify>
- `cargo test --workspace --no-fail-fast`
- `cargo run -p ctxhelm -- --help`
- `git diff --check`
</verify>
<acceptance_criteria>
- Docs do not imply default promotion unless metrics prove it.
- Phase 63 summary includes exact commands/results.
</acceptance_criteria>
</task>

</tasks>

<verification>
- focused ranking/eval tests
- two-repo reranker/fusion E2E
- `cargo test --workspace --no-fail-fast`
- `cargo run -p ctxhelm -- --help`
- `git diff --check`
</verification>

<success_criteria>
- Reranker/fusion variants are measurable and source-safe.
- Protected evidence cannot be crowded out by weaker signals.
- Promotion decisions are backed by quality, runtime, token ROI, privacy, and named-regression evidence.
</success_criteria>
