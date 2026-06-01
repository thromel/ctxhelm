# Phase 149 - Public v1.1.11 Currentness

## Goal

Publish and verify a current public `ctxhelm` release after Phase 148 real-client
diagnostic hardening, so the GitHub archive, Homebrew tap, and public
real-client evidence all match the latest product commit.

## Release

- Tag: `v1.1.11`
- Target commit: `9e23b997c4cf8985767c1194245ab6d44491b19e`
- Release URL: `https://github.com/thromel/ctxhelm/releases/tag/v1.1.11`
- Target: `aarch64-apple-darwin`
- Archive: `ctxhelm-v1.1.11-aarch64-apple-darwin.tar.gz`
- Archive SHA-256: `4c1b43d425ff2bd14491be2c120fcda30a0f8576474e75b0a0eec1fc70a8a7e8`
- Binary SHA-256: `21bd1b128652cbde26b700bebf3ffc16b892648d8650a5fc6879fb56721393f5`

## Verification

- `cargo fmt --check`
- `bash scripts/check-release-docs.sh`
- `cargo run -p ctxhelm --locked -- --help`
- `cargo test --workspace --locked`
- `bash scripts/release-package.sh`
- `bash scripts/verify-github-release.sh --repo thromel/ctxhelm --tag v1.1.11 --target 9e23b997c4cf8985767c1194245ab6d44491b19e --assets-dir dist`
- `bash scripts/check-public-release-freshness.sh --repo thromel/ctxhelm --tag v1.1.11 --require-product-current`
- `bash scripts/verify-public-archive-install.sh --repo thromel/ctxhelm --tag v1.1.11 --target-label aarch64-apple-darwin --expected-version "ctxhelm 1.1.11" --output .ctxhelm/e2e/phase149-public-v111-install.json`
- `bash scripts/smoke-public-real-clients.sh --repo thromel/ctxhelm --tag v1.1.11 --target-label aarch64-apple-darwin --expected-version "ctxhelm 1.1.11" --smoke-repo "$PWD" --output .ctxhelm/e2e/phase149-public-v111-real-clients.json`
- `bash scripts/verify-homebrew-tap.sh --tap thromel/tap --formula ctxhelm --expected-version "ctxhelm 1.1.11" --expected-url https://github.com/thromel/ctxhelm/releases/download/v1.1.11/ctxhelm-v1.1.11-aarch64-apple-darwin.tar.gz --expected-sha256 4c1b43d425ff2bd14491be2c120fcda30a0f8576474e75b0a0eec1fc70a8a7e8 --output .ctxhelm/e2e/phase149-public-v111-homebrew.json`

## Results

- Public release freshness: `current`, `commitsAhead = 0`, `productStatus = current`.
- Public archive install: checksum verification, archive audit, temporary install, version/help, `doctor`, and first-pack smoke all passed.
- Homebrew tap: formula syntax/style/audit, URL/SHA checks, install, formula test, and installed version all passed.
- Claude Code `2.1.159`: passed real-client MCP proof with explicit-repo `prepare_task` and `get_pack`; server-side request metadata records `tools/call = 2`.
- Codex CLI `0.44.0`: optional skip remains diagnostic and source-free; failure classified as `stream_disconnected` with stderr hash/line count and MCP method counts.

## Privacy

All committed proof artifacts are source-free. They record release metadata,
checksums, client versions, method counts, and privacy status only; raw prompts,
raw MCP traffic, stderr text, terminal logs, and source snippets are excluded.
