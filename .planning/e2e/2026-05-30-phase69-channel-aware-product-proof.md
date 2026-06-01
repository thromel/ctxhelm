# Phase 69 E2E: Channel-Aware Product Proof Gate

Date: 2026-05-30

## Objective

Fix the product-proof gate so it reflects ctxhelm's two output channels:

- target context files for source/docs/config evidence
- related tests and commands for validation evidence

## Changes

- Product proof corpus verdicts now include `contextRecallAt10`, `lexicalContextRecallAt10`, and `contextDeltaAt10`.
- Default promotion compares non-test context recall against lexical and requires validation-test recall when test targets exist.
- Governance project docs are added as scoped candidates for eval/planning/release tasks.
- Root governance docs get a small selection floor only when `.planning` governance docs are present, avoiding generic README/doc displacement in non-ctxhelm repositories.
- Related dependency edge ordering now preserves expansion-seed order inside the same edge direction.

## Evidence

Command:

```bash
cargo run -p ctxhelm -- eval proof --config .ctxhelm/e2e/phase62-default-config.json --format json > .ctxhelm/e2e/phase69-channel-scoped-governance-proof.json
python3 scripts/check-product-proof.py .ctxhelm/e2e/phase69-channel-scoped-governance-proof.json
```

Release gate:

```json
{
  "decision": "promote",
  "defaultPromotionAllowed": true,
  "decisionReason": "Promote: every evaluated corpus beat lexical on non-test context recall and maintained validation-test recall under local-only proof thresholds."
}
```

Corpus verdicts:

| Corpus | Status | Context Recall@10 | Lexical Context Recall@10 | Context Delta | File Recall@10 | Lexical File Recall@10 | Test Recall@10 | Protected Miss Rate@10 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | beat | 0.7778 | 0.7407 | +0.0370 | 0.7392 | 0.7792 | 1.0000 | 0.0526 |
| ctxhelm | beat | 0.3333 | 0.2857 | +0.0476 | 0.3169 | 0.2773 | 1.0000 | 0.2200 |

## Decision

Default local retrieval is promoted for the fixed two-repo proof under the channel-aware gate. This is not a claim that all retrieval work is complete. Remaining work should focus on protected evidence pressure, parser/precision misses, broader repeated-lift corpora, and real-client outcome validation.
