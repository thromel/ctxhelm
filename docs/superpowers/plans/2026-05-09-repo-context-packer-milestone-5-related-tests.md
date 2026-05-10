# Repo Context Packer Milestone 5: Related Tests

**Goal:** Add the first validation-context layer by mapping source paths to likely test files and commands.

The context broker should help agents verify changes, not only find source files. This milestone adds heuristic, inventory-backed test mapping and local git co-change hints without coverage data, package-manager introspection, or remote PR access.

## Scope

- Load or build the safe inventory.
- Accept one or more source paths.
- Find likely tests by direct naming conventions.
- Boost tests that mention source file stems or exported-ish identifiers.
- Return typed related-test records with confidence, reason, and optional command.
- Wire `ctxpack related-tests <paths...> --repo <path>`.
- Return typed co-change hints from local git history.
- Wire `ctxpack co-changes <paths...> --repo <path>`.

## Out of Scope

- Coverage ingestion.
- Framework-specific command discovery from package manifests.
- MCP `related_tests` implementation.
- Full context compiler integration.

## Acceptance Checks

- `src/auth/session.ts` maps to `tests/auth/session.test.ts`.
- Tests that import or mention source identifiers are ranked.
- Ignored/sensitive/generated files are not considered.
- CLI emits JSON related-test objects with commands.
- CLI emits JSON co-change hint objects.
- `cargo test --workspace --locked` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
