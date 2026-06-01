# Phase 130: Public v1.1.1 Release Currentness

## Purpose

Phase 130 closes the Phase 129 publication gap by publishing and verifying a
refreshed public `v1.1.1` archive release that points at the current
production-hardening commit.

This phase records durable, source-free proof that the public archive is no
longer behind `main`, can be installed from GitHub release assets, and still
works through the real Claude Code client path.

## Release

Public release:

```text
https://github.com/thromel/ctxpack/releases/tag/v1.1.1
```

Release target:

```text
6c93100fa0e4f5a5444fb7fd967c721cca49a401
```

Archive:

```text
ctxpack-v1.1.1-aarch64-apple-darwin.tar.gz
```

Archive SHA-256:

```text
8def6fd05eea21842e732e2c058f5a3147cf3cf33b85c69579785182aa7dd437
```

Binary SHA-256:

```text
745cf52d3cf4c1ac9b8611c8f9a33a86069ad151224cb82649e62da0263cccf7
```

## Evidence

Durable artifacts:

- `.ctxpack/e2e/phase130-github-release.json`
- `.ctxpack/e2e/phase130-public-release-freshness.json`
- `.ctxpack/e2e/phase130-public-archive-install.json`
- `.ctxpack/e2e/phase130-public-real-client-smoke.json`
- `.ctxpack/e2e/phase130-public-real-client-smoke-evidence/`

Freshness result:

```text
tag: v1.1.1
status: current
releaseTargetCommit: 6c93100fa0e4f5a5444fb7fd967c721cca49a401
currentCommit: 6c93100fa0e4f5a5444fb7fd967c721cca49a401
gitRelation: same
commitsAhead: 0
sourceFree: true
privacyStatus.sourceTextLogged: false
```

Public archive install result:

```text
version: ctxpack 1.1.1
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

The proof remains archive-first and source-free. The verification scripts did
not globally install ctxpack, mutate agent configuration, publish additional
assets, create tags, or run user project tests.

Homebrew, crates.io, signed installers, self-update, and required real-client
Codex proof remain deferred production-distribution work.
