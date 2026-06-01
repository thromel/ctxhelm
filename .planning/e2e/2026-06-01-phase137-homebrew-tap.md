# Phase 137 E2E: Public Homebrew Tap

## Goal

Turn the Phase 135 Homebrew readiness path into a real public install channel.

## Published Tap

Repository:

```text
https://github.com/thromel/homebrew-tap
```

Tap:

```text
thromel/tap
```

Tap commit:

```text
99af0b4ca7cb1b9756dec745810cc366e1d3c086
```

Formula:

```text
Formula/ctxpack.rb
```

## Formula Scope

- Version: `1.1.4`
- URL: `https://github.com/thromel/ctxpack/releases/download/v1.1.4/ctxpack-v1.1.4-aarch64-apple-darwin.tar.gz`
- SHA-256: `24101f411da3dae73dbd5ce7f24b0f99427ac4ab016885b72cca004ef1b619c9`
- Platform: Apple Silicon macOS (`depends_on arch: :arm64`)

## Proof

Durable source-free proof:

- `.ctxpack/e2e/phase137-homebrew-tap-proof.json`

Commands run:

```bash
brew tap thromel/tap
brew audit --strict --new ctxpack
brew install thromel/tap/ctxpack
brew test thromel/tap/ctxpack
ctxpack --version
bash scripts/verify-homebrew-tap.sh \
  --tap thromel/tap \
  --formula ctxpack \
  --expected-version "ctxpack 1.1.4" \
  --expected-url https://github.com/thromel/ctxpack/releases/download/v1.1.4/ctxpack-v1.1.4-aarch64-apple-darwin.tar.gz \
  --expected-sha256 24101f411da3dae73dbd5ce7f24b0f99427ac4ab016885b72cca004ef1b619c9 \
  --output .ctxpack/e2e/phase137-homebrew-tap-proof.json
```

All passed. The installed binary reported:

```text
ctxpack 1.1.4
```

## Main Repo Changes

- The Homebrew formula template now renders a Homebrew-style formula with typed
  and frozen-string headers, no redundant `version` stanza, and an explicit
  Apple Silicon architecture constraint.
- `scripts/verify-homebrew-tap.sh` verifies the public tap path and writes a
  source-free proof artifact.
- README, release docs, distribution docs, release checklist, release-doc
  checks, and release-packaging tests now recognize the Homebrew tap as a real
  install channel.

## Non-Goals

No crates.io package, signed installer, self-update path, global agent config
mutation, or additional platform archive was added.
