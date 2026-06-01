# Phase 42 Summary: Graph Neighborhoods & Communities

**Completed:** 2026-05-18
**Status:** Complete

## Delivered

- Added source-free graph node, edge, community, and neighborhood contracts.
- Added graph neighborhood builder using dependency edges, related tests, memory
  cards, feedback events, and task-derived anchors.
- Added capped, non-recursive graph expansion with cap diagnostics.
- Added `ctxhelm graph neighborhood`.
- Added graph documentation.
- Added `scripts/smoke-graph.sh` and wired it into release docs and release
  gate.

## Verification

- `cargo fmt --all`
- `cargo run -p ctxhelm -- graph neighborhood --help`
- `bash scripts/smoke-graph.sh`
- `bash scripts/check-release-docs.sh`
- `bash -n scripts/release-gate.sh`
- `bash -n scripts/smoke-graph.sh`
- `cargo test -p ctxhelm release_gate_script_contract -- --nocapture`
- `cargo test --workspace`

All commands passed using `CARGO_TARGET_DIR=/tmp/ctxhelm-target-phase39` where
applicable.

## Next

Phase 43 can add graph-aware policy experiments and embedding provider controls
on top of the graph neighborhood report.
