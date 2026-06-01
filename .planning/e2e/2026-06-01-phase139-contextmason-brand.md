# Phase 139 E2E: ContextMason Brand Identity

## Goal

Name the product clearly without breaking the released `ctxpack` CLI, package,
Homebrew formula, MCP namespace, or local state paths.

## Result

- Product name: ContextMason.
- CLI/package/install/MCP compatibility name: `ctxpack`.
- Descriptive category: local repo context compiler / context broker.
- Rejected name: RepoLens, because quick availability checks found public web,
  package, repository, and adjacent research collisions.

## Changes

- `README.md` now presents ContextMason as the product and `ctxpack` as the
  stable CLI.
- `docs/brand.md` records the naming model, message guardrails, and non-legal
  availability boundary.
- `docs/public-project-summary.md`, `docs/architecture.md`, `docs/release.md`,
  `AGENTS.md`, `CLAUDE.md`, and Cargo metadata use the new product framing.
- `scripts/check-release-docs.sh` now release-gates the brand document and
  public-facing ContextMason wording.

## Validation

```bash
bash scripts/check-release-docs.sh
cargo run -p ctxpack --locked -- --help
cargo test --workspace --locked
```

The first full workspace run exposed one transient
`parent_snapshot_batch_reader_extracts_multiple_paths` failure that passed when
rerun directly. The subsequent full `cargo test --workspace --locked` run
passed.

## Non-Goals

No binary rename, crate rename, MCP namespace rename, Homebrew formula rename,
repository rename, or local state migration was performed.
