# Phase 133 E2E: Product README Positioning

## Goal

Make the public entry point explain ctxhelm's product wedge before the command
tour, and keep current agent-proof claims from drifting behind local evidence.

## Changes

- Reframed the README as a local-first context compiler for existing coding
  agents.
- Added a `Why ctxhelm` section that distinguishes ctxhelm from native agent
  grep/search/read behavior.
- Added a current proof snapshot covering public `v1.1.2` install proof,
  four-repo protected target miss-rate, agent-evidence lexical comparison, and
  Claude Code workflow proof.
- Updated the client evidence boundary to Codex CLI `0.44.0` and Claude Code
  `2.1.159`.
- Updated `docs/agent-setup.md` so Codex is accurately described as optional
  source-free skip evidence while Claude Code has current workflow proof.
- Extended `scripts/check-release-docs.sh` and release packaging tests to gate
  the new public positioning strings and reject stale client-version claims in
  the public docs.

## Source-Free Proof

Durable proof:

```text
.ctxhelm/e2e/phase133-product-readme-positioning.json
```

Key facts:

- README now contains `Why ctxhelm`.
- README now contains `Current proof snapshot`.
- README and agent setup docs mention Claude Code `2.1.159`.
- README and agent setup docs mention Codex CLI `0.44.0`.
- Public release-doc checks reject stale README/agent-setup mentions of Claude
  Code `2.1.143`, Claude Code `2.1.158`, and Codex CLI `0.130.0`.

## Validation

Required validation:

```bash
bash scripts/check-release-docs.sh
cargo fmt --all -- --check
cargo test -p ctxhelm --test release_packaging release_docs_script_contract -- --nocapture
cargo test -p ctxhelm --test release_packaging release_docs_check_passes -- --nocapture
cargo run -p ctxhelm -- --help
git diff --check
```

## Boundary

This phase improves the public adoption surface and doc-gated truthfulness. It
does not publish a new archive, add Homebrew/crates.io distribution, or change
retrieval ranking.
