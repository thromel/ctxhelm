# Distribution Metadata

ctxpack v1.1.0 ships through local release archives. This document records the
preparatory distribution metadata for future package-manager channels without
making those channels blockers for the current release.

## Package Templates

- `packaging/homebrew/ctxpack.rb.template` is a Homebrew formula template, not a
  published tap formula.
- `packaging/crates/README.md` records crates.io preparation checks, not a
  registry publication claim.
- `packaging/release/update-metadata.schema.json` and
  `packaging/release/update-metadata.example.json` define machine-readable
  release metadata for future update checks.

The metadata is local-only and source-free. It does not include raw source,
prompts, secrets, terminal logs, or machine-local paths.

## Clean Extraction Verification

Verify an already-built archive from a clean temporary extraction directory:

```bash
bash scripts/verify-release-archive.sh \
  --archive dist/ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz \
  --manifest dist/ctxpack-v1.1.0-aarch64-apple-darwin.manifest.json \
  --checksums dist/sha256sums.txt
```

The verifier checks checksum consistency, extracts the archive into a temporary
directory, runs `ctxpack --version`, runs `ctxpack --help`, and runs
`ctxpack doctor` against the extracted binary and release manifest. It does not install binaries, mutate global agent configuration, publish artifacts, or run user project tests.

## Update Metadata Boundary

The release update metadata is not a self-update implementation. It only gives
future tooling a stable source-free shape for available versions, target labels,
archive checksums, manifest names, and privacy posture.

## Signing And Notarization

Current v1.1.0 archives are checksum-audited but not signed installers. Future
distribution work should add signing and notarization gaps to the release
checklist before claiming signed macOS installers or package-manager formulas.

## Candidate Decision

The v1.1.0 production candidate is archive-first:

- local archive: ready after the release gate passes with the archive binary
  and required clean fixture proof
- Homebrew formula: deferred
- crates.io package: deferred
- signed installer: deferred
- self-update: not implemented

The source-free release candidate status records these decisions alongside the
archive checksum and binary checksum. It does not publish artifacts or mutate
package-manager state.

## Verification

Run:

```bash
bash scripts/smoke-distribution-metadata.sh
```

The smoke verifies required templates, update metadata, clean-extraction
verification script syntax, source-free posture, no unsupported install claims,
and explicit signing/notarization caveats.
