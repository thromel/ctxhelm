---
phase: 12
plan: 3
title: Add optional benchmark smoke to release/adoption gates
status: complete
---

# Plan 3: Optional Gate

## Goal

Let maintainers include product proof generation in release/adoption verification without making external repos mandatory.

## Tasks

- Add `CTXHELM_BENCHMARK_CONFIG` support to `scripts/release-gate.sh`.
- Run selected binary `ctxhelm eval proof --config ... --format json`.
- Fail on report-generation errors, missing headline metrics, or non-local-only privacy status.
- Keep the gate skipped by default.

## Verification

- Release packaging test asserts the release gate contract mentions the optional benchmark proof path.
