# Phase 142 - RepoWinnow Public v1.1.7 Release

## Goal

Publish the RepoWinnow-branded `ctxpack` v1.1.7 release so public install
surfaces match the renamed product.

## Release

- GitHub release: `https://github.com/thromel/ctxpack/releases/tag/v1.1.7`
- Target commit: `13d2b9536be23eda13fe56f2c01ac55ec7d79a36`
- Archive: `ctxpack-v1.1.7-aarch64-apple-darwin.tar.gz`
- Archive SHA-256: `827e2fa7278e0df659b82e3a778aa13a2ec6cfab8981f9156e7b9e82bb3f3b64`
- Homebrew tap commit: `da7dc6f`

## Verification

- `bash scripts/verify-release-archive.sh --archive dist/ctxpack-v1.1.7-aarch64-apple-darwin.tar.gz --manifest dist/ctxpack-v1.1.7-aarch64-apple-darwin.manifest.json --checksums dist/sha256sums.txt`
  - Result: passed.
- `bash scripts/verify-github-release.sh --repo thromel/ctxpack --tag v1.1.7 --target 13d2b9536be23eda13fe56f2c01ac55ec7d79a36 --assets-dir <v1.1.7-assets>`
  - Result: passed with `assetCount = 5`.
- `bash scripts/check-public-release-freshness.sh --repo thromel/ctxpack --tag v1.1.7 --require-product-current`
  - Result: `status = current`, `productStatus = current`, `commitsAhead = 0`.
- `bash scripts/verify-public-archive-install.sh --repo thromel/ctxpack --tag v1.1.7 --target-label aarch64-apple-darwin --expected-version "ctxpack 1.1.7"`
  - Result: passed and wrote `.ctxpack/e2e/phase142-public-archive-install.json`.
- `bash scripts/smoke-public-real-clients.sh --repo thromel/ctxpack --tag v1.1.7 --target-label aarch64-apple-darwin --expected-version "ctxpack 1.1.7"`
  - Result: passed and wrote `.ctxpack/e2e/phase142-public-real-client-smoke.json`.
- `bash scripts/verify-homebrew-tap.sh --tap thromel/tap --formula ctxpack --expected-version "ctxpack 1.1.7" --expected-url https://github.com/thromel/ctxpack/releases/download/v1.1.7/ctxpack-v1.1.7-aarch64-apple-darwin.tar.gz --expected-sha256 827e2fa7278e0df659b82e3a778aa13a2ec6cfab8981f9156e7b9e82bb3f3b64`
  - Result: passed and wrote `.ctxpack/e2e/phase142-homebrew-tap.json`.

## Notes

`ctxpack` remains the CLI, package, Homebrew formula, MCP namespace, and local
state compatibility surface. RepoWinnow is the product name.
