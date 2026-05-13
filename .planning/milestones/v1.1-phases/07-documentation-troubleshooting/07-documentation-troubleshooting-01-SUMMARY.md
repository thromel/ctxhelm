---
phase: 07-documentation-troubleshooting
plan: 01
subsystem: docs
tags: [quickstart, install, first-pack, cli, mcp]
requires:
  - phase: 05-release-identity-binary-packaging
    provides: v1.1.0 archive install and release documentation
  - phase: 06-agent-setup-first-pack-adoption
    provides: setup-check and first-pack smoke behavior
provides:
  - Installed-binary README quickstart from install to first prepare-task/get-pack
  - Detailed first-pack quickstart with setup validation and MCP resource caveats
affects: [phase-08-release-gates, docs, onboarding]
tech-stack:
  added: []
  patterns: [installed-binary-first documentation, explicit repo examples]
key-files:
  created: [docs/quickstart.md]
  modified: [README.md]
key-decisions:
  - "Normal-user documentation uses the installed ctxpack binary with explicit --repo arguments; source checkout commands are kept in Development or fallback contexts."
  - "First-pack docs describe session-scoped MCP pack resources and get_pack as the durable reconnect path."
patterns-established:
  - "README keeps a concise path and delegates deeper operational explanation to dedicated docs."
requirements-completed: [DOCS-01]
duration: 3min
completed: 2026-05-13
---

# Phase 07 Plan 01: Install To First Pack Summary

**Installed-binary onboarding from release archive verification to first prepare-task/get-pack commands**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-13T19:05:51Z
- **Completed:** 2026-05-13T19:08:30Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Reworked README so the first normal-user flow uses `ctxpack`, not `cargo run`, after release archive install.
- Added `docs/quickstart.md` with explicit `--repo` setup validation, first `prepare-task`, first `get-pack`, and MCP session-scope caveats.
- Preserved source-checkout commands only as Development or maintainer validation context.

## Task Commits

1. **Task 1: Rewrite README quickstart around installed binary flow** - `830d69e` (docs)
2. **Task 2: Add detailed first-pack quickstart** - `ef0e693` (docs)

## Files Created/Modified

- `README.md` - Installed-binary first-pack path and docs links.
- `docs/quickstart.md` - Detailed first-pack reference and MCP resource scope explanation.

## Decisions Made

- Normal-user docs should assume `ctxpack` is installed and on `PATH`; source checkout commands are not the default onboarding route.
- MCP `packOptions[*].resourceUri` values are described as same-session conveniences, while `get_pack` is the durable path after reconnects.

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None.

## Issues Encountered

- The quickstart initially used a forbidden phrase while disclaiming unsupported OpenCode real-client proof. The wording was tightened before commit; no scope change.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 02 can link to the installed-binary quickstart and expand the agent setup/proof taxonomy without changing runtime behavior.

## Verification

- `python3 -c 'from pathlib import Path; r=Path("README.md").read_text(); assert "docs/quickstart.md" in r; assert "ctxpack init --repo" in r; assert "ctxpack setup-check --repo" in r; assert "ctxpack prepare-task" in r; assert "ctxpack get-pack" in r; assert "cargo run -p ctxpack -- init" not in r; assert "cargo run -p ctxpack -- serve-mcp" not in r'`
- `python3 -c 'from pathlib import Path; q=Path("docs/quickstart.md").read_text(); required=["ctxpack --version","ctxpack --help","ctxpack init --repo","ctxpack setup-check --repo","ctxpack prepare-task","ctxpack get-pack","--budget brief","explicit `--repo`","session-scoped"]; missing=[s for s in required if s not in q]; assert not missing, missing; assert "cargo run -p ctxpack -- init" not in q; assert "Cursor real-client" not in q; assert "OpenCode real-client" not in q'`
- `cargo run -p ctxpack -- --help`

## Self-Check: PASSED

- Found created file: `docs/quickstart.md`
- Found modified file: `README.md`
- Found task commit: `830d69e`
- Found task commit: `ef0e693`

---
*Phase: 07-documentation-troubleshooting*
*Completed: 2026-05-13*
