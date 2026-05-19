---
phase: 60
title: Semantic Precision Evaluation Gates And Release Proof
date: 2026-05-20
status: research
requirements:
  - GATE-01
  - GATE-02
  - GATE-03
  - GATE-04
---

# Phase 60 Research: Semantic Precision Evaluation Gates And Release Proof

## Objective

Convert semantic and precision work from "implemented feature" into release-proof evidence. The release should show whether precision-enriched semantic retrieval improves, holds neutral, or regresses retrieval quality on fixed corpora, with runtime and privacy status attached.

## Current State

The project already has:

- Historical eval options and paired baseline reports.
- Fixed corpus manifests from v2.3.
- Feature exports and product proof reports.
- Token ROI and signal saturation reports.
- Smoke scripts for semantic and precision surfaces.

The missing piece is a hard semantic/precision gate for the new v2.4 surfaces.

## Problem

Earlier E2E work showed that simply enabling semantic retrieval did not necessarily improve recall. That is acceptable only if the release proof is honest and diagnostic. The system should never claim quality lift because a backend exists.

Phase 60 must answer:

- Did enriched semantic documents improve retrieval?
- Did precision edges improve graph/semantic candidate quality?
- Did reranking improve precision without hurting anchors?
- What was the runtime cost?
- Was any source or remote provider used?
- Which named cases improved, regressed, or stayed unchanged?

## Design Direction

Add fixed-corpus variants:

- default hybrid
- lexical baseline
- lexical plus graph
- local semantic
- precision-enriched semantic
- semantic plus precision
- reranked when policy allows

Each report should include:

- Recall@K and MRR@K where gold paths are available.
- Test Recall@K.
- Candidate precision proxy.
- Token efficiency.
- Runtime and cache status.
- Provider/reranker policy decisions.
- Privacy status.
- Named wins, regressions, and misses.

## Release Gate

The gate should distinguish three outcomes:

- promote: passes thresholds and no critical regressions
- hold: neutral or mixed; feature remains opt-in or non-default
- block: material regression, unsafe privacy state, or missing required report

Default runtime behavior should only be promoted when evidence supports it.

## Risks

- Small fixture corpora can hide regressions; named-case reporting is required.
- External corpora such as RefactoringMiner can be expensive; keep them optional but supported.
- Runtime variability can make strict gates flaky; use thresholds with tolerance and include raw measurements.

## Acceptance Criteria

- v2.4 eval can run all semantic/precision variants from a fixed corpus manifest.
- Reports include quality, runtime, token, provider, precision, reranker, and privacy status.
- Release proof clearly states promote/hold/block.
- Docs explain what changed and what did not improve.
- Smoke scripts verify the full gate on local fixtures.
