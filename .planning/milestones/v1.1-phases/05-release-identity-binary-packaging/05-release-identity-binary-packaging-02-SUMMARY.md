---
phase: 05-release-identity-binary-packaging
plan: 02
subsystem: packaging
tags: [rust, cargo, shell, release-archive, checksum]
requires:
  - phase: 05-release-identity-binary-packaging
    provides: v1.1.0 release identity and binary version diagnostic
provides:
  - Repeatable local release archive script
  - Versioned tar.gz artifact with README, LICENSE, VERSION, and ctxpack binary
  - SHA-256 checksum outputs
  - Extracted-binary smoke verification outside the checkout
affects: [release-packaging, artifact-audit, docs]
tech-stack:
  added: []
  patterns: [script contract tests, local dist artifact generation, extracted binary smoke]
key-files:
  created: [scripts/release-package.sh, crates/ctxpack/tests/release_packaging.rs]
  modified: [.gitignore]
key-decisions:
  - "Use a local shell script to build GitHub Releases-style tar.gz archives with locked Cargo dependencies."
  - "Keep generated release archives under an ignored dist directory or CTXPACK_DIST_DIR override."
patterns-established:
  - "Packaging scripts must build with cargo build -p ctxpack --release --locked and smoke the extracted binary."
requirements-completed: [PKG-01, PKG-02]
duration: 6min
completed: 2026-05-13
---

# Phase 05 Plan 02: Local Binary Packaging Summary

**Local v1.1.0 release archive builder with SHA-256 checksums and extracted-binary verification**

## Performance

- **Duration:** 6 min
- **Started:** 2026-05-13T18:05:03Z
- **Completed:** 2026-05-13T18:10:51Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added `scripts/release-package.sh` for repeatable local release builds using `cargo build -p ctxpack --release --locked`.
- Produced versioned `ctxpack-v1.1.0-{target}.tar.gz` archives containing only the binary, `README.md`, `LICENSE`, and `VERSION`.
- Wrote per-archive `.sha256` files plus `sha256sums.txt`, then verified the extracted binary with `--version` and `--help`.

## Task Commits

1. **Task 1 RED: script contract guard** - `151850c` (test)
2. **Task 1 GREEN: script shell and dist ignore** - `dac62da` (feat)
3. **Task 2 RED: archive/checksum guard** - `e047eda` (test)
4. **Task 2 GREEN: archive builder** - `b7da228` (feat)

## Files Created/Modified

- `scripts/release-package.sh` - Builds, stages, archives, checksums, and smokes local release artifacts.
- `crates/ctxpack/tests/release_packaging.rs` - Guards packaging script contracts.
- `.gitignore` - Ignores generated `/dist/` artifacts.

## Decisions Made

- Kept release packaging local-only and filesystem-based; no GitHub API, package-manager publish path, signing service, or updater was added.
- Used `CTXPACK_ALLOW_DIRTY=1` only as an explicit escape hatch for local verification during this dirty multi-plan execution.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed hanging version extraction in the release script**
- **Found during:** Task 2
- **Issue:** A pipeline-based `cargo metadata | python3` command substitution left the Python reader waiting in this shell environment during the dirty-check smoke.
- **Fix:** Wrote Cargo metadata to a temporary file and had Python read that file directly.
- **Files modified:** `scripts/release-package.sh`
- **Verification:** `bash scripts/release-package.sh` now exits cleanly with code 65 on a dirty checkout, and `CTXPACK_ALLOW_DIRTY=1 CTXPACK_DIST_DIR="$(mktemp -d)" bash scripts/release-package.sh` succeeds.
- **Committed in:** `b7da228`

**Total deviations:** 1 auto-fixed (Rule 1)
**Impact on plan:** Improved packaging reliability without expanding scope or changing runtime ctxpack behavior.

## Issues Encountered

- The release build initially compiled the optimized target from cold cache and took about 1m41s. Subsequent packaging verification used the cached release build.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p ctxpack --test release_packaging release_package_script_contract -- --nocapture` passed.
- `CTXPACK_ALLOW_DIRTY=1 CTXPACK_DIST_DIR="$(mktemp -d)" bash scripts/release-package.sh` passed.
- `cargo test --workspace` passed.
- `cargo run -p ctxpack -- --help` passed.

## Next Phase Readiness

Plan 03 can add artifact privacy and hygiene auditing to the archive path created here.

## Self-Check: PASSED

- Created files exist: `scripts/release-package.sh`, `crates/ctxpack/tests/release_packaging.rs`
- Modified file exists: `.gitignore`
- Commits exist: `151850c`, `dac62da`, `e047eda`, `b7da228`

---
*Phase: 05-release-identity-binary-packaging*
*Completed: 2026-05-13*
