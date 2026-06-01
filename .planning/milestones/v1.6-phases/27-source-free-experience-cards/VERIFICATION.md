# Phase 27 Verification

Passed:

```bash
CTXHELM_BIN="$PWD/target/debug/ctxhelm" bash scripts/smoke-memory.sh
```

The smoke creates a trace, generates an experience card, checks pending review,
and verifies the SQLite database excludes the source sentinel.
