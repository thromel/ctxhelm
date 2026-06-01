---
phase: 59
title: Provider And Reranker Policy Gates
date: 2026-05-20
status: research
requirements:
  - PROVIDER-01
  - PROVIDER-02
  - PROVIDER-03
  - PROVIDER-04
---

# Phase 59 Research: Provider And Reranker Policy Gates

## Objective

Introduce explicit policy gates for semantic providers and rerankers so quality backends can be added without weakening ctxhelm's trust boundary.

The default remains local-first and source-safe. Cloud embeddings, cloud reranking, and source-snippet transfer must stay off unless a repo policy explicitly allows them.

## Current State

Phase 56 already reports provider metadata:

- provider
- model
- dimensions
- distance metric
- provider role
- quality backend
- local only
- availability

The missing layer is policy enforcement. Today provider status can tell us what is available, but there is no complete repo policy model for:

- which providers are allowed
- which data classes can leave the machine
- whether source snippets are allowed
- whether reranking is allowed
- when a degraded or unapproved provider blocks or warns

## Problem

World-class retrieval will eventually need quality backends: local embeddings, local rerankers, optional cloud embeddings, optional cloud rerankers, and perhaps enterprise-hosted precision services. Without explicit policy gates, every improvement risks trust regressions.

Reranking is especially sensitive because rerankers often want richer candidate text than embeddings. For ctxhelm, the default reranker input must be source-free unless a policy allows source snippets.

## Design Direction

Add provider and reranker policy contracts:

- `ProviderPolicy`
- `AllowedProvider`
- `DataClassPolicy`
- `RerankerPolicy`
- `ProviderDecision`
- `ProviderPolicyReport`

Policy should be loaded from repo config when present, with safe defaults when absent:

- local lexical allowed
- local graph allowed
- local hash scaffold allowed but not quality semantic
- local embedding providers allowed only when installed/enabled
- cloud providers disabled
- source snippets remote disabled
- rerankers disabled unless explicitly enabled

## Reranker Abstraction

Add a first-stage reranker abstraction without making it part of the default runtime:

- deterministic local test reranker for fixtures
- local model backend placeholder/status if installed
- cloud backend blocked by default

The reranker should consume candidate summaries and safe semantic documents by default, not raw source.

## Runtime Behavior

Provider policy decisions should flow into:

- CLI status reports.
- `prepare_task` diagnostics and warnings.
- Evaluation reports.
- Pack privacy status.

The tool should be able to say: "semantic provider available but disabled by policy" or "reranker requested but blocked because source transfer is not allowed."

## Risks

- Policy complexity can confuse users; defaults must be simple and safe.
- Reranker code can accidentally become a hidden network path; tests must assert no cloud/default behavior.
- Too many warnings can pollute MCP outputs; warnings should be structured and concise.

## Acceptance Criteria

- Provider/reranker policy contracts exist and have safe defaults.
- Provider decisions are observable in CLI/MCP/eval reports.
- Reranker is disabled by default and policy-gated when requested.
- Tests prove cloud/source transfer is blocked by default.
- Docs explain how to opt in without implying opt-in is required.
