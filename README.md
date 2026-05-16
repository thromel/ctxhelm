# Repo Context Packer

Repo Context Packer is a local-first, read-only context broker for coding agents.

The MVP exposes compact task context through:

- `AGENTS.md` for portable static instructions
- MCP tools/resources/prompts for dynamic context
- Thin native adapter files for Codex, Claude Code, Cursor, and OpenCode

ctxpack does not edit source code, run your project tests, install dependencies, or mutate global agent configuration. It writes repo-local guidance, optional adapter snippets, and local ctxpack state.

## Install v1.1.0

The v1.1.0 install path is a prebuilt GitHub Releases-style archive named like `ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz` or `ctxpack-v1.1.0-x86_64-unknown-linux-gnu.tar.gz`.

Download the archive and checksum file for your platform, then verify the SHA-256 checksums:

```bash
shasum -a 256 -c sha256sums.txt
sha256sum -c sha256sums.txt
```

Extract the archive and put the binary on your `PATH`:

```bash
tar -xzf ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz
install -m 0755 ctxpack-v1.1.0-aarch64-apple-darwin/ctxpack ~/.local/bin/ctxpack
ctxpack --version
ctxpack --help
```

The expected version diagnostic is `ctxpack 1.1.0`. See [docs/release.md](docs/release.md) for release details, source-build fallbacks, and maintainer packaging checks.

## Install To First Pack

Start from the installed `ctxpack` binary and an existing git repository:

```bash
ctxpack --version
ctxpack --help
export REPO=/path/to/repo
```

Initialize repo-local guidance and optional agent snippets:

```bash
ctxpack init --repo "$REPO" --cursor --claude --opencode
ctxpack setup-check --repo "$REPO" --cursor --claude --opencode
```

Ask for a task plan with an explicit repo and, when you know it, an active file path:

```bash
ctxpack prepare-task "fix requireSession bug" \
  --repo "$REPO" \
  --mode bug-fix \
  --path src/auth/session.ts
```

Materialize a compact context pack for the same task:

```bash
ctxpack get-pack "fix requireSession bug" \
  --repo "$REPO" \
  --mode bug-fix \
  --budget brief
```

For the longer walkthrough, including setup validation, deterministic MCP proof context, and how to interpret pack options, see [docs/quickstart.md](docs/quickstart.md).

## More Docs

- [First-pack quickstart](docs/quickstart.md)
- [Release and install guide](docs/release.md)
- [Agent setup matrix](docs/agent-setup.md)
- [Retrieval benchmarking](docs/benchmarking.md)
- [Storage](docs/storage.md)
- [Repo memory](docs/memory.md)
- [Local semantic retrieval](docs/semantic.md)
- [Parser and precision edges](docs/precision.md)
- [Troubleshooting](docs/troubleshooting.md)

## MCP Runtime

Start the local stdio MCP server:

```bash
ctxpack serve-mcp
```

Implemented MCP tools:

- `prepare_task`
- `search`
- `related`
- `get_pack`
- `related_tests`
- `current_diff`

`search` returns compact, source-free local matches from safe inventoried files and symbols. Use `kinds: ["file"]` or `kinds: ["symbol"]` to narrow the result set when an agent needs a specific evidence type. Pass `semantic: true` only when the agent explicitly wants local semantic file candidates for a conceptual query.

`current_diff` returns safe changed path lists only. Paths excluded by ignore, generated, sensitive, or other safe-inventory policy are summarized by count and source text is never returned.
`prepare_task` and `get_pack` can also accept `includeCurrentDiff: true` to add those safe changed paths as context anchors without returning source text.

`related` can expand from a `path`, `symbol`, or `includeCurrentDiff: true`. Symbol expansion resolves safe symbol matches first, and current-diff expansion uses safe changed paths; both return related tests, dependency edges, and co-change hints around the resolved file paths.
If local git history is unavailable, `related` still returns non-history context and includes a warning instead of failing the whole call.

Implemented MCP resources include `ctxpack://repo/summary`, package-aware `ctxpack://repo/test-map`, `ctxpack://repo/dependency-graph`, `ctxpack://pack/guide`, session-scoped `ctxpack://pack/<task-id>/<budget>` resources returned by `prepare_task` for brief, standard, and deep packs, safe file slices, and symbol search. Implemented prompts cover bugfix, feature, refactor, review, test-writing, and explanation workflows.

## Client Integration Status

Current local smoke status:

- Deterministic protocol proof is the required release gate: direct JSON-RPC/MCP calls verify `prepare_task`, `get_pack`, `search`, `related`, `related_tests`, `current_diff`, and same-session pack-resource reads with an explicit `repo`.
- Codex CLI `0.130.0`: optional real-client smoke support can require source-free server-side evidence for both `prepare_task` and `get_pack` with an isolated `CODEX_HOME`.
- Claude Code `2.1.140`: optional real-client smoke support can require source-free server-side evidence for both `prepare_task` and `get_pack` with strict MCP config.

When using ctxpack through MCP, pass the active repository path as `repo` whenever the client knows it. Some clients launch MCP servers from a different working directory than the project they expose.

## Safe Inventory

Build the local file inventory for a repository:

```bash
ctxpack index --repo /path/to/repo
```

The inventory respects `.gitignore`, `.ctxpackignore`, and `.cursorignore`, excludes sensitive/generated files by default, and writes JSON under `~/.ctxpack/repos/<repo-id>/inventory.json`.

To also sync source-free file metadata into the local SQLite store:

```bash
ctxpack index --repo /path/to/repo --store
ctxpack storage status --repo /path/to/repo
```

To also build local source-free semantic vector metadata:

```bash
ctxpack index --repo /path/to/repo --semantic
ctxpack search "payment webhook validation" --repo /path/to/repo --semantic
```

See [docs/storage.md](docs/storage.md), [docs/semantic.md](docs/semantic.md), and [docs/precision.md](docs/precision.md) for storage location, privacy guarantees, semantic provider details, parser/precision-edge support, repair/reset commands, and release-gate smoke coverage.

Generated and sensitive files require explicit opt-in:

```bash
ctxpack index --repo /path/to/repo --include-generated --include-sensitive
```

## Lexical Search

Search the safe inventory:

```bash
ctxpack search "requireSession" --repo /path/to/repo --limit 5
```

If no inventory exists for the repo, `ctxpack search` builds one using the safe default inventory rules before searching.

## Symbol Index

Extract language-aware symbols from safe inventoried files:

```bash
ctxpack symbols --repo /path/to/repo --limit 20
```

Search symbols by name, path, or signature:

```bash
ctxpack symbols --repo /path/to/repo --query requireSession --limit 5
```

The current local extractor covers TypeScript/JavaScript, Python, Rust, Go, Java, and Kotlin definitions. MCP symbol resources use the same symbol search path through `ctxpack://symbol/<query>`.

## Related Tests

Find likely tests for changed source files:

```bash
ctxpack related-tests src/auth/session.ts --repo /path/to/repo
```

The result includes confidence, a reason, and a best-effort targeted test command.
For JavaScript and TypeScript repos, ctxpack now checks nearby `package.json` scripts and package-manager lockfiles to prefer commands such as `pnpm vitest run <test>` or `npm test -- <test>` instead of assuming a single runner.

The MCP `ctxpack://repo/test-map` resource uses the same package-aware command inference for safe inventoried test files.

## Git Co-Change Hints

Find files that have changed together in local git history:

```bash
ctxpack co-changes src/auth/session.ts --repo /path/to/repo --limit 5
```

Co-change hints read only local git metadata and are filtered through the safe inventory.

## Dependency Graph

Inspect safe local import edges around a file:

```bash
ctxpack dependencies src/auth/session.ts --repo /path/to/repo --limit 10
```

Return the current safe dependency graph:

```bash
ctxpack dependencies --all --repo /path/to/repo --limit 50
```

Dependency edges are inferred from local TypeScript/JavaScript, Python, Rust, Java, and Kotlin imports in safe source/test files. External packages, generated files, sensitive files, and ignored files are excluded by default. MCP clients can request dependency expansion through `related` with `include: ["dependencies"]`, and can read the repository graph resource at `ctxpack://repo/dependency-graph`.

Repositories with local SCIP/LSP-derived edge exports can import source-free precision edges:

```bash
ctxpack precision import --repo /path/to/repo --input /path/to/precision-edges.json
ctxpack dependencies src/auth/middleware.ts --repo /path/to/repo
```

## Context Plan

Prepare a task-conditioned context plan:

```bash
ctxpack prepare-task "fix requireSession bug" --repo /path/to/repo --mode bug-fix
```

Pass active editor files as repeatable anchors when the host agent knows them:

```bash
ctxpack prepare-task "fix redirect behavior" --repo /path/to/repo --mode bug-fix --path src/auth/middleware.ts
```

Use safe current-diff paths as anchors for review or in-progress work:

```bash
ctxpack prepare-task "review current auth changes" --repo /path/to/repo --mode review --current-diff
```

The plan fuses active path anchors, symbol search, lexical search, related tests, local dependency edges, and local co-change hints into target files, line hints, validation commands, risk flags, and pack resource options. MCP clients can pass the same active/open files through the `paths` array on `prepare_task`.

For MCP clients, the `packOptions[*].resourceUri` values returned by `prepare_task` are loadable during the same MCP server session for brief, standard, and deep budgets. Add `.json` to a returned pack URI to read the structured pack resource instead of Markdown.

## Context Pack

Materialize a budgeted context pack:

```bash
ctxpack get-pack "fix requireSession bug" --repo /path/to/repo --mode bug-fix --budget brief
```

Use `--format json` for structured output. `get-pack` also accepts repeatable `--path <file>` anchors, and the MCP `get_pack` tool accepts the same `paths` array.
Structured and Markdown packs include source-free provenance fields: `repoId`, `taskHash`, and `targetAgent`.

## Context Cards

Generate optional repo-committable cards for cloud or disconnected agent contexts:

```bash
ctxpack cards generate --repo /path/to/repo
```

This writes `.ctxpack/cards/repo-overview.md`, `.ctxpack/cards/testing.md`, `.ctxpack/cards/dependency-graph.md`, and domain cards. Cards are deterministic, local-only, and source-snippet-free; they summarize safe inventory paths, roles, symbols, test commands, and local dependency edges. `cards generate` also stores matching source-free memory metadata for task-conditioned selection.

Generate and review experience cards from source-free local traces:

```bash
ctxpack memory generate-experience --repo /path/to/repo
ctxpack memory list --repo /path/to/repo
ctxpack memory approve experience:<task-hash> --repo /path/to/repo
```

`prepare-task`, `get-pack`, and MCP pack resources can include fresh approved or deterministic memory under `selectedMemory` and a separate `Selected memory` pack section. MCP also exposes `ctxpack://repo/memory`. See [docs/memory.md](docs/memory.md).

## Local Eval Traces

`prepare-task`, `get-pack`, and the matching MCP tools append source-free local traces under `~/.ctxpack/repos/<repo-id>/traces.jsonl`.

Inspect recent traces:

```bash
ctxpack eval traces --repo /path/to/repo --limit 20
```

Generate a manual dogfood checklist from recent traces:

```bash
ctxpack eval checklist --repo /path/to/repo --limit 5
```

Traces store task hashes, task type, target agent label, recommended files/tests/commands, optional pack id, optional budget, and created time. They do not store task text or source snippets.

Run a source-free historical retrieval eval over recent local commits:

```bash
ctxpack eval history --repo /path/to/repo --limit 20 --budget 10
```

Run a named source-free benchmark suite over multiple local repositories:

```bash
ctxpack eval benchmark --config .ctxpack/benchmarks/retrieval-quality.json
```

Compare two benchmark JSON reports and flag configured regression thresholds in the report:

```bash
ctxpack eval compare --base-report previous.json --head-report current.json --threshold fileRecallAt10=0.05
```

Generate the source-free product proof report:

```bash
ctxpack eval proof --config .ctxpack/benchmarks/retrieval-quality.json
```

See [docs/benchmarking.md](docs/benchmarking.md) for the suite JSON contract, RefactoringMiner-style setup, token ROI interpretation, gap families, and regression comparison.

```bash
ctxpack eval history --repo /path/to/repo --limit 20 --mode bug-fix
```

Use `--base <rev> --head <rev>` to freeze the evaluated commit range for apples-to-apples tuning on larger repositories.

This replays each commit subject through `prepare_task`, treats the commit's safe changed files as hidden labels, and reports File Recall@5/10, lexical and no-context baselines, ctxpack lift, Source Recall@5/10, Test Recall@5/10, signal ablations, token ROI by brief/standard/deep budget, test recommendation rate, low-information commit counts, top retrieval gaps by file role, and excluded generated/sensitive path counts. The report uses task hashes and path labels; it does not include source snippets.

## Development

Development commands require a source checkout:

```bash
cargo test --workspace
cargo run -p ctxpack -- --help
```
