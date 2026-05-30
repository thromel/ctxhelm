# Phase 95: Progressive Area Pack Guidance

## Goal

Make broad-task context packs more directly actionable without changing the
target-file budget or adding source snippets for every surfaced area.

Phase 94 increased broad `contextAreas` coverage and preserved the stable
top-10 target-file behavior. Phase 95 turns those source-free area hints into a
progressive reading path inside generated packs: agents now see which surfaced
areas had zero selected files and which representative paths to inspect next
with their native read tools.

## Implementation

- Updated the `Context areas` pack section to explain how agents should use
  area hints for progressive native reads.
- Added a `Zero-selected areas to inspect next` subsection for surfaced areas
  with candidate paths but no selected target/test files.
- Kept the section source-free: it lists paths and counts, not source text.
- Left target-file, test, validation, and snippet budgets unchanged.
- Extended the focused pack rendering test to verify zero-selected area
  guidance appears in markdown packs.

## Evidence

Focused test:

```bash
cargo test -p ctxpack-compiler compile_context_pack_renders_context_areas -- --nocapture
```

Broad proof:

```bash
cargo run --release -p ctxpack -- eval proof \
  --config .planning/e2e/2026-05-31-phase92-area-aware-gap-proof-config.json \
  --format json > /tmp/ctxpack-phase95-progressive-area-pack-proof.json

python3 scripts/check-product-proof.py \
  .ctxpack/e2e/phase95-progressive-area-pack-proof.json
```

Committed proof:

- `.ctxpack/e2e/phase95-progressive-area-pack-proof.json`

Result:

- `releaseGate.decision = promote`
- VeriSchema broad context-area recall remains `0.71851856`
- VeriSchema File Recall@10 remains `0.18449473`
- VeriSchema Source Recall@10 remains `0.31067252`
- VeriSchema Test Recall@10 remains `0.7089947`
- VeriSchema Effective Validation Recall@10 remains `1.0`
- VeriSchema protected target miss-rate remains `0.2857143`
- RefactoringMiner still promotes under the hard cold runtime ceiling at
  `4820ms`

## Notes

This phase improves agent usability rather than recall. It gives agents a
source-free, ordered way to continue investigation when a broad task cannot fit
all changed implementation areas into the first target-file list.
