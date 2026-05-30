---
phase: 65
title: v2.5 Product Proof And Release Gate
date: 2026-05-30
status: patterns
---

# Phase 65 Patterns

## Proof Is A Gate, Not A Brochure

The product proof must be allowed to say no. If one required corpus trails
lexical or only matches lexical, default promotion is blocked.

## Aggregate Lift Is Not Enough

Average Recall@10 can hide corpus-level regressions. Release decisions should
inspect every required corpus and name the failing status.

## Keep Optional Proof Strict

The default release gate may skip external corpora, but when a benchmark config
is provided through `CTXPACK_BENCHMARK_CONFIG`, the gate should fail on
non-promote results. A skipped proof is not evidence of product quality.

## Do Not Hide Test Recall

Test recall is a separate product capability. A source-file lift with zero test
recall must remain visible in product proof notes.

## Prefer Honest Hold/Block Over Premature Promotion

Production-ready means the default can be defended from measured evidence. A
large targeted improvement can complete a phase while still blocking release.
