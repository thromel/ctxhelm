# Phase 40 Plan: Local Web Pack Inspector

**Created:** 2026-05-18
**Status:** In Progress

## Scope

Implement UI-01 through UI-04:
- static/local inspector HTML from `PackInspectorView`
- dense diagnostic layout
- filters over paths, evidence/kinds, warnings, and source-bearing sections
- source-free smoke coverage

## Steps

1. Enhance `render_pack_inspector_html` with filters and complete inspector
   sections.
2. Add HTML renderer tests for UI hooks, responsive safeguards, and source
   sentinel separation.
3. Add `scripts/smoke-inspector.sh` for JSON/HTML temp-repo proof.
4. Update docs and release checks to include the inspector smoke.
5. Run formatting, smoke, CLI help, and workspace tests.

## Verification

- `cargo fmt --all`
- Focused compiler HTML test
- `bash scripts/smoke-inspector.sh`
- `bash scripts/check-release-docs.sh`
- `cargo run -p ctxpack -- inspector export --help`
- `cargo test --workspace`

## Non-goals

- No Tauri app.
- No persistent web server.
- No source snippet display in inspector artifacts.
- No GraphRAG visualization yet.
