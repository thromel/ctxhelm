# Phase 142 - ctxhelm Public v1.1.7 Release

## Goal

Publish the ctxhelm-branded `ctxhelm` v1.1.7 release so public install
surfaces match the renamed product.

## Release

- GitHub release: `https://github.com/thromel/ctxhelm/releases/tag/v1.1.7`
- Target commit: `13d2b9536be23eda13fe56f2c01ac55ec7d79a36`
- Archive: `ctxhelm-v1.1.7-aarch64-apple-darwin.tar.gz`
- Archive SHA-256: `827e2fa7278e0df659b82e3a778aa13a2ec6cfab8981f9156e7b9e82bb3f3b64`
- Homebrew tap commit: `da7dc6f`

## Verification

- `bash scripts/verify-release-archive.sh --archive dist/ctxhelm-v1.1.7-aarch64-apple-darwin.tar.gz --manifest dist/ctxhelm-v1.1.7-aarch64-apple-darwin.manifest.json --checksums dist/sha256sums.txt`
  - Result: passed.
- `bash scripts/verify-github-release.sh --repo thromel/ctxhelm --tag v1.1.7 --target 13d2b9536be23eda13fe56f2c01ac55ec7d79a36 --assets-dir <v1.1.7-assets>`
  - Result: passed with `assetCount = 5`.
- `bash scripts/check-public-release-freshness.sh --repo thromel/ctxhelm --tag v1.1.7 --require-product-current`
  - Result: `status = current`, `productStatus = current`, `commitsAhead = 0`.
- `bash scripts/verify-public-archive-install.sh --repo thromel/ctxhelm --tag v1.1.7 --target-label aarch64-apple-darwin --expected-version "ctxhelm 1.1.7"`
  - Result: passed and wrote `.ctxhelm/e2e/phase142-public-archive-install.json`.
- `bash scripts/smoke-public-real-clients.sh --repo thromel/ctxhelm --tag v1.1.7 --target-label aarch64-apple-darwin --expected-version "ctxhelm 1.1.7"`
  - Result: passed and wrote `.ctxhelm/e2e/phase142-public-real-client-smoke.json`.
- `bash scripts/verify-homebrew-tap.sh --tap thromel/tap --formula ctxhelm --expected-version "ctxhelm 1.1.7" --expected-url https://github.com/thromel/ctxhelm/releases/download/v1.1.7/ctxhelm-v1.1.7-aarch64-apple-darwin.tar.gz --expected-sha256 827e2fa7278e0df659b82e3a778aa13a2ec6cfab8981f9156e7b9e82bb3f3b64`
  - Result: passed and wrote `.ctxhelm/e2e/phase142-homebrew-tap.json`.

## Notes

`ctxhelm` remains the CLI, package, Homebrew formula, MCP namespace, and local
state compatibility surface. ctxhelm is the product name.
