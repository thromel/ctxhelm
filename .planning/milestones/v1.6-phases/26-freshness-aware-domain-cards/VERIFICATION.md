# Phase 26 Verification

Passed:

```bash
cargo test -p ctxhelm-compiler tests::generate_context_cards_writes_source_free_repo_cards
```

Also covered by the memory smoke:

```bash
CTXHELM_BIN="$PWD/target/debug/ctxhelm" bash scripts/smoke-memory.sh
```
