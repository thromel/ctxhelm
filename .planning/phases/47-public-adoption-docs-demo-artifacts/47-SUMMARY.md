# Phase 47 Summary: Public Adoption Docs & Demo Artifacts

Status: Complete

## Completed

- Added `scripts/generate-demo-artifacts.sh` and checked-in
  `docs/demo-artifacts/` with static source-free examples for pack inspector,
  retrieval health, graph neighborhood, policy/embedding, and agent preview.
- Added `scripts/smoke-demo-artifacts.sh` to validate demo artifact presence,
  local-only privacy status, and absence of raw source, prompts, secrets, and
  machine-local paths.
- Added `docs/demo.md` and `docs/public-project-summary.md`.
- Linked the public docs from `README.md`.
- Added the demo smoke to `docs/release.md`, `scripts/release-gate.sh`,
  `scripts/check-release-docs.sh`, and release packaging contract tests.

## Verification

- `bash scripts/smoke-demo-artifacts.sh` passed.
- `bash scripts/check-release-docs.sh` passed.
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-phase47 cargo test -p ctxpack --test release_packaging -- --nocapture` passed.

