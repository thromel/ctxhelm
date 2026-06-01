---
phase: 57
title: Precision-Enriched Semantic Documents
status: planned
requirements_addressed:
  - PREC-01
  - PREC-02
  - PREC-03
  - PREC-04
depends_on:
  - 56
---

# Phase 57 Plan: Precision-Enriched Semantic Documents

<objective>
Build a typed, source-free semantic document layer enriched with symbols, tests, docs/cards, dependency evidence, precision edges, and precision provider status. Wire it into retrieval evidence without changing ctxhelm's read-only or local-first defaults.
</objective>

<threat_model>
- Source leakage into semantic documents, cache records, diagnostics, or embedding text.
- Precision backend absence causing hard failures instead of degraded status.
- Enriched semantic evidence inflating context packs or displacing explicit anchors.
- Duplicate dependency and precision signals causing misleading candidate confidence.
</threat_model>

<must_haves>
- Semantic document contracts are shared through `ctxhelm-core`.
- Semantic documents are generated without reading or storing raw source bodies in output records.
- Precision status has explicit unavailable, available, stale, invalid, and degraded paths.
- Existing semantic provider status remains backward compatible.
- Planner evidence can cite semantic document facets and precision evidence.
- Tests cover source-free output and degraded precision behavior.
</must_haves>

<tasks>

<task id="57.1" name="Add semantic document and precision status contracts">
<read_first>
- `crates/ctxhelm-core/src/contracts.rs`
- `crates/ctxhelm-index/src/semantic.rs`
- `crates/ctxhelm-index/src/dependencies.rs`
</read_first>
<action>
Add serde contracts for semantic documents, semantic document facets, precision backend status, and document build reports. Keep existing semantic provider contracts compatible by adding optional fields rather than replacing current report shapes.
</action>
<verify>
- Add or update core serialization tests for the new contracts.
- Run `cargo test -p ctxhelm-core`.
</verify>
<acceptance_criteria>
- New contracts serialize in camelCase.
- Existing semantic provider status tests still pass.
- Contracts include source-free fields only: path, role, language, hash, symbol names/signatures, line ranges, relation labels, test paths, doc/card IDs, provider status, and reasons.
</acceptance_criteria>
</task>

<task id="57.2" name="Build source-free semantic documents in the index layer">
<read_first>
- `crates/ctxhelm-index/src/inventory.rs`
- `crates/ctxhelm-index/src/symbols.rs`
- `crates/ctxhelm-index/src/dependencies.rs`
- `crates/ctxhelm-index/src/tests.rs`
- `crates/ctxhelm-index/src/semantic.rs`
</read_first>
<action>
Implement a semantic document builder that joins inventory, safe symbols, dependency edges, related tests, docs/cards when available, and precision overlay edges. The builder should return a bounded report and deterministic document IDs.
</action>
<verify>
- Add index tests using a fixture with source files, tests, symbols, imports, and a `.ctxhelm/precision-edges.json` overlay.
- Assert that source body literals from fixtures do not appear in document output.
- Run `cargo test -p ctxhelm-index semantic`.
</verify>
<acceptance_criteria>
- Documents exist for eligible source/test/doc/config files.
- Symbol, test, dependency, and precision facets appear when present.
- Unsafe/ignored/sensitive paths are excluded.
- Missing precision overlay produces a non-fatal unavailable status.
</acceptance_criteria>
</task>

<task id="57.3" name="Use semantic documents for vector records and retrieval evidence">
<read_first>
- `crates/ctxhelm-index/src/semantic.rs`
- `crates/ctxhelm-compiler/src/planning.rs`
- `crates/ctxhelm-compiler/src/ranking.rs`
</read_first>
<action>
Update semantic vector record construction to use source-free semantic document text/facets instead of thin file metadata when documents are available. Extend compiler semantic evidence to include document facet IDs, precision relation labels, and provider status warnings.
</action>
<verify>
- Add compiler tests showing semantic candidates include facet-backed evidence.
- Add regression tests proving explicit anchors outrank weak semantic-only matches.
- Run `cargo test -p ctxhelm-compiler semantic ranking`.
</verify>
<acceptance_criteria>
- Semantic search remains optional and disabled unless requested by existing controls.
- Enriched semantic evidence improves explanations without expanding pack snippets by default.
- Candidate evidence identifies whether a match came from symbol, test, dependency, doc, or precision facets.
</acceptance_criteria>
</task>

<task id="57.4" name="Expose document and precision status through existing CLI/report surfaces">
<read_first>
- `crates/ctxhelm-cli/src/main.rs`
- `docs/semantic.md`
- `docs/precision.md`
- `scripts/smoke-semantic.sh`
- `scripts/smoke-precision.sh`
</read_first>
<action>
Add a bounded way to inspect semantic document reports and precision status through existing semantic/precision report surfaces. Update smoke scripts or add focused scripts to prove status and document generation.
</action>
<verify>
- Run `cargo run -p ctxhelm -- semantic status --repo . --format json` or the final equivalent command.
- Run `scripts/smoke-semantic.sh`.
- Run `scripts/smoke-precision.sh`.
</verify>
<acceptance_criteria>
- Users can inspect document count, facet count, precision status, provider status, and privacy status.
- Output remains source-free and bounded.
- No new MCP tool is required.
</acceptance_criteria>
</task>

<task id="57.5" name="Document Phase 57 behavior and update planning state">
<read_first>
- `docs/semantic.md`
- `docs/precision.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
</read_first>
<action>
Document source-free semantic documents, precision status behavior, and degraded fallback. Update planning artifacts after verification to reflect Phase 57 completion.
</action>
<verify>
- Run `cargo test --workspace`.
- Run `cargo run -p ctxhelm -- --help`.
- Check `git diff --check`.
</verify>
<acceptance_criteria>
- Docs clearly state that raw source bodies are not embedded or exported by default.
- Phase 57 is marked complete only after workspace tests and CLI help pass.
</acceptance_criteria>
</task>

</tasks>

<verification>
- `cargo test -p ctxhelm-core`
- `cargo test -p ctxhelm-index semantic`
- `cargo test -p ctxhelm-compiler semantic ranking`
- `scripts/smoke-semantic.sh`
- `scripts/smoke-precision.sh`
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
</verification>

<success_criteria>
- Source-free semantic documents are implemented and tested.
- Precision status is observable and non-fatal.
- Retrieval evidence can cite semantic document facets.
- Existing semantic, precision, and CLI behavior remain backward compatible.
</success_criteria>
