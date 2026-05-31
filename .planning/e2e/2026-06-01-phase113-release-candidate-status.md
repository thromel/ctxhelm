# Phase 113 - Release Candidate Status

Date: 2026-06-01

## Goal

Turn the clean Phase 112 release-gate proof into machine-checkable release
candidate metadata that records the archive/binary proof, clean fixture proof,
and distribution decisions.

## Change

- `scripts/release-candidate-status.sh create` now accepts `--proof-summary`.
- `ready` candidate creation fails without `--proof-summary`.
- `ready` candidates require a passed source-free release proof summary.
- `ready` candidates require `binaryIdentity.source = archive`.
- `ready` candidates require `cleanColdFixtureProductProof = passed`.
- `ready` candidates require `cleanColdFixtureRequired = true`.
- Candidate status metadata records archive checksum, binary checksum, proof
  summary checksum, required check count, clean fixture proof status, and the
  resource-backed gap-summary contract status.
- Candidate status metadata records v1.1.0 distribution decisions:
  - local archive: ready
  - Homebrew: deferred
  - crates.io: deferred
  - signed installer: deferred
  - self-update: not implemented

## Artifact

```text
.ctxpack/e2e/phase113-release-candidate-status.json
```

Generated with:

```bash
bash scripts/release-candidate-status.sh create \
  --output .ctxpack/e2e/phase113-release-candidate-status.json \
  --status ready \
  --proof-level deterministic \
  --proof-summary .ctxpack/e2e/phase112-clean-release-gate-summary.json
```

Validated with:

```bash
bash scripts/release-candidate-status.sh validate \
  --input .ctxpack/e2e/phase113-release-candidate-status.json
bash scripts/smoke-release-governance.sh
```

## Result

- `status = ready`
- `proofLevel = deterministic`
- `releaseProof.status = passed`
- `releaseProof.binarySource = archive`
- `releaseProof.cleanColdFixtureProductProof = passed`
- `releaseProof.cleanColdFixtureRequired = true`
- `distributionDecision.primaryChannel = local_archive`
- `distributionDecision.localArchive = ready`
- `distributionDecision.homebrewFormula = deferred`
- `distributionDecision.cratesIo = deferred`
- `distributionDecision.signedInstaller = deferred`
- `distributionDecision.selfUpdate = not_implemented`

## Boundary

This is a source-free readiness artifact, not publication. It does not tag,
publish, upload, install, mutate global agent configuration, or claim Homebrew,
crates.io, signed installers, self-update, Cursor real-client proof, or
OpenCode real-client proof.
