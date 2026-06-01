# Distribution Metadata

ctxhelm v1.1.7 ships through public release archives and a public Apple Silicon
Homebrew tap. This document records the distribution metadata and the boundaries
for channels that remain deferred.

## Package Templates

- `packaging/homebrew/ctxhelm.rb.template` renders the `thromel/tap/ctxhelm`
  Homebrew formula from the current release archive URL and SHA-256 digest.
- `packaging/crates/README.md` records crates.io preparation checks, not a
  registry publication claim.
- `packaging/release/update-metadata.schema.json` and
  `packaging/release/update-metadata.example.json` define machine-readable
  release metadata for future update checks.
- `scripts/render-homebrew-formula.sh` renders a local formula candidate from a
  version, GitHub release archive URL, and SHA-256 digest.
- `scripts/verify-homebrew-tap.sh` verifies the published `thromel/tap`
  formula by tapping it, checking Ruby syntax/style/audit, installing it through
  Homebrew, running the formula test, and checking `ctxhelm --version`.

The metadata is local-only and source-free. It does not include raw source,
prompts, secrets, terminal logs, or machine-local paths.

## Clean Extraction Verification

Verify an already-built archive from a clean temporary extraction directory:

```bash
bash scripts/verify-release-archive.sh \
  --archive dist/ctxhelm-v1.1.7-aarch64-apple-darwin.tar.gz \
  --manifest dist/ctxhelm-v1.1.7-aarch64-apple-darwin.manifest.json \
  --checksums dist/sha256sums.txt
```

The verifier checks checksum consistency, extracts the archive into a temporary
directory, runs `ctxhelm --version`, runs `ctxhelm --help`, and runs
`ctxhelm doctor` against the extracted binary and release manifest. It does not install binaries, mutate global agent configuration, publish artifacts, or run user project tests.

## Update Metadata Boundary

The release update metadata is not a self-update implementation. It only gives
future tooling a stable source-free shape for available versions, target labels,
archive checksums, manifest names, and privacy posture.

## Signing And Notarization

Current v1.1.7 archives are checksum-audited but not signed installers. Future
distribution work should add signing and notarization gaps to the release
checklist before claiming signed macOS installers or package-manager formulas.

## Homebrew Tap

Apple Silicon macOS users can install from the public tap:

```bash
brew tap thromel/tap
brew install ctxhelm
ctxhelm --version
```

The v1.1.7 tap proof verifies:

- `brew tap thromel/tap`
- `brew audit --strict --new ctxhelm`
- `brew install thromel/tap/ctxhelm`
- `brew test thromel/tap/ctxhelm`
- installed binary output `ctxhelm 1.1.7`

The formula is currently constrained to `arch: :arm64` because the published
archive asset is `aarch64-apple-darwin`.

## Candidate Decision

The v1.1.7 production candidate has these distribution states:

- local archive: ready after the release gate passes with the archive binary
  and required clean fixture proof
- Homebrew formula: ready through `thromel/tap`
- crates.io package: deferred
- signed installer: deferred
- self-update: not implemented

The source-free release candidate status records these decisions alongside the
archive checksum and binary checksum. It does not publish artifacts or mutate
package-manager state.

The public GitHub archive release can be verified after publication with
`scripts/verify-github-release.sh`. The verifier compares GitHub release
metadata and uploaded asset SHA-256 digests against local release artifacts; it
does not create tags, upload assets, or mutate release state.

The public release freshness status can be checked with
`scripts/check-public-release-freshness.sh`. That check compares the public
release target commit with the current commit and records `status`,
`gitRelation`, `commitsAhead`, `productStatus`, `productCommitsAhead`,
`proofOnlyCommitsAhead`, and `ignoredFreshnessPaths` as source-free metadata.
Use `--require-current` before claiming the public archive matches the exact
current commit. Use `--require-product-current` before claiming the archive has
no product-impacting commits ahead when only proof/planning metadata may have
moved after the release. This freshness check is read-only: it does not publish,
create tags, upload assets, install binaries, or mutate release state.

The public install path can be verified with
`scripts/verify-public-archive-install.sh`. That check downloads the GitHub
release assets, verifies checksums, installs only into a temporary bin
directory, and runs `ctxhelm doctor` plus the first-pack smoke.

Optional Codex CLI and Claude Code behavior against the public archive binary
can be checked with `scripts/smoke-public-real-clients.sh`. That script reuses
the public GitHub release assets, runs the existing real-client wrappers with the
extracted binary, and records source-free pass or skip evidence without global
installation or agent-config mutation. Current public archive checks keep the
stricter resource-scope assertions enabled by default. Older archives can still
be checked in compatibility mode by setting `CTXHELM_REQUIRE_RESOURCE_SCOPE=0`.

The public Homebrew tap can be verified with:

```bash
bash scripts/verify-homebrew-tap.sh \
  --tap thromel/tap \
  --formula ctxhelm \
  --expected-version "ctxhelm 1.1.7" \
  --expected-url https://github.com/thromel/ctxhelm/releases/download/v1.1.7/ctxhelm-v1.1.7-aarch64-apple-darwin.tar.gz \
  --expected-sha256 <sha256-from-release-asset>
```

This check mutates local Homebrew state by installing the formula. It does not
publish releases, edit files, mutate global agent configuration, or run user
project tests.

## Verification

Run:

```bash
bash scripts/smoke-distribution-metadata.sh
```

When `CTXHELM_DIST_DIR` points at a built release archive, the smoke also renders
a concrete Homebrew formula candidate from the exact archive digest and checks
that no placeholders or machine-local paths remain. It also runs
`cargo package --manifest-path crates/ctxhelm/Cargo.toml --locked --allow-dirty --list` and
fails if the future crates.io package boundary includes local `.ctxhelm` state,
planning proof artifacts, build output, secrets, request summaries, traces, or
machine-local paths.

The smoke verifies required templates, update metadata, clean-extraction
verification script syntax, Homebrew renderability, crates package boundaries,
source-free posture, no unsupported install claims, and explicit
signing/notarization caveats.
