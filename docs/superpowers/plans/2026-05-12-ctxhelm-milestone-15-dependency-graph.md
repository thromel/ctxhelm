# Milestone 15: Safe Dependency Graph

## Goal

Move ctxhelm beyond MVP search/history/test mapping by adding a read-only graph-lite layer: safe local import edges that agents can use as structural context.

## Scope

- Extract local dependency edges from safe source and test files.
- Resolve only local repository targets that already pass safe inventory policy.
- Expose graph edges through `ctxhelm dependencies`.
- Expose graph edges through MCP `related` and `ctxhelm://repo/dependency-graph`.
- Feed dependency edges into `prepare_task` as compact graph evidence.

## Non-Goals

- No autonomous editing.
- No shell/test execution by ctxhelm.
- No cloud graph, embeddings, or reranking.
- No full compiler/LSP precision index yet.

## Verification

- Focused unit tests for edge extraction and related edge expansion.
- MCP tests for dependency resource and related expansion.
- Compiler tests for dependency evidence in context plans.
- Full workspace test suite.
- CLI help smoke after command changes.
