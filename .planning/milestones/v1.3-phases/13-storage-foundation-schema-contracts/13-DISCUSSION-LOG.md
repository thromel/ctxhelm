# Phase 13: Storage Foundation & Schema Contracts - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-14
**Phase:** 13-Storage Foundation & Schema Contracts
**Areas discussed:** Store Location, Source-Free Boundary, Schema Breadth, Existing JSON Transition

---

## Store Location

| Option | Description | Selected |
|--------|-------------|----------|
| User-local default | Store SQLite under `CTXHELM_HOME` or `~/.ctxhelm`, matching current inventory and trace behavior. | ✓ |
| Repo-local default | Store SQLite under repo `.ctxhelm/`, easier to inspect but risks repo pollution and accidental sharing. | |
| Hybrid default | Mix user-local and repo-local behavior automatically. | |

**User's choice:** the agent selected the best option.
**Notes:** User-local default best preserves privacy and current behavior. Explicit store-path override remains allowed for tests and advanced workflows.

---

## Source-Free Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Strict source-free schema | No raw source, prompt text, snippet bodies, commit subjects, or secret-bearing columns. | ✓ |
| Pre-create source-bearing hooks | Add future tables/columns for optional snippet persistence now. | |
| Store snippets by default | Persist pack snippets for speed and later reuse. | |

**User's choice:** the agent selected the best option.
**Notes:** Strict source-free storage is the product trust contract. Future source-bearing persistence requires a separate opt-in design.

---

## Schema Breadth

| Option | Description | Selected |
|--------|-------------|----------|
| Broad schema skeleton | Create minimal future-ready records for repos, files, symbols, chunks, edges, tests, history, traces, packs, benchmarks, proof, and migrations. | ✓ |
| Inventory-only schema | Persist only inventory first, then add tables later. | |
| Fully wired schema | Create and wire every future consumer immediately. | |

**User's choice:** the agent selected the best option.
**Notes:** Broad skeleton reduces migration churn while avoiding Phase 13 overreach. Consumer wiring can expand in Phases 14 and 15.

---

## Existing JSON Transition

| Option | Description | Selected |
|--------|-------------|----------|
| Additive coexistence | Keep JSON/JSONL fallback, add SQLite path and idempotent import/sync. | ✓ |
| Immediate replacement | Replace `inventory.json` and `traces.jsonl` with SQLite now. | |
| Mandatory dual-write everywhere | Write JSON and SQLite on every touched path immediately. | |

**User's choice:** the agent selected the best option.
**Notes:** Additive coexistence avoids breaking current CLI/MCP behavior while the storage contract stabilizes.

---

## the agent's Discretion

- Planner may choose the SQLite Rust dependency.
- Planner may choose exact module layout and table normalization.
- Planner may decide which low-risk writes to introduce in Phase 13 versus deferring consumer rewiring to Phase 14.

## Deferred Ideas

- Full incremental indexing reuse.
- Storage-backed benchmark trend lookup.
- Repair/cleanup CLI commands and release-gate checks.
- Source-bearing persistence.
