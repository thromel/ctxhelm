# Phase 86: Python Package Re-Export Graph Coverage

## Goal

Reduce parser/precision blind spots for Python repositories that expose modules
through package `__init__.py` files. VeriSchema's remaining source misses include
Python package areas such as `schema_agent/nlp`, `schema_agent/text2r`, and
`schema_agent/algorithms`; many of those packages use `from package import
Symbol` or package-level re-exports.

## Change

- Python dependency extraction now records imported submodule paths for
  `from module import member` in addition to the containing module.
- Import resolution now recognizes Python package files at
  `package/__init__.py`.
- Related dependency expansion now adds a bounded `python_reexport` edge when an
  anchor imports a package `__init__.py` that re-exports concrete modules.
- `python_reexport` edges are ranked with incoming anchor edges inside the
  related dependency report so they are not dropped behind unrelated callers
  before ranking sees them.

This is intentionally narrower than general depth-2 graph expansion. It only
follows Python package re-export edges that are already present in the safe
local dependency graph.

## Local Evidence

Focused tests:

```bash
cargo test -p ctxpack-index dependency_edges_resolve_python_from_imported_submodules -- --nocapture
cargo test -p ctxpack-index related_dependency_edges_expand_python_package_reexports -- --nocapture
cargo test -p ctxpack-index dependency_edges_resolve_safe_local_imports -- --nocapture
cargo test -p ctxpack-index dependency_edges_resolve_java_and_kotlin_package_imports -- --nocapture
```

A live VeriSchema dependency query now returns Python re-export edges from
`schema_agent/agents/entity_discovery.py` to:

- `schema_agent/nlp/dependency_parser.py`
- `schema_agent/nlp/entity_extractor.py`
- `schema_agent/nlp/model_cache.py`

and from `schema_agent/gates/gate_system.py` to:

- `schema_agent/nlp/dependency_parser.py`

## Broad Proof

Command:

```bash
cargo run -p ctxpack -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxpack-phase86-python-reexport-proof.json
```

The proof still blocks on the known cold runtime gate:

```text
Blocked because proof runtime exceeded 5000ms per commit for: RefactoringMiner, ctxpack.
```

Quality movement versus Phase 84 is flat and non-regressing:

| Corpus | File Recall@10 | Source Recall@10 | Test Recall@10 | Effective Validation Recall@10 | Protected Target Miss-Rate@10 |
| --- | --- | --- | --- | --- | --- |
| RefactoringMiner | `0.6 -> 0.6` | `1.0 -> 1.0` | `1.0 -> 1.0` | `1.0 -> 1.0` | `0.0 -> 0.0` |
| ctxpack | `0.44603175 -> 0.44603175` | `0.6333333 -> 0.6333333` | `0.0 -> 0.0` | `0.0 -> 0.0` | `0.0 -> 0.0` |
| ReAgent | `0.5 -> 0.5` | `1.0 -> 1.0` | `1.0 -> 1.0` | `1.0 -> 1.0` | `0.0 -> 0.0` |
| VeriSchema | `0.17936651 -> 0.17936651` | `0.30409357 -> 0.30409357` | `0.7089947 -> 0.7089947` | `1.0 -> 1.0` | `0.2857143 -> 0.2857143` |

## Outcome

Phase 86 improves Python graph correctness and exposes real package re-export
edges, but it does not yet improve the top-10 retrieval proof. The remaining
VeriSchema gap is now more clearly a selection/budget/diversity problem: the
extra graph candidates exist, but do not displace higher-ranked context in the
standard pack.

## Next Work

- Add a task-aware area/diversity selector only if it can move VeriSchema source
  recall without increasing protected target misses.
- Continue raw related-test recall improvements for broad Python eval tasks.
- Keep general depth-2 graph expansion rejected unless a bounded variant proves
  non-regressing behavior.
