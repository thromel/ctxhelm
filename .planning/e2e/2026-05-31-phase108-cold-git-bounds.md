# Phase 108: Cold Git Bounds

## Goal

Reduce cold four-repo proof hangs without reintroducing future-context
contamination from source-repo child paths.

## What Changed

- Parent snapshot caches now live under the local ctxpack home instead of inside
  the source repository, so snapshot scans do not inherit the live repo `.git`
  ancestor.
- Parent snapshot extraction no longer uses `git archive`; that path regressed
  ReAgent quality and could hang on old generated planning/source artifacts.
- Parent snapshots use bounded `git cat-file --batch` extraction and skip a
  stalled object batch instead of recursively expanding into many stalled
  object reads.
- Parent snapshot caches now write a source-free completion manifest and are
  reused only when all object batches completed, so a bounded object-store
  failure cannot poison later warm-cache proofs as a valid empty snapshot.
- Historical subject sampling now uses the bounded no-rename `diff-tree` path
  instead of rename detection plus a slow fallback.
- Git helpers disable `core.fsmonitor` for local history commands.

## Evidence

Focused validation passed:

```text
cargo fmt --check
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase108b-target cargo test -p ctxpack-compiler parent_snapshot_batch_reader_extracts_multiple_paths -- --nocapture
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase108b-target cargo test -p ctxpack-compiler historical_eval_parent_snapshot_extracts_only_indexable_paths -- --nocapture
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase108b-target cargo test -p ctxpack-compiler historical_eval_uses_parent_snapshot_without_future_context -- --nocapture
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase108b-target cargo test -p ctxpack-compiler parent_snapshot_command_helper_times_out_instead_of_hanging -- --nocapture
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase108b-target cargo test -p ctxpack-compiler incomplete_parent_snapshot_manifests_are_not_reusable -- --nocapture
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase108b-target cargo test -p ctxpack-compiler sidecar_only_parent_snapshot_cache_is_not_reusable -- --nocapture
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase108b-target cargo test -p ctxpack-index co_change_hints_use_eval_history_sidecar_when_snapshot_has_no_git -- --nocapture
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase108b-target cargo test -p ctxpack-index historical_commit_collection_uses_fast_no_rename_diff -- --nocapture
```

Cold proof artifact:

```text
.ctxpack/e2e/phase108-cold-git-bounded-proof.json
```

The cold proof still blocks:

```text
decision = block
reason = RefactoringMiner, ctxpack, and ReAgent insufficient evidence
evaluatedRepositoryCount = 1
evaluatedCommitCount = 5
VeriSchema = beat, runtime 7463ms
```

## Remaining Blocker

The remaining blocker is not ranking or pack compilation. It is local Git/object
store availability in the cold proof environment:

- Direct reads of at least one ReAgent loose object under `.git/objects` block at
  the filesystem level.
- `git show`, `git cat-file -t`, `git cat-file -p`, `git cat-file --batch`, and
  detached worktree checkout variants time out on the same object path.
- `git ls-tree` remains fast, so history metadata can be sampled, but historical
  parent source hydration cannot be trusted when object contents are unreadable.

Phase 108 therefore improves failure mode and bounds the hang, but does not make
the cold four-repo production proof promotable. The next phase should add an
explicit source-free object-read health check and a proof verdict that separates
environment object-store failure from ctxpack retrieval quality.
