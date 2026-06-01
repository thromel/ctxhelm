---
phase: 62
title: Production Local Embedding Quality
status: complete
requirements_addressed:
  - EMBED-01
  - EMBED-02
  - EMBED-03
  - EMBED-04
depends_on:
  - 61
---

# Phase 62 Plan: Production Local Embedding Quality

<objective>
Evaluate and harden production local embedding retrieval so ctxhelm can decide whether local semantic retrieval should remain opt-in, be tuned, or be promoted under v2.5 quality gates.
</objective>

<threat_model>
- `local_hash` is mistaken for production semantic quality.
- `local_fastembed` improves a small fixture but regresses real repositories.
- Embedding cache paths pollute inventory or leak source-bearing artifacts into reports.
- Runtime cost overwhelms any recall gain.
</threat_model>

<must_haves>
- Run production local embedding eval on the Phase 61 two-repo corpus.
- Report provider/model/dimensions/cache/source-free status.
- Compare default, lexical, `local_hash`, and production local embedding variants.
- Keep semantic candidates from crowding out anchors or exact evidence.
- State beat/match/trail honestly.
</must_haves>

<tasks>

<task id="62.1" name="Verify local embedding feature and provider status">
<read_first>
- `crates/ctxhelm-index/src/semantic.rs`
- `crates/ctxhelm-compiler/src/policy.rs`
- `docs/semantic.md`
</read_first>
<action>
Build with the local embeddings feature and run semantic provider status for `local_fastembed`. Confirm provider availability, cache path, dimensions, source-free metadata, and degraded states.
</action>
<verify>
- `cargo test --workspace --features local-embeddings --no-fail-fast`
- `cargo run -p ctxhelm --features local-embeddings -- semantic status --semantic-provider local_fastembed --format json`
</verify>
<acceptance_criteria>
- Provider status is source-free and explicit.
- Unavailable model/cache conditions degrade cleanly.
</acceptance_criteria>
</task>

<task id="62.2" name="Run production embedding baseline on two real repos">
<read_first>
- `.planning/e2e/2026-05-22-v25-multirepo-baseline.md`
- `.planning/phases/61-multi-repo-quality-baselines/61-multi-repo-quality-baselines-01-SUMMARY.md`
</read_first>
<action>
Run default, `local_hash`, and `local_fastembed` variants on the same two-repo corpus. Keep large JSON under ignored `.ctxhelm/e2e/` and commit only a concise source-free summary.
</action>
<verify>
- Compare Recall@10, MRR@10, runtime, signal saturation, and named regressions.
</verify>
<acceptance_criteria>
- The summary says whether `local_fastembed` beats, matches, or trails default and lexical per repo.
- Runtime cost is visible.
</acceptance_criteria>
</task>

<task id="62.3" name="Fix cache or provider issues discovered by real eval">
<read_first>
- `crates/ctxhelm-index/src/policy.rs`
- `crates/ctxhelm-index/src/storage.rs`
- `crates/ctxhelm-index/src/semantic.rs`
</read_first>
<action>
If real eval exposes cache pollution, missing diagnostics, stale vectors, or provider metadata gaps, fix the highest-impact issue with focused tests.
</action>
<verify>
- Add targeted tests for any fixed behavior.
- Rerun the affected embedding eval.
</verify>
<acceptance_criteria>
- No cache/source artifacts are indexed as repo files.
- Provider metadata remains stable and source-free.
</acceptance_criteria>
</task>

<task id="62.4" name="Update docs and phase state">
<read_first>
- `docs/semantic.md`
- `docs/benchmarking.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
</read_first>
<action>
Document production local embedding results, limitations, and recommended use. Complete Phase 62 only after real eval and validation pass.
</action>
<verify>
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
- `git diff --check`
</verify>
<acceptance_criteria>
- Docs do not imply default promotion unless metrics prove it.
- Phase 62 summary includes exact command/results.
</acceptance_criteria>
</task>

</tasks>

<verification>
- `cargo test --workspace --features local-embeddings --no-fail-fast`
- local embedding provider status
- two-repo production embedding E2E
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
- `git diff --check`
</verification>

<success_criteria>
- Production local embedding quality is measured on real repos.
- Runtime and cache costs are visible.
- Promotion, hold, or rollback decision is evidence-backed.
</success_criteria>
