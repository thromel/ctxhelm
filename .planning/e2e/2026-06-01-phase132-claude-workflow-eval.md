# Phase 132 E2E: Claude Workflow Eval

## Goal

Add a stronger Claude Code integration proof than the protocol smoke alone:
prove that a real Claude Code client can call ctxhelm as an MCP context
workflow, while keeping committed evidence source-free.

## Changes

- Added `scripts/e2e-claude-workflow.sh`.
- The wrapper runs the deterministic Claude MCP smoke, optionally requests a
  real Claude Code run, and emits a source-free workflow report.
- The report records sanitized tool-call facts only:
  `prepare_task`, `get_pack`, explicit-repo call count, request-log hash,
  request-log line count, and privacy flags.
- The report does not persist raw prompts, raw MCP traffic, source text, local
  request-log paths, or user-project command output.
- `scripts/release-gate.sh` can now include this proof when
  `CTXHELM_RUN_CLAUDE_WORKFLOW_EVAL=1`, and can require it with
  `CTXHELM_REQUIRE_CLAUDE_WORKFLOW_EVAL=1`.
- Release docs and agent setup docs now distinguish this workflow eval from the
  lighter optional Claude smoke.

## Real-Client Result

Passed against Claude Code `2.1.159 (Claude Code)` and `ctxhelm 1.1.2` on a
temporary git fixture.

Committed source-free proof:

```text
.ctxhelm/e2e/phase132-claude-workflow-eval.json
```

Key report facts:

- `status`: `passed`
- `workflowKind`: `claude-code-mcp-context-workflow`
- `requestEvidence.explicitRepoToolCallCount`: `2`
- Observed calls: `prepare_task`, `get_pack`
- `workflowAssertions.deterministicProtocol`: `true`
- `workflowAssertions.realClientToolCalls`: `true`
- `workflowAssertions.requestLogHashOnly`: `true`
- `privacyStatus.rawMcpTrafficStored`: `false`
- `privacyStatus.rawPromptStored`: `false`
- `privacyStatus.sourceTextLogged`: `false`

## Validation

Passed:

```bash
CTXHELM_BIN="$PWD/target/debug/ctxhelm" \
  CTXHELM_SMOKE_REPO="$fixture" \
  CTXHELM_SMOKE_PATH="src/session.ts" \
  CTXHELM_SMOKE_QUERY="requireSession" \
  CTXHELM_SMOKE_TASK="fix requireSession test" \
  CTXHELM_CLAUDE_WORKFLOW_REPORT=/tmp/ctxhelm-phase132-claude-workflow-eval.json \
  CTXHELM_RUN_REAL_CLIENT=1 \
  CTXHELM_REAL_CLIENT_TIMEOUT_SECONDS=180 \
  bash scripts/e2e-claude-workflow.sh
```

Also passed in protocol-only mode with `CTXHELM_SKIP_REAL_CLIENT=1`.

## Boundary

This phase proves real Claude Code MCP workflow calls, not final patch quality
or user-project test execution. The harness intentionally does not let ctxhelm
edit files or run project commands.
