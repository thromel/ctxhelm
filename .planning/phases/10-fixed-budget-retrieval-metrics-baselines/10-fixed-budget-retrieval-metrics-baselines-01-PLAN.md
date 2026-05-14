---
phase: 10
plan: 1
title: Add baseline runners and stable benchmark metric contracts
status: complete
---

# Plan 1: Baseline Metric Contracts

## Goal

Extend historical eval contracts with an explicit no-context baseline while preserving existing lexical and hybrid metrics.

## Tasks

- Add no-context ranking metrics to `EvalComparison`.
- Add lift-vs-no-context fields for recall, precision, and MRR.
- Keep lexical lift fields for backward-compatible interpretation.
- Preserve source-free JSON naming.

## Verification

- Compiler tests assert `noContextBaseline` and `recallLiftVsNoContextAtK` in public JSON.
