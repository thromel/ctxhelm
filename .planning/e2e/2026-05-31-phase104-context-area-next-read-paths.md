# Phase 104: Context Area Next-Read Paths

## Goal

Make ranked-below-budget source/docs pressure operational for agents without
changing top-10 target-file ranking. Before this phase, broad context areas and
gap summaries could point to a resource, but generated packs still made agents
infer which concrete source or docs paths to inspect next.

## Changes

- Added `ContextArea.nextReadPaths` as a source-free list of unselected
  candidate paths an agent can read natively before asking for a deeper pack.
- Added `ContextArea.unselectedCount` so clients can tell whether an area has
  candidate pressure outside the selected target/test budget.
- Included docs candidates in broad `contextAreas`, ordered after
  source/config/schema areas and before test-only areas.
- Updated generated packs to render explicit `Next reads` in the broad
  context-area guidance section, including zero-selected areas.
- Hardened `scripts/check-product-proof.py` to fail cleanly when a benchmark
  repository has no embedded report instead of crashing while checking
  resource-backed gaps.
- Raised the git `rev-list` timeout used by history indexing from 2 seconds to
  10 seconds for large/offloaded local repositories.

## Validation

Focused tests passed:

```bash
CARGO_TARGET_DIR=/tmp/ctxhelm-phase104-target cargo test -p ctxhelm --test release_packaging product_proof_checker_accepts_promote_and_rejects_block -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxhelm-phase104-target cargo test -p ctxhelm-core context_plan_public_json_shape_is_stable -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxhelm-phase104-target cargo test -p ctxhelm-compiler context_ -- --nocapture
```

The three-repo proof excluding the locally blocked RefactoringMiner checkout
promoted and passed the product-proof checker:

```bash
python3 scripts/check-product-proof.py .ctxhelm/e2e/phase104-context-area-next-read-paths-no-refminer-proof.json
```

Reported metrics in that proof:

| Repository | File Recall@10 | Source Recall@10 | Test Recall@10 | Effective Validation Recall@10 | Broad Context-Area Recall |
|------------|---------------:|-----------------:|---------------:|--------------------------------:|--------------------------:|
| ctxhelm | 0.47460318 | 0.7166667 | n/a | n/a | 1.0 |
| ReAgent | 0.5 | 1.0 | 1.0 | 1.0 | n/a |
| VeriSchema | 0.18449473 | 0.31067252 | 0.7089947 | 1.0 | 0.71851856 |

The four-repo proof was not claimed. The local RefactoringMiner proof checkout
timed out on `git rev-list` even after the timeout increase, producing a failed
repository entry with no embedded report. The product-proof checker now reports
that shape cleanly as:

```text
embedded benchmark repository report was missing: RefactoringMiner
```

That is a local checkout/object-availability blocker for this run, not a
measured recall regression in the other repositories.

## Impact

This phase keeps top-10 ranking stable while making the source-free broad-area
channel more useful to Codex, Claude Code, Cursor, OpenCode, and generic MCP
clients. Agents now receive concrete native read targets for unselected
source/docs pressure instead of only an area label or resource URI.
