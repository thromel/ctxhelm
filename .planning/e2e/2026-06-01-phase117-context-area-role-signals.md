# Phase 117 E2E: Context Area Role Signals

Date: 2026-06-01

## Objective

Make broad context-area guidance more actionable without changing target-file
ranking. Agents should be able to tell whether a surfaced area is source-heavy,
validation-heavy, or docs-only before choosing native file reads.

## Commands

```bash
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase117-target \
  cargo run -p ctxpack -- prepare-task \
  --repo "$(pwd)" \
  --mode explain \
  --no-trace \
  "promote broad context area proof and update release docs" \
  > /tmp/phase117-plan.json

CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase117-target \
  cargo run -p ctxpack -- get-pack \
  --repo "$(pwd)" \
  --mode explain \
  --budget brief \
  --format markdown \
  --no-trace \
  "promote broad context area proof and update release docs" \
  > /tmp/phase117-pack.md
```

## Result

Durable proof:

- `.ctxpack/e2e/phase117-context-area-role-signals.json`

The proof records:

- `contextAreaCount = 13`
- plan-level `roleCounts` for surfaced context areas
- plan-level `selectedRoleCounts` for surfaced context areas
- generated pack rendering of `Role counts:`
- generated pack rendering of `Selected roles:`

Example role signals:

- `crates/ctxpack-compiler`: `roleCounts.source = 4`,
  `selectedRoleCounts.source = 4`
- `crates/ctxpack`: `roleCounts.source = 1`, `roleCounts.test = 1`,
  `selectedRoleCounts.source = 1`, `selectedRoleCounts.test = 1`
- `crates/ctxpack-core`: `roleCounts.source = 4`, no selected roles,
  four `nextReadPaths`

## Boundary

- This is additive contract metadata.
- It does not alter the MCP tool surface.
- It does not change top-10 target-file ranking.
- It does not include source snippets or raw prompt traces.
