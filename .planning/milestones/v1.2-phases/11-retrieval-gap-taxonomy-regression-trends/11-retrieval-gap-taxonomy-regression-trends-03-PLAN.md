---
phase: 11
plan: 3
title: Add benchmark run comparison and regression thresholds
status: complete
---

# Plan 3: Benchmark Comparison

## Goal

Compare two source-free benchmark JSON reports and flag retrieval regressions.

## Tasks

- Add typed comparison report contracts.
- Compare metric deltas for recall, token ROI, lift, skipped paths, and excluded paths.
- Compare gap family missed-count deltas.
- Add threshold checks with pass/fail booleans.

## Verification

- CLI compatibility test compares two benchmark reports and detects a configured recall regression.
