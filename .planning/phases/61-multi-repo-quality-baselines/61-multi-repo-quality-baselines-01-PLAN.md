---
phase: 61
title: Multi-Repo Quality Baselines
status: complete
requirements_addressed:
  - BASE-01
  - BASE-02
  - BASE-03
  - BASE-04
depends_on:
  - 60
---

# Phase 61 Plan: Multi-Repo Quality Baselines

<objective>
Add a source-free multi-repo retrieval-quality baseline path so ctxhelm can compare default, lexical, semantic, graph, reranked, and learned-policy variants across real repositories without overfitting to RefactoringMiner.
</objective>

<threat_model>
- Quality claims are overfit to one repository.
- Reports mix different revision ranges or options and become incomparable.
- Multi-repo reports leak source text or prompt text.
- Runtime becomes too high to use as a release gate.
</threat_model>

<must_haves>
- Manifest or CLI path for at least two real local repositories.
- Stable per-repo corpus identity and options in output.
- Per-repo and aggregate quality metrics.
- Named wins, misses, regressions, and repeated gap families.
- Source-free provider, cache, runtime, and privacy status.
</must_haves>

<tasks>

<task id="61.1" name="Inspect existing benchmark and proof contracts">
<read_first>
- `crates/ctxhelm-compiler/src/eval.rs`
- `crates/ctxhelm-core/src/contracts.rs`
- `docs/benchmarking.md`
- `.planning/e2e/2026-05-22-refactoringminer-semantic-fusion-regression.md`
</read_first>
<action>
Map the existing single-repo benchmark, proof, and baseline reports to a multi-repo aggregate contract. Prefer extending existing structures over inventing a parallel eval system.
</action>
<verify>
- Document selected contract fields in code comments or tests.
</verify>
<acceptance_criteria>
- The implementation path reuses existing source-free eval reports.
- No source-bearing fields are added.
</acceptance_criteria>
</task>

<task id="61.2" name="Add multi-repo baseline manifest support">
<read_first>
- `crates/ctxhelm-compiler/src/eval.rs`
- `crates/ctxhelm/src/main.rs`
- existing benchmark config parsing tests
</read_first>
<action>
Add a manifest-driven or CLI-supported multi-repo baseline command/report that runs fixed options for each repository and returns per-repo plus aggregate output.
</action>
<verify>
- Add tests for manifest parsing, missing repo diagnostics, stable options, and source-free JSON shape.
- Run `cargo test -p ctxhelm-compiler benchmark`.
</verify>
<acceptance_criteria>
- The report includes repo label, path-derived repo id, revision range, limit, budget, variants, and privacy status.
- Missing or invalid repos degrade with diagnostics instead of panicking.
</acceptance_criteria>
</task>

<task id="61.3" name="Run real two-repo baseline proof">
<read_first>
- `.planning/e2e/2026-05-22-refactoringminer-semantic-fusion-regression.md`
- local repository availability one directory up
</read_first>
<action>
Run the multi-repo baseline against RefactoringMiner and one second real local repo. Use bounded limits and parallelism, but keep enough commits to expose non-toy behavior.
</action>
<verify>
- Save a concise source-free E2E summary under `.planning/e2e/`.
- Keep large JSON outputs under ignored `.ctxhelm/e2e/`.
</verify>
<acceptance_criteria>
- The summary reports per-repo default vs lexical status.
- Named gap families and runtime are visible.
</acceptance_criteria>
</task>

<task id="61.4" name="Update docs and phase state">
<read_first>
- `docs/benchmarking.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
</read_first>
<action>
Document how to run and interpret multi-repo baselines. Update phase state only after implementation and real proof pass.
</action>
<verify>
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
- `git diff --check`
</verify>
<acceptance_criteria>
- Phase 61 remains planned until real two-repo proof exists.
- Docs do not claim retrieval lift unless metrics show it.
</acceptance_criteria>
</task>

</tasks>

<verification>
- `cargo test -p ctxhelm-compiler benchmark`
- two-repo baseline E2E
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
- `git diff --check`
</verification>

<success_criteria>
- ctxhelm can produce a source-free multi-repo retrieval-quality baseline.
- RefactoringMiner is no longer the only product-quality proof point.
- Reports make lexical/default/semantic quality status visible per repo.
</success_criteria>
