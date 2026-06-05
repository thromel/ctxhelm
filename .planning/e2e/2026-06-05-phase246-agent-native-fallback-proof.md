# Phase 246: Agent-Native Fallback Proof Gate

Date: 2026-06-05

## Purpose

Close the provable parts of the future agent-native integration work without
overclaiming real-client behavior for Cursor or OpenCode.

This phase adds a release-gated smoke that proves:

- `ctxhelm init --repo <repo> --cursor --claude --opencode` writes only
  repo-local guidance and adapter snippets.
- `ctxhelm setup-check --repo <repo> --cursor --claude --opencode` validates
  those generated artifacts without mutating global agent config.
- `ctxhelm cards fallback --repo <repo> --target-agent codex` writes
  source-free repo-local fallback cards and a disconnected agent guide.
- Generated guidance and fallback artifacts do not include source sentinel text
  or source snippets from the fixture repository.
- Generated guidance remains bounded and thin rather than becoming a broad
  static context injection path.

## Changed Artifacts

- `scripts/smoke-agent-native-fallback.sh`
- `scripts/release-gate.sh`
- `scripts/check-release-docs.sh`
- `docs/release.md`
- `crates/ctxhelm/tests/release_packaging.rs`
- `.planning/REQUIREMENTS.md`
- `.planning/MILESTONES.md`
- `.planning/STATE.md`

## Validation

Passed:

```text
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm bash scripts/smoke-agent-native-fallback.sh
bash -n scripts/smoke-agent-native-fallback.sh scripts/release-gate.sh
cargo test -p ctxhelm --test release_packaging --locked
bash scripts/check-release-docs.sh
```

## Requirement Impact

Closed:

- AGENT-03: User can install thin prompts/hooks/rules without broad static
  context injection.
- AGENT-04: User can use disconnected/cloud fallback cards when local MCP is
  unavailable.

Still open:

- AGENT-02 remains open. Cursor and OpenCode setup/protocol proof exists, but
  ctxhelm still does not claim machine-checkable Cursor/OpenCode real-client
  tool-call transcripts.
