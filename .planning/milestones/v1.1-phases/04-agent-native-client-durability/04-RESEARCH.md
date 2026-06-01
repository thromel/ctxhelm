# Phase 04: Agent-Native Client Durability - Research

**Researched:** 2026-05-13
**Domain:** MCP client durability, repo-scoped context tools, agent adapter guidance
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

## Implementation Decisions

### Real Client Smokes
- Add scriptable smoke paths for Codex CLI and Claude Code when the client binary is installed, but keep deterministic protocol-level tests as the local fallback.
- Smokes must exercise actual MCP `prepare_task` and `get_pack` flows with explicit `repo` arguments, not only config discovery or server startup.
- Scripts should skip clearly when a client is unavailable and fail only when a requested/available smoke path violates the ctxhelm contract.

### MCP Repo And Pack Semantics
- Prefer explicit repo arguments in MCP tools over relying on the MCP server process working directory.
- Make pack-resource semantics visible to users and tests: either clearly session-scoped or reconstructable from persisted source-free metadata.
- Test restart/reconnect and wrong-working-directory behavior directly so clients do not silently read from the wrong repo or fail with unclear resource errors.
- Bound or expose MCP pack cache growth so long-running clients do not accumulate invisible unbounded state.

### Adapter Guidance
- Keep generated AGENTS.md, Cursor rules, Claude commands, OpenCode config/rules, and any Codex guidance thin.
- Adapter text should direct agents to call ctxhelm dynamically for non-trivial code tasks and to load packs progressively.
- Do not inject large generated repo context into adapter rules. Static cards are acceptable only as concise fallback context for environments where dynamic MCP is unavailable.

### Scope Control
- Do not add autonomous source editing to ctxhelm.
- Do not widen the MCP tool list unless a current tool cannot represent the durability contract.
- Do not add cloud indexing, cloud embeddings, cloud reranking, or vector search.

### Claude's Discretion

No `## Claude's Discretion` section exists in CONTEXT.md. The closest discretion area is copied verbatim below from `## Specific Ideas`.

## Specific Ideas

- Add `scripts/smoke-codex-mcp.sh` and `scripts/smoke-claude-mcp.sh`, or a shared smoke harness with client-specific wrappers.
- Add a deterministic MCP JSON-RPC smoke that launches `ctxhelm serve-mcp` via stdio and calls `prepare_task` and `get_pack` with explicit repo arguments; use it as fallback when real clients are absent.
- Add tests proving `get_pack` resource reads after a restart either reconstruct from source-free metadata or return a clear session-scoped diagnostic.
- Add tests for wrong working directory: server started outside repo, explicit `repo` points to target repo, and output comes from that repo.
- Add tests or diagnostics for MCP cache size/eviction/limits.
- Add adapter snapshot tests proving generated guidance remains thin and dynamic.

### Deferred Ideas (OUT OF SCOPE)

## Deferred Ideas

- A remote MCP endpoint for cloud agents is future work.
- Persisted team context cards, hosted sync, and enterprise policy are future milestones.
- Visual UI for inspecting pack cache/resource durability is out of scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AGNT-01 | Codex CLI and Claude Code smoke scripts verify real MCP `prepare_task` and `get_pack` client paths with explicit `repo` arguments. | Use optional client-specific smoke scripts plus a deterministic stdio JSON-RPC harness; both installed clients are available locally (`codex-cli 0.130.0`, Claude Code `2.1.92`). |
| AGNT-02 | MCP pack resources are clearly session-scoped or can be reconstructed from persisted source-free metadata across server restarts. | Current code is session-scoped via an in-process `OnceLock<Mutex<BTreeMap<...>>>`; research recommends either preserving this with explicit diagnostics or adding reconstruction from source-free plan metadata, not opaque cache dependence. |
| AGNT-03 | MCP cache growth, reconnect behavior, and wrong-working-directory behavior are covered by tests or smoke scripts. | Current tests characterize session scope but do not cover cache bounds, multi-process restart, or wrong-cwd resource reads; add in-process and subprocess stdio tests around these cases. |
| AGNT-04 | Generated adapter guidance stays thin and points agents to dynamic ctxhelm calls rather than injecting large static context. | Current init constants already point to `prepare_task`; add snapshot/size/phrase tests and update wording to mention `get_pack` progressive loading and explicit `repo`. |
</phase_requirements>

## Summary

Phase 4 should harden the existing Rust + direct JSON-RPC MCP implementation rather than introducing an MCP SDK, new retrieval libraries, cloud services, or a wider tool surface. The current code already exposes the right small MCP tool set (`prepare_task`, `search`, `related`, `get_pack`, `related_tests`, `current_diff`) and all tools accept optional `repo`, but resource reads still discover the repo from the MCP server process cwd and pack resources are stored in process-local memory.

The primary implementation risk is confusing successful protocol tests with real client durability. Codex and Claude Code both have installed client binaries on this machine, but real model-driven tool invocation can be flaky or version-dependent. The durable plan is therefore two-tiered: deterministic stdio JSON-RPC smokes must verify the ctxhelm contract every time, while Codex CLI and Claude Code scripts run when installed and either prove real client invocation or skip/fail with explicit status.

**Primary recommendation:** Keep the current stack, add client smoke scripts plus deterministic MCP harnesses, make explicit `repo` mandatory in smoke paths, and either clearly document/test session-scoped pack resources or reconstruct packs from source-free metadata.

## Project Constraints (from CLAUDE.md)

- Default behavior must stay local-only and source-safe for inventory, plans, traces, historical eval reports, and generated cards.
- Packs may include safe snippets, but every snippet path must remain filtered through the safe inventory policy.
- AGENTS.md, MCP, and thin native rules/adapters are the primary product surfaces; CLI is for setup, debugging, and automation.
- ctxhelm must remain read-only with respect to user source: no source edits, user test execution, dependency installation, or auto-commits.
- It may write local ctxhelm state, traces, generated cards, adapter files, and planning/docs artifacts.
- Keep the current Rust workspace architecture and typed contracts unless measured evidence justifies a change.
- New retrieval work should be checked against source-free historical evals when practical.
- Run `cargo test --workspace` before claiming implementation complete.
- Run `cargo run -p ctxhelm -- --help` after CLI changes.
- Prefer structured reports and typed errors over ad hoc debug logging.
- Keep public contracts in `ctxhelm-core`, retrieval in `ctxhelm-index`, plan/pack construction in `ctxhelm-compiler`, MCP translation in `ctxhelm-mcp`, and CLI rendering in `ctxhelm`.
- Do not add new barrel files for `ctxhelm-index`, `ctxhelm-compiler`, or `ctxhelm-mcp` unless those crates are split into modules.

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| Rust workspace / Cargo | cargo 1.87.0, rustc 1.87.0 | Build, test, and run ctxhelm crates | Existing implementation stack; fast local CLI and typed library boundaries. |
| `serde` / `serde_json` | 1.0.228 / 1.0.149 | Public JSON contracts and JSON-RPC payloads | Existing MCP/CLI contract layer; preserves camelCase compatibility. |
| Direct MCP over stdio JSON-RPC | MCP protocol `2025-11-25` in code | Local agent-client integration | Current code already implements required methods; no SDK migration is needed for this phase. |
| `assert_cmd` + Rust tests | assert_cmd 2.2.2 | Binary CLI compatibility tests | Existing compiled-binary test pattern under `crates/ctxhelm/tests`. |
| Bash + `python3` JSON validation | Python 3.14.2 available | Smoke scripts and output validation | Existing `scripts/smoke-historical-eval.sh` pattern; portable enough for local client smoke gates. |

### Supporting

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| Codex CLI | 0.130.0 | Optional real-client MCP smoke | Run when `codex` is installed; isolate `CODEX_HOME` and avoid mutating user config. |
| Claude Code | 2.1.92 | Optional real-client MCP smoke | Run when `claude` is installed; use project `.mcp.json` or temp config and strict MCP config where available. |
| Git CLI | 2.45.1 | Fixture repos, current diff, history | Required by existing ctxhelm retrieval and tests. |
| `tempfile` | 3.27.0 | Isolated fixture repos and homes | Existing unit/integration test fixture style. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Direct JSON-RPC MCP implementation | Rust MCP SDK | Not recommended in Phase 4. Current protocol surface is small and tested; migration would add risk without addressing client durability. |
| Session-scoped pack resources | Persisted source-free plan metadata plus pack reconstruction | Viable if AGNT-02 chooses reconstructable semantics, but must revalidate source paths before reading snippets. |
| Real client smoke only | Deterministic stdio harness | Real clients are valuable but model/tool invocation can be nondeterministic; deterministic protocol smoke must remain the fallback gate. |
| Static generated repo summaries in adapters | Dynamic MCP calls plus concise cards fallback | Static dumps get stale and violate Phase 4 scope; cards are only for disconnected/cloud fallback. |

**Installation:** No new packages are required for the recommended plan.

**Version verification:** Verified locally with `cargo tree --workspace --depth 1`, `cargo --version`, `rustc --version`, `python3 --version`, `codex --version`, and `claude --version` on 2026-05-13. No registry lookup is needed because Phase 4 should not add dependencies.

## Architecture Patterns

### Recommended Project Structure

```text
scripts/
|-- smoke-mcp-protocol.sh       # Deterministic stdio JSON-RPC prepare_task/get_pack/resource checks
|-- smoke-codex-mcp.sh          # Optional Codex CLI real-client wrapper
`-- smoke-claude-mcp.sh         # Optional Claude Code real-client wrapper

crates/
|-- ctxhelm-core/src/init.rs    # Thin adapter text and snapshot tests
|-- ctxhelm-mcp/src/resources.rs # Pack cache/resource semantics
|-- ctxhelm-mcp/src/tools.rs    # Explicit repo args for tools
`-- ctxhelm-mcp/src/lib.rs      # MCP protocol/reconnect/wrong-cwd tests
```

### Pattern 1: Two-Tier Client Smoke

**What:** Add a deterministic MCP stdio smoke that launches `cargo run -p ctxhelm -- serve-mcp`, sends `initialize`, `tools/call prepare_task`, `tools/call get_pack`, and `resources/read` requests, then validates JSON with `python3`. Wrap Codex and Claude Code real-client smokes around the same fixture and expected contract.

**When to use:** Always for Phase 4 validation. Real clients should supplement, not replace, the deterministic harness.

**Example:**

```bash
CTXHELM_SMOKE_REPO="$fixture_repo" \
CTXHELM_HOME="$fixture_home" \
python3 scripts/smoke-mcp-protocol.py \
  --repo "$fixture_repo" \
  --task "fix requireSession auth bug"
```

### Pattern 2: Explicit Repo Wins Over Server CWD

**What:** For every MCP tool smoke, start the server from a directory outside the fixture repo and pass `repo` explicitly. Assert output paths come from the fixture repo. This directly tests the failure mode that clients launch MCP servers with a different cwd than the active workspace.

**When to use:** All MCP tool smokes and tests for `prepare_task`, `get_pack`, `search`, `related`, `related_tests`, and `current_diff`.

**Example JSON-RPC request:**

```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"prepare_task","arguments":{"task":"fix requireSession auth bug","repo":"/tmp/ctxhelm-fixture/repo","mode":"bug_fix","recordTrace":false}}}
```

### Pattern 3: Make Pack Resource Semantics Observable

**What:** If resources stay session-scoped, tests and docs should explicitly assert that `ctxhelm://pack/<task-id>/<budget>` works only after `prepare_task` in the same server process, and fails with a clear diagnostic after restart. If reconstruction is added, persist only source-free plan metadata and re-run safe pack compilation on `resources/read`.

**When to use:** AGNT-02 and AGNT-03. Do not leave pack resource behavior implicit.

**Example session-scoped assertion:**

```rust
clear_pack_resource_cache();
let missing = handle_line(
    r#"{"jsonrpc":"2.0","id":50,"method":"resources/read","params":{"uri":"ctxhelm://pack/not-yet-created/brief"}}"#,
).unwrap();
assert_eq!(missing["error"]["code"], -32602);
assert!(missing["error"]["message"].as_str().unwrap().contains("call prepare_task first"));
```

### Pattern 4: Thin Adapter Snapshot Tests

**What:** Add tests for generated AGENTS/Cursor/Claude/OpenCode/Codex guidance that assert small size, dynamic tool language, explicit `repo`, progressive pack loading, and absence of large generated context or source snippets.

**When to use:** AGNT-04.

**Example checks:**

```rust
assert!(AGENTS_SECTION.contains("prepare_task"));
assert!(AGENTS_SECTION.contains("repo"));
assert!(CLAUDE_BUGFIX_COMMAND.contains("standard pack"));
assert!(!AGENTS_SECTION.contains("Repository map:"));
assert!(AGENTS_SECTION.len() < 1_500);
```

### Anti-Patterns to Avoid

- **Config discovery as proof:** `codex mcp get` or `claude mcp list` only proves configuration, not that the agent called ctxhelm.
- **Wrong-cwd fallback in smokes:** Starting the MCP server inside the target repo hides client cwd bugs.
- **Unbounded pack cache:** A process-wide map without limits can grow indefinitely in long sessions.
- **Protocol-only success for AGNT-01:** Deterministic protocol tests are necessary fallback, but AGNT-01 also needs optional real Codex and Claude scripts.
- **Static adapter bloat:** Do not generate repo maps, inventories, or context packs into rules files.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Client-independent MCP verification | Prose scraping of agent replies | JSON-RPC stdio harness with structured JSON assertions | Stable, deterministic, and independent of model wording. |
| Codex/Claude client configuration | Permanent mutation of user global config | Temp homes/project fixtures and generated snippets | Avoids polluting user setup and wrong-repo state. |
| Tool output parsing | Regex over pretty text | `structuredContent` and JSON fields | MCP and ctxhelm already expose typed data. |
| Pack durability | Hidden in-memory behavior with no diagnostics | Explicit session-scoped diagnostic or source-free reconstruction | Prevents session surprises. |
| Adapter context | Generated static repo dumps | Dynamic `prepare_task` / `get_pack` instructions and concise cards fallback | Static dumps get stale and consume context. |

**Key insight:** The hard part is not adding more context. It is proving that real client sessions call the right dynamic ctxhelm surface for the right repo, and that users can predict what survives a reconnect.

## Common Pitfalls

### Pitfall 1: Real Client Smoke Only Checks Config
**What goes wrong:** A script passes after adding/listing an MCP server, but the agent never calls `prepare_task` or `get_pack`.
**Why it happens:** Client CLIs separate configuration commands from model-driven tool invocation.
**How to avoid:** Require output evidence from `structuredContent` or a known fixture target path.
**Warning signs:** Smoke only runs `codex mcp get`, `claude mcp get`, or server startup.

### Pitfall 2: Session-Scoped Pack URIs Look Durable
**What goes wrong:** A returned `ctxhelm://pack/<task-id>/<budget>` URI works in one process but fails after reconnect.
**Why it happens:** Current pack cache is process-local memory.
**How to avoid:** Either document/test session scope or persist source-free reconstruction metadata.
**Warning signs:** Resource-read tests use `clear_pack_resource_cache()` but no subprocess restart.

### Pitfall 3: Resource Reads Use Wrong Repo
**What goes wrong:** `ctxhelm://repo/summary`, file resources, or symbol resources read from the MCP server cwd instead of the active project.
**Why it happens:** Resource URIs currently do not carry repo and `read_resource` calls `discover_repo(None)`.
**How to avoid:** Prefer tool calls with explicit `repo`; for resources, either add repo-scoped semantics or make wrong-cwd behavior explicit and diagnostic.
**Warning signs:** Tests call `std::env::set_current_dir(&repo.repo)` before reading resources.

### Pitfall 4: Cache Bounds Are Invisible
**What goes wrong:** Long-running clients accumulate Markdown and JSON copies for every generated pack.
**Why it happens:** Current cache is an unbounded `BTreeMap`.
**How to avoid:** Add max entry/byte bounds or expose cache stats/eviction diagnostics.
**Warning signs:** Tests generate many packs but cannot assert cache size.

### Pitfall 5: Adapter Text Becomes Product Documentation
**What goes wrong:** AGENTS/Cursor/Claude/OpenCode files start carrying broad repo summaries or stale command descriptions.
**Why it happens:** Static guidance is easier to inspect than live dynamic tool calls.
**How to avoid:** Keep adapters command-oriented and test size/content.
**Warning signs:** Adapter constants mention concrete repo file lists, generated cards inline, or obsolete "first implemented tool" wording.

## Code Examples

Verified patterns from current code and official docs:

### Codex MCP Config Shape

```toml
# Source: OpenAI Codex MCP docs
[mcp_servers.ctxhelm]
command = "ctxhelm"
args = ["serve-mcp"]
cwd = "/path/to/repo"
startup_timeout_sec = 10
tool_timeout_sec = 60
```

Codex supports stdio MCP servers with `command`, `args`, `env`, `env_vars`, and optional `cwd`; its MCP config can live in user `~/.codex/config.toml` or trusted project `.codex/config.toml`.

### Claude Project MCP Config Shape

```json
{
  "mcpServers": {
    "ctxhelm": {
      "command": "ctxhelm",
      "args": ["serve-mcp"]
    }
  }
}
```

Claude Code supports project-scoped `.mcp.json` and names MCP tools as `mcp__<server-name>__<tool-name>` in SDK/tool allow-list contexts.

### Deterministic MCP Tool Call

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_pack","arguments":{"task":"fix requireSession auth bug","repo":"/tmp/ctxhelm-fixture/repo","mode":"bug_fix","budget":"brief","format":"json","recordTrace":false}}}
```

Assert `result.structuredContent.budget == "brief"`, `targetAgent` when supplied, and sections mention the expected safe fixture path.

## State of the Art

| Old Approach | Current Approach | When Changed / Verified | Impact |
|--------------|------------------|--------------------------|--------|
| Codex global-only MCP setup assumptions | Codex supports user config and trusted project `.codex/config.toml`; MCP has optional `cwd`, `enabled_tools`, timeouts | Official Codex docs checked 2026-05-13 | Smoke scripts should isolate `CODEX_HOME` or use project config and avoid permanent global mutation. |
| Claude `project` scope terminology only | Claude docs now describe `local`, `project`, and `user` scopes, with project `.mcp.json` approval behavior | Official Claude docs checked 2026-05-13 | Generated Claude snippets can stay project-local, but smokes must handle approval/strict config explicitly. |
| MCP tools as text-only outputs | MCP 2025-11-25 schema supports `structuredContent` and resource links not necessarily listed in `resources/list` | Official MCP schema checked 2026-05-13 | ctxhelm should assert structured output and can return tool-linked pack URIs without listing every dynamic pack resource. |
| Session behavior only characterized in unit tests | Phase 4 requires restart/reconnect/wrong-cwd subprocess smokes | Local code and requirements checked 2026-05-13 | Add black-box process tests in addition to `handle_line` tests. |

**Deprecated/outdated:**
- Adapter text saying "the first implemented tool is prepare_task" is outdated because the MCP surface now includes six tools and `get_pack`.
- Treating `ctxhelm://pack/...` resources as durable is incorrect unless Phase 4 adds reconstruction.

## Open Questions

1. **Should AGNT-02 preserve session-scoped pack resources or add reconstruction?**
   - What we know: Current code is session-scoped, tests already characterize that behavior, and `get_pack` can directly materialize packs.
   - What's unclear: Whether users expect resource URIs to survive reconnects.
   - Recommendation: Preserve session scope for now, improve diagnostics/docs, and make `get_pack` the durable path. Add reconstruction only if real client behavior strongly favors resource links.

2. **How deterministic can real Codex/Claude smokes be?**
   - What we know: Both clients are installed locally, but real agent tool invocation depends on model behavior and permissions.
   - What's unclear: Exact noninteractive flags that remain stable across future client versions.
   - Recommendation: Keep real-client scripts optional/skipping and assert contract evidence when they run; rely on protocol smoke for deterministic CI.

3. **Should resources accept repo arguments?**
   - What we know: MCP `resources/read` params currently only include `uri`; ctxhelm resource URIs do not encode repo.
   - What's unclear: Whether changing URI shapes would break existing resource compatibility.
   - Recommendation: Do not break existing URIs. Prefer explicit-repo tools for dynamic operations, and add diagnostics/docs that repo resources use server cwd unless a future additive URI shape is introduced.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust/Cargo | Build/tests/smokes | Yes | cargo 1.87.0, rustc 1.87.0 | None needed |
| Git | Fixture repos/current diff/history | Yes | 2.45.1 | None for git-dependent tests |
| Python 3 | Smoke JSON validation | Yes | 3.14.2 | Use `jq` only if deliberately added; not recommended |
| Codex CLI | Optional real-client smoke | Yes | 0.130.0 | Skip Codex smoke; run protocol smoke |
| Claude Code | Optional real-client smoke | Yes | 2.1.92 | Skip Claude smoke; run protocol smoke |
| Bash/zsh | Smoke wrappers | Yes | zsh shell active | POSIX shell subset in scripts |

**Missing dependencies with no fallback:** None found.

**Missing dependencies with fallback:** None found. Real client smokes should still skip if a future machine lacks `codex` or `claude`.

## Validation Architecture

Skipped. `.planning/config.json` sets `workflow.nyquist_validation` to `false`.

## Sources

### Primary (HIGH confidence)

- Local repo: `.planning/phases/04-agent-native-client-durability/04-CONTEXT.md` - Phase decisions and scope.
- Local repo: `.planning/REQUIREMENTS.md` - AGNT-01 through AGNT-04.
- Local repo: `.planning/ROADMAP.md` and `.planning/PROJECT.md` - Phase goal and product constraints.
- Local repo: `CLAUDE.md` and `AGENTS.md` - project constraints, validation, and architecture rules.
- Local repo: `crates/ctxhelm-mcp/src/tools.rs`, `resources.rs`, `schemas.rs`, `protocol.rs`, `lib.rs` - current MCP tool/resource/session behavior.
- Local repo: `crates/ctxhelm-core/src/init.rs` - generated adapter guidance.
- Local repo: `crates/ctxhelm/tests/cli_compat.rs`, `crates/ctxhelm/tests/common/mod.rs`, `scripts/smoke-historical-eval.sh` - existing compatibility and smoke patterns.
- OpenAI Codex MCP docs: https://developers.openai.com/codex/mcp - Codex MCP config, stdio options, timeouts, `cwd`.
- OpenAI Codex config basics: https://developers.openai.com/codex/config-basic - config scope and precedence.
- Claude Code MCP docs: https://code.claude.com/docs/en/mcp - scopes, `.mcp.json`, project approval behavior.
- Claude Agent SDK MCP docs: https://code.claude.com/docs/en/agent-sdk/mcp - `.mcp.json`, allowed tools, stdio transport.
- MCP 2025-11-25 schema: https://modelcontextprotocol.io/specification/2025-11-25/schema - `structuredContent`, resource links, tool result guidance.
- MCP 2025-11-25 overview: https://modelcontextprotocol.io/specification/2025-11-25/basic - JSON-RPC lifecycle and message rules.

### Secondary (MEDIUM confidence)

- OpenAI Codex GitHub README: https://github.com/openai/codex/blob/main/codex-rs/README.md - Codex CLI as MCP client and `codex exec`/`codex mcp` behavior.
- Local memory index from prior ctxhelm runs - used only to identify prior client-smoke risks; all actionable claims were rechecked against live repo or official docs.

### Tertiary (LOW confidence)

- None used as authoritative evidence.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - existing local dependencies and installed tools were verified directly.
- Architecture: HIGH - local source and phase context clearly identify ownership boundaries.
- Client smoke feasibility: MEDIUM - clients are installed and official docs confirm MCP support, but model-driven invocation can remain nondeterministic.
- Pack-resource semantics: HIGH - current code and tests show process-local session scope.
- Adapter guidance: HIGH - generated constants are local and easy to snapshot.

**Research date:** 2026-05-13
**Valid until:** 2026-06-12 for local architecture; 2026-05-20 for Codex/Claude client CLI behavior.
