# Phase 90: Packaged Release Gate

## Goal

Prove that the production release path passes from a clean checkout using the
packaged `ctxhelm` binary, not only the local development binary.

The active checkout contained unrelated untracked duplicate planning drafts, so
the release gate correctly refused to package from it. To avoid deleting user
files or weakening the gate with `CTXHELM_ALLOW_DIRTY=1`, the proof ran in a
temporary clean worktree at commit `0c36316`.

## Command

```bash
git worktree add --detach /tmp/ctxhelm-release-gate.MLfSJC/worktree HEAD

CTXHELM_SKIP_REAL_CLIENT=1 \
CTXHELM_BENCHMARK_CONFIG=/tmp/ctxhelm-release-gate.MLfSJC/broader-fixed-corpus-absolute.json \
bash scripts/release-gate.sh
```

The benchmark config was the pinned broader fixed-corpus fixture with absolute
paths for the local RefactoringMiner, ctxhelm, ReAgent, and VeriSchema
checkouts.

## Passed Gates

- Workspace tests.
- Release docs consistency.
- Release package and artifact audit.
- Clean archive extraction verification.
- Extracted binary identity: `ctxhelm 1.1.0`.
- First-pack, storage, memory, feedback, workspace, shared-artifact,
  inspector, retrieval-health, graph, policy/embedding, agent-preview,
  semantic, precision, v2.3 eval, and v2.4 semantic/precision smokes.
- Wrong-cwd MCP protocol smoke.
- Cursor and OpenCode setup/protocol evidence.
- Optional broad benchmark product proof through the packaged binary.
- Codex and Claude deterministic protocol gates, with real-client tool-call
  evidence intentionally skipped via `CTXHELM_SKIP_REAL_CLIENT=1`.

Final gate output:

```text
release gate passed: binary=/private/var/folders/6y/_t27xsmd06vbgzrmmj0gz9v00000gn/T/tmp.V6Hmmqsc63/extracted/ctxhelm-v1.1.0-aarch64-apple-darwin/ctxhelm archive=/var/folders/6y/_t27xsmd06vbgzrmmj0gz9v00000gn/T/tmp.V6Hmmqsc63/dist/ctxhelm-v1.1.0-aarch64-apple-darwin.tar.gz proof=/var/folders/6y/_t27xsmd06vbgzrmmj0gz9v00000gn/T/tmp.V6Hmmqsc63/proof-bundle/release-proof-summary.json
```

## Rejected Attempts

Three source-selection heuristics were tested before the release-gate proof and
were not kept:

1. Treating shell scripts as config plus increasing the broad config floor.
   - Result: VeriSchema File Recall@10 regressed from `0.18449473` to
     `0.17939667`.
   - The release proof blocked on runtime in that run.

2. Splitting nested broad source areas into deeper path groups.
   - Result: proof promoted, but VeriSchema File Recall@10 regressed from
     `0.18449473` to `0.17782804` and Source Recall@10 regressed from
     `0.31067252` to `0.29678363`.

3. Reducing broad source-area representatives from four to three per area.
   - Result: proof promoted, but File Recall@10, Source Recall@10, validation
     coverage, and protected-target metrics did not improve.
   - Gap taxonomy worsened for some no-candidate families, so the change had
     no production-quality benefit.

All three were reverted because they did not improve production readiness under
the pinned corpus evidence.

## Result

The packaged release gate passes in a clean worktree with the broader benchmark
proof enabled. Remaining source-candidate gaps are still real, but they are now
post-release quality-improvement work rather than blockers for the current
packaged release path.
