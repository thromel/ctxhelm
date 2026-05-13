---
phase: 05-release-identity-binary-packaging
plan: 03
subsystem: packaging
tags: [shell, release-audit, privacy, local-first]
requires:
  - phase: 05-release-identity-binary-packaging
    provides: local release archive builder
provides:
  - Release artifact audit script
  - Synthetic archive audit tests
  - Packaging integration that gates checksums and success output on audit pass
affects: [release-packaging, docs, verification]
tech-stack:
  added: []
  patterns: [local archive member audit, text payload leakage scan, package-script audit gate]
key-files:
  created: [scripts/audit-release-artifact.sh]
  modified: [scripts/release-package.sh, crates/ctxpack/tests/release_packaging.rs]
key-decisions:
  - "Audit archive member paths for local ctxpack state, traces, request logs, caches, target debris, git internals, and secret-looking names."
  - "Scan included text payloads for machine-specific paths and secret-looking assignments without printing source contents."
patterns-established:
  - "Release packaging must call the artifact audit immediately after archive creation and before checksum success output."
requirements-completed: [PKG-05]
duration: 3min
completed: 2026-05-13
---

# Phase 05 Plan 03: Artifact Audit Summary

**Local-only release artifact audit that blocks state/cache/secret leakage before packaging succeeds**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-13T18:12:24Z
- **Completed:** 2026-05-13T18:15:43Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added `scripts/audit-release-artifact.sh` for `.tar.gz` member inspection and extracted text-payload scanning.
- Added synthetic archive tests proving `.ctxpack/repos/.../traces.jsonl` fails and a minimal release archive passes.
- Integrated the audit into `scripts/release-package.sh` before checksum writing and success output.

## Task Commits

1. **Task 1 RED: audit guards** - `ca5f4bb` (test)
2. **Task 1 GREEN: audit script** - `be613c0` (feat)
3. **Task 2 RED: package integration guard** - `e74f7ba` (test)
4. **Task 2 GREEN: package audit gate** - `c0cef8b` (feat)

## Files Created/Modified

- `scripts/audit-release-artifact.sh` - Audits release archive names and text payloads.
- `scripts/release-package.sh` - Runs the audit after archive creation.
- `crates/ctxpack/tests/release_packaging.rs` - Adds audit contract and synthetic archive tests.

## Decisions Made

- Treated `.ctxpack`, traces, caches, `.git`, `target/`, request logs, and secret-looking names as forbidden archive member paths.
- Kept text scanning focused on machine-specific paths and secret-looking assignments so normal README descriptions of local ctxpack behavior do not create false positives.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p ctxpack --test release_packaging release_artifact_audit -- --nocapture` passed.
- `CTXPACK_ALLOW_DIRTY=1 CTXPACK_DIST_DIR="$(mktemp -d)" bash scripts/release-package.sh` passed and printed `audit passed`.
- `cargo test --workspace` passed.

## Next Phase Readiness

Plan 04 can document the release install, checksum, fallback, and audit flow against the actual scripts.

## Self-Check: PASSED

- Created file exists: `scripts/audit-release-artifact.sh`
- Modified files exist: `scripts/release-package.sh`, `crates/ctxpack/tests/release_packaging.rs`
- Commits exist: `ca5f4bb`, `be613c0`, `e74f7ba`, `c0cef8b`

---
*Phase: 05-release-identity-binary-packaging*
*Completed: 2026-05-13*
