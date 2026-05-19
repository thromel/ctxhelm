---
phase: 59
title: Provider And Reranker Policy Gates
date: 2026-05-20
status: patterns
---

# Phase 59 Patterns

## Existing Patterns To Reuse

- Privacy status objects should stay close to existing semantic provider status and pack privacy fields.
- Configuration should follow existing repo policy/config loading instead of adding an isolated format.
- Compiler warnings are already structured in plan/pack reports; extend those rather than printing free-form warnings.
- Feature-gated providers should compile without optional dependencies by default.

## Closest Code Analogs

- `crates/ctxpack-compiler/src/policy.rs`: provider status and policy experiment reporting.
- `crates/ctxpack-index/src/semantic.rs`: semantic provider config and availability.
- `crates/ctxpack-core/src/contracts.rs`: report and policy-shaped contracts.
- `docs/policy-embedding.md`: current policy framing for embeddings.

## Implementation Notes

- Build policy evaluation first, then wire it into semantic/reranker execution.
- Keep reranker input source-free by default.
- Avoid adding new MCP tools; return policy state through existing tool/report outputs.
- Add explicit tests for denied cloud/default paths.
