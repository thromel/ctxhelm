# Phase 141 E2E: RepoWinnow Brand Identity

## Goal

Replace the short-lived ContextMason product name with a cleaner public brand
after availability review found the name too close to an adjacent Mason
MCP/code-context product.

## Decision

- Product name: RepoWinnow.
- Compatibility surface: keep `ctxpack` for CLI, packages, Homebrew formula,
  MCP namespace, and local state.
- Rejected names:
  - RepoLens: crowded across public web, package, and repository surfaces.
  - ContextMason: too close to adjacent Mason context tooling.
  - Bare Winnow: already used by LLM-context and AI tools.

## Validation

Commands run:

```bash
bash scripts/check-release-docs.sh
cargo run -p ctxpack --locked -- --help
cargo test --workspace --locked
```

Results:

- Release docs check passed.
- CLI help starts with `RepoWinnow: agent-native context packs for coding agents`.
- Full workspace test suite passed.

