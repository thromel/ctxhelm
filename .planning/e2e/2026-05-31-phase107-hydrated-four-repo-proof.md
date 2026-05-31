# Phase 107 E2E: Hydrated Four-Repo Proof Path

## Goal

Make the pinned four-repo production proof hydrate every configured repository
with embedded, source-free reports instead of hanging, returning `report: null`,
or dropping RefactoringMiner from the measured corpus.

## Issue

The broad proof config in
`.planning/e2e/2026-05-31-phase92-area-aware-gap-proof-config.json` previously
failed the production-readiness path in two different ways:

- Parent snapshot hydration could wedge on unbounded full-tree `git ls-tree -r`
  and `git archive` calls against older ctxpack revisions.
- RefactoringMiner commit collection could return zero usable commits because
  `git diff-tree -M` rename detection exceeded the per-commit timeout on the
  large historical repository.

Those failures made the full proof shape weaker than the three-repo and warm
proof paths because a large real repository could become missing evidence
instead of an explicit, measurable corpus verdict.

## Changes

- Historical commit collection now falls back to `git diff-tree` without rename
  detection when the rename-detection path times out.
- Parent snapshot existence checks are path-limited with bounded subprocess
  timeouts instead of scanning the full tree.
- Parent snapshot extraction uses bounded per-file `git show` reads instead of
  piping a whole revision archive through `tar`.
- Bad single parent-snapshot paths are skipped after recursive chunk splitting
  instead of blocking the whole repository proof.
- Unrelated historical `.ctxpack/e2e`, `.planning/e2e`, and
  `.planning/milestones` archive artifacts are filtered from parent snapshot
  hydration unless they are actual safe changed files for the sampled commit.

The fix intentionally does not tune release thresholds and does not add a
top-10 ranking workaround. The proof remains honest about cold runtime.

## Evidence

Cold force proof:

```text
artifact: .ctxpack/e2e/phase107-hydrated-four-repo-cold-proof.json
decision: block
reason: Blocked because proof runtime exceeded 5000ms per commit for: ctxpack.
evaluatedRepositoryCount: 4
evaluatedCommitCount: 12

RefactoringMiner: match, runtime 3936ms, context Recall@10 1.0 vs lexical 1.0
ctxpack: beat, runtime 34867ms, File Recall@10 0.43650794 vs lexical 0.32539684
ReAgent: beat, runtime 5312ms, context Recall@10 1.0 vs lexical 0.5
VeriSchema: beat, runtime 3532ms, File Recall@10 0.18449473 vs lexical 0.122021124
```

Warm/cache proof:

```text
artifact: .ctxpack/e2e/phase107-hydrated-four-repo-warm-proof.json
decision: promote
reason: Promote: every evaluated corpus beat lexical or reached a perfect lexical ceiling on non-test context recall, while maintaining validation-test recall under local-only proof thresholds.
evaluatedRepositoryCount: 4
evaluatedCommitCount: 12

RefactoringMiner: match, runtime 4ms, context Recall@10 1.0 vs lexical 1.0
ctxpack: beat, runtime 3ms, File Recall@10 0.43650794 vs lexical 0.32539684
ReAgent: beat, runtime 3ms, context Recall@10 1.0 vs lexical 0.5
VeriSchema: beat, runtime 7ms, File Recall@10 0.18449473 vs lexical 0.122021124
```

## Validation

Passed:

```bash
cargo fmt --check
CARGO_TARGET_DIR=/tmp/ctxpack-phase107-target cargo test -p ctxpack-index historical_commit_collection_falls_back_when_rename_diff_times_out -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-phase107-target cargo test -p ctxpack-compiler parent_snapshot_command_helper_times_out_instead_of_hanging -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-phase107-target cargo test -p ctxpack-compiler historical_eval_parent_snapshot_extracts_only_indexable_paths -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-phase107-target cargo test -p ctxpack-compiler parent_snapshot_candidates_keep_changed_archives_but_drop_unrelated_archives -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-phase107-target cargo run -p ctxpack -- eval proof --config .planning/e2e/2026-05-31-phase92-area-aware-gap-proof-config.json --format json
CARGO_TARGET_DIR=/tmp/ctxpack-phase107-target cargo run -p ctxpack -- eval proof --config .planning/e2e/2026-05-31-phase92-area-aware-gap-warm-proof-config.json --format json
CARGO_TARGET_DIR=/tmp/ctxpack-phase107-target cargo run -p ctxpack -- eval proof --config .planning/e2e/2026-05-31-phase92-area-aware-gap-warm-proof-config.json --format json
```

Deferred full gate:

- The cold four-repo proof still blocks on ctxpack runtime, but it now does so
  with all four repositories hydrated and verdicted. The remaining work is
  cold runtime reduction, not missing reports or large-repo proof hydration.
- The first warm proof after the cache-schema bump rebuilt fresh reports and
  therefore also blocked on runtime; the second warm proof used the current
  cache schema and promoted with resource-backed gap summaries.
- The cache schema was bumped to `historical-eval-cache-v2.3.5` so cached
  reports cannot reuse the pre-resource-backed or pre-spooled-output snapshot
  hydration shape.
