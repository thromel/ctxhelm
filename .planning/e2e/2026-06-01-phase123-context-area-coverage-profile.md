# Phase 123 E2E: Context Area Coverage Profile

Date: 2026-06-01

## Objective

Make source-free context-area MCP resources more decision-ready for agents.
Agents should be able to see whether an area is implementation-heavy,
validation-heavy, docs-only, or mixed before choosing a progressive native read
batch.

## Commands

```bash
cargo fmt --all -- --check
bash -n scripts/smoke-mcp-protocol.sh scripts/check-release-docs.sh
cargo test -p ctxhelm-mcp resources_public_uri_shapes_are_stable -- --nocapture
cargo build -p ctxhelm
CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" \
  CTXHELM_SMOKE_REPO="$(pwd)" \
  bash scripts/smoke-mcp-protocol.sh
bash scripts/check-release-docs.sh
```

## Result

Durable proof:

- `.ctxhelm/e2e/phase123-context-area-coverage-profile.json`

The proof records:

- `ctxhelm://repo/context-areas` entries include `coverageProfile`.
- `ctxhelm://repo/context-area/{encoded-area}` includes `coverageProfile`.
- `coverageProfile.sourceTextLogged = false`.
- Specific area resources expose `recommendedFirstBatch`.
- MCP protocol smoke checks the new field.

## Boundary

- This is additive source-free MCP resource metadata.
- It does not add MCP tools.
- It does not read or store source text.
- It does not change target-file ranking or benchmark floors.
