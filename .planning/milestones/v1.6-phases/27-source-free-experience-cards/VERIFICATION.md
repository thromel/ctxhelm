# Phase 27 Verification

Passed:

```bash
CTXPACK_BIN="$PWD/target/debug/ctxpack" bash scripts/smoke-memory.sh
```

The smoke creates a trace, generates an experience card, checks pending review,
and verifies the SQLite database excludes the source sentinel.
