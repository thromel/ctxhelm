# Repo Context Packer Milestone 6: Context Plan Compiler

**Goal:** Make `prepare_task` produce a useful task-conditioned `ContextPlan` instead of an empty plan.

This is the first compiler pass. It fuses existing local signals: lexical search, related tests, and git co-change hints. It does not yet emit full context packs or snippets.

## Scope

- Add a compiler API that accepts repo root, task text, and task type.
- Use lexical search to select likely target files.
- Use related-test mapping for selected source files.
- Use co-change hints as supporting evidence and risk context.
- Emit `ContextPlan` with target files, related tests, commands, pack options, risk flags, confidence, and privacy status.
- Wire `ctxpack prepare-task <task> --repo <path> --mode <mode>`.

## Out of Scope

- Full `ContextPack` rendering.
- Snippet materialization.
- MCP `prepare_task` repo-aware implementation.
- Tree-sitter symbols.
- Learned rank fusion.

## Acceptance Checks

- A temp repo task mentioning an identifier returns the source file as a target.
- Related tests and validation commands appear in the plan.
- Co-change hints appear as risk flags.
- Empty or low-signal tasks return an explicit missing-info question.
- `cargo test --workspace --locked` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
