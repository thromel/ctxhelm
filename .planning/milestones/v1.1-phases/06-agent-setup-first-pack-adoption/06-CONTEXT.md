# Phase 6: Agent Setup & First-Pack Adoption - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning
**Mode:** Autonomous smart-discuss defaults

<domain>
## Phase Boundary

This phase turns the v1.1 binary into a usable first-run agent setup experience. It covers `ctxpack init` reporting, generated Codex/Claude/Cursor/OpenCode setup guidance, setup validation, and a first useful prepare-task/get-pack journey. It does not publish a package, alter retrieval ranking, auto-edit global agent configuration, or replace the agent clients' native permission and editing flows.

</domain>

<decisions>
## Implementation Decisions

### Init User Experience
- `ctxpack init` should clearly report repo-local files written, skipped, and unchanged.
- The init output should provide exact next steps: verify binary, run deterministic MCP smoke, configure one agent explicitly, then request a first context plan/pack.
- JSON output, if present or added, should remain testable and stable enough for scripts.
- Init should continue to write only repo-local ctxpack/adaptor artifacts unless a future explicit apply mode is designed separately.

### Agent Setup Guidance
- Codex CLI setup should be copy/paste-oriented and avoid hidden global config mutation by default.
- Claude Code setup should prefer project-local or mergeable MCP guidance and slash-command context, not global side effects.
- Cursor and OpenCode artifacts should remain thin rules/snippets that direct the agent to dynamic ctxpack MCP calls.
- All setup guidance must include explicit `repo` arguments and a progressive `prepare_task` -> native file reads -> `get_pack` flow.

### Validation And First Pack
- Add a user-facing validation path for generated setup artifacts: expected files, JSON/syntax shape where applicable, command availability, no large static context, and absolute binary path troubleshooting.
- Use deterministic MCP protocol smoke as the hard proof of ctxpack behavior before asking users to debug agent auth/model issues.
- Treat Codex/Claude real-client proof as optional and versioned; do not claim Cursor/OpenCode real-client tool-call evidence until machine-checkable.
- The first-pack journey should work from a real repo using installed `ctxpack`, not only `cargo run`.

### the agent's Discretion
- The planner may choose whether setup validation is a new `ctxpack` subcommand, a script, or an extension of existing init output, provided it is testable and user-facing.
- The implementation may reuse existing smoke scripts if that keeps the behavior simpler and avoids duplicating MCP logic.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/ctxpack-core/src/init.rs` owns generated `AGENTS.md`, `.ctxpack/ctxpack.toml`, Cursor rule, Claude command, Claude MCP snippet, and OpenCode snippet templates.
- `crates/ctxpack/src/main.rs` owns `init`, `prepare-task`, `get-pack`, `serve-mcp`, and CLI rendering.
- `crates/ctxpack/tests/cli_compat.rs` already guards init/adaptor guidance and MCP compatibility.
- `scripts/smoke-mcp-protocol.sh`, `scripts/smoke-codex-mcp.sh`, and `scripts/smoke-claude-mcp.sh` already provide deterministic and optional real-client proof paths.
- Phase 5 added release packaging and installed-binary documentation that Phase 6 should build on.

### Established Patterns
- Keep adapter text concise and dynamic; static files should tell agents to call ctxpack, not embed repo context.
- Use explicit repo paths in MCP calls because client cwd varies.
- Preserve existing CLI/MCP/JSON contracts unless additive changes are deliberately tested.
- Keep real-client claims machine-checkable and separate from deterministic protocol proof.

### Integration Points
- Init reports and adapter templates connect through `ctxpack-core::init`.
- User-facing setup validation likely belongs in the CLI, backed by core/template checks where useful.
- First-pack quickstart can be exercised via CLI commands and existing MCP protocol smoke.
- Documentation can be updated later in Phase 7, but Phase 6 should produce enough behavior and output for those docs to describe.

</code_context>

<specifics>
## Specific Ideas

- Consider adding `ctxpack doctor` or `ctxpack setup-check` if a dedicated command makes validation simpler.
- If adding a command, keep it local-only and read-only: inspect generated files, command availability, and config syntax; do not mutate agent configs.
- Add tests that generated agent text contains `prepare_task`, `get_pack`, explicit `repo`, and session-scope caveats.
- Add a smoke script that runs init in a temp repo, validates generated setup artifacts, and obtains a first plan/pack with the installed binary.

</specifics>

<deferred>
## Deferred Ideas

- Automatic global agent config writes.
- Cursor/OpenCode real-client tool-call automation without machine-checkable proof.
- Hosted setup service, UI wizard, telemetry, or package-manager integrations.
- Retrieval-quality changes, storage migration, semantic search, parser expansion, and team features.

</deferred>
