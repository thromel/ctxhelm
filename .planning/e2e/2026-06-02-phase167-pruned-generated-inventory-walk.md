# Phase 167: Pruned Generated Inventory Walk

Date: 2026-06-02

## Goal

Continue R&D after Phase 166 by attacking the shared large-repo setup cost.
Phase 166 made repeated `local_fastembed` query/candidate vectors reusable, but
RefactoringMiner still showed roughly 14-15s for lexical search and semantic
status because inventory freshness walked generated fixture trees.

## Findings

The clean RefactoringMiner fixture contains large generated test-resource
subtrees that ctxhelm already classifies as generated:

```text
36205 src/test/resources/oracle
 1677 src/test/resources/astDiff
  332 src/test/resources/mappings
```

Before this phase, ctxhelm still descended into those trees before excluding
their files. That made default evidence-safe workflows pay for tens of thousands
of files that could never enter the context pack.

## Changes

- Inventory schema bumped to v4 so old cache manifests rebuild under the faster
  traversal contract.
- Inventory and freshness walkers now prune known generated directories when
  `include_generated=false`.
- The walkers also prune `.git` and known sensitive directories when sensitive
  files are excluded.
- Generated files that are not under pruned directories are still classified and
  counted, so root-level generated sentinels such as lockfiles remain visible as
  generated exclusions.
- Freshness ignores newly created files inside pruned generated directories, so
  generated fixture churn does not force stale inventory rebuilds.

## RefactoringMiner Proof

Fixture:

```text
/Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean
```

Feature-enabled binary:

```bash
cargo build -p ctxhelm --features local-embeddings --locked
```

Lexical setup proof from a fresh `CTXHELM_HOME`:

```bash
rm -rf /tmp/ctxhelm-phase167-pruned-inventory-home
mkdir -p /tmp/ctxhelm-phase167-pruned-inventory-home

/usr/bin/time -p env \
  CTXHELM_HOME=/tmp/ctxhelm-phase167-pruned-inventory-home \
  target/debug/ctxhelm search "Improvement in TypeScriptVisitor" \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 5

/usr/bin/time -p env \
  CTXHELM_HOME=/tmp/ctxhelm-phase167-pruned-inventory-home \
  target/debug/ctxhelm search "Improvement in TypeScriptVisitor" \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 5
```

Observed:

```text
first lexical search with fresh v4 inventory: 3.70s
second lexical search with cache hit:        0.08s
previous Phase 166 lexical/status setup:     roughly 14-15s
```

Semantic status proof:

```bash
/usr/bin/time -p env \
  CTXHELM_HOME=/tmp/ctxhelm-phase167-pruned-inventory-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm semantic status \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic-provider local_fastembed \
  --semantic-model AllMiniLML6V2Q
```

Observed:

```text
real 0.10
```

Feature-enabled semantic proof from a fresh `CTXHELM_HOME`:

```bash
rm -rf /tmp/ctxhelm-phase167-semantic-home
mkdir -p /tmp/ctxhelm-phase167-semantic-home

/usr/bin/time -p env \
  CTXHELM_HOME=/tmp/ctxhelm-phase167-semantic-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm index \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic \
  --semantic-provider local_fastembed \
  --semantic-model AllMiniLML6V2Q
```

Observed:

```text
files: 647
generated excluded by default: 25
previous generated excluded count: 38242
semantic vector records: 16
real 5.18
previous Phase 166 semantic seed: 55.65s
```

Back-to-back semantic searches:

```bash
/usr/bin/time -p env \
  CTXHELM_HOME=/tmp/ctxhelm-phase167-semantic-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm search "Improvement in TypeScriptVisitor" \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic \
  --semantic-provider local_fastembed \
  --semantic-model AllMiniLML6V2Q \
  --limit 5
```

Observed:

```text
first semantic search:  0.98s
second semantic search: 0.11s
top result both times: src/main/java/gr/uom/java/xmi/decomposition/TypeScriptVisitor.java
semantic vector records after write-through: 31
semantic query vector records after write-through: 1
```

Comparison against Phase 166:

```text
Phase 166 semantic seed:                       55.65s
Phase 167 semantic seed:                        5.18s
Phase 166 second fresh-process semantic search: 12.08s
Phase 167 second fresh-process semantic search:  0.11s
```

## Interpretation

This phase removes the main measured latency blocker from the RefactoringMiner
semantic R&D loop. The win comes from avoiding irrelevant generated fixture
trees before freshness and inventory work, not from changing retrieval scoring.

Semantic retrieval should still remain opt-in until quality gates prove it adds
semantic-only target hits or better agent outcomes across corpora. The latency
path is now much closer to interactive use.

## Validation

Focused checks:

```bash
cargo test -p ctxhelm-index inventory_respects_ignores_and_default_exclusions --locked
cargo test -p ctxhelm-index freshness_ignores_created_files_inside_pruned_generated_dirs --locked
cargo test -p ctxhelm-index inventory_metadata_records_safe_file_manifest --locked
```

Final gates:

```bash
cargo fmt --check
cargo test -p ctxhelm-index --locked
bash scripts/check-release-docs.sh
cargo run -p ctxhelm --locked -- --help >/tmp/ctxhelm-phase167-help.txt
cargo test --workspace --locked --no-fail-fast
cargo clippy --workspace --locked --all-targets -- -D warnings
git diff --check
```

Result: all passed.
