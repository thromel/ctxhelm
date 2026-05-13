# Feature Research

**Domain:** Local-first agent-native repository context compiler for coding agents
**Researched:** 2026-05-13
**Confidence:** HIGH for agent-native surfaces and local ctxpack state; MEDIUM for competitor feature emphasis because vendor docs expose capabilities but not full ranking internals.

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Agent-native setup artifacts | Current coding agents already use repository instruction files such as `AGENTS.md`, `.github/copilot-instructions.md`, `CLAUDE.md`, rules, and MCP configuration. | MEDIUM | Keep `ctxpack init` focused on repo-local `AGENTS.md`, `.ctxpack/ctxpack.toml`, and thin adapter snippets. Do not mutate global client config automatically. |
| Small MCP tool/resource/prompt surface | MCP standardizes tools, resources, and prompts; coding-agent clients expect typed discovery and structured outputs. | MEDIUM | Preserve a small surface: `prepare_task`, `get_pack`, `search`, `related`, `related_tests`, `current_diff`, plus browsable repo/pack/file/symbol resources and workflow prompts. |
| Safe repository inventory | Codebase-aware tools index repositories and respect ignore files; users expect generated, vendored, and sensitive files not to pollute context. | HIGH | Must respect `.gitignore`, `.ctxpackignore`, `.cursorignore`, generated-file exclusions, and secret-path exclusions. Add freshness checks before expanding use. |
| Hybrid local retrieval | Agents can already run `rg`; ctxpack must at least combine exact path anchors, lexical search, symbol search, current diff, dependency hints, related tests, and co-change hints. | HIGH | Retrieval should be typed, source-safe where possible, and explain which signal selected each file. Lexical-only search is not enough post-MVP. |
| Task-conditioned context plans | The product promise is not "search my repo"; users expect a plan that names target files, tests, validation commands, risk flags, and pack options for a specific task. | MEDIUM | `prepare_task` remains the main product action. Plans should be compact enough for agent use and rich enough for follow-up pack/resource reads. |
| Related tests and targeted commands | Coding agents are judged by whether their patch is testable; users expect likely tests and runnable commands, not only source files. | HIGH | Include confidence and reason. Improve package-manager/workspace inference and expose weak-command diagnostics rather than pretending certainty. |
| Budgeted context packs | Agents need brief, standard, and deep context depending on task complexity and model budget. | MEDIUM | Keep Markdown and JSON outputs. Packs may include safe snippets, but every snippet path must be revalidated against current safe inventory before read. |
| Current-diff awareness | Real agent sessions often start from in-progress changes; users expect changed safe files to anchor bugfix, review, and continuation tasks. | LOW | Keep source text out of `current_diff`; use safe changed paths as anchors for plans and related expansion. |
| Source-free traces and eval reports | Trust requires measuring retrieval without storing prompts or source snippets. | MEDIUM | Keep traces under local ctxpack state. Add retention and `--no-trace`/config controls so read-like operations remain operationally predictable. |
| Operational diagnostics | A weak plan should say whether the cause is stale cache, unreadable files, missing git history, skipped large files, parse gaps, or low-information task text. | MEDIUM | This is table stakes for post-MVP trust because otherwise users cannot distinguish product failure from repository/task ambiguity. |
| Real client compatibility | The product surface is agent-native, so CLI-only success is insufficient. | MEDIUM | Maintain smoke paths for Codex CLI, Claude Code, Cursor/OpenCode-style adapters, and explicit `repo` arguments because MCP servers may launch outside the project cwd. |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required for a generic repo search tool, but critical for proving ctxpack is better than agents doing grep themselves.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Measured lift over lexical/grep baseline | Converts "better context" from a claim into evidence. | HIGH | Make historical eval the product scorecard: File Recall@5/10, Source Recall, Test Recall, test recommendation rate, missing-role breakdown, and ctxpack-vs-lexical lift. This is the top post-MVP differentiator. |
| Graph-first context expansion | Finds related implementation, tests, examples, configs, and constraints that text search misses. | HIGH | Promote dependency edges, co-change clusters, symbol relationships, and current-diff anchors into first-class ranking features rather than only risk-flag evidence. |
| Evidence-set compiler, not search results | Agents need the smallest useful evidence set, ordered by actionability. | HIGH | Compile target files, related tests, examples, constraints, commands, risk flags, and checklist sections into one task-conditioned object. |
| Signal attribution and rationale | Lets users and agents understand why a file was recommended. | MEDIUM | For every selected file/test, expose signals such as lexical, symbol, path anchor, current diff, dependency, co-change, test mapping, and examples. This makes debugging retrieval possible. |
| Source-free provenance | Enables cloud-agent handoff and local audit without leaking source or prompt text. | MEDIUM | Preserve `repoId`, `taskHash`, `targetAgent`, privacy status, and source-free cards/traces. This is stronger than generic codebase indexing when privacy matters. |
| Agent-specific shaping | Different clients load instructions, rules, prompts, and MCP resources differently. | MEDIUM | Generate thin Codex/Claude/Cursor/OpenCode/GitHub Copilot guidance around one stable core contract instead of branching product logic per agent. |
| Dynamic context discovery assets | Cursor's 2026 direction favors fewer static tokens and more pull-on-demand context. | MEDIUM | Keep static AGENTS/rules small. Put deeper packs, cards, file slices, symbol resources, and diagnostics behind discoverable MCP/resources so agents fetch only when needed. |
| Freshness-aware incremental local cache | Users trust results only if active repo changes are reflected. | HIGH | Add cheap invalidation using inventory metadata, ignore-file hashes, file mtimes/sizes/hashes, tool version, and scan options. Rebuild or warn before plans/search use stale inventory. |
| Configurable privacy policy | Local-first value depends on repo-specific secret and generated-file conventions. | MEDIUM | Move hard-coded denylist into a tested policy module with default conservative rules plus opt-in project config for secret paths, generated paths, vendored paths, and safe exceptions. |
| Historical retrieval gap reports | Makes roadmap work data-driven by showing which roles/signals are missed. | MEDIUM | Rank recurring misses by role and path family, e.g. docs/config/test files lagging source recall. Feed this into feature prioritization. |
| Persistent/reconstructable MCP pack resources | Real MCP clients reconnect, parallelize, and launch multiple server processes. | MEDIUM | Either persist pack metadata by task id or make pack resources regenerate from source-free plan/provenance. This differentiates ctxpack from session-fragile demos. |
| Local context cards for disconnected/cloud agents | Gives non-local or disconnected agents source-free orientation without uploading repository contents. | LOW | Keep deterministic `.ctxpack/cards/` summaries for repo overview, tests, and dependency graph. Treat cards as handoff artifacts, not full source context. |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Autonomous editing, test execution, dependency install, or commits | Looks like a complete coding agent. | Duplicates Codex/Claude/Cursor responsibilities and breaks the read-only trust contract. | Stay a context compiler; recommend files, tests, commands, and constraints. |
| Cloud indexing, cloud embeddings, or cloud reranking by default | Promises semantic search and large-repo scale. | Undercuts local-first privacy and creates vendor/backend trust questions. | Default to local lexical/symbol/graph/history; consider optional local semantic search only if eval proves lift. |
| Standalone chat app or editor replacement | Gives a visible product surface. | Users already work in coding agents and IDEs; a separate daily UI adds adoption friction. | Keep AGENTS.md, MCP, cards, and thin native adapters as the product surface. |
| Giant static repo map injected into every session | Feels comprehensive. | Consumes tokens, becomes stale, and can reduce instruction adherence. | Use compact always-on guidance plus dynamic MCP resources/packs. |
| Large MCP tool catalog | More tools seem more capable. | Long tool descriptions bloat context and increase client decision complexity. | Maintain a small tool surface with typed parameters and richer resources behind it. |
| Sourceful telemetry or prompt logging | Easier debugging and eval. | Violates privacy expectations and blocks use on sensitive repositories. | Store task hashes, path labels, roles, signal metadata, and source-free metrics. |
| Silent fallback to empty results | Avoids surfacing errors. | Agents misread infrastructure failures as "nothing relevant exists." | Return structured warnings for stale cache, unreadable files, parse failures, git timeouts, and skipped files. |
| Language-perfect static analysis as the next goal | Sounds rigorous. | It can consume months before proving user value and still miss framework conventions. | Add parser-backed improvements only where historical eval or dogfood gaps show retrieval lift. |
| Insecure one-click MCP/global config mutation | Makes setup frictionless. | Local MCP server commands execute with user privileges; hidden global mutation damages trust. | Print/copy explicit config snippets, show exact commands, and keep repo-local generated artifacts. |
| Indexing every file by default including generated/vendor/secret-like paths | Maximizes recall. | Pollutes rankings, slows indexing, and risks source/secret exposure. | Exclude by default, report exclusions, and require explicit opt-in with visible privacy status. |

## Feature Dependencies

```
Safe inventory + privacy policy
    |--requires--> Freshness/invalidation metadata
    |--enables--> Hybrid retrieval
    |--enables--> MCP file/symbol resources
    `--enables--> Safe snippets in packs

Hybrid retrieval
    |--requires--> Lexical + symbol + path/current-diff anchors
    |--enhanced-by--> Dependency graph + co-change history
    |--enhanced-by--> Related test inference
    `--feeds--> Task-conditioned context plan

Task-conditioned context plan
    |--feeds--> Budgeted context packs
    |--feeds--> MCP pack resources
    |--feeds--> Source-free traces
    `--requires--> Signal attribution + diagnostics

Historical eval
    |--requires--> Source-free labels and traces
    |--measures--> Hybrid retrieval lift
    `--prioritizes--> Graph/test/example/constraint improvements

Agent-native adapters
    |--requires--> Stable core contracts
    |--requires--> Small static instructions
    `--enhanced-by--> Dynamic pack/resources/cards

Cloud indexing by default
    `--conflicts--> Local-first privacy contract
```

### Dependency Notes

- **Safe inventory requires freshness metadata:** every search, plan, pack, file resource, symbol resource, and current-diff expansion inherits privacy and correctness from inventory. Stale inventory is therefore a feature blocker, not only a cache bug.
- **Graph/test expansion requires hybrid retrieval:** graph edges and related tests should rerank or expand already plausible candidates; used alone, they can flood plans with noisy neighbors.
- **Historical eval requires source-free labels:** the product cannot rely on sourceful logs to prove retrieval quality because privacy is part of the value proposition.
- **Agent-specific shaping requires stable contracts:** adapters should change prompts and setup files, not fork the planner output schema.
- **Dynamic context discovery requires small static instructions:** if AGENTS/rules become huge, ctxpack loses the main advantage of on-demand MCP resources and budgeted packs.
- **Cloud indexing conflicts with local-first defaults:** optional remote features can be revisited later, but default behavior must remain local-only.

## MVP Definition

### Current MVP Baseline (Already Present)

Minimum viable product already exists in this brownfield codebase.

- [x] Agent-native initialization through `AGENTS.md`, `.ctxpack/ctxpack.toml`, and thin adapters.
- [x] MCP tools/resources/prompts around task prep, search, related files, packs, tests, and current diff.
- [x] Safe local inventory with generated/sensitive exclusions and ignore-file support.
- [x] Hybrid retrieval across lexical, symbols, related tests, dependency hints, current diff, and co-change hints.
- [x] Budgeted context packs with Markdown/JSON output and source-free provenance.
- [x] Source-free traces, historical eval, and deterministic context cards.

### Add After Validation (Post-MVP P1)

Features to add now because they directly determine whether ctxpack beats agent grep.

- [ ] Graph-first ranking lift - promote dependency/co-change/test signals into file selection and prove File Recall@5/10 lift over lexical baseline.
- [ ] Inventory freshness and cache diagnostics - prevent stale or unsafe recommendations in active repositories.
- [ ] Weak-plan diagnostics - explain missing git, parse gaps, skipped files, unreadable files, stale cache, and low-signal prompts.
- [ ] Privacy policy hardening - broaden secret filename coverage and support repo-specific policy config.
- [ ] Signal attribution - show why each file/test/command was selected.

### Future Consideration (v2+)

Features to defer until measured local context quality is strong.

- [ ] Optional local semantic retrieval - add only if historical eval shows lift beyond lexical+symbol+graph and it remains local by default.
- [ ] Parser-backed language plugins - prioritize languages and constructs that historical eval identifies as recurring misses.
- [ ] Multi-repo/workspace context - defer until single-repo freshness, graph, and privacy are reliable.
- [ ] Team/shared index reuse - useful for large repos, but it introduces trust, access-control, and cache-coherency complexity.
- [ ] Hosted enterprise sync/admin - outside the current local-first product contract.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Graph-first ranking lift | HIGH | HIGH | P1 |
| Inventory freshness/invalidation | HIGH | HIGH | P1 |
| Operational weak-plan diagnostics | HIGH | MEDIUM | P1 |
| Privacy policy hardening/config | HIGH | MEDIUM | P1 |
| Signal attribution per recommendation | HIGH | MEDIUM | P1 |
| Related test command confidence improvements | HIGH | HIGH | P1 |
| Persistent/reconstructable MCP pack resources | MEDIUM | MEDIUM | P2 |
| Historical gap reports by role/signal | MEDIUM | MEDIUM | P2 |
| Agent-specific adapter refinements | MEDIUM | MEDIUM | P2 |
| Context card expansion | MEDIUM | LOW | P2 |
| Optional local semantic retrieval | MEDIUM | HIGH | P3 |
| Multi-repo workspace context | MEDIUM | HIGH | P3 |
| Team/shared index reuse | MEDIUM | HIGH | P3 |
| Hosted backend/admin | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for post-MVP proof
- P2: Should have once quality and reliability are stable
- P3: Future consideration after local-first value is proven

## Competitor Feature Analysis

| Feature | Cursor | Claude/GitHub/Copilot/VS Code | Aider/Sourcegraph/Continue | Our Approach |
|---------|--------|--------------------------------|----------------------------|--------------|
| Instruction files | Rules, skills, and context discovery are part of the agent harness. | Claude uses `CLAUDE.md`; GitHub/VS Code support repository instructions and `AGENTS.md`. | Aider has repo conventions but less cross-agent instruction standardization. | Generate portable `AGENTS.md` plus thin native adapters; keep static files small and source-controlled. |
| Codebase retrieval | Cursor emphasizes codebase indexing, semantic search, secure large-repo indexing, and dynamic context discovery. | Coding agents can inspect files and use MCP/tooling, but retrieval quality varies by client. | Sourcegraph Cody uses search-backed context; Continue historically combined embeddings and keyword search; Aider uses repo maps. | Stay local-first and typed: lexical + symbols + graph + tests + history + current diff, measured against a lexical baseline. |
| Context budgeting | Cursor explicitly optimizes dynamic context and tool-token usage. | Claude docs warn instruction files consume context and favor concise/path-scoped rules. | Aider repo maps compress whole-repo symbols into context. | Budgeted packs and resources should let agents pull brief/standard/deep context on demand. |
| Test and validation selection | Often handled by the agent reading repo scripts or IDE/test integrations. | Instruction files commonly encode build/test commands. | Search tools can find tests but usually do not compile targeted validation plans. | Make likely tests, command confidence, and validation plans first-class outputs. |
| Privacy posture | Cursor documents secure indexing mechanics but indexing may involve service-side components depending on product mode. | Claude/Copilot docs focus on instruction and client behavior, not ctxpack-style source-free eval. | Continue legacy codebase indexing used local embeddings by default; Sourcegraph can be managed/self-hosted. | Default local-only, source-free traces/cards, safe snippets only after inventory filtering. |
| Quality proof | Cursor publishes eval-style claims for semantic search and token reductions. | Client docs describe mechanisms more than retrieval metrics. | Aider/Sourcegraph/Continue describe context mechanisms, not per-repo historical recall lift. | Use `ctxpack eval history` as the proof engine and roadmap feedback loop. |

## Sources

- Project context: `.planning/PROJECT.md`, `.planning/codebase/ARCHITECTURE.md`, `.planning/codebase/CONCERNS.md`, `README.md` (HIGH confidence for current ctxpack state).
- Model Context Protocol server concepts: https://modelcontextprotocol.io/docs/learn/server-concepts (HIGH confidence for MCP tools/resources/prompts shape).
- Model Context Protocol security best practices: https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices (HIGH confidence for least-privilege/local-server safety implications).
- Claude Code memory/instructions docs: https://code.claude.com/docs/en/memory (HIGH confidence for `CLAUDE.md`, path rules, `AGENTS.md` import, and instruction-size guidance).
- GitHub Copilot repository instructions: https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/add-custom-instructions/add-repository-instructions (HIGH confidence for `AGENTS.md` and repository/path-specific instructions).
- VS Code Copilot custom instructions: https://code.visualstudio.com/docs/copilot/customization/custom-instructions (HIGH confidence for always-on and file-based instruction patterns).
- Cursor dynamic context discovery, Jan 6 2026: https://cursor.com/blog/dynamic-context-discovery (MEDIUM confidence; vendor research/blog, but directly relevant to context budgeting).
- Cursor secure codebase indexing, Jan 27 2026: https://cursor.com/blog/secure-codebase-indexing (MEDIUM confidence; vendor research/blog, useful for large-repo indexing expectations).
- Aider repository map docs: https://aider.chat/docs/repomap.html (HIGH confidence for repo-map feature framing).
- Sourcegraph Cody context docs: https://sourcegraph.com/docs/cody/core-concepts/context (HIGH confidence for search-backed context framing).
- Continue deprecated codebase provider docs: https://docs.continue.dev/reference/deprecated-codebase (MEDIUM confidence for historical local embedding + keyword expectations; feature is explicitly deprecated).

---
*Feature research for: local-first agent-native repo context compiler*
*Researched: 2026-05-13*
