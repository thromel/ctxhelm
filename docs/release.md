# ctxpack v1.1.0 Release Guide

This document describes the local binary release path for ctxpack v1.1.0. The primary user path is a prebuilt archive plus SHA-256 checksums; source builds are fallback paths.

## User Install

Download the archive for your platform from the release artifacts. Archive names follow this shape:

```text
ctxpack-v1.1.0-{target}.tar.gz
sha256sums.txt
```

Verify checksums before installing:

```bash
shasum -a 256 -c sha256sums.txt
sha256sum -c sha256sums.txt
```

Extract and install the binary on `PATH`:

```bash
tar -xzf ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz
install -m 0755 ctxpack-v1.1.0-aarch64-apple-darwin/ctxpack ~/.local/bin/ctxpack
ctxpack --version
ctxpack --help
```

The expected diagnostic is:

```text
ctxpack 1.1.0
```

## Source Build Fallbacks

Build from the tagged repository with locked dependencies:

```bash
cargo install --git https://github.com/thromel/ctxpack --tag v1.1.0 ctxpack --locked
ctxpack --version
ctxpack --help
```

Build from a local checkout:

```bash
cargo install --path crates/ctxpack --locked
cargo build -p ctxpack --release --locked
target/release/ctxpack --version
target/release/ctxpack --help
```

## Maintainer Packaging

From a clean checkout at the v1.1.0 tag, run:

```bash
bash scripts/release-package.sh
```

The script builds with:

```bash
cargo build -p ctxpack --release --locked
```

It writes release artifacts under `dist/` by default, or under `CTXPACK_DIST_DIR` when that environment variable is set:

```text
dist/ctxpack-v1.1.0-{target}.tar.gz
dist/ctxpack-v1.1.0-{target}.tar.gz.sha256
dist/sha256sums.txt
```

The package script stages only the `ctxpack` binary, `README.md`, `LICENSE`, and `VERSION`, then extracts the archive in a temporary directory and verifies:

```bash
ctxpack --version
ctxpack --help
```

During multi-plan local work, maintainers can set `CTXPACK_ALLOW_DIRTY=1` for verification, but release artifacts should be produced from a clean checkout.

## Release Gate

Before publishing or announcing a release, run the local release gate:

```bash
bash scripts/release-gate.sh
```

This is the pre-publication blocker for v1.1.0. When `CTXPACK_BIN` is not set, the gate runs `scripts/release-package.sh`, audits the archive, extracts the generated artifact, and uses the extracted `ctxpack` binary for installed-binary proof.

To prove a selected installed or previously extracted binary, pass an absolute path:

```bash
CTXPACK_BIN=/absolute/path/to/ctxpack bash scripts/release-gate.sh
```

The release gate preserves the packaging script's clean-checkout guard by default. During multi-plan local verification, set `CTXPACK_ALLOW_DIRTY=1` explicitly; release artifacts intended for publication should be built from a clean checkout.

The release gate runs these required checks:

- `cargo test --workspace`
- `scripts/check-release-docs.sh`
- `scripts/release-package.sh`, including `scripts/audit-release-artifact.sh`
- selected or extracted binary `ctxpack --version`
- selected or extracted binary `ctxpack --help`
- `scripts/smoke-first-pack.sh`
- `scripts/smoke-mcp-protocol.sh` from a wrong cwd with an explicit `--repo`/MCP `repo` argument

The optional real-client evidence wrappers are:

- `scripts/smoke-codex-mcp.sh`
- `scripts/smoke-claude-mcp.sh`

The gate passes the same selected or extracted `CTXPACK_BIN` into the first-pack smoke, MCP protocol smoke, and optional real-client wrappers. Real-client proof is not required by default. Use these environment variables when needed:

- `CTXPACK_SKIP_REAL_CLIENT=1` keeps Codex and Claude checks deterministic-only after the protocol proof.
- `CTXPACK_REQUIRE_REAL_CLIENT=1` makes missing Codex or Claude tool-call evidence fail the gate.
- `CTXPACK_REAL_CLIENT_EVIDENCE_DIR=/absolute/path/to/evidence` writes stable JSON evidence files with client version, ctxpack version, repo path, `prepare_task`, and `get_pack` proof when real-client checks run.

The release gate does not publish, upload, or create GitHub releases, and does not create tags. It does not mutate global agent config and does not run user project tests. Cursor and OpenCode real-client proof is not claimed for v1.1.0.

## Artifact Audit

`scripts/release-package.sh` runs `scripts/audit-release-artifact.sh` immediately after archive creation and before checksum success output.

The audit lists archive members and extracts the artifact to a temporary directory. It fails on local state, traces, request logs, cache or target debris, git internals, secret-looking filenames, absolute local paths, and text payloads with machine-specific or secret-looking values. It does not upload artifacts or call cloud scanning services.

You can audit an existing archive directly:

```bash
bash scripts/audit-release-artifact.sh dist/ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz
```

## Out of Scope for v1.1

The v1.1.0 release does not require crates.io publishing, Homebrew taps, self-update support, signed installers, cloud telemetry, cloud indexing, cloud embeddings, hosted release services, or global agent config mutation.

ctxpack remains local-first and read-only. Release scripts build and audit ctxpack artifacts only; they do not mutate user repositories, global Codex or Claude configuration, MCP client config, or package-manager registries.
