# Phase 131: Product-Aware Freshness Release

## Purpose

Phase 131 fixes a release-governance edge case found while recording the
`v1.1.1` public proof: a proof-only commit after a release made the exact
freshness check report the public archive as stale, even when no
product-impacting code or documentation changed.

The phase adds product-aware freshness metadata, publishes `v1.1.2`, verifies
the public archive, and refreshes real Claude Code integration evidence against
the released binary.

## Implementation

- Extended `scripts/check-public-release-freshness.sh` with:
  - `productStatus`
  - `productCommitsAhead`
  - `proofOnlyCommitsAhead`
  - `ignoredFreshnessPaths`
  - `--require-product-current`
- Kept `--require-current` strict for exact commit matches.
- Added Rust release-packaging coverage for proof-only commits.
- Updated release, distribution, governance, quickstart, troubleshooting, and
  packaging docs for `v1.1.2`.

## Public Release

```text
https://github.com/thromel/ctxhelm/releases/tag/v1.1.2
```

Release target:

```text
ac6dc97f04cd18b5f2c6c32f7a1eca49e3ef5587
```

Archive:

```text
ctxhelm-v1.1.2-aarch64-apple-darwin.tar.gz
```

Archive SHA-256:

```text
1227a3e06b508c70688ac4a879520dacf0a2016a4d2dc58affdf9fd2b6406255
```

Binary SHA-256:

```text
e22a436b9c6c7b445aeb9d81c49904723c70ee70cc15cfff23e1935bd6a9b1df
```

## Evidence

Durable artifacts:

- `.ctxhelm/e2e/phase131-github-release.json`
- `.ctxhelm/e2e/phase131-github-release-verify.json`
- `.ctxhelm/e2e/phase131-public-release-freshness.json`
- `.ctxhelm/e2e/phase131-public-archive-install.json`
- `.ctxhelm/e2e/phase131-public-real-client-smoke.json`
- `.ctxhelm/e2e/phase131-public-real-client-smoke-evidence/`

Freshness result at release target:

```text
tag: v1.1.2
status: current
productStatus: current
releaseTargetCommit: ac6dc97f04cd18b5f2c6c32f7a1eca49e3ef5587
currentCommit: ac6dc97f04cd18b5f2c6c32f7a1eca49e3ef5587
gitRelation: same
commitsAhead: 0
productCommitsAhead: 0
proofOnlyCommitsAhead: 0
```

Public archive install result:

```text
version: ctxhelm 1.1.2
downloadedPublicAssets: true
checksumsVerified: true
archiveVerified: true
installedToTemporaryBin: true
versionPassed: true
helpPassed: true
doctorPassed: true
firstPackSmokePassed: true
```

Real-client result:

```text
Claude Code 2.1.158: passed
  prepare_task: true
  get_pack: true
  explicitRepoToolCallCount: 2
  deterministicProtocol: true
  deterministicContextAreaResourceRead: true

Codex CLI 0.44.0: skipped
  prepare_task: false
  get_pack: false
  explicitRepoToolCallCount: 0
  reason: optional client did not produce machine-checkable tool-call evidence
```

## Boundary

The public archive remains local-first and source-free. Verification did not
globally install ctxhelm, mutate agent configuration, run user project tests,
or upload source to remote embedding or reranking services.

Homebrew, crates.io, signed installers, self-update, stronger Codex CLI
machine-checkable proof, and deeper world-class retrieval research remain
future production-readiness work.
