# Phase 45 Context: Clean Release Gate & Proof Bundle

**Created:** 2026-05-18
**Status:** Complete

## Goal

Maintainers can produce a clean, source-free release candidate with a full proof
bundle.

## Requirements

- REL-01: Clean-checkout release gate verifies workspace tests, docs, release
  packaging, artifact audit, selected-binary smokes, MCP protocol proof,
  diagnostic smokes, and optional benchmark proof without publishing side
  effects.
- REL-02: Release packaging produces versioned archives, checksums, release
  manifest metadata, and artifact audit reports.
- REL-03: Artifact audit rejects local state, traces, caches, source-control
  internals, secrets, machine paths, and source-bearing diagnostic leaks.
- REL-04: Release gate generates a proof bundle recording commands, versions,
  binary identity, smoke outcomes, optional proof status, and privacy status.

## Decisions

- Keep release hardening local-first and source-free.
- Keep real-client proof optional and explicitly marked as skipped unless run.
- Keep Cursor/OpenCode real-client proof out of claimed v1.1 release evidence.
- Record file names and checksums in proof summaries, not machine-local binary
  or repository paths.
- Allow `CARGO_TARGET_DIR` for release packaging so a stale shared target cache
  does not block clean verification.

## Non-goals

- No publishing, tags, registry upload, Homebrew tap changes, signing, or
  notarization.
- No global agent config mutation.
- No cloud indexing, embedding, reranking, telemetry, or upload.
