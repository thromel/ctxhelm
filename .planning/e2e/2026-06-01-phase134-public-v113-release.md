# Phase 134 E2E: Public v1.1.3 Release Currentness

## Goal

Make the public downloadable archive current with the product-facing README and
agent evidence updates from Phase 133.

## Product Commit

```text
f17bd4cb27f1989e696717ac706868808ff01151
```

## Published Release

```text
https://github.com/thromel/ctxpack/releases/tag/v1.1.3
```

Uploaded assets:

- `ctxpack-v1.1.3-aarch64-apple-darwin.tar.gz`
- `ctxpack-v1.1.3-aarch64-apple-darwin.manifest.json`
- `ctxpack-v1.1.3-aarch64-apple-darwin.audit.json`
- `ctxpack-v1.1.3-aarch64-apple-darwin.tar.gz.sha256`
- `sha256sums.txt`

## Source-Free Proof Artifacts

```text
.ctxpack/e2e/phase134-github-release.json
.ctxpack/e2e/phase134-github-release-verify.json
.ctxpack/e2e/phase134-public-release-freshness.json
.ctxpack/e2e/phase134-public-archive-install.json
.ctxpack/e2e/phase134-public-real-client-smoke.json
```

## Verification

Passed:

```bash
bash scripts/verify-github-release.sh \
  --repo thromel/ctxpack \
  --tag v1.1.3 \
  --target f17bd4cb27f1989e696717ac706868808ff01151 \
  --assets-dir dist

bash scripts/check-public-release-freshness.sh \
  --repo thromel/ctxpack \
  --tag v1.1.3 \
  --require-product-current \
  --output .ctxpack/e2e/phase134-public-release-freshness.json

bash scripts/verify-public-archive-install.sh \
  --repo thromel/ctxpack \
  --tag v1.1.3 \
  --expected-version "ctxpack 1.1.3" \
  --output .ctxpack/e2e/phase134-public-archive-install.json

CTXPACK_RUN_REAL_CLIENT=1 bash scripts/smoke-public-real-clients.sh \
  --repo thromel/ctxpack \
  --tag v1.1.3 \
  --expected-version "ctxpack 1.1.3" \
  --smoke-repo "$PWD" \
  --output .ctxpack/e2e/phase134-public-real-client-smoke.json
```

Key results:

- GitHub release target: `f17bd4cb27f1989e696717ac706868808ff01151`
- Public freshness: `status = current`, `productStatus = current`,
  `commitsAhead = 0`, `productCommitsAhead = 0`
- Public install checks: checksum, archive verification, temporary install,
  version, help, doctor, and first-pack smoke all passed
- Claude Code `2.1.159`: passed explicit-repo `prepare_task` and `get_pack`
  evidence against `ctxpack 1.1.3`
- Codex CLI `0.44.0`: remains an optional source-free skip because it did not
  produce machine-checkable `prepare_task` / `get_pack` evidence

## Boundary

This phase refreshes the public archive channel only. Homebrew, crates.io,
signed installers, self-update, and cloud telemetry remain intentionally
deferred.
