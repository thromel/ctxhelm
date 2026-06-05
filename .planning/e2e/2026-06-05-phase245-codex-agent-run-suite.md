# Phase 245 - Codex Agent-Run Suite And Harness Consumption

## Summary

Phase 245 extends the Codex real-client outcome proof from a single task to a source-free multi-task suite and fixes the harness/guidance retrieval gap it exposed.

The first real suite showed that ctxhelm could aggregate two Codex tasks, but it also found two real issues:

- `Check selected memory native read guidance` initially missed `crates/ctxhelm-mcp/src/tools.rs` and `crates/ctxhelm-compiler/src/planning.rs` in ctxhelm evidence for some lanes.
- `Improve Codex agent-run harness` surfaced `scripts/e2e-agent-run-codex.sh` as evidence, but the `ctxhelm-plan` lane did not read it because the script ranked outside Codex's first native-read batch.

## Changes

- Added `--suite` support to `scripts/e2e-agent-run-codex.sh`.
- Added source-free suite aggregation for task counts, lane summaries, target coverage/read coverage deltas, irrelevant/read/command deltas, required ctxhelm-call compliance, client failures, forbidden commands, evidence misses, evidence-only targets, under-read targets, outcome claim, and recommended R&D actions.
- Fixed skipped-suite accounting so skipped/no-real-client lanes do not create false missing-required-call failures.
- Added agent-native guidance implementation-surface promotion in the planner for Codex/Claude/Cursor/MCP/memory native-read guidance and harness tasks.
- Promoted existing implementation-surface lexical hits instead of only adding absent paths.
- Treated named shell harness scripts as source-like candidates for this bounded promotion so they enter the native first-read window.
- Isolated spawned Codex real-client runs from the current Desktop thread environment and enabled `--ignore-rules` when available.
- Banned bootstrap/setup/install/hook/superpowers commands in the e2e prompt and evaluator.
- Added bounded shell-command guidance so the native baseline cannot wander until timeout.

## Proof

Final artifact: `.ctxhelm/e2e/phase245-agent-run-codex-suite-real-bounded-final.json`

Rendered result:

- Status: `passed`
- Client: `codex-cli 0.137.0`
- Tasks: `2`
- Comparison eligible tasks: `2`
- Comparable ctxhelm lanes: `8`
- Outcome claim: `ctxhelm_improved`
- Target coverage delta average: `0.25`
- Target read coverage delta average: `0.25`
- Irrelevant read delta sum: `0`
- Read file delta sum: `-1`
- Command execution delta sum: `11`
- Missing required ctxhelm calls: `false`
- Invalid required ctxhelm calls: `false`
- Client failures: `false`
- Forbidden commands: `false`
- ctxhelm evidence misses: `false`
- ctxhelm evidence-only targets: `false`
- ctxhelm under-read targets: `false`
- Recommended R&D action: `preserve_current_agent_contract(p3)`

Lane summary:

- Baseline: average target read coverage `0.75`, read files `9`, missed targets `1`.
- `ctxhelm-plan`: average target read coverage `1.00`, read files `10`, missed targets `0`.
- `ctxhelm-brief`: average target read coverage `1.00`, read files `10`, missed targets `0`.
- `ctxhelm-standard`: average target read coverage `1.00`, read files `10`, missed targets `0`.
- `ctxhelm-memory`: average target read coverage `1.00`, read files `10`, missed targets `0`.

Strict command accounting:

- Observed command verbs: `head`, `nl`, `pwd`, `rg`, `sed`.
- `superpowers-codex` observed: `false`.
- Forbidden command count: `0`.

## Interpretation

This phase improves the R&D harness and the product behavior.

The harness improvement is that Codex outcome evidence is now suite-level rather than single-task only. The suite reports aggregate lane behavior and routes next R&D actions from the observed failure mode instead of forcing manual interpretation of individual task artifacts.

The product improvement is that agent-native guidance and harness tasks now place the actual implementation surfaces into the agent's native first-read batch. The final suite proves the earlier `ctxhelm-plan` evidence-only script gap is gone: every ctxhelm lane reads all expected target files for both tasks, while the baseline misses one target under the same bounded read budget.

The result should still be interpreted as a two-task real-agent proof, not a broad benchmark. It is strong evidence for the fixed failure class and a cleaner harness for future larger suites.
