# Phase 122: Public Real-Client Protocol Compatibility

## Goal

Keep optional public-archive real-client proof usable after the source tree adds
new MCP protocol assertions that are not present in an already-published archive.

## Gap

`scripts/smoke-public-real-clients.sh` downloads the public `v1.1.0` archive but
uses the current source-tree wrapper scripts. After Phase 118, the current
`scripts/smoke-mcp-protocol.sh` required `resourceScope` fields on context-area
resources. The public `ctxpack 1.1.0` archive predates that assertion, so the
public real-client smoke failed before Codex or Claude wrappers could write
source-free pass/skip evidence.

This was a release-evidence compatibility bug, not a user-facing ctxpack runtime
bug.

## Changes

- Added `CTXPACK_REQUIRE_RESOURCE_SCOPE`, defaulting to `1`, to
  `scripts/smoke-mcp-protocol.sh`.
- Kept current build/release-candidate protocol checks strict by default.
- Made `scripts/smoke-public-real-clients.sh` set
  `CTXPACK_REQUIRE_RESOURCE_SCOPE=0` only for the published `ctxpack 1.1.0`
  archive, unless overridden by `CTXPACK_PUBLIC_SMOKE_REQUIRE_RESOURCE_SCOPE`.
- Added Rust contract coverage for the new compatibility knob.
- Updated release and distribution docs so the compatibility boundary is
  explicit and not confused with weakening current release gates.

## Validation

Passed focused proof:

```bash
CTXPACK_REQUIRE_RESOURCE_SCOPE=0 \
  CTXPACK_BIN=<downloaded public v1.1.0 archive binary> \
  CTXPACK_SMOKE_REPO=/Users/romel/Documents/GitHub/ctxpack-release-gate-clean-20260601 \
  bash scripts/smoke-mcp-protocol.sh

CTXPACK_BIN=/Users/romel/Documents/GitHub/ctxpack-release-gate-clean-20260601/target/debug/ctxpack \
  CTXPACK_SMOKE_REPO=/Users/romel/Documents/GitHub/ctxpack-release-gate-clean-20260601 \
  bash scripts/smoke-mcp-protocol.sh

CTXPACK_RUN_REAL_CLIENT=1 \
  bash scripts/smoke-public-real-clients.sh \
    --smoke-repo /Users/romel/Documents/GitHub/ctxpack-release-gate-clean-20260601 \
    --output /tmp/ctxpack-public-real-client-rerun.json
```

Observed public real-client results:

- Public archive verification: passed.
- Claude Code `2.1.158`: passed with explicit-repo `prepare_task` and
  `get_pack` evidence.
- Codex CLI `0.44.0`: source-free optional skip; deterministic protocol passed,
  but Codex still produced no machine-checkable `prepare_task`/`get_pack` tool
  calls.
- Missing evidence: resolved.

## Boundary

This does not claim Codex CLI real-client success. It makes the optional skip
machine-checkable again and keeps current protocol assertions strict for current
builds.
