# Phase 115 - Public Archive Install Verification

Date: 2026-06-01

## Goal

Prove that the documented user install path works from the public `v1.1.0`
GitHub release, not only from local release artifacts.

## Verification Command

```bash
bash scripts/verify-public-archive-install.sh \
  --repo thromel/ctxhelm \
  --tag v1.1.0 \
  --target-label aarch64-apple-darwin \
  --expected-version "ctxhelm 1.1.0" \
  --output .ctxhelm/e2e/phase115-public-archive-install.json
```

## Result

The verifier downloaded the public release assets from:

```text
https://github.com/thromel/ctxhelm/releases/download/v1.1.0
```

It verified:

- release checksums
- archive clean extraction
- temporary-bin install
- `ctxhelm --version`
- `ctxhelm --help`
- `ctxhelm doctor --release-manifest ...`
- first-pack smoke through the installed binary

Durable source-free evidence:

```text
.ctxhelm/e2e/phase115-public-archive-install.json
```

Key proof values:

- release URL: `https://github.com/thromel/ctxhelm/releases/tag/v1.1.0`
- archive: `ctxhelm-v1.1.0-aarch64-apple-darwin.tar.gz`
- archive SHA-256: `81f5ecd6d944d13ec70141b55a110cc808f584fc0e7b64a0ba087eda5e18f664`
- binary SHA-256: `92700827037f34b72e24fde627dd8b9f6506037cd0bf2a6e11dc66b3ac9887ee`
- version: `ctxhelm 1.1.0`

## Boundary

This proof installs only into a temporary bin directory. It does not install
globally, mutate shell startup files, mutate agent configuration, publish, tag,
upload, or run user project tests.
