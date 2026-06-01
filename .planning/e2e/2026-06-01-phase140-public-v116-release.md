# Phase 140 E2E: Public v1.1.6 Release Currentness

## Goal

Publish the ContextMason branding product change as a current public `ctxpack`
release, keeping the archive and Homebrew install paths current with `main`.

## Release

- Tag: `v1.1.6`
- URL: `https://github.com/thromel/ctxpack/releases/tag/v1.1.6`
- Target commit: `d1a602c6fbce9e69c2fd2e80e8e2b98a7a5dc8f6`
- Archive: `ctxpack-v1.1.6-aarch64-apple-darwin.tar.gz`
- Archive SHA-256: `7805f240956a2d7fdb3f26d04d8858692526242c597f75748421964fd9600b84`
- Binary SHA-256: `2d4d954836326fd24a36c999cdeea21e01218812715b64dedada87599212b146`
- Public CI: `https://github.com/thromel/ctxpack/actions/runs/26749403843`

## Homebrew Tap

- Tap repository: `https://github.com/thromel/homebrew-tap`
- Tap commit: `3c05f5e`
- Formula: `thromel/tap/ctxpack`
- Formula URL: `https://github.com/thromel/ctxpack/releases/download/v1.1.6/ctxpack-v1.1.6-aarch64-apple-darwin.tar.gz`
- Formula SHA-256: `7805f240956a2d7fdb3f26d04d8858692526242c597f75748421964fd9600b84`
- Platform: Apple Silicon macOS (`depends_on arch: :arm64`)

## Proof

Durable source-free proof artifacts:

- `.ctxpack/e2e/phase140-github-release.json`
- `.ctxpack/e2e/phase140-github-release-verify.json`
- `.ctxpack/e2e/phase140-public-release-freshness.json`
- `.ctxpack/e2e/phase140-public-archive-install.json`
- `.ctxpack/e2e/phase140-homebrew-tap-proof.json`
- `.ctxpack/e2e/phase140-public-real-client-smoke.json`

Commands run:

```bash
cargo test --workspace --locked
bash scripts/check-release-docs.sh
cargo run -p ctxpack --locked -- --help
bash scripts/release-package.sh
bash scripts/verify-release-archive.sh \
  --archive dist/ctxpack-v1.1.6-aarch64-apple-darwin.tar.gz \
  --manifest dist/ctxpack-v1.1.6-aarch64-apple-darwin.manifest.json \
  --checksums dist/sha256sums.txt
bash scripts/verify-github-release.sh \
  --repo thromel/ctxpack \
  --tag v1.1.6 \
  --target d1a602c6fbce9e69c2fd2e80e8e2b98a7a5dc8f6
bash scripts/check-public-release-freshness.sh \
  --repo thromel/ctxpack \
  --tag v1.1.6 \
  --require-product-current
bash scripts/verify-public-archive-install.sh \
  --repo thromel/ctxpack \
  --tag v1.1.6 \
  --target-label aarch64-apple-darwin \
  --expected-version "ctxpack 1.1.6"
bash scripts/verify-homebrew-tap.sh \
  --tap thromel/tap \
  --formula ctxpack \
  --expected-version "ctxpack 1.1.6" \
  --expected-url https://github.com/thromel/ctxpack/releases/download/v1.1.6/ctxpack-v1.1.6-aarch64-apple-darwin.tar.gz \
  --expected-sha256 7805f240956a2d7fdb3f26d04d8858692526242c597f75748421964fd9600b84
bash scripts/smoke-public-real-clients.sh \
  --repo thromel/ctxpack \
  --tag v1.1.6 \
  --target-label aarch64-apple-darwin \
  --expected-version "ctxpack 1.1.6"
```

All required checks passed.

## Client Evidence

- Claude Code `2.1.159` passed explicit-repo `prepare_task` and `get_pack`
  evidence against the public v1.1.6 archive binary.
- Codex CLI `0.44.0` remained an optional source-free skip because it still did
  not produce machine-checkable `prepare_task` / `get_pack` evidence.

## Non-Goals

No crates.io package, signed installer, self-update path, cloud indexing, cloud
embedding, hosted service, global agent config mutation, or additional platform
archive was added.
