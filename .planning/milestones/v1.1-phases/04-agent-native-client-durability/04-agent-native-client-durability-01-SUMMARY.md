---
phase: 04-agent-native-client-durability
plan: 01
subsystem: testing
tags: [mcp, stdio, json-rpc, explicit-repo, smoke-test]

requires:
  - phase: 03-measured-retrieval-lift-eval-gates
    provides: "Stable CLI/MCP compatibility surfaces and attributed retrieval outputs"
provides:
  - "Deterministic stdio MCP protocol smoke with explicit repo arguments from a wrong server cwd"
  - "Binary regression test for prepare_task, get_pack, search, related, related_tests, and current_diff through ctxhelm serve-mcp"
  - "Same-session pack resource read validation using prepare_task-returned resource URIs"
affects: [agent-native-client-durability, mcp, smoke-scripts, cli-compat]

tech-stack:
  added: []
  patterns:
    - "Portable bash smoke delegates JSON-RPC protocol validation to python3"
    - "Compiled-binary MCP tests launch serve-mcp from outside the target repo and pass repo explicitly"

key-files:
  created:
    - scripts/smoke-mcp-protocol.sh
    - .planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-01-SUMMARY.md
  modified:
    - crates/ctxhelm/tests/cli_compat.rs

key-decisions:
  - "Use deterministic JSON-RPC stdio as the Phase 4 hard gate before optional real-client smokes."
  - "Keep all repo-accepting MCP tool calls explicit-repo and validate from a server cwd outside the target repository."
  - "Read pack resources from the same server process using the resource URI returned by prepare_task rather than assuming fixed URI names."

patterns-established:
  - "Protocol smoke scripts should validate structuredContent and resource bodies with JSON parsing, not prose scraping."
  - "Wrong-cwd MCP tests should create a safe current-diff fixture so current_diff cannot pass vacuously."

requirements-completed: [AGNT-01, AGNT-03]

duration: 5m15s
completed: 2026-05-13
---

# Phase 4 Plan 01: Deterministic MCP Protocol Smoke Summary

**Explicit-repo stdio MCP hard gate with wrong-cwd tool coverage and same-session pack resource validation**

## Performance

- **Duration:** 5m15s
- **Started:** 2026-05-13T16:19:50Z
- **Completed:** 2026-05-13T16:25:05Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added a binary compatibility test that launches `ctxhelm serve-mcp` from a temp directory outside the fixture repo and proves `prepare_task`, `get_pack`, `search`, `related`, `related_tests`, and `current_diff` all use the explicit `repo` argument.
- Added `scripts/smoke-mcp-protocol.sh`, a portable protocol gate that launches the stdio MCP server from the wrong cwd, sends newline-delimited JSON-RPC, validates structured content with `python3`, and reads the returned pack resource in the same server session.
- Verified the new protocol gate together with the full workspace tests and CLI help.

## Task Commits

1. **Task 1: Add deterministic wrong-cwd protocol smoke coverage** - `8ceaf1b` (test)
2. **Task 2: Run the protocol gate with workspace validation** - `362718a` (test, empty validation commit)

## Files Created/Modified

- `crates/ctxhelm/tests/cli_compat.rs` - Adds compiled-binary wrong-cwd MCP regression coverage across all repo-accepting implemented tools.
- `scripts/smoke-mcp-protocol.sh` - Adds the deterministic JSON-RPC stdio smoke, including same-session pack resource read validation.

## Decisions Made

- Use the deterministic protocol smoke as the hard gate for Phase 4 client durability, with real-client scripts left to later plans.
- Validate resource reads through the task-scoped URI returned by `prepare_task`; pack resource names are not assumed to be stable fixed strings.
- Keep validation local-only and dependency-light: bash, cargo, and `python3` JSON parsing only.

## Deviations from Plan

None - plan executed as written.

## Known Stubs

None.

## Issues Encountered

- The initial RED test intentionally tried a fixed pack URI and failed because pack resource URIs are task-scoped. The final smoke reads `prepare_task.packOptions[0].resourceUri` and appends `.json`, which matches the existing MCP session semantics.
- Parallel Phase 4 Plan 02 changes landed while this plan was executing. This plan only staged its owned test and script files.

## Verification

- `cargo test -p ctxhelm --test cli_compat mcp_protocol -- --nocapture`
- `CTXHELM_SMOKE_REPO="$PWD" CTXHELM_SMOKE_TASK="harden MCP explicit repo handling" bash scripts/smoke-mcp-protocol.sh`
- `CTXHELM_SMOKE_REPO="$PWD" CTXHELM_SMOKE_TASK="harden MCP explicit repo handling" bash scripts/smoke-mcp-protocol.sh && cargo test --workspace && cargo run -p ctxhelm -- --help`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 4 now has a deterministic protocol-level hard gate. Later plans can focus on pack cache/session visibility and optional Codex CLI / Claude Code wrappers without using real-client success as a substitute for this protocol proof.

## Self-Check: PASSED

- Found `crates/ctxhelm/tests/cli_compat.rs`
- Found `scripts/smoke-mcp-protocol.sh`
- Found `.planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-01-SUMMARY.md`
- Found commit `8ceaf1b`
- Found commit `362718a`

---
*Phase: 04-agent-native-client-durability*
*Completed: 2026-05-13*
