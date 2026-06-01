# Phase 48 Plan: Distribution Metadata & Package Prep

## Tasks

1. Add future Homebrew and crates.io preparation metadata without claiming
   current package-manager publication.
2. Add machine-readable update metadata schema and example with local-only
   privacy posture, explicit self-update=false, and signedInstaller=false.
3. Add a clean-extraction release archive verifier that checks checksums,
   manifest consistency, extracted binary identity, `--version`, `--help`, and
   `ctxhelm doctor`.
4. Document the distribution boundary, clean extraction command, update
   metadata boundary, and signing/notarization gaps.
5. Wire distribution metadata smoke and archive verification into the release
   gate, docs checks, and release packaging tests.

## Verification

- `bash scripts/smoke-distribution-metadata.sh`
- `bash scripts/check-release-docs.sh`
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target-phase48 cargo test -p ctxhelm --test release_packaging -- --nocapture`
- `CTXHELM_ALLOW_DIRTY=1 CTXHELM_DIST_DIR=/tmp/ctxhelm-phase48-dist CARGO_TARGET_DIR=/tmp/ctxhelm-target-phase48-package bash scripts/release-package.sh`
- `bash scripts/verify-release-archive.sh --archive /tmp/ctxhelm-phase48-dist/ctxhelm-v1.1.0-aarch64-apple-darwin.tar.gz --manifest /tmp/ctxhelm-phase48-dist/ctxhelm-v1.1.0-aarch64-apple-darwin.manifest.json --checksums /tmp/ctxhelm-phase48-dist/sha256sums.txt`

