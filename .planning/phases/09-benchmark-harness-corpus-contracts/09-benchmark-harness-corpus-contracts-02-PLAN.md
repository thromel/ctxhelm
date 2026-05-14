---
phase: 9
plan: 2
title: Implement multi-repo bounded benchmark execution
status: complete
---

# Plan 2: Multi-Repo Runner

## Goal

Run existing historical evals over every repository listed in a benchmark suite.

## Tasks

- Load a JSON suite file.
- Resolve relative repo paths from the suite file directory.
- Apply defaults and per-repo overrides.
- Run `evaluate_historical_commits` for each repo.
- Preserve per-repo errors without aborting the whole suite.

## Verification

- Compiler unit test runs a suite over two temporary git repositories.
