# Research: Features for v2.1 Pack Inspector & GraphRAG Retrieval

## Question

How should the v2.1 pack inspector and GraphRAG/embedding retrieval features
work from a user and agent perspective?

## Table Stakes

### Pack Inspector

- Open a recent or generated `ContextPack`.
- Show target files, related tests, validation commands, selected memory,
  warnings, privacy status, token estimate, budget, and target agent.
- Show why each file/snippet/test was selected: lexical, symbol, graph,
  semantic, history, test relation, active path, workspace, feedback, or memory.
- Show omitted candidates and budget trade-offs so users can see what did not
  fit.
- Keep the UI read-only: no source edits, no project test execution, no
  automatic commits.

### Retrieval Health

- Show benchmark trend summaries and recent historical eval results.
- Show repeated retrieval gap families by path, role, subsystem, and signal.
- Show token ROI and low-value chunks where existing reports support it.
- Show workspace routing quality when workspace plans are available.

### GraphRAG Retrieval

- Add graph communities or module clusters as source-free retrieval evidence.
- Add task-conditioned graph neighborhoods around selected candidates.
- Add feedback-weighted graph edges where prior sessions repeatedly read or
  edit files together.
- Keep graph expansion budgeted and non-recursive by default.

### Embedding Controls

- Show semantic provider status, dimensions, vector metadata count, and whether
  semantic retrieval was used.
- Keep local deterministic embeddings default.
- Make any cloud embedding/reranking provider explicit, policy-gated, and
  visible in privacy status.

## Differentiators

- A context decision debugger, not a repo chat UI.
- A graph view that explains why an agent was pointed at files and tests.
- Direct comparison of lexical, graph, semantic, feedback, memory, and workspace
  signals.
- Pack previews for Codex, Claude Code, Cursor, OpenCode, and generic MCP usage.

## Anti-Features

- No autonomous edits.
- No interactive code chat.
- No hidden cloud calls.
- No giant always-on UI process required for normal CLI/MCP use.
- No broad source viewer that bypasses safe inventory policy.

## Complexity Notes

The UI should start as an inspector over existing JSON contracts before adding
new retrieval behavior. GraphRAG improvements should start as source-free
reports and policy experiments before changing default pack ranking.

