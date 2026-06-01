# Phase 143 - Paired Agent-Run Outcome Harness

## Goal

Close the thin real-client outcome-proof gap by adding a source-free paired
Claude Code run that compares native repository exploration against
ctxpack-assisted exploration.

## Implementation

- Added `scripts/e2e-agent-run.sh`.
  - Runs three read-only Claude Code lanes: `baseline`, `ctxpack-plan`, and
    `ctxpack-brief`.
  - Keeps raw prompts, raw stream output, raw MCP traffic, source snippets,
    terminal logs, and project test output out of the persisted report.
  - Writes an honest skipped report unless `CTXPACK_RUN_REAL_CLIENT=1` is set.
- Added `ctxpack eval agent-run --report <path>` for Markdown/JSON rendering of
  the source-free report contract.
- Added focused tests for the CLI renderer and shell-script contract.
- Documented the paired run in `docs/feedback.md` and `docs/agent-setup.md`.

## Claude Code E2E

Command:

```bash
CTXPACK_RUN_REAL_CLIENT=1 CTXPACK_AGENT_RUN_TIMEOUT_SECONDS=150 \
  bash scripts/e2e-agent-run.sh \
  --repo . \
  --task "Identify the files relevant to improving the Claude workflow eval harness without editing files" \
  --target-file scripts/e2e-claude-workflow.sh \
  --target-file scripts/smoke-claude-mcp.sh \
  --output .ctxpack/e2e/phase143-agent-run-claude.json
```

Result:

- Report schema: `ctxpack-agent-run-eval-v1`
- Status: `passed`
- Client: Claude Code `2.1.159`
- Best lane: `ctxpack-brief`
- Target coverage delta: `0.00`
- Irrelevant read delta: `3`
- Outcome claim: `ctxpack_improved`
- Privacy: `sourceTextLogged = false`, `rawPromptStored = false`,
  `rawTranscriptStored = false`, `rawMcpTrafficStored = false`

Lane summary:

| Lane | Status | Target coverage | Read files | Irrelevant reads | Tool calls | ctxpack calls |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `baseline` | passed | 1.00 | 7 | 5 | 12 | 0 |
| `ctxpack-plan` | passed | 1.00 | 8 | 6 | 15 | 1 |
| `ctxpack-brief` | passed | 1.00 | 4 | 2 | 10 | 2 |

Observed ctxpack calls:

- `prepare_task` with explicit repo and task in both ctxpack lanes.
- `get_pack` with `budget = brief` and `format = json` in the brief lane.

## Notes

This is process evidence for one real Claude Code task, not a global claim that
ctxpack always beats native agent search. It does prove the current integration
can reduce irrelevant reads while preserving target-file coverage on a real
agent run, with a source-free persisted artifact.

