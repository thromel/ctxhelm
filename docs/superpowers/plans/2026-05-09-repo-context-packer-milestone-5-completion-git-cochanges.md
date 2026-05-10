# Repo Context Packer Milestone 5 Completion: Git Co-Change Hints

**Goal:** Complete the roadmap milestone for "Test Mapper and Git Co-Change Hints" by adding local git history co-change retrieval.

The previous related-tests milestone covered validation mapping but intentionally left git co-change hints out of scope. This completion pass adds the missing local-history signal without remote PR access or persistent git databases.

## Scope

- Read local git history only.
- Accept one or more anchor paths.
- Parse commit-file changes from `git log --name-only`.
- Count files that changed in the same commits as the anchor paths.
- Filter hints to safe inventoried files.
- Return typed co-change hints with path, commit count, confidence, and sample commits.
- Wire `ctxpack co-changes <paths...> --repo <path>`.

## Out of Scope

- Remote PR or issue history.
- Persistent SQLite git history tables.
- Commit diff hunk indexing.
- Learned co-change weights.
- MCP exposure.

## Acceptance Checks

- A local temp git repo can produce co-change hints.
- Hints exclude sensitive/generated/ignored files.
- CLI emits JSON co-change hint objects.
- `cargo test --workspace --locked` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
