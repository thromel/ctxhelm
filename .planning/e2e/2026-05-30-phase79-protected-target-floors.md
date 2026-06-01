# Phase 79 Protected Target Floors

## Goal

Reduce protected retrieval-target misses without weakening the product-proof
gate. Protected exact lexical/symbol evidence is a production-quality signal:
when it also belongs to a historical retrieval target, it should not be
displaced by lower-value archive artifacts, scripts, or broad docs.

## Change

- Added a bounded lexical config floor so exact manifest/config evidence can
  survive script/doc-heavy tasks.
- Increased source lexical capacity by one slot at standard budget.
- Added a general source symbol reserve for no-active-context tasks.
- Treated `docs/agent-setup.md` as governance context.
- Deferred `.planning/e2e`, `.planning/phases`, and `.planning/milestones`
  archive artifacts during lexical/final fill, while keeping them searchable as
  fallback context.

The change keeps archive/history artifacts available, but gives fresh source,
config, and governance evidence first chance at the standard 10-file budget.

## Required Two-Repo Proof

Command:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .ctxhelm/e2e/v25-multirepo-baseline-config.json \
  --format json > /tmp/ctxhelm-phase79-required-proof.json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase79-required-proof.json
```

Committed artifact:

- `.ctxhelm/e2e/phase79-protected-target-floors-proof.json`

Result:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Effective validation recall | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `beat` | 0.778 | 0.741 | 1.000 | 0.000 |
| ctxhelm | `beat` | 0.541 | 0.351 | 0.000 | 0.042 |

Gate decision: `promote`.

RefactoringMiner protected retrieval-target misses drop to zero. The required
ctxhelm slice still has one protected source-symbol target below budget:
`crates/ctxhelm-compiler/src/planning.rs`.

## Broader Fixed-Corpus Proof

Command:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxhelm-phase79-broader-proof.json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase79-broader-proof.json
```

Committed artifact:

- `.ctxhelm/e2e/phase79-broader-protected-target-floors-proof.json`

Result:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Effective validation recall | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `match` | 1.000 | 1.000 | 1.000 | 0.000 |
| ctxhelm | `beat` | 0.389 | 0.306 | 0.000 | 0.100 |
| ReAgent | `beat` | 1.000 | 0.571 | 1.000 | 0.000 |
| VeriSchema | `beat` | 0.192 | 0.082 | 1.000 | 0.000 |

Gate decision: `promote`.

VeriSchema protected retrieval-target misses drop to zero. The broader ctxhelm
slice still has one protected source-symbol target below budget:
`crates/ctxhelm-index/src/related_tests.rs`.

## Validation

```bash
cargo fmt --check
cargo test -p ctxhelm-compiler selection_reserves_symbol_floor_when_ordinary_lexical_files_fill_budget -- --nocapture
cargo test -p ctxhelm-compiler selection_reserves_exact_config_when_scripts_and_docs_dominate -- --nocapture
cargo test -p ctxhelm-compiler selection_treats_agent_setup_as_governance_context -- --nocapture
cargo test -p ctxhelm-compiler selection_defers_archive_artifacts_in_final_fill -- --nocapture
cargo test -p ctxhelm-compiler selection_reserves_symbol_floor_when_archive_docs_fill_budget -- --nocapture
cargo test --workspace --no-fail-fast
bash scripts/check-release-docs.sh
git diff --check
cargo run -p ctxhelm -- --help
cargo run -p ctxhelm -- eval proof --config .ctxhelm/e2e/v25-multirepo-baseline-config.json --format json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase79-required-proof.json
cargo run -p ctxhelm -- eval proof --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json --format json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase79-broader-proof.json
```
