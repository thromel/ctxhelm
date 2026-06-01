# Phase 47 Plan: Public Adoption Docs & Demo Artifacts

## Tasks

1. Add a deterministic demo artifact generator that emits pack inspector,
   retrieval health, graph neighborhood, policy/embedding, and agent-preview
   examples.
2. Add a demo artifact smoke test that validates required files, local-only
   privacy status, and absence of source/prompts/secrets/local paths.
3. Add public-facing docs for demo artifacts and a concise public project
   summary with accurate current capabilities and non-claims.
4. Link the new docs from the README and wire the demo smoke into release docs,
   release-gate execution, and release-packaging contract tests.
5. Regenerate checked-in source-free demo artifacts and run focused validation.

## Verification

- `bash scripts/smoke-demo-artifacts.sh`
- `bash scripts/check-release-docs.sh`
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target-phase47 cargo test -p ctxhelm --test release_packaging -- --nocapture`

