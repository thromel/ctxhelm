---
phase: 9
plan: 1
title: Define benchmark suite configuration and source-free corpus contracts
status: complete
---

# Plan 1: Benchmark Suite Contracts

## Goal

Define typed contracts for named benchmark suites, per-repo configuration, effective run configuration, and source-free suite reports.

## Tasks

- Add suite and repository config structs.
- Add effective config structs with limit, ranking budget, mode, target agent, revision range, and role filters.
- Add suite and repository report structs with source-free metadata.
- Ensure privacy status is explicit and local-only.

## Verification

- Compiler unit test asserts multi-repo suite metadata and source-free JSON.
