# Codebase Concerns

**Analysis Date:** 2026-05-13

## Tech Debt

**Stale inventory cache as the default read path:**
- Issue: `load_or_build_inventory` loads `~/.ctxhelm/repos/<repo-id>/inventory.json` whenever it exists and only rebuilds when the file is missing. It does not compare current filesystem state, stored hashes, ignore files, or option changes before using cached entries.
- Files: `crates/ctxhelm-index/src/lib.rs`
- Impact: `lexical_search`, `symbol_search`, `related_tests`, `test_map`, `dependency_edges`, `prepare_context_plan`, MCP `search`, MCP `prepare_task`, and MCP `get_pack` can miss newly created files, include deleted paths, or use stale file roles until `ctxhelm index` refreshes the cache.
- Fix approach: Store inventory metadata such as tool version, options, ignore-file hashes, and scan timestamp; add a cheap invalidation check before `load_inventory` returns. For user-facing search and plan APIs, prefer rebuilding when tracked files or ignore config changed.

**Large single-file modules with mixed responsibilities:**
- Issue: Indexing, lexical search, symbol extraction, dependency inference, test command inference, git history, current diff, and trace storage live in one file. Compiler planning, historical eval, card generation, pack rendering, snippet rendering, and tests live in one file. MCP schema, transport, resources, tools, cache, and tests live in one file.
- Files: `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`, `crates/ctxhelm-mcp/src/lib.rs`
- Impact: Changes to privacy policy, retrieval scoring, MCP serialization, or pack content require editing very broad files. Regression risk is high because unrelated concerns share helpers and inline tests.
- Fix approach: Split along behavioral boundaries: inventory/privacy policy, search/scoring, symbols, dependencies, tests, git/history, traces; compiler planning, packs, cards, eval; MCP transport, schemas, tool handlers, resources. Keep public crate exports stable during the split.

**Hand-rolled language parsing and import resolution:**
- Issue: Symbol extraction and dependency graph construction use line-based parsing for TypeScript/JavaScript, Python, Rust, and Go rather than AST parsers. Import resolution covers common local imports but not aliases, workspaces, tsconfig paths, package exports, Python namespace packages, Rust module edge cases, or Go package layout.
- Files: `crates/ctxhelm-index/src/lib.rs`
- Impact: `symbol_search`, `dependency_edges`, `related_dependency_edges`, context planning risk flags, MCP `related`, and generated dependency cards can produce false negatives or misleading edges in realistic repositories.
- Fix approach: Keep the current parser as a fast fallback, but introduce language-specific parser traits behind focused fixtures. Add alias-aware JS/TS resolution and Rust module tests before expanding graph-dependent planning.

**Test command inference is heuristic and package-manager biased:**
- Issue: JavaScript test command inference defaults to `pnpm` when no lockfile is present, chooses command shapes from substring checks in the `test` script, and does not model monorepo package runners beyond nearest `package.json`.
- Files: `crates/ctxhelm-index/src/lib.rs`
- Impact: `related_tests`, `test_map`, MCP `ctxhelm://repo/test-map`, and context plans can recommend commands that do not run in npm/yarn/bun projects or workspaces with custom runners.
- Fix approach: Represent test command inference as structured confidence plus reason. Add fixtures for npm/yarn/bun, workspace packages, scripts with `--`, scripts that require file filters after a runner-specific separator, and missing lockfiles.

**Read-oriented operations have persistent write side effects:**
- Issue: `prepare-task`, `get-pack`, MCP `prepare_task`, and MCP `get_pack` append traces under `~/.ctxhelm/repos/<repo-id>/traces.jsonl`. Search-like calls also call `load_or_build_inventory`, which may write an inventory under `~/.ctxhelm/repos/<repo-id>/inventory.json`.
- Files: `crates/ctxhelm/src/main.rs`, `crates/ctxhelm-mcp/src/lib.rs`, `crates/ctxhelm-index/src/lib.rs`
- Impact: The product is described as a local-first read-only context broker, but normal context retrieval mutates user home state and can fail when the home directory is read-only or quota-limited.
- Fix approach: Document these writes as local cache/telemetry artifacts, add `--no-trace` or config-level trace control, make trace append failures non-fatal for MCP context retrieval, and add retention controls.

## Known Bugs

**Cached inventory can return deleted or role-changed files:**
- Symptoms: After a file is removed, renamed, moved into `dist/`, renamed to `.env`, or newly ignored, existing cached inventory entries still drive searches and plans until the inventory is explicitly rebuilt.
- Files: `crates/ctxhelm-index/src/lib.rs`
- Trigger: Run any API that creates an inventory, change repo files or ignore rules, then call `ctxhelm search`, `ctxhelm symbols`, `ctxhelm prepare-task`, MCP `search`, or MCP `prepare_task` without `ctxhelm index`.
- Workaround: Run `cargo run -p ctxhelm -- index --repo /path/to/repo` after file layout or ignore-policy changes.

**MCP pack resources are process-local and unavailable across sessions:**
- Symptoms: `prepare_task` returns `ctxhelm://pack/<task-id>/<budget>` URIs, but `resources/read` for those URIs only succeeds in the same MCP server process because the cache is an in-memory `OnceLock<Mutex<BTreeMap<...>>>`.
- Files: `crates/ctxhelm-mcp/src/lib.rs`
- Trigger: A client reconnects, starts another server process, or sends `resources/read` to a different ctxhelm process than the one that handled `prepare_task`.
- Workaround: Call MCP `get_pack` directly, or call `prepare_task` again in the same MCP session before reading the returned pack resource.

**Read failures silently look like weak matches:**
- Symptoms: Several paths use `fs::read_to_string(...).unwrap_or_default()`, so unreadable or non-UTF-8 files become empty content rather than warnings or errors.
- Files: `crates/ctxhelm-index/src/lib.rs`
- Trigger: Inventoried files are unreadable at search time, are binary with a recognized extension, or contain invalid UTF-8.
- Workaround: Rebuild inventory and inspect file permissions manually; there is no structured warning surfaced by search or planning APIs.

## Security Considerations

**Sensitive-file detection is a denylist:**
- Risk: Secret-like files that do not match the current name/path checks can enter the safe inventory and later be exposed through MCP file resources or context packs. Examples to add coverage for include `.npmrc`, `.pypirc`, `.netrc`, `id_rsa`, `id_ed25519`, `serviceAccountKey.json`, cloud credential JSON files, and inline-secret compose/config files.
- Files: `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-mcp/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`
- Current mitigation: `is_sensitive_path` excludes `.env`, `.env.*`, certificate/key extensions, dumps, SQL gzip files, and paths containing `secret` or `credentials`. MCP file reads re-check safe inventory before returning text.
- Recommendations: Move privacy classification into a dedicated policy module with explicit tests for known secret filename families, package-manager auth files, SSH keys, cloud credentials, and nested secret directories. Treat unknown dotfiles and credential-shaped JSON names conservatively unless opted in.

**Generated context packs intentionally return source snippets:**
- Risk: MCP `get_pack`, `ctxhelm get-pack`, and session-scoped pack resources include target and test snippets from safe files. This is useful for local agents, but it means the privacy guarantee depends entirely on correct inventory filtering.
- Files: `crates/ctxhelm-compiler/src/lib.rs`, `crates/ctxhelm-mcp/src/lib.rs`
- Current mitigation: Planning selects files from the safe inventory, and pack rendering reads paths from the plan. Eval traces avoid source text.
- Recommendations: Keep docs explicit that packs include safe source snippets. Add pack-level assertions that every snippet path is revalidated against current safe inventory immediately before reading, especially while the stale inventory behavior exists.

**External process execution trusts local `git` and `tar`:**
- Risk: Historical eval and git-history helpers spawn `git` and `tar` from `PATH`; compromised PATH or platform differences can affect behavior.
- Files: `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`
- Current mitigation: Commands use argument arrays rather than shell strings.
- Recommendations: Keep shell-free invocation, add clearer error surfaces, timeouts around historical eval archive/extract operations, and tests for missing `tar` or failing external commands.

## Performance Bottlenecks

**Full-file reads during inventory, search, symbol extraction, dependency graph, and pack rendering:**
- Problem: `build_inventory` reads every included file into memory to hash it. `lexical_search`, `extract_symbols`, `related_tests`, and `dependency_edges` read file contents repeatedly. Pack and file resources read full files before slicing.
- Files: `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`, `crates/ctxhelm-mcp/src/lib.rs`
- Cause: There are no file-size caps, streaming hashes, content caches, binary checks, or per-operation read budgets.
- Improvement path: Stream hashes during inventory; store size and skip/flag files over configurable limits; cache decoded text per operation; enforce snippet read limits before loading huge files; surface skipped-file counts in API results.

**Git history scans are sequential and timeout-sensitive:**
- Problem: Co-change hints scan up to 50 commits and run one `git diff-tree` per commit with a 250 ms timeout. Historical eval multiplies commit samples, parent snapshots, inventory work, lexical search, symbol search, dependency graph, and co-change hints.
- Files: `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`
- Cause: History analysis uses repeated external git commands instead of batched log formats or shared commit metadata.
- Improvement path: Batch file-change extraction with `git log --name-only` where possible, cache parsed commit file sets per repo/revision range, expose partial-result warnings, and keep limits applied before expensive work.

**Unbounded trace and pack-resource storage:**
- Problem: Eval traces append forever to `traces.jsonl`, and MCP cached pack resources accumulate in a process-wide map with Markdown and JSON copies for each budget.
- Files: `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-mcp/src/lib.rs`
- Cause: There is no retention, cache eviction, or maximum bytes setting.
- Improvement path: Add configurable trace retention by count/age/bytes. Evict pack resources by task id, LRU, or maximum total bytes. Include cache size in debug logs or resource diagnostics.

## Fragile Areas

**Safe inventory and privacy policy:**
- Files: `crates/ctxhelm-index/src/lib.rs`
- Why fragile: File role classification controls what can be indexed, searched, exposed as snippets, returned by current-diff anchors, and written into context cards. It is centralized but implemented as name/path string checks.
- Safe modification: Add table-driven tests before changing `is_sensitive_path`, `is_generated_path`, `classify_path`, or `language_for_path`. Include both positive and negative examples, and verify MCP pack/file behavior, not just inventory counts.
- Test coverage: Existing tests cover `.env`, `private.key`, generated paths, current diff filtering, and historical samples. Coverage is missing for package-manager auth files, SSH private key names, cloud credential JSON files, Docker compose secrets, and large/binary files.

**MCP JSON schemas and response compatibility:**
- Files: `crates/ctxhelm-mcp/src/lib.rs`, `crates/ctxhelm-core/src/contracts.rs`
- Why fragile: MCP clients depend on camelCase JSON, tool names, resource URI shapes, prompt names, and structuredContent. All handlers live together and most behavior is tested through JSON assertions.
- Safe modification: Preserve existing fields when adding new fields. Add tests for both `content[0].text` and `structuredContent` for every changed tool.
- Test coverage: Unit tests cover listing, initialize, tool calls, resources, prompts, and errors. Coverage is missing for multi-process resource reads, concurrent clients, malformed-but-valid JSON-RPC edge cases, and client-specific smoke tests in CI.

**Historical eval snapshot construction:**
- Files: `crates/ctxhelm-compiler/src/lib.rs`, `crates/ctxhelm-index/src/lib.rs`
- Why fragile: Historical evaluation builds parent snapshots from current inventory paths, extracts files through `git archive | tar`, and evaluates commit subjects. Deleted files or files outside the current inventory disappear from labels and snapshots.
- Safe modification: Keep source-free guarantees, but test commits with deleted files, renamed files, generated files, sensitive files, and files that existed only in historical revisions.
- Test coverage: Current tests cover source-free reports and excluded generated/sensitive files. Coverage is thin for renames, deletes, missing external tools, large histories, and timeout behavior.

## Scaling Limits

**Repository size:**
- Current capacity: Defaults are tuned for small-to-medium local repositories: search limits around 8-10, dependency graph resource capped at 200 edges, co-change scans at 50 commits, MCP request limits clamped to 50.
- Limit: Large monorepos with many safe files, large source files, deep JS workspaces, or long git histories will hit repeated full scans, stale cache risk, and incomplete dependency/test recommendations.
- Scaling path: Introduce incremental inventory updates, per-language indexes, stable cache invalidation, operation budgets, and batched git metadata.

**MCP session lifecycle:**
- Current capacity: Pack resources are useful inside one process and one active server cache.
- Limit: Clients that restart servers, parallelize MCP processes, or use different working directories can lose resource state or resolve the wrong repo when `repo` is omitted.
- Scaling path: Persist pack metadata by task id or make resource reads regenerate packs from source-free plan metadata. Continue recommending explicit `repo` arguments in generated prompts and docs.

## Dependencies at Risk

**System `git` and `tar`:**
- Risk: Core retrieval and evaluation features depend on external tools that may be missing, slow, old, or platform-incompatible.
- Impact: `co_change_hints`, `current_diff_summary`, `historical_commit_samples`, and `evaluate_historical_commits` can fail or silently drop history signals.
- Migration plan: Keep `git` as the source of truth, but centralize process execution with timeout, version/error diagnostics, and fallback behavior. Consider a Rust git library only if process overhead or platform issues become the bottleneck.

**Manual parsers instead of language parser libraries:**
- Risk: Retrieval quality depends on simple string parsing as repository language coverage expands.
- Impact: Symbol search and dependency graph recommendations degrade in modern TypeScript, Python, Rust, and Go codebases.
- Migration plan: Add parser-backed implementations behind the current `CodeSymbol` and `DependencyEdge` contracts; keep the current parser as a lightweight fallback.

## Missing Critical Features

**Cache invalidation and cache management:**
- Problem: There is no user-visible way to know whether inventory is fresh, why cached data was reused, or how large trace/cache artifacts are.
- Blocks: Reliable use in long-running agent sessions and large active repositories.

**Configurable privacy policy:**
- Problem: `InventoryOptions` only exposes generated/sensitive include booleans, while the actual policy is hard-coded.
- Blocks: Teams with repo-specific secret locations, generated-source conventions, vendored code policies, or compliance constraints.

**Operational diagnostics:**
- Problem: Search, planning, dependency, and history APIs do not consistently report skipped unreadable files, stale inventory, timed-out git commands, or partial graph coverage.
- Blocks: Users cannot tell whether a weak context plan means the task is underspecified, the repo is too large, the cache is stale, or a subsystem failed.

## Test Coverage Gaps

**End-to-end CLI behavior:**
- What's not tested: Full binary-level command execution for `ctxhelm index`, `search`, `prepare-task`, `get-pack`, `serve-mcp`, `cards generate`, and eval subcommands.
- Files: `crates/ctxhelm/src/main.rs`
- Risk: Clap wiring, output formats, path handling, and write side effects can regress while library unit tests stay green.
- Priority: High

**Cache freshness and filesystem mutation:**
- What's not tested: Behavior after files are created, deleted, renamed, moved into generated paths, made sensitive, or ignored after an inventory exists.
- Files: `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`, `crates/ctxhelm-mcp/src/lib.rs`
- Risk: Agents receive stale or unsafe recommendations.
- Priority: High

**Privacy corpus coverage:**
- What's not tested: Broader secret filename/path families, credential JSON files, package-manager auth files, SSH private keys, Docker compose inline secrets, and non-UTF-8 files with sensitive names.
- Files: `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-mcp/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`
- Risk: Sensitive file snippets can be returned through MCP file resources or packs.
- Priority: High

**Concurrency and multi-session MCP:**
- What's not tested: Parallel requests, cache growth, pack URI reads from a different process, client reconnects, and repo discovery when the MCP server cwd differs from the active project.
- Files: `crates/ctxhelm-mcp/src/lib.rs`
- Risk: Real client behavior differs from unit-level protocol smoke tests.
- Priority: Medium

**Large repository and large file stress tests:**
- What's not tested: Monorepos, huge source files, binary files with source-like extensions, long git histories, large dependency graphs, and many generated/sensitive exclusions.
- Files: `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`
- Risk: Context retrieval becomes slow, memory-heavy, or incomplete without clear user diagnostics.
- Priority: Medium

---

*Concerns audit: 2026-05-13*
