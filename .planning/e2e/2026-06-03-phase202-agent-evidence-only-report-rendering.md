# Phase 202 - Agent Evidence Only Report Rendering

## Goal

Make the Phase 201 agent-evidence-only diagnostics visible in the normal
source-free markdown report path. Phase 201 added JSON fields, but the
historical eval report still only rendered aggregate agent-evidence recovery.

## Change

Historical eval markdown now renders:

- `Agent-evidence-only recovery`
- `Agent-evidence-only roles`
- `Top agent-evidence-only areas`

This keeps the report source-free and makes validation-only residual gaps
visible during normal proof review without requiring raw JSON inspection.

## Validation

Focused command:

```bash
cargo test -p ctxhelm historical_eval_report_renders_source_free_metrics \
  --locked -- --nocapture
```

Result: pass.

The focused test fixture now includes a representative test-only
agent-evidence-only gap and asserts that the markdown includes:

- `Agent-evidence-only recovery: 1 missed@10 path(s)`
- `Agent-evidence-only roles: test=1`
- `Top agent-evidence-only areas: tests/auth=1`

## Interpretation

This phase does not claim retrieval lift. It closes a product-observability gap:
the data added in Phase 201 is now visible where maintainers review benchmark
reports. The next R&D loop can target validation evidence consumption with the
same source-free report surface instead of requiring JSON inspection.
