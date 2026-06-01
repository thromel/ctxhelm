# Phase 124: Context Area Inspection Strategy

## Goal

Make source-free context-area MCP resources more directly actionable for coding
agents without changing target-file ranking, retrieval budgets, or source-text
logging.

## Change

`ctxhelm://repo/context-areas` and
`ctxhelm://repo/context-area/{encoded-area}` now expose an
`inspectionStrategy` object derived only from safe inventory metadata:

- `initialBatch`: the first next-read batch agents should inspect.
- `preferredOrder`: progressive batch order such as `primary`, `validation`,
  and `docs`.
- `pathBudget`: small suggested native-read budget for the area.
- `stopRule`: source-free stopping guidance so agents do not over-read broad
  areas.
- `sourceTextLogged: false`: explicit privacy marker.

This builds on Phase 123 coverage profiles. The coverage profile describes the
area shape; the inspection strategy converts that shape into a bounded native
read plan.

## Rejected Ranking Experiment

A shell-script role experiment was tested first by classifying `.sh`, `.bash`,
and `.zsh` paths as `Config` and bumping the parent-snapshot schema. It was
rejected and reverted before this phase because the scratch four-repo proof
blocked and VeriSchema regressed:

- VeriSchema File Recall@10 changed from `0.17423832` to `0.16914026`.
- VeriSchema Effective Validation Recall@10 changed from `1.0` to
  `0.85714287`.
- The release gate decision changed from `promote` to `block`.

The lesson is that script retrieval needs a validation-preserving selection
policy, not a broad role-classification change that spends the existing config
floor.

## Validation

Focused MCP resource contract:

```bash
cargo test -p ctxhelm-mcp resources_public_uri_shapes_are_stable -- --nocapture
```

Result: passed.

Pinned clean fixture proof:

```bash
cargo run -p ctxhelm -- eval proof --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json --format json
```

Result: `releaseGate.decision = promote`.

The proof preserves the promoted baseline: RefactoringMiner remains a lexical
ceiling match, while ctxhelm, ReAgent, and VeriSchema beat lexical on the
measured context channel and VeriSchema keeps Effective Validation Recall@10
at `1.0`.

## Production Notes

- Ranking output is intentionally unchanged.
- The MCP tool surface is unchanged.
- No cloud embedding, cloud reranking, or source transfer is introduced.
- The new fields are safe for agents to consume before native file reads.
