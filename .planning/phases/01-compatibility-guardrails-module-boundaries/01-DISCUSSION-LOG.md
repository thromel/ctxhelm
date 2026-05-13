# Phase 1: Compatibility Guardrails & Module Boundaries - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-05-13
**Phase:** 01-compatibility-guardrails-module-boundaries
**Areas discussed:** Compatibility strategy, CLI guardrails, MCP guardrails, Module boundaries

---

## Compatibility Strategy

The structured question UI was unavailable in this Codex mode, so the workflow fallback selected the recommended conservative default: characterize public behavior before refactoring.

| Option | Description | Selected |
|--------|-------------|----------|
| Characterize first | Add contract, CLI, and MCP compatibility tests before splitting modules. | yes |
| Split first | Extract modules immediately and rely on the existing unit suite. | |
| Redesign contracts | Use this phase to change public JSON and protocol shapes. | |

**User's choice:** Fallback selected recommended default.
**Notes:** Phase 1 exists to protect behavior. Contract redesign and product behavior changes are out of scope.

---

## CLI Guardrails

| Option | Description | Selected |
|--------|-------------|----------|
| Binary-level smoke coverage | Exercise representative commands through the compiled binary with temp repos and temp `CTXPACK_HOME`. | yes |
| Renderer/helper-only tests | Continue testing only internal render helpers and lower-layer functions. | |
| Full shell-script E2E suite | Build a broad shell-driven suite for every command immediately. | |

**User's choice:** Fallback selected recommended default.
**Notes:** The selected approach fills the current gap without turning Phase 1 into packaging or CI-infrastructure work.

---

## MCP Guardrails

| Option | Description | Selected |
|--------|-------------|----------|
| Preserve current small surface | Characterize current tools, resources, prompts, `content[0].text`, and `structuredContent`. | yes |
| Add new MCP tools | Expand capability during compatibility work. | |
| Fix session durability now | Solve process-local pack resource behavior in this phase. | |

**User's choice:** Fallback selected recommended default.
**Notes:** Session durability is important but mapped to Phase 4. Phase 1 should only lock current semantics so later changes are deliberate.

---

## Module Boundaries

| Option | Description | Selected |
|--------|-------------|----------|
| Split behind stable facades | Move large-file internals into private modules while preserving crate-root public APIs. | yes |
| Public API redesign | Move and rename public functions as part of cleanup. | |
| No module split | Add tests only and leave the large files untouched. | |

**User's choice:** Fallback selected recommended default.
**Notes:** The selected approach directly supports `CONT-04` while protecting `CONT-02`.

---

## the agent's Discretion

- Exact test crate layout and binary-test harness are left to the planner.
- Exact module names are left to the planner, constrained by current crate responsibilities and existing codebase map guidance.
- Exact number of golden fixtures is left to the planner, with a preference for stable structured assertions.

## Deferred Ideas

- Freshness/privacy/diagnostics work belongs to Phase 2.
- Retrieval fusion/eval lift/parser precision work belongs to Phase 3.
- Real client durability and reconnect semantics belong to Phase 4.
