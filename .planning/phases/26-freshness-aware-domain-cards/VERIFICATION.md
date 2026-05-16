# Phase 26 Verification

Passed:

```bash
cargo test -p ctxpack-compiler tests::generate_context_cards_writes_source_free_repo_cards
```

Also covered by the memory smoke:

```bash
CTXPACK_BIN="$PWD/target/debug/ctxpack" bash scripts/smoke-memory.sh
```
