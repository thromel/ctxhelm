---
phase: 11
plan: 1
title: Normalize source-free gap families across benchmark suites
status: complete
---

# Plan 1: Gap Family Taxonomy

## Goal

Group repeated retrieval misses into source-free families that can be compared across benchmark runs.

## Tasks

- Add `package`, `targetStatus`, and `recommendationArea` to retrieval gap summaries.
- Preserve role, signal gap, path family, missed count, and example paths.
- Keep all fields source-free.

## Verification

- Compiler tests assert the new gap taxonomy fields in public JSON.
