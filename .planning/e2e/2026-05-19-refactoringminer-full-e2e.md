# RefactoringMiner Full E2E

Date: 2026-05-19

## Scope

This run used a real large repository instead of toy fixtures:

- Subject repo: `/tmp/refminer-ctxhelm-e2e`
- Source: clean clone of `/Users/romel/Documents/GitHub/RefactoringMiner`
- Scale: 1.9 GB, 38,753 files
- ctxhelm release binary tested: `/tmp/ctxhelm-claude-e2e/ctxhelm-v1.1.0-aarch64-apple-darwin/ctxhelm`
- patched local binary tested: `target/debug/ctxhelm`
- Isolated ctxhelm home: `/tmp/ctxhelm-refminer-home`
- Evidence directory: `/tmp/ctxhelm-refminer-e2e`

The original RefactoringMiner checkout was not modified.

## User-Visible Answer

ctxhelm is useful, but the real e2e shows it is not yet consistently better than baseline lexical retrieval on a large Java repository.

What works:

- Local indexing completes on a large repo.
- `prepare-task`, `get-pack`, graph, inspector, cards, memory, and agent preview all return structured source-free outputs.
- Memory storage and selection work after card generation.
- Claude Code can call ctxhelm through MCP against the real RefactoringMiner clone.
- Historical eval now evaluates real commits after fixing a sampler bug found by this run.
- A follow-up ranking fix moved historical Recall@10 above the lexical baseline on this slice.

What is not good enough yet:

- Retrieval quality is only slightly above lexical baseline at Recall@10 and only tied at Recall@5.
- Related-test selection is still noisy on sparse Java/AST tasks.
- Some relevant source files remain unrecoverable from sparse historical commit titles without stronger semantic or repository-history signals.
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

- Initial `prepare-task` hit 4 of 5 gold files and missed `documentation/mcp.md`.
- After the ranking fixes, the task context hit all 5 gold files across target files plus related tests.
- Historical eval for the same commit also hit all 5 gold files after context ranking was changed to keep validation tests inside the fixed budget.

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

- Initial `prepare-task` hit 1 of 2 gold files.
- After the ranking fixes, live `prepare-task` hit both `VariableDeclaration.java` and `MethodMatcher.java`.
- Historical eval still misses `MethodMatcher.java` from the parent snapshot because the commit title has no lexical/symbol signal for that file; the live repo can recover it through co-change history after that history exists.

## Historical Eval

The first release-binary run incorrectly returned:

- `evaluatedCommits: 0`
- real time: 3.32s

Root cause:

- RefactoringMiner has recent commits where `git diff-tree` takes seconds.
- Example timings from the first 20 commits included about 5.7s, 8.4s, and 37.7s diff-tree calls.
- ctxhelm had a 250ms per-commit diff timeout and treated one slow commit as a failure of the whole sample set.

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

Follow-up ranking result after debugging:

- `evaluatedCommits: 20`
- `fileRecallAt5: 0.4532`
- `fileRecallAt10: 0.5186`
- `lexicalBaselineRecallAt5: 0.4532`
- `lexicalBaselineRecallAt10: 0.5008`
- `ctxhelmLiftAt5: 0.0`
- `ctxhelmLiftAt10: 0.0179`
- `sourceRecallAt10: 0.4611`
- `testRecallAt10: 0.4722`
- real time: 279.18s

Interpretation:

- The hybrid policy is no longer worse than lexical on this real slice.
- The lift is small; this is a correctness recovery, not a strong product win yet.

Follow-up eval-diagnostics and related-test ranking result:

- `evaluatedCommits: 20`
- `fileRecallAt5: 0.4532`
- `fileRecallAt10: 0.5186`
- `lexicalBaselineRecallAt5: 0.4532`
- `lexicalBaselineRecallAt10: 0.5008`
- `ctxhelmLiftAt5: 0.0`
- `ctxhelmLiftAt10: 0.0179`
- `sourceRecallAt10: 0.4611`
- `testRecallAt10: 0.4722`
- runtime total: `191097ms`
- runtime commit loop: `184130ms`
- runtime overhead: `6967ms`
- average commit runtime: `9206.5ms`

Interpretation:

- Aggregate retrieval quality stayed stable after the related-test ranking fix.
- The latest MCP commit improved from one missing test file at 10 to zero missing files at 10 on the two-commit smoke slice.
- Runtime diagnostics confirm the expensive part is per-commit parent-snapshot planning, not fixed eval overhead.

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
| `eval history --limit 20` after ranking fixes | 279.18s |
| `eval history --limit 20` after runtime/test ranking fixes | 191.10s |
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
  "ctxhelmVersion": "ctxhelm 1.1.0",
  "getPack": true,
  "prepareTask": true,
  "repo": "/private/tmp/refminer-ctxhelm-e2e",
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

- `crates/ctxhelm-index/src/git.rs`
- `crates/ctxhelm-index/src/lib.rs`

### REF-E2E-002: All-Agent Preview Recomputed Too Much

Status: improved.

Problem:

- `agent preview --target-agent all` took 112.11s because it recomputed retrieval and pack materialization per target agent.

Fix:

- Shared one retrieval plan across all target agents.
- Shared one pack resource URI across all preview entries.

Files:

- `crates/ctxhelm-compiler/src/agent_preview.rs`

Remaining issue:

- Debug-build runtime is still 45.41s on RefactoringMiner, so more caching or preview-specific lightweight planning is needed.

### REF-E2E-003: Hybrid Ranking Dropped Strong Lexical Evidence

Status: fixed.

Problem:

- Strong exact lexical hits such as `documentation/mcp.md` and `RefactoringMinerMcpService.java` were present in candidates but fell outside the selected context because symbol, graph, and history candidates saturated the top ranks.

Fix:

- Added a lexical floor to target selection.
- Preserved original lexical rank through candidate fusion so exact-search ordering survives score saturation.
- Reduced generic `node` query weight and ignored common task verbs such as `fix`, `handle`, `default`, and `support`.

Files:

- `crates/ctxhelm-index/src/search.rs`
- `crates/ctxhelm-compiler/src/ranking.rs`

### REF-E2E-004: History and Graph Signals Were Misweighted

Status: improved.

Problem:

- Dependency expansion was too greedy and co-change evidence was too weak for bug-fix style tasks.

Fix:

- Prioritized incoming dependency edges before outgoing dependency edges.
- Increased bug/refactor/review co-change candidate coverage.
- Added a co-change floor to target selection.
- Lowered dependency signal weight and raised co-change signal weight.

Files:

- `crates/ctxhelm-index/src/dependencies.rs`
- `crates/ctxhelm-compiler/src/planning.rs`
- `crates/ctxhelm-compiler/src/ranking.rs`

### REF-E2E-005: Java Tests Had No Runnable Commands

Status: fixed.

Problem:

- RefactoringMiner related tests were returned without useful Gradle/JUnit class-level commands.

Fix:

- Added Java test command inference for Gradle and Maven.
- RefactoringMiner MCP tests now return commands such as `./gradlew test --tests org.refactoringminer.mcp.RefactoringMinerMcpToolsTest`.

Files:

- `crates/ctxhelm-index/src/related_tests.rs`

## Remaining Product Gaps

### REF-E2E-006: Retrieval Policy Has Only Small Lift Over Lexical

Status: partially fixed.

Evidence:

- original ctxhelm file recall@10: `0.2569`
- fixed ctxhelm file recall@10: `0.5186`
- lexical baseline recall@10: `0.5008`

Likely causes:

- Sparse commit titles still leave files with no recoverable lexical/symbol signal.
- Java package/path families need better precision than import-only graph expansion.
- Graph/test/history candidates can still add noise before improving recall.

### REF-E2E-007: Docs Missed for MCP Task

Status: fixed for the observed task.

Evidence:

- Latest commit task missed `documentation/mcp.md`.
- Historical gaps show `documentation/*.md` as `lexical_only_miss`.

Needed:

- Docs should be preserved for tasks with explicit integration/tool names like MCP, CLI, setup, docs, config, or user-facing behavior.

### REF-E2E-008: Historical NPE Task Still Misses MethodMatcher

Status: partially fixed.

Evidence:

- NPE task hit `VariableDeclaration.java` but missed `MethodMatcher.java`.
- Live `prepare-task` now includes `MethodMatcher.java`.
- Parent-snapshot historical eval still misses `MethodMatcher.java`.

Needed:

- Stronger exact-symbol extraction for Java method names and AST wrapper class names.
- Better history/co-change weighting around recent AST-diff files.

### REF-E2E-009: Related Tests Are Noisy

Status: improved.

Evidence:

- Related test output still includes GUI and broad regression tests for sparse NPE tasks.
- Commands are now useful Gradle/JUnit class-level commands.
- Latest MCP commit two-commit smoke moved from one missing test file at 10 to zero missing files at 10.

Fixed:

- Java/Gradle test command mapping.
- Package-proximity and class-name matching should dominate broad resource/test references.
- Content-only related-test matches are capped so broad helper tests do not outrank structurally matched class tests.

Still needed:

- Better Java package proximity for sparse AST tasks where no test class name directly matches the changed source file.

### REF-E2E-010: Deep Historical Eval Is Expensive

Status: open.

Evidence:

- Fixed `eval history --limit 20` took 265.65s on the RefactoringMiner clone.
- Runtime-instrumented `eval history --limit 20` took 191.10s total, with 184.13s spent inside per-commit planning and 6.97s in overhead.

Needed:

- Cache parent snapshots or candidate plans.
- Add a cheaper first-pass history eval mode.
- Use the new slow-commit diagnostics in `HistoricalEvalReport` to choose the next performance optimization.

## Validation

Commands passed:

```bash
cargo fmt --check
git diff --check
CARGO_INCREMENTAL=0 cargo test --workspace
CARGO_INCREMENTAL=0 cargo run -p ctxhelm -- --help
```

Targeted tests passed:

```bash
CARGO_INCREMENTAL=0 cargo test -p ctxhelm-index historical_commit_collection_skips_per_commit_diff_failures -- --nocapture
CARGO_INCREMENTAL=0 cargo test -p ctxhelm-compiler agent_preview -- --nocapture
CARGO_INCREMENTAL=0 cargo test -p ctxhelm-compiler ranking -- --nocapture
CARGO_INCREMENTAL=0 cargo test -p ctxhelm-index lexical_search_ignores_common_task_verbs -- --nocapture
CARGO_INCREMENTAL=0 cargo test -p ctxhelm-index related_tests_uses_gradle_java_test_class_command -- --nocapture
CARGO_INCREMENTAL=0 cargo test -p ctxhelm-index related_tests -- --nocapture
CARGO_INCREMENTAL=0 cargo test -p ctxhelm-compiler historical_eval -- --nocapture
CARGO_INCREMENTAL=0 cargo test -p ctxhelm historical_eval_report_renders_source_free_metrics -- --nocapture
CARGO_INCREMENTAL=0 cargo test -p ctxhelm --test cli_compat search_related_tests_dependencies_and_eval_history_emit_json_shapes -- --nocapture
CARGO_INCREMENTAL=0 cargo test -p ctxhelm-mcp related_call -- --nocapture
```

Claude Code e2e passed:

```bash
CTXHELM_BIN=/tmp/ctxhelm-claude-e2e/ctxhelm-v1.1.0-aarch64-apple-darwin/ctxhelm \
CTXHELM_REQUIRE_REAL_CLIENT=1 \
CTXHELM_REAL_CLIENT_EVIDENCE_DIR=/tmp/ctxhelm-refminer-claude-evidence \
CTXHELM_ROOT="/Users/romel/Documents/GitHub/Agent Memory" \
CTXHELM_SMOKE_REPO="/tmp/refminer-ctxhelm-e2e" \
CTXHELM_SMOKE_TASK="Default MCP repository path to working directory" \
CTXHELM_SMOKE_PATH="src/main/java/org/refactoringminer/mcp/RefactoringMinerMcpTools.java" \
CTXHELM_SMOKE_QUERY="Default MCP repository path" \
bash scripts/smoke-claude-mcp.sh
```
