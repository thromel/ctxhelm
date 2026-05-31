# Phase 114 - Public Archive Release

Date: 2026-06-01

## Goal

Close the gap between the archive-ready release candidate and public production
availability by publishing and verifying the v1.1.0 GitHub archive release.

## Release Gate

The full release gate was run from clean checkout
`68383cbfc2fff00c4f53fbd2b7bf90527ac4bd7e`:

```bash
CARGO_NET_OFFLINE=true \
CARGO_TARGET_DIR=/tmp/ctxpack-phase114-release-target \
CTXPACK_DIST_DIR=/tmp/ctxpack-v1.1.0-release-dist \
CTXPACK_PROOF_DIR=/tmp/ctxpack-phase114-release-proof \
CTXPACK_REQUIRE_CLEAN_FIXTURE_PROOF=1 \
bash scripts/release-gate.sh
```

Result:

- release gate: `passed`
- version: `ctxpack 1.1.0`
- binary source: `archive`
- archive: `ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz`
- archive SHA-256: `81f5ecd6d944d13ec70141b55a110cc808f584fc0e7b64a0ba087eda5e18f664`
- binary SHA-256: `92700827037f34b72e24fde627dd8b9f6506037cd0bf2a6e11dc66b3ac9887ee`
- clean cold fixture proof: `passed`
- clean cold fixture proof required: `true`
- required checks: `28`

## Public Release

Published release:

```text
https://github.com/thromel/ctxpack/releases/tag/v1.1.0
```

The release tag targets:

```text
68383cbfc2fff00c4f53fbd2b7bf90527ac4bd7e
```

Uploaded assets:

- `ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz`
- `ctxpack-v1.1.0-aarch64-apple-darwin.manifest.json`
- `ctxpack-v1.1.0-aarch64-apple-darwin.audit.json`
- `ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz.sha256`
- `sha256sums.txt`

## Verification

Verified the GitHub release tag, target commit, draft/prerelease status, and
asset SHA-256 digests with:

```bash
bash scripts/verify-github-release.sh \
  --tag v1.1.0 \
  --target 68383cbfc2fff00c4f53fbd2b7bf90527ac4bd7e \
  --assets-dir /tmp/ctxpack-v1.1.0-release-dist \
  --release-json .ctxpack/e2e/phase114-github-release.json
```

Verifier output:

```json
{"assetCount": 5, "sourceFree": true, "tag": "v1.1.0", "target": "68383cbfc2fff00c4f53fbd2b7bf90527ac4bd7e", "url": "https://github.com/thromel/ctxpack/releases/tag/v1.1.0"}
```

Durable source-free evidence:

- `.ctxpack/e2e/phase114-release-proof-summary.json`
- `.ctxpack/e2e/phase114-release-candidate-status.json`
- `.ctxpack/e2e/phase114-github-release.json`

## Boundary

This release publishes the local archive channel only. It does not claim
Homebrew, crates.io, signed installers, self-update, hosted sync, cloud
indexing, cloud embeddings, Cursor real-client proof, or OpenCode real-client
proof.
