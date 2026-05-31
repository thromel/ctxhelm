# Phase 119 E2E: Index Test Environment Lock

Date: 2026-06-01

## Objective

Remove an observed full-suite flake before treating the release gate as stable.
The failing test was:

- `ctxpack-index::storage::tests::json_inventory_and_trace_fallback_remain_intact`

The failure happened once during a full workspace run when
`inventory_report.inventory_path.exists()` was false, then passed when rerun in
isolation.

## Root Cause

Several `ctxpack-index` test modules mutate process-global `CTXPACK_HOME`.
Those modules each used their own local `OnceLock<Mutex<()>>`, so tests in
different modules could still run concurrently and overwrite or remove
`CTXPACK_HOME` while another module was writing inventory or trace artifacts.

## Fix

Added one crate-wide test environment lock:

- `crate::test_env_lock()`

Updated local `env_lock()` helpers in:

- `crates/ctxpack-index/src/lib.rs`
- `crates/ctxpack-index/src/freshness.rs`
- `crates/ctxpack-index/src/storage.rs`

## Evidence

Durable proof:

- `.ctxpack/e2e/phase119-index-env-lock-proof.json`

Stress validation:

```bash
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase119-target \
  cargo fmt --all -- --check

for run in 1 2 3; do
  CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase119-target \
    cargo test -p ctxpack-index --lib -- --nocapture
done
```

Result:

- 3 parallel `ctxpack-index` lib test runs passed.
- The previously observed fallback test passed in each run.

## Boundary

- This changes test synchronization only.
- It does not alter runtime indexing, storage, retrieval, or MCP behavior.
- It removes nondeterminism from release validation evidence.
