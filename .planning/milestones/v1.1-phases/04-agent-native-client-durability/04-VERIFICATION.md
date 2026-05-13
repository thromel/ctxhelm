---
phase: 04-agent-native-client-durability
verified: 2026-05-13T17:04:33Z
status: passed
score: 4/4 must-haves verified
re_verification:
  previous_status: gaps_found
  previous_score: 3/4
  gaps_closed:
    - "Codex CLI and Claude Code smoke scripts verify real MCP `prepare_task` and `get_pack` client paths with explicit `repo` arguments."
  gaps_remaining: []
  regressions: []
---

# Phase 4: Agent-Native Client Durability Verification Report

**Phase Goal:** Users can rely on ctxpack from real coding-agent clients without session surprises, wrong-repo behavior, or static context dumps.  
**Verified:** 2026-05-13T17:04:33Z  
**Status:** passed  
**Re-verification:** Yes - after gap closure commit `4380390`

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Codex CLI and Claude Code smoke scripts verify real MCP `prepare_task` and `get_pack` client paths with explicit `repo` arguments. | VERIFIED | Fresh required-mode runs passed: `CTXPACK_REQUIRE_REAL_CLIENT=1 ... smoke-codex-mcp.sh` and `... smoke-claude-mcp.sh` both ran the protocol gate and recorded server-side `prepare_task` plus `get_pack` calls with `repo=/Users/romel/Documents/GitHub/Agent Memory`. |
| 2 | User can tell whether MCP pack resources are session-scoped or reconstructed from persisted source-free metadata after server restarts. | VERIFIED | `ctxpack://pack/guide` and missing-resource diagnostics state pack resources are MCP-session scoped and `get_pack` is reconnect-safe; restart test passed. |
| 3 | Maintainer can test MCP cache growth, reconnect behavior, and wrong-working-directory behavior without relying on manual client inspection. | VERIFIED | Wrong-cwd protocol smoke/test, pack-resource restart test, and cache-bound tests all pass. |
| 4 | Generated adapter guidance stays thin and directs agents to dynamic ctxpack calls instead of injecting large static repository context. | VERIFIED | Adapter guidance tests pass and assert `prepare_task`, explicit `repo`, progressive `get_pack`, size bounds, and no static-dump phrases. |

**Score:** 4/4 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `scripts/smoke-mcp-protocol.sh` | Deterministic stdio MCP hard gate from wrong cwd with explicit repo. | VERIFIED | Calls initialize, `prepare_task`, `get_pack`, search/related tools, `current_diff`, and same-session `resources/read`; fresh smoke passed. |
| `crates/ctxpack/tests/cli_compat.rs` | Binary-level regression tests for wrong-cwd, restart, and wrapper contracts. | VERIFIED | Contains and passes focused tests for explicit repo JSON-RPC calls, pack-resource restart diagnostics, and wrapper syntax/contracts. |
| `crates/ctxpack-core/src/init.rs` | Thin dynamic adapter constants plus tests. | VERIFIED | Guidance uses dynamic calls and explicit repo; adapter tests passed. |
| `crates/ctxpack-mcp/src/resources.rs` | Session-scoped pack resource diagnostics and bounded/test-visible cache behavior. | VERIFIED | Implements `MAX_PACK_RESOURCE_CACHE_ENTRIES`, deterministic insertion-order eviction, clear missing-resource diagnostic, and guide text. |
| `crates/ctxpack-mcp/src/lib.rs` | In-process MCP resource/cache tests. | VERIFIED | Pack resource same-session, session-scope, and cache-bound tests passed. |
| `scripts/smoke-codex-mcp.sh` | Optional/required Codex real-client MCP wrapper. | VERIFIED | Runs protocol gate first, isolates temp state, uses `--dangerously-bypass-approvals-and-sandbox`, and required mode passed with server-side tool-call evidence. |
| `scripts/smoke-claude-mcp.sh` | Optional/required Claude Code real-client MCP wrapper. | VERIFIED | Runs protocol gate first, uses temp strict MCP config and normal Claude auth path without `--bare`; required mode passed with server-side tool-call evidence. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `scripts/smoke-mcp-protocol.sh` | `ctxpack serve-mcp` | `cargo run --manifest-path ... -p ctxpack -- serve-mcp` from temp cwd | WIRED | Fresh protocol smoke passed with explicit repo and same-session pack resource read. |
| `crates/ctxpack/tests/cli_compat.rs` | MCP tool handlers | JSON-RPC `tools/call` for `prepare_task`, `get_pack`, search/related/current_diff tools | WIRED | Focused `mcp_protocol` and `pack_resource` tests passed. |
| `crates/ctxpack-mcp/src/tools.rs` | `crates/ctxpack-mcp/src/resources.rs` | `prepare_task` calls `cache_pack_resources` | WIRED | Manual trace found the call; cache/resource tests passed. |
| `scripts/smoke-codex-mcp.sh` | `scripts/smoke-mcp-protocol.sh` | Protocol hard gate before `codex exec` | WIRED | gsd-tools key-link check passed; required smoke confirms ordering at runtime. |
| `scripts/smoke-claude-mcp.sh` | `scripts/smoke-mcp-protocol.sh` | Protocol hard gate before `claude -p` | WIRED | gsd-tools key-link check passed; required smoke confirms ordering at runtime. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `scripts/smoke-mcp-protocol.sh` | `structuredContent`, pack resource JSON | Live `ctxpack serve-mcp` subprocess over stdio | Yes | FLOWING |
| `scripts/smoke-codex-mcp.sh` | Request evidence log | Server-side tee of Codex client JSON-RPC stdin | Yes | FLOWING |
| `scripts/smoke-claude-mcp.sh` | Request evidence log | Server-side tee of Claude client JSON-RPC stdin | Yes | FLOWING |
| `crates/ctxpack-mcp/src/resources.rs` | Cached pack entries | `cache_pack_resources` compiles packs from live plan/repo data | Yes | FLOWING |
| `crates/ctxpack-core/src/init.rs` | Generated guidance text | Static adapter constants generated by `run_init` | Yes, intentionally static guidance only | VERIFIED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Script syntax | `bash -n scripts/smoke-mcp-protocol.sh && bash -n scripts/smoke-codex-mcp.sh && bash -n scripts/smoke-claude-mcp.sh` | exit 0 | PASS |
| Wrapper contract tests | `cargo test -p ctxpack --test cli_compat real_client_smoke_scripts -- --nocapture` | 1 passed | PASS |
| Codex required real-client path | `CTXPACK_REQUIRE_REAL_CLIENT=1 CTXPACK_SMOKE_REPO="$PWD" CTXPACK_SMOKE_TASK="harden MCP explicit repo handling" bash scripts/smoke-codex-mcp.sh` | protocol gate passed; server-side instrumentation recorded `prepare_task` and `get_pack` with explicit repo | PASS |
| Claude required real-client path | `CTXPACK_REQUIRE_REAL_CLIENT=1 CTXPACK_SMOKE_REPO="$PWD" CTXPACK_SMOKE_TASK="harden MCP explicit repo handling" bash scripts/smoke-claude-mcp.sh` | protocol gate passed; server-side instrumentation recorded `prepare_task` and `get_pack` with explicit repo | PASS |
| Wrong-cwd protocol test | `cargo test -p ctxpack --test cli_compat mcp_protocol -- --nocapture` | 1 passed | PASS |
| Pack resource restart test | `cargo test -p ctxpack --test cli_compat pack_resource -- --nocapture` | 1 passed | PASS |
| MCP pack resource/cache tests | `cargo test -p ctxpack-mcp pack_resource -- --nocapture` | 3 passed | PASS |
| Adapter guidance tests | `cargo test -p ctxpack-core adapter -- --nocapture` | 7 passed | PASS |
| Deterministic protocol smoke | `CTXPACK_SMOKE_REPO="$PWD" CTXPACK_SMOKE_TASK="harden MCP explicit repo handling" bash scripts/smoke-mcp-protocol.sh` | protocol smoke ok; explicit repo and pack resource verified | PASS |
| Formatting | `cargo fmt --all --check` | exit 0 | PASS |
| Workspace validation | `cargo test --workspace` | 177 tests passed, 0 failed, plus doc-tests | PASS |
| CLI help | `cargo run -p ctxpack -- --help` | help lists core commands including `serve-mcp` | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| AGNT-01 | Plans 01, 04 | Codex CLI and Claude Code smoke scripts verify real MCP `prepare_task` and `get_pack` client paths with explicit `repo` arguments. | SATISFIED | Required Codex and Claude smokes passed with server-side evidence for both tools and explicit repo. |
| AGNT-02 | Plan 03 | MCP pack resources are clearly session-scoped or can be reconstructed across restarts. | SATISFIED | Guide/diagnostics explain session scope and `get_pack` reconnect-safe path; restart test passed. |
| AGNT-03 | Plans 01, 03 | MCP cache growth, reconnect behavior, and wrong-working-directory behavior are covered by tests or smoke scripts. | SATISFIED | Wrong-cwd protocol test/smoke, restart test, and cache-bound tests passed. |
| AGNT-04 | Plan 02 | Generated adapter guidance stays thin and points agents to dynamic ctxpack calls. | SATISFIED | Adapter tests enforce dynamic `prepare_task`, explicit `repo`, progressive `get_pack`, size bounds, and no static-dump phrases. |

No orphaned Phase 4 requirements found in `.planning/REQUIREMENTS.md`.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `crates/ctxpack/tests/cli_compat.rs` | 903 | Empty match arm | Info | Intentional cleanup/error-handling branch, not a stub. |
| `crates/ctxpack-core/src/init.rs` | 335-336 | Empty match arms | Info | Intentional ignore of remove-file success/not-found in test cleanup, not product behavior. |

No blocker placeholder/TODO/static-return implementation patterns were found in the phase files scanned.

### Human Verification Required

None. The formerly missing real-client proof is machine-checkable and now passes in required mode on this machine.

### Gaps Summary

No gaps remain. The previous blocker was closed by making the Codex smoke allow non-interactive MCP tool calls and making the Claude smoke use the normal authenticated Claude Code path. Phase 4 now has deterministic protocol coverage, explicit-repo wrong-cwd coverage, session-scoped pack resource diagnostics, bounded cache tests, thin dynamic adapter guidance, and required-mode real-client proof for both Codex CLI and Claude Code.

---

_Verified: 2026-05-13T17:04:33Z_  
_Verifier: Claude (gsd-verifier)_
