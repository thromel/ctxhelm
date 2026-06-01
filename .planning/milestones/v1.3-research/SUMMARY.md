# Research Summary: v1.1 Packaging & Adoption

**Synthesized:** 2026-05-13
**Scope:** Make the completed v1 ctxhelm product installable, configurable, documented, and smoke-testable without changing the retrieval engine or product boundaries.

## Decision

v1.1 should be a thin adoption layer around the existing Rust CLI/MCP product. The milestone should not add retrieval signals, semantic search, durable storage, UI, cloud services, team sync, autonomous edits, or global agent-config mutation.

The release should answer one practical question:

> Can a developer install ctxhelm, wire it into an agent, prove MCP works, and get a first useful context pack from a real repository?

## Recommended v1.1 Shape

1. **Release identity and packaging**
   - Bump workspace versions consistently before publishing a v1.1 tag.
   - Add `ctxhelm --version` as the first support diagnostic.
   - Prefer GitHub Releases with platform archives and SHA-256 checksums as the first public binary channel.
   - Keep `cargo install --git ... --tag ... --locked` and local checkout builds as fallback paths.
   - Defer crates.io, Homebrew, self-update, signed installers, and package-manager ecosystems until the binary release path is proven.

2. **Installed-binary smoke gates**
   - Release gates must smoke the installed or extracted `ctxhelm` binary, not only `cargo run`.
   - Required gates should include `ctxhelm --help`, `ctxhelm --version`, `ctxhelm init`, deterministic MCP protocol smoke from the wrong cwd, and artifact audits.
   - Existing Codex/Claude real-client smokes should remain optional by default and required only on provisioned maintainer machines.

3. **Agent-native setup**
   - `ctxhelm init` should remain repo-local and non-invasive.
   - It should generate or print thin setup artifacts for Codex, Claude Code, Cursor, and OpenCode.
   - It should not mutate global Codex, Claude, Cursor, or OpenCode configuration by default.
   - All guidance should emphasize explicit `repo` arguments, dynamic `prepare_task`, and progressive `get_pack`.

4. **Docs and troubleshooting**
   - Split a short quickstart from reference docs.
   - Cover install, PATH, absolute binary paths, `CTXHELM_HOME`, wrong-cwd MCP behavior, pack-resource session scope, generated files, smoke commands, and uninstall/state cleanup.
   - Add a compatibility matrix for verified client versions and make real-client claims evidence-based.

5. **Privacy and release audit**
   - Release artifacts must exclude `.ctxhelm`, traces, request logs, local MCP configs, temp homes, secrets, and absolute local paths.
   - No telemetry, cloud indexing, cloud embeddings, or update checks should appear in v1.1.

## Main Risks

| Risk | Mitigation |
|------|------------|
| Install works only from the source checkout | Test an installed/extracted binary from a clean temp directory. |
| MCP client cannot find `ctxhelm` on PATH | Document absolute binary paths and make generated guidance diagnose PATH issues. |
| Agent starts MCP from the wrong cwd | Keep explicit `repo` in every adapter and smoke path. |
| Client version drift breaks setup docs | Record verified Codex/Claude/Cursor/OpenCode versions and keep protocol smoke as the hard gate. |
| Release artifacts include local state | Add archive-content audits for caches, traces, secrets, logs, and absolute local paths. |
| Packaging scope becomes v1.2/v2 work | Defer retrieval, storage, semantic search, UI, cloud, and team features. |

## Requirement Implications

- Packaging requirements should cover version identity, release artifacts, checksums, source-build fallback, and artifact audits.
- Adoption requirements should cover repo-local init reporting, agent setup snippets, absolute-path guidance, config linting, and first-pack quickstart.
- Smoke requirements should cover installed binary CLI smoke, deterministic MCP protocol smoke, optional real-client Codex/Claude smoke, and generated docs/config checks.
- Documentation requirements should cover install, quickstart, troubleshooting, compatibility matrix, release checklist, and privacy/state cleanup.

## Source Research Files

- `.planning/research/STACK.md`
- `.planning/research/FEATURES.md`
- `.planning/research/ARCHITECTURE.md`
- `.planning/research/PITFALLS.md`
