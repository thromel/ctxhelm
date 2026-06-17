# ctxhelm

ctxhelm, powered by the `ctxhelm` CLI, is a local-first, read-only context
compiler for coding agents.

It does not try to replace Codex, Claude Code, Cursor, OpenCode, Aider, or any
other coding agent. It makes those agents better at deciding what to inspect
first: target files, related tests, graph neighbors, history hints, constraints,
and progressive context packs.

The package, binary, MCP namespace, local state directory, and install channel
are now `ctxhelm`. The name reflects the product's job: steering coding agents
toward the right context before they edit.

## Why ctxhelm

Agents can grep, search, and read files on their own. ctxhelm is useful when
that is not enough:

- It turns a task into a ranked context plan instead of a generic search result.
- It combines exact lexical matches with symbols, dependency edges, related
  tests, git co-change hints, local semantic metadata, memory cards, and current
  diff anchors.
- It returns source-free evidence, confidence, warnings, validation commands,
  and MCP resource URIs so agents can read progressively instead of dumping the
  repo into context.
- It stays read-only: ctxhelm does not edit code, run user project commands, or
  mutate global agent configuration.

Current proof snapshot:

- Public `v2.4.5` archive install is current and verified through checksum,
  archive, temporary install, version/help, doctor, and first-pack checks.
- The four-repo product proof reports zero protected target misses across
  RefactoringMiner, ctxhelm, ReAgent, and VeriSchema.
- The agent-evidence retrieval channel beats or matches lexical on every
  measured corpus, with average Recall@10 delta `+0.19379663`.
- Codex CLI `0.137.0` currently produces real explicit-repo `prepare_task` and
  `get_pack` MCP calls and a source-free outcome lift in the retry-enabled
  Codex R&D breadth suite: all ctxhelm lanes reach `1.00` average target-read
  coverage versus baseline `0.7083333333333333`, with no evidence misses,
  evidence-only targets, under-read targets, forbidden commands, client
  failures, or rate limits.
- Claude Code `2.1.163` is currently rate-limited in source-free availability
  and in the preflight-disabled four-task paired suite; older Claude workflow
  proof remains historical evidence only.

## Product Surface

ctxhelm exposes compact task context through:

- `AGENTS.md` for portable static instructions
- MCP tools/resources/prompts for dynamic context
- Thin native adapter files for Codex, Claude Code, Cursor, and OpenCode

ctxhelm writes repo-local guidance, optional adapter snippets, and local ctxhelm
state only.

## Install v2.4.5

On Apple Silicon macOS, install from the public Homebrew tap:

```bash
brew tap thromel/tap
brew install ctxhelm
ctxhelm --version
ctxhelm --help
```

The archive install path remains available for manual verification and
non-Homebrew workflows. The v2.4.5 archive is named like
`ctxhelm-v2.4.5-aarch64-apple-darwin.tar.gz`.

Download the archive and checksum file for your platform, then verify the SHA-256 checksums:

```bash
shasum -a 256 -c sha256sums.txt
sha256sum -c sha256sums.txt
```

Extract the archive and put the binary on your `PATH`:

```bash
tar -xzf ctxhelm-v2.4.5-aarch64-apple-darwin.tar.gz
install -m 0755 ctxhelm-v2.4.5-aarch64-apple-darwin/ctxhelm ~/.local/bin/ctxhelm
ctxhelm --version
ctxhelm --help
ctxhelm doctor --binary "$(command -v ctxhelm)" --release-manifest ctxhelm-v2.4.5-aarch64-apple-darwin.manifest.json
```

The expected version diagnostic is `ctxhelm 2.4.5`. See [docs/release.md](docs/release.md) for release details, source-build fallbacks, and maintainer packaging checks.

## Install To First Pack

Start from the installed `ctxhelm` binary and an existing git repository:

```bash
ctxhelm --version
ctxhelm --help
export REPO=/path/to/repo
ctxhelm doctor --repo "$REPO"
```

Initialize repo-local guidance and optional agent snippets:

```bash
ctxhelm setup repo --repo "$REPO"
ctxhelm init --repo "$REPO" --cursor --claude --opencode
ctxhelm doctor --repo "$REPO"
ctxhelm setup-check --repo "$REPO" --cursor --claude --opencode
```

`ctxhelm setup repo` is the easiest secure path: it writes repo-local guidance
for Cursor, Claude Code, and OpenCode, merges a project-local `.mcp.json` entry
for `ctxhelm serve-mcp`, uses an absolute ctxhelm binary path, and does not
mutate global agent config. Use `--dry-run` to preview the files first.
Use `--format json` when scripting setup verification or collecting
source-free onboarding evidence.

Ask for a task plan with an explicit repo and, when you know it, an active file path:

```bash
ctxhelm prepare-task "fix requireSession bug" \
  --repo "$REPO" \
  --mode bug-fix \
  --path src/auth/session.ts
```

Materialize a compact context pack for the same task:

```bash
ctxhelm get-pack "fix requireSession bug" \
  --repo "$REPO" \
  --mode bug-fix \
  --budget brief
```

Inspect that pack decision path as source-free metadata:

```bash
ctxhelm inspector export "fix requireSession bug" \
  --repo "$REPO" \
  --mode bug-fix \
  --budget brief \
  --format json
```

Open the same diagnostics through a localhost-only, read-only shell:

```bash
ctxhelm inspector serve "fix requireSession bug" \
  --repo "$REPO" \
  --mode bug-fix \
  --budget brief
```

For the longer walkthrough, including setup validation, deterministic MCP proof context, and how to interpret pack options, see [docs/quickstart.md](docs/quickstart.md).

## More Docs

- [Brand and naming](docs/brand.md)
- [First-pack quickstart](docs/quickstart.md)
- [Release and install guide](docs/release.md)
- [Agent setup matrix](docs/agent-setup.md)
- [Architecture and trade-offs](docs/architecture.md)
- [Component guide](docs/components.md)
- [Data contracts](docs/data-contracts.md)
- [Context compiler](docs/context-compiler.md)
- [Pack inspector](docs/inspector.md)
- [Agent preview](docs/agent-preview.md)
- [Public demo artifacts](docs/demo.md)
- [Public project summary](docs/public-project-summary.md)
- [Distribution metadata](docs/distribution.md)
- [Release governance](docs/release-governance.md)
- [Agent integrations](docs/integrations.md)
- [Workspace manifests](docs/workspace.md)
- [Shared artifacts and team policy](docs/shared-artifacts.md)
- [Retrieval health](docs/retrieval-health.md)
- [Graph neighborhoods](docs/graph.md)
- [Policy and embedding controls](docs/policy-embedding.md)
- [Context governor](docs/context-governor.md)
- [Retrieval benchmarking](docs/benchmarking.md)
- [Storage](docs/storage.md)
- [Repo memory](docs/memory.md)
- [Feedback and policy learning](docs/feedback.md)
- [Local semantic retrieval](docs/semantic.md)
- [Parser and precision edges](docs/precision.md)
- [Troubleshooting](docs/troubleshooting.md)

## MCP Runtime

Start the local stdio MCP server:

```bash
ctxhelm serve-mcp
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

Implemented MCP resources include `ctxhelm://repo/summary`, package-aware `ctxhelm://repo/test-map`, `ctxhelm://repo/dependency-graph`, `ctxhelm://repo/memory`, `ctxhelm://workspace/status`, `ctxhelm://workspace/shared-artifacts`, `ctxhelm://pack/guide`, session-scoped `ctxhelm://pack/<task-id>/<budget>` resources returned by `prepare_task` for brief, standard, and deep packs, safe file slices, and symbol search. Implemented prompts cover bugfix, feature, refactor, review, test-writing, and explanation workflows.

## Client Integration Status

Current local smoke status:

- Deterministic protocol proof is the required release gate: direct JSON-RPC/MCP calls verify `prepare_task`, `get_pack`, `search`, `related`, `related_tests`, `current_diff`, and same-session pack-resource reads with an explicit `repo`.
- Codex CLI `0.137.0`: current local real-client MCP smoke produces source-free server-side evidence for explicit-repo `prepare_task` and `get_pack` calls with strict MCP config, and the Codex agent-run matrix produces outcome claim `ctxhelm_improved`.
- Claude Code `2.1.163`: current availability preflight is source-free but rate-limited with API status `429`; earlier Claude workflow evidence remains historical rather than current availability proof.

When using ctxhelm through MCP, pass the active repository path as `repo` whenever the client knows it. Some clients launch MCP servers from a different working directory than the project they expose.

## Safe Inventory

Build the local file inventory for a repository:

```bash
ctxhelm index --repo /path/to/repo
```

The inventory respects `.gitignore`, `.ctxhelmignore`, and `.cursorignore`, excludes sensitive/generated files by default, and writes JSON under `~/.ctxhelm/repos/<repo-id>/inventory.json`.

To also sync source-free file metadata into the local SQLite store:

```bash
ctxhelm index --repo /path/to/repo --store
ctxhelm storage status --repo /path/to/repo
```

To also build local source-free semantic vector metadata:

```bash
ctxhelm index --repo /path/to/repo --semantic
ctxhelm search "payment webhook validation" --repo /path/to/repo --semantic
```

See [docs/storage.md](docs/storage.md), [docs/semantic.md](docs/semantic.md), [docs/precision.md](docs/precision.md), and [docs/context-governor.md](docs/context-governor.md) for storage location, privacy guarantees, semantic provider details, parser/precision-edge support, adaptive policy decisions, repair/reset commands, and release-gate smoke coverage.

Generated and sensitive files require explicit opt-in:

```bash
ctxhelm index --repo /path/to/repo --include-generated --include-sensitive
```

## Lexical Search

Search the safe inventory:

```bash
ctxhelm search "requireSession" --repo /path/to/repo --limit 5
```

If no inventory exists for the repo, `ctxhelm search` builds one using the safe default inventory rules before searching.

## Symbol Index

Extract language-aware symbols from safe inventoried files:

```bash
ctxhelm symbols --repo /path/to/repo --limit 20
```

Search symbols by name, path, or signature:

```bash
ctxhelm symbols --repo /path/to/repo --query requireSession --limit 5
```

The current local extractor covers TypeScript/JavaScript, Python, Rust, Go, Java, and Kotlin definitions. MCP symbol resources use the same symbol search path through `ctxhelm://symbol/<query>`.

## Related Tests

Find likely tests for changed source files:

```bash
ctxhelm related-tests src/auth/session.ts --repo /path/to/repo
```

The result includes confidence, a reason, and a best-effort targeted test command.
For JavaScript and TypeScript repos, ctxhelm now checks nearby `package.json` scripts and package-manager lockfiles to prefer commands such as `pnpm vitest run <test>` or `npm test -- <test>` instead of assuming a single runner.

The MCP `ctxhelm://repo/test-map` resource uses the same package-aware command inference for safe inventoried test files.

## Git Co-Change Hints

Find files that have changed together in local git history:

```bash
ctxhelm co-changes src/auth/session.ts --repo /path/to/repo --limit 5
```

Co-change hints read only local git metadata and are filtered through the safe inventory.

## Dependency Graph

Inspect safe local import edges around a file:

```bash
ctxhelm dependencies src/auth/session.ts --repo /path/to/repo --limit 10
```

Return the current safe dependency graph:

```bash
ctxhelm dependencies --all --repo /path/to/repo --limit 50
```

Dependency edges are inferred from local TypeScript/JavaScript, Python, Rust, Java, and Kotlin imports in safe source/test files. External packages, generated files, sensitive files, and ignored files are excluded by default. MCP clients can request dependency expansion through `related` with `include: ["dependencies"]`, and can read the repository graph resource at `ctxhelm://repo/dependency-graph`.

Repositories with local SCIP/LSP-derived edge exports can import source-free precision edges:

```bash
ctxhelm precision import --repo /path/to/repo --input /path/to/precision-edges.json
ctxhelm dependencies src/auth/middleware.ts --repo /path/to/repo
```

## Context Plan

Prepare a task-conditioned context plan:

```bash
ctxhelm prepare-task "fix requireSession bug" --repo /path/to/repo --mode bug-fix
```

Pass active editor files as repeatable anchors when the host agent knows them:

```bash
ctxhelm prepare-task "fix redirect behavior" --repo /path/to/repo --mode bug-fix --path src/auth/middleware.ts
```

Use safe current-diff paths as anchors for review or in-progress work:

```bash
ctxhelm prepare-task "review current auth changes" --repo /path/to/repo --mode review --current-diff
```

The plan fuses active path anchors, symbol search, lexical search, related tests, local dependency edges, and local co-change hints into target files, line hints, validation commands, risk flags, and pack resource options. MCP clients can pass the same active/open files through the `paths` array on `prepare_task`.

For MCP clients, the `packOptions[*].resourceUri` values returned by `prepare_task` are loadable during the same MCP server session for brief, standard, and deep budgets. Add `.json` to a returned pack URI to read the structured pack resource instead of Markdown.

## Context Pack

Materialize a budgeted context pack:

```bash
ctxhelm get-pack "fix requireSession bug" --repo /path/to/repo --mode bug-fix --budget brief
```

Use `--format json` for structured output. `get-pack` also accepts repeatable `--path <file>` anchors, and the MCP `get_pack` tool accepts the same `paths` array.
Structured and Markdown packs include source-free provenance fields: `repoId`, `taskHash`, and `targetAgent`.

## Context Governor

Inspect the task-conditioned retrieval, budget, memory, validation, semantic,
and policy-profile decision before handing work to an agent:

```bash
ctxhelm governor decide "fix requireSession bug" \
  --repo /path/to/repo \
  --mode bug-fix \
  --path src/auth/session.ts
```

Use `--format json` for the source-free
`ctxhelm-context-governor-report-v1` contract. The report includes selected
evidence, omitted evidence, rollout controls, active policy profile state, and
privacy metadata without source snippets. See
[docs/context-governor.md](docs/context-governor.md).

## Context Cards

Generate optional repo-committable cards for cloud or disconnected agent contexts:

```bash
ctxhelm cards generate --repo /path/to/repo
```

This writes `.ctxhelm/cards/repo-overview.md`, `.ctxhelm/cards/testing.md`, `.ctxhelm/cards/dependency-graph.md`, and domain cards. Cards are deterministic, local-only, and source-snippet-free; they summarize safe inventory paths, roles, symbols, test commands, and local dependency edges. `cards generate` also stores matching source-free memory metadata for task-conditioned selection.

Generate and review experience cards from source-free local traces:

```bash
ctxhelm memory generate-experience --repo /path/to/repo
ctxhelm memory list --repo /path/to/repo
ctxhelm memory approve experience:<task-hash> --repo /path/to/repo
```

`prepare-task`, `get-pack`, and MCP pack resources can include fresh approved or deterministic memory under `selectedMemory` and a separate `Selected memory` pack section. MCP also exposes `ctxhelm://repo/memory`. See [docs/memory.md](docs/memory.md).

## Local Eval Traces

`prepare-task`, `get-pack`, and the matching MCP tools append source-free local traces under `~/.ctxhelm/repos/<repo-id>/traces.jsonl`.

Inspect recent traces:

```bash
ctxhelm eval traces --repo /path/to/repo --limit 20
```

Generate a manual dogfood checklist from recent traces:

```bash
ctxhelm eval checklist --repo /path/to/repo --limit 5
```

Traces store task hashes, task type, target agent label, recommended files/tests/commands, optional pack id, optional budget, and created time. They do not store task text or source snippets.

Run a source-free historical retrieval eval over recent local commits:

```bash
ctxhelm eval history --repo /path/to/repo --limit 20 --budget 10
```

Run a named source-free benchmark suite over multiple local repositories:

```bash
ctxhelm eval benchmark --config .ctxhelm/benchmarks/retrieval-quality.json
```

Compare two benchmark JSON reports and flag configured regression thresholds in the report:

```bash
ctxhelm eval compare --base-report previous.json --head-report current.json --threshold fileRecallAt10=0.05
```

Generate the source-free product proof report:

```bash
ctxhelm eval proof --config .ctxhelm/benchmarks/retrieval-quality.json
```

See [docs/benchmarking.md](docs/benchmarking.md) for the suite JSON contract, RefactoringMiner-style setup, token ROI interpretation, gap families, and regression comparison.

```bash
ctxhelm eval history --repo /path/to/repo --limit 20 --mode bug-fix
```

Use `--base <rev> --head <rev>` to freeze the evaluated commit range for apples-to-apples tuning on larger repositories.

This replays each commit subject through `prepare_task`, treats the commit's safe changed files as hidden labels, and reports File Recall@5/10, lexical and no-context baselines, ctxhelm lift, Source Recall@5/10, Test Recall@5/10, signal ablations, token ROI by brief/standard/deep budget, test recommendation rate, low-information commit counts, top retrieval gaps by file role, and excluded generated/sensitive path counts. The report uses task hashes and path labels; it does not include source snippets.

## Development

Development commands require a source checkout:

```bash
cargo test --workspace
cargo run -p ctxhelm -- --help
```
