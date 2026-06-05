# Phase 248: Local Inspector Shell

**Date:** 2026-06-06
**Requirements:** UX-01, UX-02, UX-03, UX-04
**Boundary:** localhost-only, read-only, source-free diagnostic UX

## What Changed

- Added `ctxhelm inspector serve`.
- Reused the existing context planner/compiler path used by `inspector export`.
- Added a loopback-only HTTP shell bound to `127.0.0.1`.
- Added source-free routes:
  - `/`
  - `/pack-inspector.html`
  - `/pack-inspector.json`
  - `/graph.html`
  - `/graph.json`
  - `/setup-status.json`
  - `/health.json`
- Added filterable graph-neighborhood HTML for nodes, edges, and communities.
- Added setup/status JSON from the existing read-only `setup-check` contract.
- Kept the shell diagnostic only; it does not edit files, mutate global agent configuration, run user project tests, or replace daily coding inside existing agents.

## Proof Commands

```bash
cargo test -p ctxhelm inspector_serve_command_parses_local_shell_options --locked
```

```bash
bash scripts/smoke-inspector.sh
```

## Smoke Coverage

`scripts/smoke-inspector.sh` now:

- creates a temporary repository with a source sentinel,
- exports source-free JSON and HTML pack inspector artifacts,
- starts `ctxhelm inspector serve`,
- fetches `/`,
- fetches `/pack-inspector.json`,
- fetches `/graph.html`,
- fetches `/graph.json`,
- fetches `/setup-status.json`,
- fetches `/health.json`,
- verifies local shell and graph UI hooks,
- verifies read-only/source-free health metadata,
- rejects the source sentinel in all inspected outputs.

## Requirement Status

- UX-01 complete: user can open a local inspector shell for diagnostic review.
- UX-02 complete: user can interactively filter graph neighborhoods and retrieval diagnostics through graph and pack-inspector views.
- UX-03 complete: user can inspect setup/status output from the shell without editing source files.
- UX-04 complete: the shell is diagnostic-only and explicitly keeps daily coding inside existing agents.
