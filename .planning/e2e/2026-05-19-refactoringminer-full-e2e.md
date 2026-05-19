# RefactoringMiner Full E2E

Date: 2026-05-19

## Scope

This run used a real large repository instead of toy fixtures:

- Subject repo: `/tmp/refminer-ctxpack-e2e`
- Source: clean clone of `/Users/romel/Documents/GitHub/RefactoringMiner`
- Scale: 1.9 GB, 38,753 files
- ctxpack release binary tested: `/tmp/ctxpack-claude-e2e/ctxpack-v1.1.0-aarch64-apple-darwin/ctxpack`
- patched local binary tested: `target/debug/ctxpack`
- Isolated ctxpack home: `/tmp/ctxpack-refminer-home`
- Evidence directory: `/tmp/ctxpack-refminer-e2e`

The original RefactoringMiner checkout was not modified.

## User-Visible Answer

ctxpack is useful, but the real e2e shows it is not yet consistently better than baseline lexical retrieval on a large Java repository.

What works:

- Local indexing completes on a large repo.
- `prepare-task`, `get-pack`, graph, inspector, cards, memory, and agent preview all return structured source-free outputs.
- Memory storage and selection work after card generation.
- Claude Code can call ctxpack through MCP against the real RefactoringMiner clone.
- Historical eval now evaluates real commits after fixing a sampler bug found by this run.

What is not good enough yet:

- Retrieval quality underperforms lexical baseline on this RefactoringMiner historical slice.
- Related-test selection is noisy and lacks Gradle/JUnit class-level commands.
- Some relevant docs/source files are missed.
- Deep eval is expensive on large histories.

## Correctness

### Latest Commit Task

Task title:

```text
Default MCP repository path to working directory
```

Gold changed files:

- `documentation/mcp.md`
- `src/main/java/org/refactoringminer/mcp/RefactoringMinerMcpService.java`
- `src/main/java/org/refactoringminer/mcp/RefactoringMinerMcpTools.java`
- `src/test/java/org/refactoringminer/mcp/RefactoringMinerMcpServiceRepositoryTest.java`
- `src/test/java/org/refactoringminer/mcp/RefactoringMinerMcpToolsTest.java`

Result:

- `prepare-task` hit 4 of 5 gold files.
- Missed `documentation/mcp.md`.
- Historical eval for the same commit also hit the four Java/test MCP files and missed `documentation/mcp.md`.

Top target files:

1. `src/main/java/org/refactoringminer/mcp/RefactoringMinerMcpTools.java`
2. `src/test/java/org/refactoringminer/mcp/RefactoringMinerMcpServiceRepositoryTest.java`
3. `src/test/java/org/refactoringminer/mcp/RefactoringMinerMcpToolsTest.java`
4. `src/main/java/org/refactoringminer/mcp/WorktreeChangeCollector.java`
5. `src/main/java/gr/uom/java/xmi/diff/UMLModelDiff.java`
6. `src/main/java/gui/webdiff/dir/DirectoryDiffView.java`
7. `src/main/java/org/refactoringminer/mcp/RefactoringMinerMcpService.java`
8. `src/main/java/org/refactoringminer/util/GitServiceImpl.java`

### NPE Task

Task title:

```text
Handle NPE in getScopeNode(ISwc4jAst node)
```

Gold changed files:

- `src/main/java/gr/uom/java/xmi/decomposition/VariableDeclaration.java`
- `src/main/java/org/refactoringminer/astDiff/matchers/wrappers/MethodMatcher.java`

Result:

- `prepare-task` hit 1 of 2 gold files.
- It found `VariableDeclaration.java`.
- It missed `MethodMatcher.java`.
- It over-selected broad `RefactoringMiner`, API, and extension AST nodes.

## Historical Eval

The first release-binary run incorrectly returned:

- `evaluatedCommits: 0`
- real time: 3.32s

Root cause:

- RefactoringMiner has recent commits where `git diff-tree` takes seconds.
- Example timings from the first 20 commits included about 5.7s, 8.4s, and 37.7s diff-tree calls.
- ctxpack had a 250ms per-commit diff timeout and treated one slow commit as a failure of the whole sample set.

Fix applied:

- Historical commit collection now skips per-commit metadata/diff failures instead of collapsing the full eval to zero samples.
- Added a regression test for per-commit diff failure handling.

Fixed local-binary result:

- `evaluatedCommits: 20`
- `fileRecallAt5: 0.1018`
- `fileRecallAt10: 0.2569`
- `lexicalBaselineRecallAt5: 0.4594`
- `lexicalBaselineRecallAt10: 0.5008`
- `sourceRecallAt10: 0.2944`
- `testRecallAt10: 0.4167`
- real time: 265.65s

Interpretation:

- The eval system now works on this large repo.
- The current retrieval policy is worse than lexical baseline for this slice and needs ranking work before we can claim better context selection.

Top retrieval gaps:

1. MCP source files with `no_candidate_signal`.
2. MCP test files with `no_candidate_signal`.
3. `documentation/*.md` docs with `lexical_only_miss`.

## Efficiency

Release-binary timings:

| Operation | Time |
| --- | ---: |
| `index --store --semantic` | 11.38s |
| `prepare-task` latest commit | 20.06s |
| `get-pack --budget brief` | 22.34s |
| `get-pack --budget standard` | 22.19s |
| `prepare-task` NPE commit | 19.72s |
| `graph neighborhood` | 25.53s |
| `inspector export` | 22.37s |
| `agent preview --target-agent all` before fix | 112.11s |
| `cards generate` | 11.04s |
| `memory generate-experience` | 5.25s |
| `memory list` | < 1s |

Patched local-binary timings:

| Operation | Time |
| --- | ---: |
| `eval history --limit 20` after sampler fix | 265.65s |
| `agent preview --target-agent all` after shared-plan fix | 45.41s |

Efficiency conclusions:

- `prepare-task` and `get-pack` are usable but not fast on a 1.9 GB Java repo.
- Historical eval is now correct but expensive.
- `agent preview --target-agent all` improved from five independent recomputations to one shared pack/resource path, but still has a large fixed retrieval/pack cost.

## Memory

Memory path tested:

1. `cards generate --limit 40`
2. `memory generate-experience`
3. `memory list`
4. `prepare-task` again

Observed results:

- `cards generate` produced 8 deterministic domain cards.
- `memory generate-experience` produced 4 experience-card reports.
- `memory list` returned 10 memory cards.
- After memory generation, `prepare-task` selected 3 memory cards.

Selected memory:

- `domain:testing`, score `0.632`
- `domain:domain-documentation`, score `0.096`
- `domain:repo-overview`, score `0.096`

Memory conclusion:

- Working: durable cards are stored and selected by later planning.
- Correct behavior: pending experience cards are not selected before review.
- Gap: selected memory includes a weak generic repo-overview card; memory ranking should become more ROI-aware.

## Claude Code E2E

Real-client Claude Code smoke against RefactoringMiner passed.

Evidence:

```json
{
  "client": "claude",
  "clientVersion": "2.1.143 (Claude Code)",
  "ctxpackVersion": "ctxpack 1.1.0",
  "getPack": true,
  "prepareTask": true,
  "repo": "/private/tmp/refminer-ctxpack-e2e",
  "required": true
}
```

Conclusion:

- The Claude Code MCP integration is not just protocol-only; real Claude Code invoked `prepare_task` and `get_pack` against the large repo.

## Bugs Fixed During This Run

### REF-E2E-001: Historical Eval False Zero

Status: fixed.

Problem:

- A slow commit diff caused `eval history --limit 20` to return zero evaluated commits.

Fix:

- Changed historical commit collection to skip per-commit metadata/diff failures instead of failing the whole sample set.
- Added regression coverage.

Files:

- `crates/ctxpack-index/src/git.rs`
- `crates/ctxpack-index/src/lib.rs`

### REF-E2E-002: All-Agent Preview Recomputed Too Much

Status: improved.

Problem:

- `agent preview --target-agent all` took 112.11s because it recomputed retrieval and pack materialization per target agent.

Fix:

- Shared one retrieval plan across all target agents.
- Shared one pack resource URI across all preview entries.

Files:

- `crates/ctxpack-compiler/src/agent_preview.rs`

Remaining issue:

- Debug-build runtime is still 45.41s on RefactoringMiner, so more caching or preview-specific lightweight planning is needed.

## Remaining Product Gaps

### REF-E2E-003: Retrieval Policy Underperforms Lexical Baseline

Status: open.

Evidence:

- ctxpack file recall@10: `0.2569`
- lexical baseline recall@10: `0.5008`

Likely causes:

- Java package/path families are not weighted strongly enough.
- Exact title-token matches are not dominating enough for historical commit eval.
- Docs and tests are demoted too aggressively in bug-fix mode.
- Graph/test/history candidates sometimes add noise before improving recall.

### REF-E2E-004: Docs Missed for MCP Task

Status: open.

Evidence:

- Latest commit task missed `documentation/mcp.md`.
- Historical gaps show `documentation/*.md` as `lexical_only_miss`.

Needed:

- Docs should be preserved for tasks with explicit integration/tool names like MCP, CLI, setup, docs, config, or user-facing behavior.

### REF-E2E-005: NPE Task Misses MethodMatcher

Status: open.

Evidence:

- NPE task hit `VariableDeclaration.java` but missed `MethodMatcher.java`.

Needed:

- Stronger exact-symbol extraction for Java method names and AST wrapper class names.
- Better history/co-change weighting around recent AST-diff files.

### REF-E2E-006: Related Tests Are Noisy

Status: open.

Evidence:

- Related test output included GUI and broad regression tests for narrow MCP and NPE tasks.
- Commands were not useful Gradle/JUnit class-level commands.

Needed:

- Java/Gradle test command mapping.
- Package-proximity and class-name matching should dominate broad resource/test references.

### REF-E2E-007: Deep Historical Eval Is Expensive

Status: open.

Evidence:

- Fixed `eval history --limit 20` took 265.65s on the RefactoringMiner clone.

Needed:

- Cache parent snapshots or candidate plans.
- Add a cheaper first-pass history eval mode.
- Surface skipped/slow commit diagnostics in `HistoricalEvalReport`.

## Validation

Commands passed:

```bash
CARGO_INCREMENTAL=0 cargo test --workspace
CARGO_INCREMENTAL=0 cargo run -p ctxpack -- --help
```

Targeted tests passed:

```bash
CARGO_INCREMENTAL=0 cargo test -p ctxpack-index historical_commit_collection_skips_per_commit_diff_failures -- --nocapture
CARGO_INCREMENTAL=0 cargo test -p ctxpack-compiler agent_preview -- --nocapture
```

Claude Code e2e passed:

```bash
CTXPACK_BIN=/tmp/ctxpack-claude-e2e/ctxpack-v1.1.0-aarch64-apple-darwin/ctxpack \
CTXPACK_REQUIRE_REAL_CLIENT=1 \
CTXPACK_REAL_CLIENT_EVIDENCE_DIR=/tmp/ctxpack-refminer-claude-evidence \
CTXPACK_ROOT="/Users/romel/Documents/GitHub/Agent Memory" \
CTXPACK_SMOKE_REPO="/tmp/refminer-ctxpack-e2e" \
CTXPACK_SMOKE_TASK="Default MCP repository path to working directory" \
CTXPACK_SMOKE_PATH="src/main/java/org/refactoringminer/mcp/RefactoringMinerMcpTools.java" \
CTXPACK_SMOKE_QUERY="Default MCP repository path" \
bash scripts/smoke-claude-mcp.sh
```
