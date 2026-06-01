# Phase 136 E2E: Public v1.1.4 Release Currentness

## Goal

Publish the Phase 135 distribution-readiness gate through the public archive
install path and verify that the public release is current with `main`.

## Release

Release URL:

```text
https://github.com/thromel/ctxpack/releases/tag/v1.1.4
```

Target commit:

```text
186fbebc8a4e9131b09665809a426c021eb5f13b
```

Assets:

- `ctxpack-v1.1.4-aarch64-apple-darwin.tar.gz`
- `ctxpack-v1.1.4-aarch64-apple-darwin.manifest.json`
- `ctxpack-v1.1.4-aarch64-apple-darwin.audit.json`
- `ctxpack-v1.1.4-aarch64-apple-darwin.tar.gz.sha256`
- `sha256sums.txt`

## Proof Artifacts

- `.ctxpack/e2e/phase136-github-release.json`
- `.ctxpack/e2e/phase136-github-release-verify.json`
- `.ctxpack/e2e/phase136-public-release-freshness.json`
- `.ctxpack/e2e/phase136-public-archive-install.json`
- `.ctxpack/e2e/phase136-public-real-client-smoke.json`
- `.ctxpack/e2e/phase136-distribution-readiness.json`

## Verification

Commands run:

```bash
cargo fmt --all -- --check
bash scripts/check-release-docs.sh
CARGO_TARGET_DIR=/tmp/ctxpack-phase136-target cargo test -p ctxpack --test cli_compat --test release_packaging --locked -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-phase136-target cargo test --workspace --locked
CARGO_TARGET_DIR=/tmp/ctxpack-phase136-target cargo run -p ctxpack --locked -- --help
CTXPACK_DIST_DIR="$PWD/dist" bash scripts/smoke-distribution-metadata.sh
bash scripts/release-package.sh
bash scripts/verify-release-archive.sh --archive dist/ctxpack-v1.1.4-aarch64-apple-darwin.tar.gz --manifest dist/ctxpack-v1.1.4-aarch64-apple-darwin.manifest.json --checksums dist/sha256sums.txt
bash scripts/verify-github-release.sh --repo thromel/ctxpack --tag v1.1.4 --target 186fbebc8a4e9131b09665809a426c021eb5f13b --assets-dir dist
bash scripts/check-public-release-freshness.sh --repo thromel/ctxpack --tag v1.1.4 --require-product-current
bash scripts/verify-public-archive-install.sh --repo thromel/ctxpack --tag v1.1.4 --target-label aarch64-apple-darwin --expected-version "ctxpack 1.1.4"
bash scripts/smoke-public-real-clients.sh --repo thromel/ctxpack --tag v1.1.4 --target-label aarch64-apple-darwin --expected-version "ctxpack 1.1.4"
```

All passed.

## Results

- Public freshness: `status = current`, `productStatus = current`, `commitsAhead = 0`.
- Public archive install: checksum/archive/version/help/doctor/first-pack checks passed.
- Distribution readiness: Homebrew formula render passed from the exact
  `v1.1.4` archive digest; crates package boundary check passed; Homebrew and
  crates.io publication remain deferred.
- Claude Code `2.1.159`: passed with explicit-repo `prepare_task` and `get_pack`
  calls against `ctxpack 1.1.4`.
- Codex CLI `0.44.0`: optional source-free skip; it still does not produce
  machine-checkable `prepare_task`/`get_pack` evidence.

## Non-Goals

No Homebrew tap, crates.io package, signed installer, self-update path, global
install mutation, or global agent config mutation was added.
