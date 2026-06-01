# ctxhelm Product Spec

Date: 2026-05-09
Status: Draft for user review
Artifact order: Product spec first, implementation roadmap second, initial repo scaffold third

## 1. Product Thesis

ctxhelm is an agent-native, local-first context layer for coding agents.

The product should not become another "ask my repo" app. Its job is to make existing coding agents choose better files, tests, examples, constraints, and context budgets before they edit code.

Core promise:

```text
Given a software task, produce the smallest evidence set that helps an existing coding agent make a correct change.
```

Primary users:

- Entry point: solo power users who already use Codex, Claude Code, Cursor, OpenCode, or similar agents.
- MVP optimization target: small engineering teams that want repeatable repo setup, shared agent instructions, and better context discipline without adopting a new daily app.

## 2. Product Boundary

ctxhelm is a read-only context broker.

It should:

- Inspect repository structure.
- Index safe, non-ignored source context.
- Classify coding tasks.
- Recommend target files, related tests, examples, constraints, and validation commands.
- Return small structured context plans and budgeted context packs.
- Integrate through AGENTS.md, MCP, and thin agent-native rule files.

It should not initially:

- Edit files.
- Run arbitrary build commands.
- Auto-commit changes.
- Replace Codex, Claude Code, Cursor, OpenCode, Aider, Continue, or editor agents.
- Upload source code to cloud services by default.
- Become a standalone daily chat UI.

The daily workflow should stay inside the user's chosen coding agent:

```text
Developer prompts existing agent
-> agent follows AGENTS.md or native rule
-> agent calls ctxhelm.prepare_task
-> ctxhelm returns target files, related tests, pack URI, and constraints
-> agent reads files with native tools
-> agent edits and validates with native tools
```

## 3. MVP Scope

The MVP is the smallest version that makes existing agents behave measurably better in real repositories.

### 3.1 Core Capabilities

1. Repo initialization
   - Detect repo root and package boundaries.
   - Generate `AGENTS.md` or append a bounded `ctxhelm` section safely.
   - Generate thin native rule files for Cursor, Claude Code, and OpenCode.
   - Provide Codex MCP config guidance as the first verified setup.

2. Read-only context engine
   - File inventory with ignore handling.
   - Sensitive and generated file exclusion.
   - Basic language and file-role classification.
   - Tree-sitter symbol extraction for TypeScript/JavaScript and Python first.
   - Lexical search over paths, symbols, identifiers, docs, and tests.
   - Basic source-to-test mapping from naming, imports, and directory conventions.
   - Git co-change hints from local history when available.

3. MCP runtime
   - `prepare_task`: classify the task and return a compact context plan.
   - `search`: find files, symbols, docs, tests, commits, or config entries.
   - `related`: expand around a file, symbol, or diff.
   - `get_pack`: return a brief, standard, or deep context pack.
   - `related_tests`: return likely tests and commands.
   - `current_diff`: summarize changed files and local diff metadata.

4. Context compiler
   - Task classification for bug fix, feature, refactor, review, test, and explain tasks.
   - Candidate fusion from anchors, lexical search, lightweight graph edges, tests, docs, and git history.
   - Budgeted Markdown and JSON packs.
   - Evidence labels explaining why each file or snippet was selected.
   - Privacy status on every pack.

5. Early team path
   - Repo-committable config at `.ctxhelm/ctxhelm.toml`.
   - Optional repo-committable generated context cards, enabled explicitly.
   - Repeatable onboarding command for another developer on the same repo.
   - Local session logging that records context decisions for later evaluation.

### 3.2 Non-Goals

- No autonomous edits.
- No cloud indexing by default.
- No VS Code or Cursor extension.
- No hosted team sync.
- No SCIP/LSP precision index.
- No cloud reranker.
- No large visual UI.
- No full benchmark suite yet.

## 4. Architecture

The first implementation should be split into focused packages or crates:

```text
ctxhelm-core
  repo scanning, ignore/privacy policy, file inventory, symbols, test mapping

ctxhelm-index
  lexical index, lightweight graph edges, git history/co-change data

ctxhelm-compiler
  task classifier, candidate fusion, budget manager, pack renderer

ctxhelm-mcp
  MCP tools/resources/prompts over the compiler
```

### 4.1 Storage

Persistent local state:

```text
.ctxhelm/ctxhelm.toml          repo config, committable when desired
~/.ctxhelm/repos/<repo-id>/    local private index/cache by default
```

Storage should start simple:

- SQLite for files, symbols, chunks, edges, tests, commits, packs, and episodes.
- Tantivy or equivalent for BM25 lexical search.
- No vector database in the default MVP.
- Optional vector retrieval only behind an experimental feature flag.

This keeps privacy, install size, and correctness tighter for the first version.

### 4.2 Runtime Flow

```text
agent calls prepare_task(task, active_context?)
-> classify task
-> collect anchors from explicit paths, current diff, and active hints
-> run lexical/symbol/doc/test search
-> expand graph-lite around high-confidence candidates
-> add related tests and git co-change hints
-> fuse, diversify, and rank candidates
-> return tiny ContextPlan plus pack resource URI
```

`get_pack` materializes a budgeted context object:

```text
Task restatement
Target file list
Validation commands
Constraints
Relevant snippets
Related tests
Docs/cards/history only if budget allows
Final checklist
Privacy status
```

The system remains read-only. It can suggest test commands, but the coding agent runs them using its own shell and permission model.

## 5. Data Contracts

The integration layer should depend on stable contracts, not on specific retrieval internals.

### 5.1 ContextPlan

```ts
type ContextPlan = {
  taskId: string
  taskType: "bug_fix" | "feature" | "refactor" | "review" | "test" | "explain"
  confidence: number
  targetFiles: TargetFile[]
  relatedTests: RelatedTest[]
  recommendedCommands: Command[]
  packOptions: PackOption[]
  missingInfoQuestions: string[]
  riskFlags: RiskFlag[]
  privacyStatus: PrivacyStatus
}
```

### 5.2 ContextCandidate

```ts
type ContextCandidate = {
  id: string
  kind: "file" | "symbol" | "test" | "doc" | "commit" | "diff" | "config"
  path?: string
  symbol?: string
  lineRange?: [number, number]
  score: number
  sourceScores: {
    lexical?: number
    graph?: number
    history?: number
    tests?: number
    active?: number
  }
  reason: string
  evidence: Evidence[]
}
```

### 5.3 ContextPack

```ts
type ContextPack = {
  id: string
  repoHash: string
  taskHash: string
  budget: "brief" | "standard" | "deep"
  targetAgent: "codex" | "claude-code" | "cursor" | "opencode" | "generic"
  tokenEstimate: number
  confidence: number
  warnings: string[]
  sections: PackSection[]
  privacyStatus: PrivacyStatus
}
```

## 6. MCP API

The MCP surface should stay intentionally small. More tools increase cognitive and context overhead for agents.

### 6.1 Tools

```text
ctxhelm.prepare_task
ctxhelm.search
ctxhelm.related
ctxhelm.get_pack
ctxhelm.related_tests
ctxhelm.current_diff
```

Tool output policy:

- `prepare_task`, `search`, `related`, and `related_tests` should be small enough to read inline.
- Larger content should be returned through resource URIs or `get_pack`.
- `prepare_task` is the default first call for non-trivial coding tasks.

### 6.2 Resources

```text
ctxhelm://repo/summary
ctxhelm://repo/test-map
ctxhelm://pack/{id}/brief
ctxhelm://pack/{id}/standard
ctxhelm://pack/{id}/deep
ctxhelm://file/{path}?lines=42-119
ctxhelm://symbol/{fqname}
```

### 6.3 Prompts

```text
ctxhelm.bugfix
ctxhelm.feature
ctxhelm.refactor
ctxhelm.review_diff
ctxhelm.write_tests
ctxhelm.explain_area
```

Prompts should orchestrate a workflow, not duplicate full repository context.

## 7. Integration Policy

The integration design should keep one portable behavior contract, then adapt it thinly per agent.

### 7.1 Universal Layer

`AGENTS.md` should contain stable, high-signal instruction:

```md
For non-trivial code changes, call ctxhelm.prepare_task before planning edits.
Use ctxhelm for likely target files, related tests, relevant examples,
architecture constraints, and validation commands.
Read actual files with the agent's native tools before editing.
Do not request deep packs unless the brief or standard pack is insufficient.
```

### 7.2 Codex

Codex remains a primary target, but current local smoke coverage only verifies MCP registration/config discovery for Codex CLI.

- Configure the local stdio MCP server.
- Use `AGENTS.md` as the repo instruction source.
- Keep `.ctxhelm/cards/` optional for cloud or disconnected contexts.
- Next verification target: a sample repo task where Codex calls `prepare_task`, reads target files, edits with native tools, and runs suggested validation.

### 7.3 Claude Code

- Current local smoke coverage verifies end-to-end MCP tool use with `.mcp.json`, `--strict-mcp-config`, and an explicit `repo` argument.
- Add `CLAUDE.md` or `.claude/commands/*` only when the user opts in.
- Expose MCP prompts as slash-command workflows.
- Do not auto-inject a full pack on every prompt.

### 7.4 Cursor

- Generate `.cursor/rules/ctxhelm.mdc`.
- Tell Cursor to use its own index for broad awareness and ctxhelm for task-specific ranked context, tests, history, and constraints.
- Keep rules short and version-controlled.

### 7.5 OpenCode

- Generate minimal `opencode.jsonc` or instruction wiring.
- Keep MCP tool count low.
- Use `AGENTS.md` as fallback behavior.

Integration success means the developer keeps using their existing agent and ctxhelm appears only as a context-planning call, not as a separate daily app.

## 8. Privacy and Safety

Default policy:

- Local indexing only.
- Local metadata only.
- Local lexical search and graph-lite retrieval.
- Cloud embeddings disabled.
- Cloud reranking disabled.
- Sensitive files excluded by default.

Files to exclude by default:

- `.env` and dotenv variants.
- Private keys, certificates, tokens, and local credentials.
- Local database dumps.
- Dependency folders.
- Build outputs.
- Generated/minified files unless explicitly enabled.
- Files ignored by `.gitignore`, `.cursorignore`, or `.ctxhelmignore`.

Every pack should include:

```json
{
  "privacyStatus": {
    "localOnly": true,
    "remoteEmbeddingsUsed": false,
    "remoteRerankingUsed": false,
    "redactionsApplied": 0
  }
}
```

## 9. Evaluation and Success Metrics

Integration comes first, but every MVP flow should leave enough local traces to evaluate context quality later.

Example local-only telemetry:

```json
{
  "task_hash": "...",
  "task_type": "bug_fix",
  "pack_id": "...",
  "recommended_files": ["src/auth/middleware.ts"],
  "recommended_tests": ["tests/auth/redirect.test.ts"],
  "agent_target": "codex",
  "budget": "brief",
  "tool": "prepare_task",
  "created_at": "..."
}
```

No source code leaves the machine by default.

### 9.1 MVP Success Metrics

- Setup success: user can initialize a repo and connect at least one target agent through MCP; current verified end-to-end client is Claude Code, while Codex CLI registration is verified separately.
- Agent adoption: agent calls `prepare_task` for non-trivial tasks via `AGENTS.md` or native rules.
- Context compactness: `prepare_task` stays under roughly 1,500 tokens; standard packs remain budgeted.
- File relevance: in dogfood tasks, ctxhelm target files overlap files the agent actually reads or edits.
- Test relevance: ctxhelm recommends at least one plausible targeted validation command when tests exist.
- Privacy trust: ignored, sensitive, and generated files are excluded and pack privacy status is visible.
- Team repeatability: another developer can clone the repo, install ctxhelm, and reuse repo config without inheriting private local indexes.

### 9.2 Deferred Evaluation

- Historical PR retrieval eval.
- Agent A/B comparison with and without ctxhelm.
- Token and cost reduction.
- Graph/history/test ablation.
- SWE-bench-style external benchmarking.

The spec should not claim a specific performance lift until there is measurement. Targets like "15-30% more tasks solved" can remain aspirations, not launch claims.

## 10. Open Product Questions

1. Should generated context cards be committed by default for team repos, or always opt-in?
2. Should the first install flow configure all supported agents at once, or ask the user which agents to wire?
3. How much local telemetry should be retained by default before pruning?
4. Should `ctxhelm.prepare_task` accept active editor/open-file hints through each agent adapter, or only through explicit user-provided paths in MVP?
5. What should the first dogfood benchmark repo be?

## 11. Source References

- AGENTS.md official site: https://agents.md/
- Model Context Protocol tools: https://modelcontextprotocol.io/docs/concepts/tools
- Model Context Protocol resources: https://modelcontextprotocol.io/docs/concepts/resources
- Model Context Protocol prompts: https://modelcontextprotocol.io/docs/concepts/prompts
- OpenAI Docs MCP quickstart: https://platform.openai.com/docs/docs-mcp
- OpenAI Codex configuration reference: https://developers.openai.com/codex/config-reference
- Claude Code MCP documentation: https://code.claude.com/docs/en/mcp
- Claude Code slash commands: https://code.claude.com/docs/en/slash-commands
- Cursor rules documentation: https://docs.cursor.com/context/rules
- Cursor MCP documentation: https://docs.cursor.com/advanced/model-context-protocol
- OpenCode rules documentation: https://opencode.ai/docs/rules/
- OpenCode configuration documentation: https://opencode.ai/docs/config

## 12. Approval Gate

This product spec is the first artifact. After user review and approval, the next artifact should be an implementation roadmap. The initial repo scaffold comes after the roadmap is approved.
