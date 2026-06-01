# Technology Stack: v1.1 Packaging & Adoption

**Project:** ctxhelm / ctxhelm
**Researched:** 2026-05-13
**Scope:** Packaging and adoption only for the existing Rust CLI plus stdio MCP server.
**Overall confidence:** HIGH for repo state and Cargo behavior; MEDIUM for `dist`/cargo-dist workflow until tested in this repo CI.

## Current Repo State

| Area | Observed State | v1.1 Implication |
|------|----------------|------------------|
| Binary package | `crates/ctxhelm` exposes one binary named `ctxhelm`; workspace packages are still `0.1.0`. | Release should make `ctxhelm` the only user-installed binary and bump package/tag version together. |
| Release automation | No `.github/workflows` release workflow and no release packaging scripts detected. | v1.1 needs a first release pipeline, not a custom installer ecosystem. |
| Metadata | Package manifests lack `description`, `repository`, `readme`, `rust-version`, and a root `LICENSE` file is not present. | Add release metadata before publishing artifacts or enabling `cargo install --git` documentation. |
| Runtime surface | MCP is launched as `ctxhelm serve-mcp`; generated adapters already assume `ctxhelm` is on `PATH`. | Packaging must prioritize reliable `PATH` installation and MCP stdio cleanliness. |
| Local state | `CTXHELM_HOME` overrides state; default is `$HOME/.ctxhelm`, falling back to repo-local `.ctxhelm` if `HOME` is unavailable. | Install docs must explain state location and env forwarding for MCP clients. |
| Privacy | Inventory/traces are local, source-free where required, and safe inventory filters generated/sensitive files by default. | Release/install tooling must not add telemetry, cloud indexing, or global agent config mutation. |
| Smoke scripts | `scripts/smoke-mcp-protocol.sh`, `scripts/smoke-codex-mcp.sh`, and `scripts/smoke-claude-mcp.sh` currently run via `cargo run`. | v1.1 should add installed-binary smokes so release assets prove the same path users run. |

## Recommended Stack

### Release Artifacts

| Technology | Version / Choice | Purpose | Why |
|------------|------------------|---------|-----|
| GitHub Releases | Tags like `v1.1.0` | Public release channel | Matches the current public-snapshot workflow, supports downloadable assets and API-visible asset digests, and avoids introducing a package registry before metadata is ready. |
| `dist` / cargo-dist | Current docs show v0.31.0; pin exact tool version in CI | Build archives/checksums/release workflow | Use because it is purpose-built for Rust binary releases, can generate GitHub CI, archives, installers, and sha256 checksum files. Keep v1.1 config minimal: archives plus checksums first. |
| Platform archives | `ctxhelm-v1.1.0-{target}.tar.gz` for macOS/Linux, `.zip` for Windows | Manual install path | Developers can inspect/extract a single binary. This is simpler and more privacy-transparent than making package managers the first path. |
| SHA-256 checksums | One unified `sha256.sum` plus per-asset checksums if generated | Integrity verification | `dist` generates sha256 checksums by default; publish them and document `sha256sum -c` or `shasum -a 256 -c`. |
| Release smoke scripts | Repo scripts plus an installed-binary variant | Publish gate | Keep existing protocol and real-client gates, but run at least one smoke against `./ctxhelm serve-mcp` extracted from the release archive. |

**Targets for v1.1:** start with `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`. Add Linux arm64 and musl only after the basic release path is green.

### Versioning

| Decision | Recommendation |
|----------|----------------|
| Public version | Use `1.1.0` for the v1.1 milestone if the project is being presented as post-v1; otherwise rename the milestone before release. Do not ship a `v1.1.0` tag with package version `0.1.0`. |
| Workspace versions | Bump all workspace crates together for v1.1. The binary depends on local crates, so mixed versions add confusion without benefit. |
| CLI version output | Add Clap version metadata so `ctxhelm --version` returns the package version. This becomes the first support diagnostic. |
| Release tags | Use annotated tags `v1.1.0`, `v1.1.1`, etc. Keep prereleases as `v1.1.0-rc.1` only if testing public assets before final. |
| MSRV | Add `rust-version` after deciding the supported minimum. Current local baseline is `rustc/cargo 1.87.0`; v1.1 can either set `rust-version = "1.87"` for honesty or test a lower MSRV before claiming it. |

### Install Paths

| Path | Command / Flow | Use In v1.1? | Notes |
|------|----------------|--------------|-------|
| Prebuilt archive | Download, verify checksum, extract `ctxhelm`, move to a directory on `PATH`. | Yes, primary | Most transparent path; no Rust toolchain required. |
| Shell/PowerShell installer from `dist` | Optional generated installer script. | Yes, secondary | Useful for adoption, but docs should still show manual archive + checksum first. |
| `cargo install --git` | `cargo install --git <repo-url> --tag v1.1.0 ctxhelm --locked` | Yes, fallback | Works for source builds; document that Rust is required and `--locked` uses the committed lockfile. |
| Build from checkout | `cargo install --path crates/ctxhelm --locked` or `cargo build -p ctxhelm --release --locked` | Yes, developer fallback | Good for contributors and unsupported platforms. |
| crates.io | `cargo install ctxhelm --locked` | Defer | Current manifests are not crates.io-ready: path-only internal deps, sparse package metadata, and no release need to publish all internal crates yet. |
| Homebrew / package managers | `brew install ...` | Defer | Valuable after one GitHub Release proves stable asset naming and checksums. |
| Self-update | Built-in updater | Avoid | Adds network behavior and trust surface that conflicts with local-only positioning for v1.1. |

## PATH and Environment Handling

- Installation docs should verify `ctxhelm --version` and `ctxhelm --help` before any MCP setup.
- MCP setup snippets should continue to call `ctxhelm serve-mcp`, but troubleshooting must explain that the MCP client may have a different `PATH` than the user's shell.
- For robust client configs, recommend an absolute binary path when users hit PATH issues, for example `/Users/name/.local/bin/ctxhelm` or `%USERPROFILE%\.local\bin\ctxhelm.exe`.
- Document `CTXHELM_HOME` as the only supported state override. Default state remains `$HOME/.ctxhelm`; examples should set `CTXHELM_HOME` only for tests, CI, or isolated client smoke.
- Do not make installers mutate Codex, Claude, Cursor, or OpenCode global config. Keep `ctxhelm init --repo ... --claude --cursor --opencode` as the repo-local setup step.

## Privacy Constraints for Packaging

| Constraint | v1.1 Rule |
|------------|-----------|
| No cloud default | Release binaries must not contact hosted indexing, embedding, reranking, analytics, or update services. |
| Read-only product role | Installers may place the `ctxhelm` binary; ctxhelm may write its own caches, traces, cards, and adapter files, but must not edit source code or agent global configs. |
| Source-safe release assets | Release artifacts should include the binary, README, LICENSE, and completion/docs if added. Do not bundle `.ctxhelm`, traces, local inventories, test fixtures with secrets, or developer target directories. |
| Stdio MCP safety | `ctxhelm serve-mcp` must keep stdout reserved for JSON-RPC. Diagnostics belong on stderr or structured MCP responses. |
| Trust diagnostics | The first-run guide should state where local state is written and how to delete it: remove `$CTXHELM_HOME` or `$HOME/.ctxhelm`. |

## Minimal v1.1 Implementation Sequence

1. Add release metadata: root `LICENSE`, package `description`, `repository`, `readme`, `rust-version`, and `ctxhelm --version`.
2. Configure `dist` for the single `ctxhelm` binary, archive artifacts, GitHub Releases, and sha256 checksums.
3. Add release CI gates: `cargo test --workspace`, `cargo run -p ctxhelm -- --help`, `scripts/smoke-mcp-protocol.sh`, and an extracted-binary `ctxhelm serve-mcp` smoke.
4. Document install flows in this order: prebuilt archive, optional installer script, `cargo install --git ... --locked`, build from checkout.
5. Update adapter/setup docs to cover PATH failures, absolute binary paths, explicit `repo` arguments, and `CTXHELM_HOME`.

## Avoid in v1.1

| Avoid | Why | Do Instead |
|-------|-----|------------|
| crates.io as the first release channel | Requires package metadata and internal crate publishing cleanup, while GitHub binaries solve adoption faster. | Use GitHub Releases plus `cargo install --git` fallback. |
| Homebrew tap before first binary release | Adds maintenance before asset names and support workflow are proven. | Revisit after v1.1.0 and at least one patch release. |
| Auto-writing global MCP configs | Breaks the "thin native adapters, repo-local setup" trust model. | Print/copy snippets and keep repo-local generated files. |
| Curl-pipe installer as the only documented install path | Fast but weaker for trust-conscious developers. | Lead with manual archive + checksum; list installer as convenience. |
| Adding cloud telemetry/update checks | Conflicts with the local-only privacy claim. | Keep release checking manual via GitHub Releases. |

## Sources

- Repo inspection: `Cargo.toml`, `crates/ctxhelm/Cargo.toml`, `README.md`, `crates/ctxhelm/src/main.rs`, `crates/ctxhelm-core/src/init.rs`, `crates/ctxhelm-index/src/inventory.rs`, `scripts/smoke-*.sh`. Confidence: HIGH.
- Project context: `.planning/PROJECT.md` and `.planning/STATE.md` identify v1.1 Packaging & Adoption as the active milestone. Confidence: HIGH.
- Cargo install docs: https://doc.rust-lang.org/stable/cargo/commands/cargo-install.html. Confidence: HIGH.
- Cargo manifest docs: https://doc.rust-lang.org/cargo/reference/manifest.html. Confidence: HIGH.
- GitHub release asset docs: https://docs.github.com/en/rest/releases/assets?apiVersion=2026-03-10. Confidence: HIGH.
- cargo-dist docs: https://axodotdev.github.io/cargo-dist/ and https://axodotdev.github.io/cargo-dist/book/artifacts/checksums.html. Confidence: MEDIUM until exercised in ctxhelm CI.
