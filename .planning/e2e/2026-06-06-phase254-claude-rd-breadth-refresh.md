# Phase 254 - Claude R&D Breadth Refresh

## Goal

Refresh the stale Claude Code R&D breadth-suite status with the current local
client instead of relying on the Phase 252 rate-limit artifact.

## Evidence

Source-free artifact:

- `.ctxhelm/e2e/phase254-agent-run-claude-rd-breadth-suite.json`

Run scope:

- Claude Code `2.1.163`
- ctxhelm `1.1.12`
- R&D breadth suite hash:
  `d04f86b3b8fb792a6d8dad7b493f728b1b78901a63a473dd004f2247b2b54afe`
- 4 tasks

Aggregate result:

| Metric | Value |
| --- | ---: |
| Status | `degraded` |
| Client preflights | `4` |
| Client preflight failures | `4` |
| Client preflight rate limits | `4` |
| Comparison-eligible tasks | `0` |
| Comparable ctxhelm lanes | `0` |
| Evidence miss observed | `false` |
| Under-read observed | `false` |
| Missing/invalid required ctxhelm calls | `false` |

Privacy remained local-only with no source text, raw prompts, raw transcripts,
raw MCP traffic, remote embeddings, or remote reranking.

## Interpretation

This is not retrieval-quality evidence. Claude Code is still unavailable for a
fresh paired outcome suite because every task preflight reports rate limiting.
The correct R&D action remains `retry_real_client_when_available`.
