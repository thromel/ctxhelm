# Milestones

## Active

### v2.5 Production Retrieval Quality

**Goal:** Prove and improve retrieval quality across real repositories so production local embeddings, reranking, graph/test/history fixes, and learned fusion can beat lexical baseline while staying local-first and source-safe.

**Status:** Active: 2026-05-22. Phases 61-67 complete; Phase 69 promoted default local retrieval under the channel-aware product-proof gate; Phase 70 refreshed Codex CLI and Claude Code real-client MCP proof; Phase 71 reduced ctxhelm archive-artifact retrieval noise; Phase 72 broadened repeated-lift validation and improved validation-test recall seeding; Phase 73 committed a pinned broader fixed-corpus probe; Phase 74 separated protected retrieval-target misses from non-target protected pressure; Phase 75 restored parent-bounded eval history and reserved co-changed validation tests; Phase 76 split historical parent history into validation-only mode and enriched co-changed test commands; Phase 77 added broad validation fallback commands and effective validation-command coverage metrics; Phase 78 made the broader proof gate ceiling-aware and promoted the fixed four-repo corpus; Phase 79 added protected target floors; Phase 80 fixed symbol-floor duplicate accounting and cleared protected target misses; Phase 81 fixed warm-cache runtime reporting; Phase 82 made warm-cache runtime enforceable; Phase 83 made context-vs-all-file divergence machine-checkable; Phase 84 added broad-scope accounting and scoped dependency source floors; Phase 85 added source-free context-area hints for broad prepare-task plans and packs; Phase 86 added bounded Python package re-export graph coverage; Phase 87 fixed validation gap accounting; Phase 88 added broad source-area candidates; Phase 89 reduced inventory freshness overhead and promoted the broader release proof; Phase 90 proved the packaged release gate from a clean worktree with the broad benchmark enabled; Phase 91 added broad context-area recall metrics and implementation-first area ordering; Phase 92 added area-aware gap taxonomy, clean large-repo warm proof, historical eval cache invalidation, and large-repo eval runtime caching; Phase 93 added source-free symbol/dependency index caches and promoted clean cold large-repo proof; Phase 94 increased broad context-area guidance while preserving target-file/test/validation metrics; Phase 95 made broad context-area pack guidance progressive and actionable; Phase 96 exposed broad context areas as source-free MCP resources; Phase 97 improved broad governance/proof task classification; Phase 98 split broad context-area classification from target-file source floors for archive/docs tasks; Phase 99 added source-free role buckets, path families, and next-read batches to context-area resources; Phase 100 made retrieval-gap summaries resource-backed; Phase 101 made the resource-backed gap shape release-gated; Phase 102 made explicit-repo MCP resources consumable from non-repo server cwd; Phase 103 added broad fixed-corpus floors; Phase 104 added source-free next-read paths and unselected counts to broad context areas; Phase 105 made history-unavailable benchmark repos produce embedded insufficient-evidence reports; Phase 106 hardened Codex/Claude real-client proof artifacts with source-free request metadata; Phase 107 fixed the hydrated full four-repo proof path; Phase 108 bounded cold Git failures; Phase 109 made environment-health blockers machine-readable; Phase 110 promoted the clean cold full-fixture proof; Phase 111 wired that proof into the packaged release gate; Phase 112 passed the clean packaged release gate with the clean fixture proof required; Phase 113 recorded source-free release-candidate status and archive-first distribution decisions; Phase 114 published and verified the public v1.1.0 archive release; Phase 115 verified the public archive install path from GitHub release assets; Phase 116 refreshed optional real-client evidence against the public archive binary; Phase 117 added source-free role signals to broad context-area guidance; Phase 118 added safe-inventory scope metadata to context-area MCP resources; Phase 119 removed an observed `CTXHELM_HOME` test-environment race from `ctxhelm-index`; Phase 120 added public CI release-gate enforcement; Phase 121 moved public CI JavaScript actions to Node 24 and verified no Node 20 warning text; Phase 122 fixed public archive real-client smoke compatibility with post-release MCP protocol assertions; Phase 123 added source-free context-area coverage profiles; Phase 124 added source-free context-area inspection strategies; Phase 125 added source-free lexical comparison summaries to product proof; Phase 126 added agent-evidence lexical comparison for the actual context/test/validation evidence set; Phase 127 added a narrow-plan validation-test reserve that eliminates target-file lexical trailing corpora without regressing broad context-area plans; Phase 128 added broad operational floors that clear protected target misses across all four measured corpora; Phase 129 added public release freshness proof; Phase 130 published and verified a current public v1.1.1 archive release; Phase 131 added product-aware public release freshness and published/verified v1.1.2 with Claude Code evidence; Phase 132 added and verified a source-free Claude Code workflow eval with real-client explicit-repo `prepare_task` and `get_pack` evidence; Phase 133 sharpened README positioning, current proof snapshot, and release-doc drift checks; Phase 134 published and verified the current public v1.1.3 archive release; Phase 135 added Homebrew/crates readiness checks without publishing those channels; Phase 136 published and verified the current public v1.1.4 archive release; Phase 137 published and verified the public Homebrew tap; Phase 138 published and verified the current public v1.1.5 archive and refreshed Homebrew tap; Phase 139 introduced a short-lived interim product name while preserving the `ctxhelm` compatibility surface; Phase 140 published and verified the public v1.1.6 archive and Homebrew tap; Phase 141 finalized ctxhelm after availability review and bumped the release line to v1.1.7; Phase 142 published and verified the ctxhelm v1.1.7 archive and Homebrew tap; Phase 143 added paired Claude Code agent-run outcome proof showing `ctxhelm-brief` preserved target coverage while reducing irrelevant reads.

Latest follow-up: Phase 340 makes Claude Code setup easier without weakening config safety. `ctxhelm setup claude --repo <repo>` now initializes repo-local Claude guidance, writes or merges project-local `.mcp.json` with only `mcpServers.ctxhelm`, uses an absolute ctxhelm binary path, runs setup validation, and keeps global Claude Code config untouched. The CLI contract test covers the one-command setup path, preservation of existing MCP entries, and `--dry-run` no-write behavior.

Previous follow-up: Phase 339 makes single-run agent-run proof summaries expose the command/tool-call audit fields that suite summaries already publish. `scripts/check-agent-run-proof.py` now includes `metrics.commandExecutionDelta` and `metrics.ctxhelmToolCallsObserved` for `--workflow run`, so a standalone accepted report can be audited for command-cost and ctxhelm-call observation without requiring suite JSON. The focused contract test now asserts the Phase 322 single-run summary records command delta `2` and observed ctxhelm tool calls `true`.

Previous follow-up: Phase 338 makes saved agent-run proof audit metrics expose every validated top-level comparison aggregate. `scripts/check-agent-run-proof.py` now includes `metrics.commandExecutionDeltaSum` and `metrics.ctxhelmToolCallsObserved` alongside the validated coverage/read delta fields, so the JSON audit artifact no longer omits fields covered by `strictComparisonAggregateChecks`. The focused contract test now asserts the Phase 322 summary records command delta `9` and observed ctxhelm tool calls `true`.

Previous follow-up: Phase 337 makes saved agent-run proof audit privacy and boundary fields unambiguous. `scripts/check-agent-run-proof.py` now emits factual `privacyStatus` values from the source report, separate `privacyChecks` pass/fail fields, factual `boundaryStatus` values from the aggregate/comparison report, and existing `boundaryChecks` pass/fail fields. The focused contract test now asserts the Phase 322 summary reports `privacyStatus.sourceTextLogged = false`, source-free privacy checks passed, boundary facts such as `clientFailuresObserved = false`, and boundary checks passed.

Previous follow-up: Phase 336 makes saved agent-run proof audit metric names match the validated aggregate contract. `scripts/check-agent-run-proof.py` now emits `metrics.targetReadCoverageDeltaAverage` and `metrics.targetCoverageDeltaAverage` in JSON proof summaries instead of stale shortened `...DeltaAvg` aliases that produced `null`. The focused contract test now asserts the Phase 322 summary records the non-null `0.3125` target-read coverage delta and `0.25` target coverage delta under the exact validated field names.

Previous follow-up: Phase 335 makes saved agent-run proof suite-envelope status strict. `scripts/check-agent-run-proof.py` now derives `suite.taskCount` from `tasks[*]` and derives `report.status` from nested task statuses, comparison eligibility, and strict boundary flags, rejecting stale suite-count or stale status labels even when the aggregate proof still looks clean. The JSON audit artifact records `suiteConsistency.strictSuiteStatusChecks`, `suiteConsistency.derivedTaskCount`, `suiteConsistency.derivedComparisonEligibleCount`, `suiteConsistency.derivedBoundaryObserved`, `suiteConsistency.derivedStatus`, `suiteConsistency.matchesDerivedTaskCount`, and `suiteConsistency.matchesDerivedStatus`; the focused contract test now rejects synthetic stale suite task-count and stale report-status reports.

Previous follow-up: Phase 334 makes saved agent-run proof outcome routing strict. `scripts/check-agent-run-proof.py` now derives `aggregate.outcomeClaim` and `aggregate.recommendedResearchActions` from source-free comparison aggregates and boundary flags, rejecting stale outcome labels and stale R&D routing actions. The JSON audit artifact records `aggregateConsistency.strictOutcomeRoutingChecks`, `aggregateConsistency.derivedOutcomeClaim`, and `aggregateConsistency.checkedRecommendedResearchActionCount`; the focused contract test now rejects synthetic stale outcome-claim and stale research-action reports.

Previous follow-up: Phase 333 makes saved agent-run proof comparison aggregates strict. `scripts/check-agent-run-proof.py` now derives top-level comparison aggregate metrics from `tasks[*].comparison`, rejecting stale `targetReadCoverageDeltaAverage`, `targetCoverageDeltaAverage`, `readFileDeltaSum`, `irrelevantReadDeltaSum`, `commandExecutionDeltaSum`, and `ctxhelmToolCallsObserved` values. The JSON audit artifact records `aggregateConsistency.strictComparisonAggregateChecks` and `aggregateConsistency.checkedComparisonAggregateMetricCount`; the focused contract test now rejects synthetic stale aggregate-delta and stale ctxhelm-tool-call-observed reports.

Previous follow-up: Phase 332 makes saved agent-run proof retry-cost and read-efficiency aggregates strict. `scripts/check-agent-run-proof.py` now derives `aggregate.retryCost` from per-task `comparison.retryCost` blocks and derives `aggregate.readEfficiency` from `aggregate.laneSummaries`, rejecting stale retry overhead, evidence-only target, recovered-target, extra-read, precision, and irrelevant-read-rate claims. The JSON audit artifact records `aggregateConsistency.strictRetryCostConsistencyChecks`, `aggregateConsistency.checkedRetryCostMetricCount`, `aggregateConsistency.strictReadEfficiencyConsistencyChecks`, and `aggregateConsistency.checkedReadEfficiencyMetricCount`; the focused contract test now rejects synthetic stale retry-cost and read-efficiency reports.

Previous follow-up: Phase 331 makes saved agent-run proof lane-summary metrics strict. `scripts/check-agent-run-proof.py` now derives `aggregate.laneSummaries[*]` metrics from `tasks[*].lanes[*].metrics` and lane role-count maps, rejecting stale read counts, target-read coverage, read precision, irrelevant-read rate, required-call counts, evidence counters, role counts, and failure/rate-limit counters even when the nested lane records remain clean. The JSON audit artifact records `aggregateConsistency.strictLaneSummaryMetricChecks`, `aggregateConsistency.checkedLaneSummaryCount`, and `aggregateConsistency.checkedLaneSummaryMetricCount`; the focused contract test now rejects a synthetic stale lane-summary metric report.

Previous follow-up: Phase 330 makes saved agent-run proof aggregate consistency strict. `scripts/check-agent-run-proof.py` now derives suite task count, comparison-eligible count, comparable ctxhelm lane count, strict boundary booleans, and aggregate lane-summary names from `tasks[*]`, rejecting saved reports whose top-level aggregate fields drift from derived task comparisons. The JSON audit artifact records `aggregateConsistency.strictAggregateConsistencyChecks`, `aggregateConsistency.derivedTaskCount`, `aggregateConsistency.derivedComparisonEligibleCount`, `aggregateConsistency.derivedComparableCtxhelmLaneCount`, `aggregateConsistency.derivedLaneNameCount`, `aggregateConsistency.laneSummaryCount`, and `aggregateConsistency.matchesDerivedAggregates`; the focused contract test now rejects synthetic stale aggregate-count and stale lane-summary-name reports whose nested task data is unchanged.

Previous follow-up: Phase 329 makes saved agent-run proof suite-task strict. When `--current-suite` is provided, `scripts/check-agent-run-proof.py` now re-parses the current suite and requires every saved `tasks[*].taskSha256` and `tasks[*].targetFiles` entry to match the current suite task text hash and target list. The JSON audit artifact records `suiteTaskChecks.strictCurrentSuiteTaskChecks`, `suiteTaskChecks.reportTaskCount`, `suiteTaskChecks.currentSuiteTaskCount`, and `suiteTaskChecks.matchesCurrentSuiteTasks`; the focused contract test now rejects a synthetic stale-suite-task report whose top-level `suite.suiteSha256` and aggregate fields are unchanged.

Previous follow-up: Phase 328 makes saved agent-run proof task-lane strict. `scripts/check-agent-run-proof.py` now validates every suite task comparison and every nested task lane, rejecting hidden client failures, rate limits, forbidden commands, missing/invalid required ctxhelm calls, under-read targets, evidence misses, evidence-only targets, and ctxhelm target-read coverage below the release floor even when aggregate summaries still look clean. The JSON audit artifact records `taskLaneChecks.strictTaskLaneChecks`, `taskLaneChecks.taskLaneCount`, and `taskLaneChecks.ctxhelmTaskLaneCount`; the focused contract test now rejects a synthetic stale nested-lane report whose aggregate fields are unchanged.

Previous follow-up: Phase 327 makes saved agent-run proof identity strict. `scripts/check-agent-run-proof.py` now accepts `--expected-ctxhelm-version`, `--expected-client-name`, and `--expected-client-version`, and rejects a report whose `ctxhelmVersion`, `client.name`, or `client.version` no longer matches the release-gate proof claim. The release gate passes the selected binary's `ctxhelm --version`, Codex as the expected client, and Codex `codex-cli 0.137.0` by default, with maintainer overrides through `CTXHELM_AGENT_RUN_EXPECTED_CLIENT_NAME` and `CTXHELM_AGENT_RUN_EXPECTED_CLIENT_VERSION`. The JSON audit artifact records `identity.*` match fields and the threshold values, and the focused contract test now rejects a synthetic stale-client report.

Previous follow-up: Phase 326 makes saved agent-run proof suite freshness strict. `scripts/check-agent-run-proof.py` now accepts `--current-suite` and rejects a suite report whose `suite.suiteSha256` does not match the current checked-in R&D breadth suite. The release gate passes `.planning/e2e/2026-06-06-phase251-codex-rd-suite.json` whenever `CTXHELM_AGENT_RUN_PROOF_REPORT` is set, and the JSON audit artifact records `currentSuiteName`, `currentSuiteSha256`, `matchesCurrentSuite`, and `requireCurrentSuite`. The committed Phase 322 report still passes because its suite hash matches the current four-task suite, while the focused contract test now rejects a synthetic stale-suite report.

Previous follow-up: Phase 325 makes saved agent-run proof freshness strict. `scripts/check-agent-run-proof.py` now accepts `--current-runner-script` and rejects a report whose `runner.scriptSha256` does not match the current local runner script hash. The release gate passes `scripts/e2e-agent-run-codex.sh` whenever `CTXHELM_AGENT_RUN_PROOF_REPORT` is set, and the JSON audit artifact records `currentRunnerScriptName`, `currentRunnerScriptSha256`, `matchesCurrentRunnerScript`, and `requireCurrentRunnerScript`. The committed Phase 322 report still passes because its runner hash matches the current Codex runner, while the focused contract test now rejects a synthetic stale-runner report.

Previous follow-up: Phase 324 makes the saved agent-run proof gate auditable after release validation. `scripts/check-agent-run-proof.py` now supports `--format json` and `--output`, emitting `ctxhelm-agent-run-proof-check-v1` with the saved report filename, report SHA-256, thresholds, source-free privacy checks, runner metadata, aggregate metrics, boundary checks, and lane quality summaries. When `CTXHELM_AGENT_RUN_PROOF_REPORT` is set, `scripts/release-gate.sh` writes `agent-run-outcome-proof.json` into `CTXHELM_PROOF_DIR` and records `agentRunOutcomeProofReport` in `release-proof-summary.json`, still without rerunning live clients or storing prompts, transcripts, MCP traffic, command output, source snippets, or absolute local report paths.

Previous follow-up: Phase 323 makes the real-agent outcome claim enforceable without rerunning live clients during release validation. `scripts/check-agent-run-proof.py` validates source-free paired agent-run JSON reports for schema, workflow, status, privacy fields, Codex runner fingerprint metadata, suite/task/comparable-lane floors, ctxhelm target-read coverage floors, retry-cost fields, read-delta thresholds, required-call compliance, and strict absence of client failures, rate limits, forbidden commands, evidence misses, evidence-only targets, and under-read targets. `scripts/release-gate.sh` now accepts `CTXHELM_AGENT_RUN_PROOF_REPORT` and optional `CTXHELM_REQUIRE_AGENT_RUN_PROOF=1`, validates the saved report with the checker, and records `agentRunOutcomeProof` plus `agentRunOutcomeProofRequired` in `release-proof-summary.json`.

Previous follow-up: Phase 322 turns the target-first prompt work into broad real-Codex evidence. The four-task Codex `0.137.0` breadth suite is `passed` with 4/4 comparison-eligible tasks, 16 comparable ctxhelm lanes, outcome `ctxhelm_improved`, target-read coverage delta average `+0.3125`, target coverage delta average `+0.25`, no forbidden commands, no missing/invalid ctxhelm calls, no client failures, no rate limits, no evidence misses, no evidence-only targets, and no under-read targets. Every ctxhelm lane reaches `1.0` average target-read coverage. Compared with Phase 318, noisy lanes are materially cleaner: plan reads `36 -> 23` and irrelevant `23 -> 10`, brief reads `35 -> 22` and irrelevant `22 -> 9`, standard reads `35 -> 23` and irrelevant `22 -> 10`, while memory reads `22 -> 21` and irrelevant `9 -> 8`. The efficient lane is `ctxhelm-memory`: baseline target-read coverage `0.6875 -> 1.0`, read precision `0.5294 -> 0.6190`, irrelevant-read rate `0.4706 -> 0.3810`, 4 recovered target reads, and 0 extra irrelevant reads. Do not claim absolute read reduction yet: the suite still reports `readFileDeltaSum = -2`, meaning two extra reads across best lanes to recover missed targets and reduce irrelevant reads.

Previous follow-up: Phase 321 starts the read-efficiency intervention for noisy Codex ctxhelm lanes. `ctxhelm-plan`, `ctxhelm-brief`, and `ctxhelm-standard` now use a target-first efficiency probe with at most 6 post-ctxhelm shell commands, no more than 6 current files read, a guard against batch-reading broad context-area/pack-neighbor/planning-doc lists, and an explicit stop rule: stop immediately after target-backed reads are enough to name the key files. The target-consumption requirements remain intact: read `targetFiles` first, treat docs/config/schema/script targets as first-class, preserve selected-memory reads, and keep read-only/no-test/no-write boundaries. A focused governor/release probe after the first command-budget-only edit preserved target-read coverage and improved plan/brief, but exposed standard-lane batching (`11` reads, `7` irrelevant), which motivated the file-count/batch-read guard. The final focused Codex `0.137.0` rerun is `passed`/`ctxhelm_improved`: every ctxhelm lane reached `1.0` target-read coverage with no forbidden commands, failures, rate limits, evidence misses, evidence-only targets, or under-read targets; `ctxhelm-standard` dropped to `6` reads and `2` irrelevant reads.

Previous follow-up: Phase 320 makes Codex suite checkpoints runner-fingerprinted before the next prompt/read-efficiency optimization round. `scripts/e2e-agent-run-codex.sh` now records a source-free `runner` block with name, contract version, script SHA-256, and `runner_fingerprint_v1` validation label in single-task and suite reports. Suite checkpoint reuse now requires matching runner fingerprint, ctxhelm version, Codex client version, task hash, target-file list, schema/workflow, non-empty lanes, and source-free privacy contract; stale checkpoints are deleted and rerun. This prevents future prompt or lane-ordering edits from silently reusing Phase 318/319 task reports.

Previous follow-up: Phase 319 adds source-free Codex read-efficiency metrics so the Phase 318 overhead finding is actionable. Per-lane reports now include `targetReadPrecision`, `irrelevantReadRate`, and `readsPerTargetRead`; single-task and suite reports add `readEfficiency` with baseline lane, efficient ctxhelm lane, target-read coverage delta, read precision delta, irrelevant-read-rate delta, recovered target reads, and extra reads per recovered target. Re-rendering the Phase 318 aggregate from checkpoints identifies `ctxhelm-memory` as the efficient lane: target-read coverage `0.625 -> 1.0`, read precision `0.5333 -> 0.5909`, irrelevant-read rate `0.4667 -> 0.4091`, with 7 extra reads and 2 extra irrelevant reads to recover 5 target reads. Plan/brief/standard remain noisy at roughly `0.36-0.37` read precision and `0.63-0.64` irrelevant-read rate, so the next implementation gate is to preserve 1.0 target-read coverage while moving those lanes toward memory-lane precision.

Previous follow-up: Phase 318 completes the checkpointed real Codex breadth-suite retry-cost proof. The four-task suite passed with 4/4 comparison-eligible tasks, 16 comparable ctxhelm lanes, outcome `ctxhelm_improved`, average target-read coverage `1.0` for every ctxhelm lane versus baseline `0.625`, average target-read delta `+0.375`, no evidence misses, no evidence-only targets after retry, no under-read targets, no forbidden commands, no client failures, and no rate limits. Retry triggered and selected in 4 lanes, closed evidence-only targets `6 -> 0`, and raised retry target-read coverage `0.5416666666666666 -> 1.0`. Efficiency is not yet proven: aggregate irrelevant-read delta is `-2`, read-file delta is `-7`, and retry increased avg reads `5.0 -> 6.75` and avg irrelevant reads `3.25 -> 3.5`. `scripts/e2e-agent-run-codex.sh` now emits `optimize_agent_read_efficiency(p2)` when ctxhelm improves target consumption at extra read cost, so the next R&D slice should target lane ordering, target-first stop rules, and memory-lane compaction rather than another retrieval algorithm.

Previous follow-up: Phase 317 records the first real Codex retry-cost proof and fixes the long-suite durability gap it exposed. A full four-task real Codex breadth-suite rerun timed out before writing an aggregate, which is a harness checkpointing gap rather than retrieval-quality evidence. A focused governor/release-proof run passed with 4 comparable ctxhelm lanes, outcome `ctxhelm_improved`, target-read coverage delta `+0.25`, no evidence misses, no evidence-only targets, no under-read targets, no forbidden commands, no client failures, and no rate limits. Retry was triggered and selected in all 4 ctxhelm lanes, closed evidence-only targets `6 -> 0`, and raised retry target-read coverage `0.625 -> 1.0`; average reads increased `5.0 -> 6.5` while average irrelevant reads stayed `2.5 -> 2.5`. `scripts/e2e-agent-run-codex.sh --suite` now accepts `--suite-work-dir`/`CTXHELM_AGENT_RUN_SUITE_WORK_DIR`, writes durable source-free per-task checkpoints, validates reused reports against the privacy contract, and records `checkpointEnabled`, `checkpointDirSha256`, `reusedTaskCount`, and per-task `reusedCheckpoint` in suite aggregates. Do not claim broad-suite retry efficiency until the checkpointed four-task suite completes.

Previous follow-up: Phase 316 continues Agent Outcome Reliability by adding source-free retry-cost accounting to Codex agent-run reports. `scripts/e2e-agent-run-codex.sh` now records per-lane retry eligibility, trigger, selection, read-file deltas, irrelevant-read deltas, target-read coverage deltas, and evidence-only target before/after counts. Single-run reports expose `comparison.retryCost`, suite reports expose `aggregate.retryCost`, and `ctxhelm eval agent-run` renders the new counters. This quantifies retry overhead without claiming efficiency until a fresh real Codex suite reruns with the new fields.

Previous follow-up: Phase 315 starts Agent Outcome Reliability. The first PR makes distribution metadata smoke temp-path clean by default, preserves an explicit `CTXHELM_UPDATE_DISTRIBUTION_METADATA=1` path for refreshing `.ctxhelm/distribution-metadata-smoke.json`, and adds release-gate post-smoke/final worktree cleanliness checks so validation does not leave tracked proof artifacts dirty.

Previous follow-up: Phase 314 applies the R&D completion gate. The active R&D scope is closed: Codex Phase 310/311 remains the current comparable outcome proof, memory Phase 253/255 remains the guard, semantic/default promotion is explicitly closed as diagnostic-only after the Phase 252-303 rejected same-family branches, and Claude cross-agent replication is classified as an external availability blocker based on Phase 312 rate-limit/client-failure evidence with zero comparable lanes.

Previous follow-up: Phase 313 records the strict R&D completion gate. Current head is strong but not fully complete: CI/docs are green, Codex Phase 310/311 closes the measured evidence-only consumption gap, memory Phase 253/255 remains guarded, and optional client/auth blockers are documented; semantic/default promotion still needs an explicit diagnostic-only closeout or no-regress lift proof, and Claude needs either a comparable paired suite or a formal external-availability closeout.

Previous follow-up: Phase 312 refreshes cross-agent proof after the Codex retry work. A tiny manual Claude Opus probe returned `OK`, but source-free availability still records Claude Code `2.1.163` as `rate_limited`, and the preflight-disabled four-task Claude R&D breadth suite is `degraded`: 0/4 comparison-eligible tasks, 0 comparable ctxhelm lanes, outcome claim `insufficient_comparable_lanes`, client failures and rate limits observed in every lane. The run did observe ctxhelm tool calls and memory-lane forbidden `Bash` calls, but it is not retrieval-quality evidence because every lane is non-comparable.

Previous follow-up: Phase 310-311 adds source-free Codex target-consumption retries for otherwise eligible ctxhelm lanes with evidence-only targets. The targeted governor proof clears evidence-only targets after selecting the `ctxhelm-memory` retry, and the four-task Codex R&D breadth suite passes with 4/4 comparison-eligible tasks, 16 comparable ctxhelm lanes, `ctxhelm_improved`, average target-read coverage delta `0.2916666666666667`, no evidence misses, no evidence-only targets, no under-read targets, no forbidden commands, no client failures, and no rate limits. Claude Code `2.1.163` remains rate-limited, so fresh Claude paired outcome proof is still classified as client availability rather than retrieval quality.

Previous follow-up: Phase 303 adds margin and threshold-sweep diagnostics to `ctxhelm eval retention-separator-train-test` after Claude Code recommended checking whether Phase 302's `0` eligible families were a threshold artifact. Across the same four recent-to-older proofs, only 8 of 117 train families have positive target/non-target margin (`0.068376`), below the pre-registered `15%` continuation bar; the best relaxed aggregate row recovers only 3 dropped targets while inserting 21 non-targets. Treat semantic retention separation as structurally weak under this proxy and pivot the next R&D branch toward fresh agent outcome proof rather than another separator relaxation.

Previous follow-up: Phase 302 adds `ctxhelm eval retention-separator-train-test`, a source-free held-out diagnostic that trains strict retention-family eligibility on a recent range and applies it to dropped supported semantic candidates in a disjoint older range. Across ctxhelm, ReAgent, RefactoringMiner, and VeriSchema, the report sees 117 train families and 377 test dropped profiles, but zero eligible families and zero applications. The decision is `insufficient_train_families` on all four repos. Keep semantic retention separation diagnostic-only and do not relax eligibility inside the same proof just to force applications.

Previous follow-up: Phase 301 adds `semanticCandidateRetentionSummary` to historical eval and semantic gate reports so supported semantic candidates are measured as retained/dropped target/non-target cells without changing ranking. Across the eight Phase 299/300 slices, the retention surface has 1071 profiles: 52 retained targets, 22 dropped targets, 232 retained non-targets, and 765 dropped non-targets. Target retention is `0.702703`, non-target drop rate is `0.767302`, and there are 12 recoverable dropped-target family rows. This is useful held-out separator input, not runtime/default promotion evidence.

Previous follow-up: Phase 300 adds `supportedSemanticCandidateProfileSummary` to historical eval and semantic gate reports so supported semantic candidate base rates are machine-readable without adding another eval-only predictor. Across the eight Phase 299 slices, supported semantic candidate profiles have 787 rows, 22 targets, 765 non-targets, and only `0.027954` global target precision; the per-slice shape surface has 171 rows, 90 thin cells, and only one repeated-target shape, which is noisy docs/planning evidence rather than a promotable source shape. Keep supported-shape work diagnostic-only and move semantic R&D toward broader source-free candidate-retention evidence or a held-out learned separator.

Previous follow-up: Phase 299 broadens Phase 298's source-free `semantic_supported_shape_tail_slot_reranked` validation across RefactoringMiner, ctxhelm, ReAgent, and VeriSchema older/recent ranges after a clean feature-enabled Jina rebuild. Across 159 evaluated commits, the supported-shape predictor is safe but sparse: target-hit delta `+1`, improved commits `1`, regressed commits `0`, and default-only target hits `0`. The only lift remains the original VeriSchema older `3507d7c932c4` case recovering `schema_agent/core/state.py`; the other seven slices are neutral. Keep the variant eval-only and treat the next semantic R&D path as broader source-free candidate-retention or held-out learned-separator work, not runtime/default promotion.

Previous follow-up: Phase 298 replaces Phase 297's gold-backed oracle dependency with the eval-only source-free `semantic_supported_shape_tail_slot_reranked` variant and `supportedShapeTailSlotSemanticRerankerContribution`. The variant uses generated `supportedSemanticCandidateProfilesAt10` rows, not target-miss profiles, and only inserts supported candidates matching the measured `symbol_identifier` / `python_source` / `dependency_co_change` shape into protected tail slots. On the same feature-enabled VeriSchema older-range Jina `candidate-path-hints` proof, it recovers `schema_agent/core/state.py` for commit `3507d7c932c4`, matches the oracle's target-hit lift `13 -> 14`, raises file Recall@10 to `0.3438596`, and has zero regressions/default-only target churn. Keep it eval-only until broader cross-repo/range proof shows the narrow shape is repeatable.

Previous follow-up: Phase 297 adds the eval-only `semantic_supported_candidate_tail_slot_oracle` variant and `supportedCandidateTailSlotRerankerContribution` to measure the upper bound from Phase 296's supported candidate-missed surface. On the feature-enabled VeriSchema older-range Jina `candidate-path-hints` proof, the oracle recovers `schema_agent/core/state.py` for commit `3507d7c932c4`, improves target hits `13 -> 14`, raises file Recall@10 to `0.3438596`, and has zero regressions/default-only target churn. This is not runtime-promotable because it consumes eval target-miss profiles; it points the next semantic R&D slice at a source-free predictor for `symbol_identifier` / `python_source` supported semantic candidates.

Previous follow-up: Phase 296 adds `semanticCandidateMissedSupportProfiles` and per-query-family `candidateMissedSupportProfiles` so semantic-generated target candidates dropped by final top-K selection are grouped by non-semantic source-free support. On the same feature-enabled VeriSchema older-range Jina `candidate-path-hints` proof, the only semantic candidate miss is `schema_agent/core/state.py` with `dependency_co_change` support. The gate remains held (`local_semantic` Recall@10 unchanged, recall delta `+0.000`), but it now emits `semantic_candidate_fusion_supported_gap`, pointing the next semantic R&D slice at fusion/top-K ordering for supported semantic candidates rather than broader documents, MiniLM12 swaps, next-read promotion, or default Jina policy.

Previous follow-up: Phase 295 tests the documented `AllMiniLML12V2Q` and `AllMiniLML12V2` local-fastembed model branch on the same VeriSchema older-range `candidate-path-hints` proof after confirming a feature-enabled binary. Both variants are hard negatives versus corrected Jina: candidate misses are worse (`1 -> 3`), semantic-only non-targets rise (`12 -> 18/19`), bounded next-read noise rises (`1 -> 3` non-target appends with zero target appends), `local_semantic` Recall@10 remains `0.29122806`, `semantic_corroborated_reranked targetHitDelta = -1`, runtime ratio is `6.75x/7.34x`, and tail-slot reranking remains neutral. Reject MiniLM12 as a simple model-swap path; corrected Jina remains the useful diagnostic backend, not a default policy.

Previous follow-up: Phase 294 adds `semanticNextReadContribution`, a source-free eval diagnostic that preserves the full default top K and measures up to two semantic-ranked next-read paths after that protected budget. On the targeted VeriSchema older-range Jina candidate-path proof, semantic appends only one next-read path and it is a non-target (`tests/core/test_state_validator.py`): `appendedTargetHitCount = 0`, `appendedNonTargetCount = 1`, diagnostic `semantic_next_read_noise_hold`. This rejects semantic next-read promotion for the current Jina/candidate-path setup; keep the field diagnostic-only.

Previous follow-up: Phase 293 fixes the explicit `JinaEmbeddingsV2BaseCode` fastembed contract after the first model probe exposed zero semantic candidates from a dimension mismatch. Jina now normalizes to `768` dimensions, renders provider-specific `query:` / `passage:` text, and hashes the provider-specific document text contract so stale old-shape vectors are not reused. The targeted VeriSchema older-range proof with `--semantic-model JinaEmbeddingsV2BaseCode --semantic-query-mode candidate-path-hints` improves candidate quality versus Phase 289 AllMini candidate-path hints: candidate misses fall `3 -> 1`, selected semantic targets rise `11 -> 12`, and semantic-only non-targets fall `16 -> 12`. It is still held, not promoted: `local_semantic` recall remains `0.29122806`, `semantic_corroborated_reranked` still has `targetHitDelta = -1`, tail-slot reranking remains neutral, and runtime ratio is `6.01x` with recall delta `+0.000`.

Previous follow-up: Phase 292 adds `--semantic-query-mode candidate-path-concept-hints`, an eval-only source-free query-construction probe that keeps candidate-path aliases and appends up to four generic software-domain concept terms such as formal/solver/constraint for verification or state/transition/orchestration for workflow. The targeted VeriSchema older-range proof rejects it: candidate targets fall `14 -> 13`, candidate misses worsen `3 -> 4`, selected semantic targets fall `11 -> 9`, semantic-only non-targets rise `16 -> 19`, `semantic_corroborated_reranked` still has `targetHitDelta = -1`, and `semantic_tail_slot_reranked` now regresses with `targetHitDelta = -1`. Stop adding terms to a single local-fastembed query; the next semantic branch needs a materially different source-free document construction experiment or a narrower learned separator with held-out no-regress evidence.

Previous follow-up: Phase 291 adds `--semantic-query-mode candidate-sibling-path-hints`, an eval-only source-free query-construction probe that appends bounded same-directory and mirrored-test aliases near top lexical candidates. The targeted VeriSchema older-range proof rejects it: candidate misses worsen from Phase 289 `3 -> 4`, selected semantic targets fall `11 -> 10`, `semantic_corroborated_reranked` still has `targetHitDelta = -1`, and `semantic_tail_slot_reranked` remains neutral. Do not pursue more sibling path aliases as the semantic fix; the next branch should be richer source-free document/query construction for prompt/workflow/verification concepts or a narrower learned separator.

Previous follow-up: Phase 290 adds the eval-only `semantic_tail_slot_reranked` variant, which preserves the top `ceil(0.8 * K)` default context files and lets semantic-corroborated candidates compete only for the remaining tail slots. On the targeted VeriSchema older-range candidate-path-hints proof, it removes the known semantic-corroborated regression (`targetHitDelta = 0`, `regressedCommitCount = 0`, `defaultOnlyTargetHitCount = 0`) but adds no target hits (`rerankerOnlyTargetHitCount = 0`). This is safety evidence, not promotion evidence; richer query/document construction or a narrower learned separator remains the semantic path.

Previous follow-up: Phase 289 adds `--semantic-query-mode candidate-path-hints`, an eval-only task-specific query-construction probe that appends bounded source-free aliases from top lexical candidate paths to the semantic query after lexical search. The targeted VeriSchema older-range proof improves candidate quality versus plain/source-role hints: candidate misses fall to `3`, selected semantic target hits rise to `11`, semantic-only targets stay `2`, and semantic-only non-targets fall to `16`. It still rejects promotion because `local_semantic` recall remains `0.29122806`, `semantic_corroborated_reranked` remains `0.28245613`, and the corroborated variant still has `targetHitDelta = -1` with one regressed commit.

Previous follow-up: Phase 288 adds `ctxhelm eval learned-policy-cross-repo`, a source-free artifact-level diagnostic that aggregates saved learned-policy train/test reports across repositories and requires repeated repo support, positive inserted target hits, zero inserted non-targets, and zero lost default targets. The four-repo Phase 285 strict aggregation rejects promotion with `28` candidate profiles and `0` eligible profiles; the repeated `symbol_identifier/docs` key has only `3` inserted targets but `25` inserted non-targets and `20` lost default targets. The Phase 286 backoff aggregation is also rejected with `9` candidates and `0` eligible profiles; `*/docs` has `16` inserted targets but `140` inserted non-targets and `125` lost defaults. Cross-repo learned-policy aggregation is not the current semantic promotion path.

Previous follow-up: Phase 287 adds `--semantic-query-mode source-role-hints` as an eval-only semantic query-construction probe. The mode appends generic coding-role terms plus dominant source languages to explicit eval semantic queries while keeping runtime/default behavior at `plain` and making cache keys mode-distinct. The targeted VeriSchema older-range local-fastembed proof rejects the idea: candidate target hits rise only `13 -> 14`, candidate misses rise `4 -> 6`, selected semantic targets fall `9 -> 8`, semantic-only targets fall `2 -> 0`, and `semantic_corroborated_reranked` keeps the same `targetHitDelta = -1` with one regressed commit. Generic source-role words are not the next promotion path; query/document work needs task-specific construction or a cross-repo learned rule with a held-out no-regress bar.

Previous follow-up: Phase 286 adds `--profile-key-mode path-family-backoff` to `ctxhelm eval learned-policy-train-test` as an eval-only diagnostic relaxation. It aggregates training profiles by path family (`queryFamily = "*"`) while keeping the same support and zero-harm gates and emitting per-query-family breakdown rows. The four-repo recent-to-older local-fastembed proof rejects this backoff for promotion: coarse train/test overlap increases, but all path-family aggregates are blocked by inserted semantic non-targets, missing target hits, or lost default targets; every repo still has `eligibleProfileCount = 0`, `appliedCommitCount = 0`, and `targetHitDelta = 0`. The next semantic branch should move away from profile-key relaxation and toward semantic query/document construction or a richer cross-repo rule that does not collapse noisy query families.

Previous follow-up: Phase 285 tests whether Phase 284's empty learned-policy train/test result is only a small-slice power problem by rerunning the same source-free recent-to-older ranges with `--train-limit 40 --test-limit 40`. It remains empty under the pre-registered bar: ctxhelm/ReAgent/RefactoringMiner only expose `25` train commits in the selected recent range, VeriSchema reaches `38` train and `39` test commits, but every repo still has `trainTestEligibleProfileOverlapCount = 0` and `appliedCommitCount = 0`. This is not a no-regress win; it is a no-signal result. Stop doubling these ranges and move next to a pre-registered backoff/cross-repo aggregation rule or semantic query/document construction.

Previous follow-up: Phase 284 adds `ctxhelm eval learned-policy-train-test`, a source-free evaluator that trains learned semantic policy profiles on one revision range and applies them to a disjoint test range while reporting support histograms, train/test profile overlap, applied commits/files, target-hit delta, regressions, and `runtimePromotable = false`. The four-repo recent-to-older local-fastembed proof is negative: ctxhelm, ReAgent, and VeriSchema have `0` eligible training profiles, while RefactoringMiner trains the single eligible `symbol_identifier/docs` profile but has `0` eligible-profile overlap and `0` test applications. This confirms the current learned-policy signal is still too sparse for promotion; next work should increase statistical surface, add pre-registered backoff/cross-repo aggregation, or pivot back to query/document construction for semantic candidate quality.

Previous follow-up: Phase 283 adds `--base` and `--head` to `ctxhelm eval gate` so semantic gate variants can be measured on stable source-free revision slices. Older/recent four-repo local-fastembed probes show the current learned-policy signal still does not repeat across slices: RefactoringMiner recent recreates the only eligible durable row (`symbol_identifier/docs`, observed in `2` commits) and learned-profile lift (`0.41857147 -> 0.51857144`), but RefactoringMiner older and all ctxhelm/ReAgent/VeriSchema slices have `0` eligible profiles and every held-out exported-policy variant has `appliedCommitCount = 0`. The next policy experiment needs cross-repo aggregation or a true train-on-range/apply-on-disjoint-test-range evaluator.

Previous follow-up: Phase 282 broadens the learned-policy holdout probe from `limit 20` to `limit 40` to test whether Phase 281 was only a small-slice artifact. It is still no-regress but still too sparse for promotion: RefactoringMiner keeps learned-profile lift (`0.38511905 -> 0.43511906`) and exports only one eligible durable row (`symbol_identifier/docs`, observed in `2` commits), while the held-out exported-policy variant applies to `0` commits on RefactoringMiner, ctxhelm, ReAgent, and VeriSchema. Same-gate broadening is therefore insufficient; the next semantic learned-policy work needs explicit train/test revision ranges or cross-repo profile aggregation with no-regress gates.

Previous follow-up: Phase 281 adds eval-only `semantic_learned_policy_holdout_reranked`, `learnedSemanticPolicyHoldout`, and `learnedPolicySemanticHoldoutContribution`. The source-free holdout applies the Phase 280 repeated-support threshold (`minimumSupportCommitCount = 2`) while leaving each measured commit out of its own profile evidence. The four-repo local-fastembed proof is no-regress but too sparse for promotion: RefactoringMiner's full learned-profile lift remains (`0.41857147 -> 0.51857144`), but the held-out exported-policy variant applies to `0` commits on RefactoringMiner, ctxhelm, ReAgent, and VeriSchema, so all holdout reports decide `insufficient_eligible_holdout_profiles` and keep `runtimePromotable = false`.

Previous follow-up: Phase 280 adds `learnedSemanticPolicy` to the semantic gate report. The artifact is source-free and deterministic, records schema version, policy id, source variant, eval range, ranking budget, `minimumSupportCommitCount = 2`, profile eligibility/blocking reasons, staleness status, and holdout status. The four-repo local-fastembed proof preserves Phase 278's learned-profile metrics: RefactoringMiner still improves (`0.41857147 -> 0.51857144`, `+2`, no regressions), while ctxhelm, ReAgent, and VeriSchema remain neutral. Only RefactoringMiner exports an eligible durable profile (`symbol_identifier/docs`, observed in `2` commits); all artifacts set `runtimePromotable = false` until cross-repo holdout proof exists.

Previous follow-up: Phase 279 promotes safe changed paths from task-mentioned commits into prepare-task anchors. When the task explicitly says `commit` and includes 7-40 hex revision tokens, ctxhelm runs local `git diff-tree --name-status -z` for those revisions, filters the paths through the safe inventory, adds them as explicit path facets, and emits source-free diagnostics. This targets public-commit reproduction tasks without logging commit subjects or source text.

Previous follow-up: Phase 278 adds eval-only `semantic_learned_profile_reranked` and `learnedProfileSemanticRerankerContribution`. The variant uses leave-one-out source-free profile evidence keyed by `(query_family, path_family)` and admits semantic-corroborated candidates only when other commits show inserted target hits with zero inserted non-targets and zero lost default targets. This is the first post-Phase-270 semantic reranker probe with four-repo no-regress evidence: RefactoringMiner improves (`0.41857147 -> 0.51857144`, `+2` target hits, `0` regressions), while ctxhelm, ReAgent, and VeriSchema remain exactly neutral with no default-only target churn. Keep it eval-only until there is a durable learned-policy artifact, staleness contract, support threshold, and broader holdout proof.

Previous follow-up: Phase 277 adds eval-only `semantic_family_budget_reranked` and `familyBudgetSemanticRerankerContribution` to test a source-free path-family budget constraint after Phase 276's displacement findings. The variant is mixed and rejected for promotion: RefactoringMiner improves cleanly but weakly (`0.41857147 -> 0.43690476`, `+2` target hits), ReAgent improves (`0.35 -> 0.4`, `+1`), but ctxhelm regresses hard (`0.44620585 -> 0.37543336`, `-10`, `9` regressed commits) and VeriSchema regresses (`0.39382353 -> 0.34382352`, `-1`). Keep the eval-only diagnostics; do not promote. The next semantic path should be learned/listwise allocation with explicit no-regress constraints or a much narrower corpus/profile-specific policy.

Previous follow-up: Phase 276 adds source-free displacement diagnostics to `RerankerContributionSummary`. `displacementContributions` now shows which reranked-only top-K path families entered the fixed budget when default target files were lost, plus the lost target path families. The four-repo semantic-corroborated gate keeps RefactoringMiner's clean lift unchanged (`0.41857147 -> 0.5619047`, no displacement rows), while explaining the blockers: ctxhelm inserts planning/scripts/docs non-targets while losing planning/docs/Rust-source targets, ReAgent inserts docs/paper/planning non-targets while losing planning/script targets, and VeriSchema inserts docs/scripts/paper while losing Python source targets. The new `semantic_corroborated_displacement_pressure` diagnostic confirms the next semantic step should be target-preserving budget constraints or learned/listwise allocation, not another handwritten route.

Previous follow-up: Phase 275 adds source-free query/path shape diagnostics to `semanticCorroboratedRerankerContribution` and rejects a temporary shape-routed semantic insertion experiment. The diagnostics show RefactoringMiner's clean lift is concentrated in `symbol_identifier/docs +2`, `commit_clue/config +1`, `domain_phrase/java_source +1`, and `domain_phrase/java_test +1`, but cross-repo blockers remain: ctxhelm loses/churns Rust source and planning shapes, ReAgent blocks `symbol_identifier/scripts` and `symbol_identifier/planning`, and VeriSchema blocks Python source shapes. The temporary shape-routed variant failed to preserve lift (`RefactoringMiner 0.41857147 -> 0.40023813`, ReAgent `0.35 -> 0.325`) by displacing default target tests, so it was removed. Shape diagnostics stay; shape-routed promotion is rejected.

Previous follow-up: Phase 274 tests path-derived Python package/module semantic facets for VeriSchema and rejects them. The temporary source-free facets increased semantic candidate target hits from `11` to `12`, but semantic-corroborated recall fell from `0.36960787` to `0.32294118`, semantic candidate missed targets appeared (`0 -> 2`), semantic-only non-targets increased (`17 -> 18`), and Python source stayed churned. The code was reverted; the remaining VeriSchema semantic path is task/query construction or fusion constraints around Python source targets.

Previous follow-up: Phase 273 probes the VeriSchema Python-source semantic gap by raising `CTXHELM_FASTEMBED_DOCUMENT_LIMIT=256`. The clean feature-enabled run leaves metrics unchanged: local semantic `0.39382353`, semantic-corroborated `0.36960787`, semantic candidate target hits `11`, semantic-only targets `6`, semantic-only non-targets `17`, and `python_source -10`. Increasing the local-fastembed prefilter cap is rejected; the remaining VeriSchema semantic path is query/document construction, Python package/module facets, or fusion around Python source targets.

Previous follow-up: Phase 272 adds source-free path-family contribution diagnostics to `RerankerContributionSummary` and reruns the four-repo semantic-corroborated gate. The result rejects broad path-family routing: RefactoringMiner's `+5` target-hit delta is clean across `docs`, `config`, `java_source`, and `java_test`, but ctxhelm has `docs +11` with churn plus `rust_source -18`, ReAgent loses `scripts` and `planning`, and VeriSchema loses `python_source -10`. Semantic remains opt-in; the next R&D should test narrower corpus-shape constraints such as Java/MCP/package coherence or focus on VeriSchema query/document construction.

Previous follow-up: Phase 271 adds source-free `semanticCorroboratedRerankerContribution` diagnostics for the eval-only `semantic_corroborated_reranked` variant. Fresh four-repo `limit 20` local-fastembed gates reject query-family-only routing: RefactoringMiner has clean route candidates for `domain_phrase` (`+2`), `symbol_identifier` (`+2`), and `commit_clue` (`+1`) with no default-only target churn, but `domain_phrase` blocks on ctxhelm/ReAgent/VeriSchema and `symbol_identifier` churns ctxhelm while regressing ReAgent. Semantic remains opt-in; the next R&D should test corpus-shape/path-role constraints for the RefactoringMiner-shaped lift and separate query/document construction for VeriSchema.

Previous follow-up: Phase 270 adds the eval-only `semantic_corroborated_reranked` variant to test the Phase 269 fusion hypothesis. The variant preserves protected source evidence and gives bounded semantic credit only when another source-free signal corroborates the candidate. Fresh four-repo `limit 20` local-fastembed gates reject global promotion: RefactoringMiner improves strongly (`0.41857147 -> 0.5619047`) with `5` named wins and `0` regressions, but ctxhelm regresses (`0.44620585 -> 0.44333735`, `7` wins and `9` regressions), ReAgent regresses (`0.35 -> 0.325`, `2` wins and `3` regressions), and VeriSchema regresses (`0.39382353 -> 0.36960787`, `0` wins and `3` regressions). Semantic remains opt-in; the next R&D should route the RefactoringMiner-shaped lift by query family/corpus shape or reject it if no stable separator exists.

Previous follow-up: Phase 269 adds semantic candidate-generation versus fusion diagnostics. `semanticContribution` now reports semantic candidate target hits separately from selected semantic target hits, plus `semantic_candidate_generation_gap` and `semantic_candidate_fusion_gap` diagnostics. Fresh four-repo `limit 20` local-fastembed gates show the next semantic bottleneck is split: ctxhelm selected `7` target hits but had `24` candidate target hits with `17` candidate misses; ReAgent selected `2` but had `5` candidates with `3` candidate misses; RefactoringMiner selected `9` but had `12` candidates with `3` candidate misses; VeriSchema selected and generated `11` target hits with no semantic candidate misses, so its remaining gap is candidate generation/query coverage. Semantic remains opt-in; the next R&D should test fusion/budget constraints for the first three repos and separate query/model/document coverage for VeriSchema.

Previous follow-up: Phase 268 adds the eval-only `support_profile_routed_semantic` variant proposed by Phase 267. The synthetic route inserts semantic-only candidates only when their query-family/support-family profile had semantic-only target hits and no semantic-only non-targets, while excluding profile noise holds and leaving runtime policy unchanged. Fresh four-repo `limit 20` local-fastembed gates show the hypothesis is safe but not useful enough: ctxhelm, ReAgent, RefactoringMiner, and VeriSchema all have zero support-profile-routed regressions, but also zero support-profile-routed wins; Recall@10 remains unchanged versus default on all four repos. This rejects support-profile routing as the next promotion path and moves semantic R&D toward better local query/model/fusion construction.

Previous follow-up: Phase 267 adds support-profile-level semantic diagnostics after Phase 266 rejected generic corroboration. Gate reports now emit `semantic_support_profile_route_candidate`, `semantic_support_profile_mixed_hold`, and `semantic_support_profile_noise_hold`. Fresh four-repo `limit 20` local-fastembed gates find sparse profile-level route candidates, but still no runtime promotion: ctxhelm has `1` route candidate, `1` mixed hold, and `3` noise holds; ReAgent has `0` route candidates and `8` noise holds; RefactoringMiner has `0` route candidates, `2` mixed holds, and `6` noise holds; VeriSchema has `3` route candidates, `2` mixed holds, and `13` noise holds while the gate remains blocked. The next semantic R&D should test an eval-only variant that includes route-candidate support profiles and excludes noise profiles, then measure recall/churn/regressions before any provider-policy exposure.

Previous follow-up: Phase 266 adds semantic-only support profiles to test the next stricter within-family hypothesis after Phase 265 rejected coarse family routing. `semanticContribution.queryFamilyContributions` now reports whether semantic-only targets and non-targets had non-semantic source-free support, plus `supportProfiles` grouped by support family. Fresh four-repo `limit 20` local-fastembed gates reject the simple "semantic + any other signal" policy: ctxhelm `broad_scope`/`domain_phrase`, RefactoringMiner `symbol_identifier`, and VeriSchema `domain_phrase`/`broad_scope` all have corroborated targets and corroborated non-targets; ReAgent families and several other families have corroborated semantic-only non-targets with no corroborated semantic-only targets. Semantic remains opt-in; the next semantic R&D should test support-family-specific constraints or alternate local query/document construction.

Previous follow-up: Phase 265 broadens semantic query-family validation from Phase 264's `limit 10` slice to four `limit 20` local-fastembed gates and adds commit-level stability counters to `semanticContribution.queryFamilyContributions`. The new fields report semantic-only target-hit commits, semantic-only non-target commits, clean target-only commits, mixed commits, noise-only commits, and matching rates. Fresh evidence rejects coarse semantic family routing for now: ctxhelm's Phase 264 `broad_scope` candidate becomes unstable (`1` clean target-only commit and `1` noise-only commit), VeriSchema's `domain_phrase` candidate becomes unstable (`2` clean target-only commits and `1` noise-only commit), ReAgent remains pure semantic noise by family, and RefactoringMiner `symbol_identifier` has small semantic lift but heavy noise (`2` target-hit commits, `5` non-target commits, `1` mixed commit, and `4` noise-only commits). Semantic remains opt-in; the next semantic R&D should test stricter within-family gating or alternate local query/document construction against these stability counters.

Previous follow-up: Phase 264 adds semantic query-family contribution diagnostics and reruns four local-fastembed gates. `semanticContribution.queryFamilyContributions` now reports semantic-selected files, semantic-only target hits, semantic-only non-targets, missed targets, gap families, and example cases by query family. Fresh evidence keeps semantic opt-in: RefactoringMiner gets small semantic lift (`0.5383333 -> 0.5583333`) but `symbol_identifier` is mixed (`1` target, `9` non-targets); ctxhelm `broad_scope` has `2` semantic-only targets and `0` non-targets but no aggregate lift; ReAgent has semantic-only non-targets with no unique target hits; VeriSchema `domain_phrase` has `3` semantic-only targets and `0` non-targets but the gate remains blocked by named regressions. Phase 264 also confirms Claude Code `2.1.163` is still rate-limited while Codex CLI remains available.

Previous follow-up: Phase 263 promotes the Phase 262 routed-reranker finding into opt-in runtime policy without changing defaults. `.ctxhelm/provider-policy.json` can now set `enableQueryFamilyRoutedReranker: true`; planner/provider policy reports expose the decision as `local_metadata_routed`, and runtime ranking applies the local metadata reranker only for the measured route-safe `commit_clue` family. Unproven families such as `symbol_identifier` keep default ranking and emit `query_family_routed_reranker_held`. CLI proof shows both applied and held branches stay source-free and local-only.

Previous follow-up: Phase 262 broadens routed-reranker proof across RefactoringMiner, ctxhelm, ReAgent, and VeriSchema and tightens the route policy. The broader run rejected the Phase 261 `symbol_identifier` route because ReAgent regressed by 5 target hits, so `query_family_routed_reranked` now routes only `commit_clue` and leaves `symbol_identifier` at default ranking. Fresh four-repo evidence shows routed reranking has zero routed regressions and zero routed default-only churn on all four repos; ctxhelm keeps useful lift (`default Recall@10 0.3536905 -> routed 0.5005952`, `routedRerankerContribution.targetHitDelta = +9`), while RefactoringMiner, ReAgent, and VeriSchema are neutral under the routed variant. Full reranking remains unsafe on ReAgent and VeriSchema.

Previous follow-up: Phase 261 adds an eval-only `query_family_routed_reranked` variant that applies the local metadata reranker to initial route-safe families (`commit_clue` and `symbol_identifier`). Fresh gates show RefactoringMiner remains neutral with no churn, while ctxhelm improves from default Recall@10 `0.3036905` to routed Recall@10 `0.48845237`, with `routedRerankerContribution.targetHitDelta = +11`, `regressedCommitCount = 0`, and `defaultOnlyTargetHitCount = 0`. Phase 262 later rejected the `symbol_identifier` part of this policy after broader ReAgent proof.

Previous follow-up: Phase 260 adds source-free query-family routing diagnostics to `rerankerContribution`. The semantic/precision gate now groups `local_metadata_reranked` lift, regressions, default-only churn, and routing recommendations by primary query family. Fresh gates show RefactoringMiner remains family-neutral (`domain_phrase` and `symbol_identifier` both `hold_neutral`), while ctxhelm splits into clean route candidates for `commit_clue` and `symbol_identifier` and a `domain_phrase` family held for churn. This gives learned/routed reranker R&D concrete family-level evidence without enabling default reranking.

Previous follow-up: Phase 259 aligns the policy-enabled runtime local metadata reranker with the eval safety contract. `rerank_with_local_metadata` now uses a protected-source two-stage sort: source candidates with anchor/current-diff/lexical/symbol evidence sort before remaining metadata-ranked candidates, while tests stay outside that protected source floor and continue through normal selection budgets. A planner test proves `.ctxhelm/provider-policy.json` with `enableLocalMetadataReranker: true` allows and applies the reranker through `prepare_context_plan`. Fresh gates still have zero named regressions; RefactoringMiner remains neutral (`targetHitDelta = 0`) and ctxhelm shows `targetHitDelta = +16` with the known two `.planning/STATE.md` churn cases. This promotes runtime safety for opt-in reranking, not default reranking.

Previous follow-up: Phase 258 adds source-free `rerankerContribution` diagnostics to semantic/precision gate reports. The report now counts improved/regressed/neutral commits, target-hit delta, reranker-only target hits, default-only target hits, protected miss-rate deltas, and target-churn cases. Fresh gates show RefactoringMiner is neutral (`targetHitDelta = 0`, `improvedCommitCount = 0`, `regressedCommitCount = 0`) while ctxhelm has strong net lift (`targetHitDelta = +14`, `improvedCommitCount = 8`, `regressedCommitCount = 0`) plus explicit target churn (`defaultOnlyTargetHitCount = 2`, both `.planning/STATE.md`). This supports routing/learned-fusion R&D; it does not justify unconditional default reranking.

Previous follow-up: Phase 257 makes the eval-only local metadata reranker preserve protected source evidence from the default top-K ranking and keep the existing validation-test reserve. Fresh source-free gates have zero named regressions on both checked corpora. RefactoringMiner is safe but neutral (`ctxhelm_default` Recall@10 `0.5383333`, `local_metadata_reranked` `0.5383333`) while protected miss-rate improves from `0.35164836` to `0.20879121`. ctxhelm improves strongly (`0.30929655 -> 0.5414069`) while protected miss-rate improves from `0.5670103` to `0.15463917` and test recall stays `1.0`. The gate remains `hold`, so the change promotes safety and continued eval, not default reranking.

Previous follow-up: Phase 256 tests and rejects richer source-free semantic search documents for `local_fastembed`. Temporarily adding symbol/dependency facets to semantic search/index vectors preserved privacy but removed the RefactoringMiner semantic lift (`local_semantic` Recall@10 `0.5583333 -> 0.5383333`), kept ctxhelm neutral (`0.31237346 -> 0.31237346`), and worsened semantic runtime (`14457ms -> 101561ms` on RefactoringMiner and `9938ms -> 23837ms` on ctxhelm). The change was reverted. Semantic remains opt-in; the next semantic R&D should target query construction, alternate local model/fusion, or safe metadata-reranker promotion constraints rather than richer default semantic documents.

Previous follow-up: Phase 255 tightens the Codex `ctxhelm-memory` lane into a memory-efficiency probe and reruns the same three-repo memory outcome suite as Phase 253. Target consumption is preserved (`memoryTargetReadImprovedPairCount = 3`, `memoryTargetReadMatchedOrImprovedPairCount = 3`) and memory irrelevant-read improvement moves from 0/3 to 3/3. Per-repo memory reads drop from `6 -> 2`, `6 -> 4`, and `5 -> 2`, while memory irrelevant reads drop from `5 -> 1`, `5 -> 3`, and `4 -> 1`. The report remains source-free and local-only with no evidence misses, under-read targets, client failures, rate limits, forbidden commands, or malformed required ctxhelm calls.

Previous follow-up: Phase 254 refreshes the Claude Code R&D breadth-suite state with Claude Code `2.1.163` and the same four-task suite hash as Phase 252. The report is still correctly `degraded`: all four client preflights report rate limiting, leaving zero comparison-eligible tasks and zero comparable ctxhelm lanes. This is current client-availability evidence, not retrieval-quality evidence.

Previous follow-up: Phase 253 adds a source-free cross-repo Codex memory outcome suite. `scripts/e2e-codex-memory-outcome-suite.sh` scans each requested repository for repeated-file historical pairs, seeds one approved source-free experience card from the older pair task, and runs the existing read-only Codex paired harness against the newer task in an isolated `CTXHELM_HOME`. The committed artifact covers VeriSchema, ReAgent, and RefactoringMiner, reports `passed`, 3 comparison-eligible pairs, `improvedPairCount = 3`, `memoryTargetReadImprovedPairCount = 3`, no evidence misses, no under-read targets, no client failures, no rate limits, no forbidden commands, and no malformed required ctxhelm calls. This closes the current cross-repo memory target-consumption gap, but not memory read efficiency: `memoryIrrelevantReadImprovedPairCount = 0`.

Previous follow-up: Phase 252 fixes the semantic/precision gate classification so eval-only local metadata reranker regressions remain visible in `namedRegressions` but no longer force the top-level semantic default decision to `block`. Fresh source-free `local_fastembed` gates now report the accurate state: RefactoringMiner is `hold` with a small semantic lift (`0.5104166` vs default `0.48541665`) and one semantic-only target hit, while ctxhelm is `hold` with neutral recall (`0.3212704` vs `0.3212704`). Semantic therefore remains local, source-free, and opt-in; default promotion still requires repeated query-family lift. Phase 252 also attempted a fresh Claude paired R&D suite with Claude Code `2.1.163`; the report is correctly `degraded` because client preflight observed rate limiting, so no retrieval-quality conclusion is drawn from that run.

Previous follow-up: Phase 251 adds a four-task real Codex R&D breadth suite covering selected-memory native reads, semantic contribution diagnostics, GraphRAG edge budget work, and governor/release proof artifacts. The dry probe exposed a GraphRAG implementation-surface miss, so graph-edge R&D tasks now promote `crates/ctxhelm-compiler/src/eval.rs`, `crates/ctxhelm-compiler/src/ranking.rs`, and `crates/ctxhelm-index/src/dependencies.rs`. The Codex harness now distinguishes discovery from consumption and forbids `awk`/redirection in read-only prompts. The final source-free suite artifact reports `passed`, `ctxhelm_improved`, four comparison-eligible tasks, 16 comparable ctxhelm lanes, no evidence misses, no evidence-only targets, no under-read targets, no forbidden commands, no client failures, all ctxhelm lanes at `1.00` average target-read coverage, and local-only privacy.

Previous follow-ups: Phase 250 fixes a real Codex under-read regression for context-governor and release-gate R&D tasks. The before proof matched baseline but missed `docs/context-governor.md`, `scripts/smoke-governor.sh`, and `crates/ctxhelm-core/src/contracts.rs` in some ctxhelm lanes; the after proof reports `ctxhelm_improved`, best lane `ctxhelm-standard`, no ctxhelm evidence misses, no ctxhelm under-read targets, and `1.00` target-read coverage in every ctxhelm lane. Phase 249 adds `ctxhelm governor decide`, a source-free context-governor report for task-conditioned retrieval, budget, memory, validation, semantic, and policy-profile decisions. `scripts/smoke-governor.sh` proves selected/omitted evidence, rollout controls, active learned-profile visibility, policy apply/rollback reflection, release-gate wiring, and source sentinel rejection. Phase 248 adds `ctxhelm inspector serve`, a localhost-only, read-only diagnostic shell for pack inspector, graph neighborhood, setup status, and shell health routes. `scripts/smoke-inspector.sh` now starts the shell, fetches `/`, `/pack-inspector.json`, `/graph.html`, `/graph.json`, `/setup-status.json`, and `/health.json`, and rejects source sentinel leakage. Phase 247 added optional Cursor Agent CLI and OpenCode real-client smoke wrappers with source-free server-side request evidence. `scripts/smoke-opencode-real-client.sh` passes locally with OpenCode `1.14.25` and records explicit-repo `prepare_task` plus `get_pack` tool calls; `scripts/smoke-cursor-real-client.sh` has the same proof contract and uses an isolated temporary Cursor workspace, but current local required proof is auth-blocked because Cursor Agent CLI `3.6.21` reports not logged in. The release gate now records Cursor/OpenCode optional proof status and can require OpenCode proof without requiring Cursor auth. Phase 246 added a release-gated agent-native fallback smoke for thin repo-local guidance and disconnected source-free cards. `scripts/smoke-agent-native-fallback.sh` proves `ctxhelm init --cursor --claude --opencode`, `setup-check`, and `cards fallback --target-agent codex` produce repo-local, bounded guidance without broad static source injection or source sentinel leakage; the release gate and release docs contract now require that smoke. Phase 245 added a strict multi-task Codex real-client suite and fixed the harness/guidance first-read gap it exposed. Earlier suite evidence showed ctxhelm could surface `scripts/e2e-agent-run-codex.sh` for `Improve Codex agent-run harness`, but the `ctxhelm-plan` lane did not read it because the script ranked outside Codex's first native-read batch. Phase 245 adds `--suite` aggregation, promotes bounded agent-native guidance implementation surfaces, treats named shell harnesses as source-like for that promotion, isolates spawned Codex runs from Desktop thread environment leakage, forbids bootstrap/setup/superpowers commands, and caps read-only exploration. The source-free artifact `.ctxhelm/e2e/phase245-agent-run-codex-suite-real-bounded-final.json` passes with Codex CLI `0.137.0`, two comparison-eligible tasks, eight comparable ctxhelm lanes, no forbidden commands, no client failures, no ctxhelm evidence misses, no evidence-only targets, no ctxhelm under-read targets, and outcome claim `ctxhelm_improved`: baseline average target-read coverage is `0.75`, while every ctxhelm lane reaches `1.00`.

**Phases planned:** Phases 61-65 plus Phase 66-293 production-readiness follow-ups, 7 planned phase files plus measured proof follow-up artifacts

**Planned capabilities:**

- Multi-repo fixed-corpus baselines across RefactoringMiner and a second real repository.
- Production local embedding quality proof with bounded local cache and provider metadata.
- Reranker and fusion promotion gates that protect anchors, current diff, lexical evidence, and symbols.
- Gap-family retrieval fixes with before/after proof on real misses.
- v2.5 product proof and release gate that honestly reports beat/match/trail status per corpus and variant.
- Correct validation-test recall measurement through the dedicated `recommended_tests` channel.
- Correct retrieval metrics to use parent-snapshot `retrievalTargetFiles` while preserving `safeChangedFiles`.
- Channel-aware product proof that compares context recall separately from validation-test recall.
- Real-client MCP proof refresh for Codex CLI and Claude Code.
- Archive artifact dampening for historical planning proof files.
- Broader repeated-lift validation across additional local repositories, plus related-test budget and co-change/dependency test-seed fixes.
- Pinned broader fixed-corpus probe config for repeatable optional production-readiness validation.
- Protected-evidence diagnostics that separate retrieval-target misses from non-target exact/symbol pressure.
- Parent-bounded source-free eval history sidecars and co-changed validation-test reservation.
- Parent-bounded validation-only history mode for historical eval snapshots, plus runnable commands for co-changed tests selected from history.
- Broad validation fallback commands and effective validation-command coverage for multi-area smoke/eval tasks.
- Machine-checkable context-vs-all-file divergence accounting that blocks unexplained all-file lexical deficits.
- Broad-scope task accounting plus bounded dependency source floors for multi-area workflow/eval/lint tasks.
- Source-free context-area hints for broad prepare-task plans and packs without displacing target-file or validation budgets.
- Source-free broad context-area recall metrics that show whether broad tasks expose the right implementation areas beyond the top-10 file budget.
- A broader source-free context-area guidance cap for wide tasks when top-10 target-file churn would regress measured recall.
- Progressive pack guidance that tells agents which zero-selected broad areas to inspect next with native reads.
- Source-free MCP context-area resources that make broad area hints inspectable without adding MCP tools or source text.
- Broader governance/proof/eval task classification for source-free planning docs and area signals.
- Progressive broad classification that covers archive/docs retrieval tasks without spending target-file budget on broad source floors.
- Deeper source-free context-area resources with role buckets, path families, and next-read batches for progressive native reads.
- Source-free resource-backed gap summaries that point agents from measured misses to context-area resources and next-read paths.
- Release-gated resource-backed gap summary contracts so future product proofs cannot silently drop progressive read evidence.
- Explicit-repo MCP resource resolution so agents can follow repo-scoped resource URIs even when their MCP server process starts outside the workspace.
- Source-free `nextReadPaths` and `unselectedCount` in broad context areas, including docs areas, so agents have concrete native read targets without top-10 churn.
- Embedded insufficient-evidence benchmark reports for repos whose git history is unavailable, without caching transient zero-commit failures.
- Source-free real-client request evidence with request-log hashes, request line counts, explicit repo tool-call counts, and sanitized observed tool-call summaries.
- Bounded Python package re-export graph coverage without general depth-2 graph expansion.
- Area-aware retrieval-gap taxonomy that distinguishes files covered by surfaced context areas from true no-candidate failures.
- Clean large-repo proof fixtures and warm-cache evidence that do not depend on dirty interactive checkouts.
- Source-free symbol/dependency index caches that reduce repeated cold planner work without weakening privacy or release gates.
- Clean cold four-repo proof fixtures that promote without dirty interactive checkouts or stale parent-snapshot inventories.
- Packaged release-gate controls that run or require the clean cold fixture proof and record source-free proof-summary status.
- Clean release-candidate proof that the extracted archive binary passes the complete release gate with clean fixture proof required.
- Source-free release-candidate status that binds `ready` to the archive-binary proof, required clean fixture proof, and explicit archive-first distribution decisions.
- Public archive release verification that checks GitHub release tag, target commit, draft/prerelease status, and uploaded asset SHA-256 digests.
- Public archive install verification that downloads release assets, verifies checksums, installs to a temporary bin, and runs version/help/doctor/first-pack checks.
- Public archive real-client smoke verification that runs optional Codex CLI and Claude Code wrappers against the extracted release binary and records source-free pass or skip evidence.
- Source-free plan-level role signals that expose `roleCounts` and `selectedRoleCounts` in broad context areas and render those signals in generated packs.
- Source-free context-area MCP resource scope metadata that labels inventory-wide counts and read batches as `safeInventoryArea` and `taskConditioned = false`.
- Source-free context-area MCP coverage profiles that summarize implementation, validation, docs, or mixed area shape and recommend the first native read batch.
- Source-free context-area MCP inspection strategies that give agents progressive read order, small path budget, and stop rules for broad areas.
- Source-free context-area pressure breakdowns that explain whether unread broad-area pressure comes from source/config/schema, validation, or docs paths.
- Source-free context-area pressure summaries that aggregate broad-area pressure across eval/proof reports and identify highest-pressure areas.
- Source-free context-area next-read recovery summaries that quantify whether progressive read paths recover files missed by top-10 context.
- Source-free context-area next-read ordering that prioritizes stronger local signals before weaker progressive reads without changing selected target-file budget.
- Adaptive source-free context-area next-read budgets that expose more progressive reads for high-pressure source-like and validation-heavy areas without selected target-file churn.
- Source-free candidate coverage accounting that separates generated-but-unselected missed@10 files from true no-candidate misses before the next ranking experiment.
- Source-free candidate miss pressure profiles that summarize generated-but-unselected misses by role, signal, and context area.
- Source-free contextual README doc reserve that recovers nested README evidence in broad tasks after config/workflow evidence has first chance.
- Source-free agent-evidence-only gap profiles that show which misses are covered by related tests or other agent evidence but absent from progressive next-read paths.
- Source-free markdown rendering for agent-evidence-only gap counts, roles, and top areas.
- Source-free related-test evidence in generated packs so agents can consume selected validation evidence even when those tests are not repeated in progressive next-read lists.
- Source-free forbidden-tool accounting in paired real-agent reports so read-only outcome proof cannot hide shell/edit/write attempts.
- Source-free target-read coverage and role-count diagnostics in paired real-agent reports so target discovery is not mistaken for actual context consumption.
- Source-free consumption guidance in `prepare_task`, generated agent setup files, and generated packs so agents are told to read returned targets natively before relying on discovered paths or pack snippets.
- Source-free lane-comparability diagnostics in paired real-agent reports so ctxhelm outcome claims require the expected `prepare_task` and `get_pack` calls instead of treating failed/no-call lanes as comparable evidence.
- Source-free client-failure diagnostics in paired real-agent reports so rate limits, API errors, timeouts, and generic client exits are not mistaken for ctxhelm retrieval or pack failures.
- Source-free required-call argument validation in paired real-agent reports so wrong-repo or malformed ctxhelm MCP calls cannot satisfy the comparability contract.
- Source-free ctxhelm evidence attribution in paired real-agent reports so outcome gaps distinguish ctxhelm not surfacing a target from the agent failing to read a surfaced target.
- Hyphenated identifier query aliases that map task terms like `agent-run` to code identifiers like `agent_run` for lexical and symbol retrieval.
- Source-free recommended R&D actions in paired real-agent, historical eval, and product-proof reports so client availability, retrieval misses, consumption misses, required-call issues, no-lift outcomes, candidate gaps, progressive-read gaps, memory reuse gaps, graph-budget opportunities, runtime gaps, and fixture/evidence gaps route to different next steps.
- Release-gated memory lift proof at deterministic plan, historical eval, and benchmark/product-proof layers while preserving source-free storage and report boundaries.
- Parent-snapshot historical evals can consume approved source-repo memory, fixing the real-history storage-identity gap exposed by RefactoringMiner.
- Source-free lexical comparison proof summaries that distinguish target-file-only, agent-evidence, and context-channel lexical claims.
- Narrow-plan validation-test reservation that lets changed tests enter top-10 context ranking without perturbing broad context-area plans.
- Broad operational floors that reserve root governance docs, exact config evidence, and workflow lifecycle scripts before lower-priority expansion.
- Source-free public release freshness metadata that separates verified public archive status from latest current-main production-hardening claims.
- Refreshed public archive verification that proves the latest public release is current, installable from GitHub assets, and covered by refreshed optional real-client smoke evidence.
- Product-aware public release freshness that separates proof/planning-only commits from product-impacting drift.
- Public README positioning that explains the product wedge, current proof snapshot, and current client-evidence boundaries before the command tour.
- Current public `v1.1.5` archive release with verified GitHub asset digests, install proof, product freshness, refreshed Homebrew tap proof, and optional real-client evidence.
- Short-lived interim product branding with `ctxhelm` retained as the stable CLI, package, Homebrew formula, MCP namespace, and local state compatibility surface.
- Public `v1.1.6` archive release for the superseded brand with verified GitHub asset digests, install proof, product freshness, refreshed Homebrew tap proof, and optional real-client evidence.
- Active ctxhelm product branding with `ctxhelm` retained as the stable CLI, package, Homebrew formula, MCP namespace, and local state compatibility surface.
- Public `v1.1.7` archive release for the ctxhelm brand with verified GitHub asset digests, install proof, product freshness, refreshed Homebrew tap proof, and optional real-client evidence.
- Public `v1.1.11` archive release and Homebrew tap with verified GitHub asset digests, install proof, product freshness, refreshed Homebrew tap proof, and public real-client evidence.
- Public `v1.1.12` release-currentness candidate that can publish verified Linux x64, macOS Intel, and macOS Apple Silicon release assets from the version-tag workflow.
- Multi-platform release-artifact workflow that builds and verifies Linux x64, macOS Intel, and macOS Apple Silicon archives; manual dispatch uploads workflow artifacts, while version-tag pushes create/update the matching GitHub release and upload verified target assets without creating tags, package-manager commits, crates.io packages, signed installers, or self-update metadata.
- Public Apple Silicon Homebrew tap with strict formula audit, install, test, and source-free proof against the current archive.
- Homebrew formula renderability for the published tap and crates package boundary checks for deferred crates.io publication, without mutating unsupported channels.
- Stable `ctxhelm-index` release-validation tests with crate-wide synchronization for process-global `CTXHELM_HOME`.
- Public GitHub Actions CI that enforces formatting, clippy, locked workspace tests, CLI help, release docs, and release-gate smoke without publishing.

**Active artifacts:**

- Roadmap: `.planning/ROADMAP.md`
- Requirements: `.planning/REQUIREMENTS.md`
- Phases: `.planning/phases/61-multi-repo-quality-baselines/` through `.planning/phases/65-v25-product-proof-release-gate/`

**Current evidence:**

- Phase 61 E2E: `.planning/e2e/2026-05-22-v25-multirepo-baseline.md`
- Phase 62 E2E: `.planning/e2e/2026-05-30-phase62-production-local-embedding-quality.md`
- Phase 65 E2E: `.planning/e2e/2026-05-30-phase65-product-proof-release-gate.md`
- Phase 66 E2E: `.planning/e2e/2026-05-30-phase66-test-recall-eval-channel.md`
- Phase 67 E2E: `.planning/e2e/2026-05-30-phase67-retrievable-target-eval-denominator.md`
- Phase 69 E2E: `.planning/e2e/2026-05-30-phase69-channel-aware-product-proof.md`
- Phase 70 E2E: `.planning/e2e/2026-05-30-phase70-real-client-mcp-proof.md`
- Phase 71 E2E: `.planning/e2e/2026-05-30-phase71-archive-artifact-dampening.md`
- Phase 72 E2E: `.planning/e2e/2026-05-30-phase72-broader-repeated-lift-validation.md`
- Phase 73 E2E: `.planning/e2e/2026-05-30-phase73-broader-fixed-corpus-fixture.md`
- Phase 73 config: `.planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json`
- Phase 74 E2E: `.planning/e2e/2026-05-30-phase74-protected-evidence-diagnostics.md`
- Phase 74 proof: `.ctxhelm/e2e/phase74-protected-evidence-diagnostics-proof.json`
- Phase 74 broader proof: `.ctxhelm/e2e/phase74-broader-protected-evidence-diagnostics-proof.json`
- Phase 75 E2E: `.planning/e2e/2026-05-30-phase75-parent-history-test-reserve.md`
- Phase 75 proof: `.ctxhelm/e2e/phase75-parent-history-test-reserve-proof.json`
- Phase 75 broader proof: `.ctxhelm/e2e/phase75-broader-parent-history-test-reserve-proof.json`
- Phase 76 E2E: `.planning/e2e/2026-05-30-phase76-parent-bounded-validation-history.md`
- Phase 76 proof: `.ctxhelm/e2e/phase76-parent-bounded-validation-history-proof.json`
- Phase 76 broader proof: `.ctxhelm/e2e/phase76-broader-parent-bounded-validation-history-proof.json`
- Phase 77 E2E: `.planning/e2e/2026-05-30-phase77-validation-command-coverage.md`
- Phase 77 proof: `.ctxhelm/e2e/phase77-validation-command-coverage-proof.json`
- Phase 77 broader proof: `.ctxhelm/e2e/phase77-broader-validation-command-coverage-proof.json`
- Phase 78 E2E: `.planning/e2e/2026-05-30-phase78-ceiling-aware-broader-gate.md`
- Phase 78 broader proof: `.ctxhelm/e2e/phase78-ceiling-aware-broader-proof.json`
- Phase 79 E2E: `.planning/e2e/2026-05-30-phase79-protected-target-floors.md`
- Phase 79 proof: `.ctxhelm/e2e/phase79-protected-target-floors-proof.json`
- Phase 79 broader proof: `.ctxhelm/e2e/phase79-broader-protected-target-floors-proof.json`
- Phase 80 E2E: `.planning/e2e/2026-05-30-phase80-unique-symbol-floor.md`
- Phase 80 proof: `.ctxhelm/e2e/phase80-unique-symbol-floor-proof.json`
- Phase 80 broader proof: `.ctxhelm/e2e/phase80-broader-unique-symbol-floor-proof.json`
- Phase 81 E2E: `.planning/e2e/2026-05-30-phase81-warm-cache-latency.md`
- Phase 81 config: `.planning/e2e/2026-05-30-phase81-warm-cache-proof-config.json`
- Phase 81 cold proof: `.ctxhelm/e2e/phase81-warm-cache-cold-proof.json`
- Phase 81 warm proof: `.ctxhelm/e2e/phase81-warm-cache-warm-proof.json`
- Phase 82 E2E: `.planning/e2e/2026-05-30-phase82-warm-cache-gate.md`
- Phase 82 cold proof: `.ctxhelm/e2e/phase82-warm-cache-gate-cold-proof.json`
- Phase 82 warm proof: `.ctxhelm/e2e/phase82-warm-cache-gate-warm-proof.json`
- Phase 83 E2E: `.planning/e2e/2026-05-30-phase83-context-divergence-accounting.md`
- Phase 83 proof: `.ctxhelm/e2e/phase83-context-divergence-proof.json`
- Phase 84 E2E: `.planning/e2e/2026-05-31-phase84-broad-scope-dependency-floors.md`
- Phase 84 proof: `.ctxhelm/e2e/phase84-broad-scope-dependency-proof.json`
- Phase 85 E2E: `.planning/e2e/2026-05-31-phase85-broad-context-areas.md`
- Phase 85 proof: `.ctxhelm/e2e/phase85-context-areas-warm-proof.json`
- Phase 86 E2E: `.planning/e2e/2026-05-31-phase86-python-package-reexports.md`
- Phase 87 E2E: `.planning/e2e/2026-05-31-phase87-validation-gap-accounting.md`
- Phase 87 proof: `.ctxhelm/e2e/phase87-validation-gap-accounting-proof.json`
- Phase 88 E2E: `.planning/e2e/2026-05-31-phase88-broad-source-area-candidates.md`
- Phase 88 proof: `.ctxhelm/e2e/phase88-broad-source-area-candidates-proof.json`
- Phase 89 E2E: `.planning/e2e/2026-05-31-phase89-fast-inventory-freshness.md`
- Phase 89 debug proof: `.ctxhelm/e2e/phase89-fast-inventory-freshness-proof.json`
- Phase 89 release proof: `.ctxhelm/e2e/phase89-fast-inventory-freshness-release-proof.json`
- Phase 90 E2E: `.planning/e2e/2026-05-31-phase90-packaged-release-gate.md`
- Phase 91 E2E: `.planning/e2e/2026-05-31-phase91-broad-context-area-eval.md`
- Phase 91 release proof: `.ctxhelm/e2e/phase91-broad-context-area-release-proof.json`
- Phase 92 E2E: `.planning/e2e/2026-05-31-phase92-area-aware-gap-taxonomy.md`
- Phase 92 clean force proof: `.ctxhelm/e2e/phase92-area-aware-gap-taxonomy-clean-force-proof.json`
- Phase 92 warm proof: `.ctxhelm/e2e/phase92-area-aware-gap-taxonomy-warm-proof.json`
- Phase 93 E2E: `.planning/e2e/2026-05-31-phase93-source-free-index-cache.md`
- Phase 93 cold proof: `.ctxhelm/e2e/phase93-index-cache-cold-proof.json`
- Phase 94 E2E: `.planning/e2e/2026-05-31-phase94-broad-context-area-cap.md`
- Phase 94 proof: `.ctxhelm/e2e/phase94-context-area-cap-proof.json`
- Phase 95 E2E: `.planning/e2e/2026-05-31-phase95-progressive-area-pack-guidance.md`
- Phase 95 proof: `.ctxhelm/e2e/phase95-progressive-area-pack-proof.json`
- Phase 96 E2E: `.planning/e2e/2026-05-31-phase96-context-area-resources.md`
- Phase 96 proof: `.ctxhelm/e2e/phase96-context-area-resources-proof.json`
- Phase 97 E2E: `.planning/e2e/2026-05-31-phase97-broad-governance-classification.md`
- Phase 97 proof: `.ctxhelm/e2e/phase97-broad-governance-classification-proof.json`
- Phase 98 E2E: `.planning/e2e/2026-05-31-phase98-progressive-broad-classification.md`
- Phase 98 proof: `.ctxhelm/e2e/phase98-broader-broad-task-classification-proof.json`
- Phase 99 E2E: `.planning/e2e/2026-05-31-phase99-context-area-read-batches.md`
- Phase 99 proof: `.ctxhelm/e2e/phase99-context-area-read-batches-proof.json`
- Phase 100 E2E: `.planning/e2e/2026-05-31-phase100-resource-backed-gap-summaries.md`
- Phase 100 proof: `.ctxhelm/e2e/phase100-resource-backed-gap-summaries-proof.json`
- Phase 101 E2E: `.planning/e2e/2026-05-31-phase101-release-gated-gap-summary-contract.md`
- Phase 102 E2E: `.planning/e2e/2026-05-31-phase102-explicit-repo-mcp-resource-consumption.md`
- Phase 103 E2E: `.planning/e2e/2026-05-31-phase103-broad-fixed-corpus-floors.md`
- Phase 104 E2E: `.planning/e2e/2026-05-31-phase104-context-area-next-read-paths.md`
- Phase 105 E2E: `.planning/e2e/2026-05-31-phase105-history-unavailable-report.md`
- Phase 106 E2E: `.planning/e2e/2026-05-31-phase106-real-client-request-evidence.md`
- Phase 107 E2E: `.planning/e2e/2026-05-31-phase107-hydrated-four-repo-proof.md`
- Phase 107 cold proof: `.ctxhelm/e2e/phase107-hydrated-four-repo-cold-proof.json`
- Phase 107 warm proof: `.ctxhelm/e2e/phase107-hydrated-four-repo-warm-proof.json`
- Phase 108 E2E: `.planning/e2e/2026-05-31-phase108-cold-git-bounds.md`
- Phase 108 cold proof: `.ctxhelm/e2e/phase108-cold-git-bounded-proof.json`
- Phase 109 E2E: `.planning/e2e/2026-05-31-phase109-environment-health-verdict.md`
- Phase 109 cold proof: `.ctxhelm/e2e/phase109-environment-health-proof.json`
- Phase 110 E2E: `.planning/e2e/2026-05-31-phase110-clean-cold-fixture-proof.md`
- Phase 110 config: `.planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json`
- Phase 110 clean fixture proof: `.ctxhelm/e2e/phase110-clean-fixture-proof.json`
- Phase 111 E2E: `.planning/e2e/2026-06-01-phase111-clean-fixture-release-gate.md`
- Phase 112 E2E: `.planning/e2e/2026-06-01-phase112-clean-release-gate-required.md`
- Phase 112 release summary: `.ctxhelm/e2e/phase112-clean-release-gate-summary.json`
- Phase 113 E2E: `.planning/e2e/2026-06-01-phase113-release-candidate-status.md`
- Phase 113 candidate status: `.ctxhelm/e2e/phase113-release-candidate-status.json`
- Phase 114 E2E: `.planning/e2e/2026-06-01-phase114-public-archive-release.md`
- Phase 114 release proof summary: `.ctxhelm/e2e/phase114-release-proof-summary.json`
- Phase 114 candidate status: `.ctxhelm/e2e/phase114-release-candidate-status.json`
- Phase 114 GitHub release metadata: `.ctxhelm/e2e/phase114-github-release.json`
- Phase 115 E2E: `.planning/e2e/2026-06-01-phase115-public-archive-install.md`
- Phase 115 public install proof: `.ctxhelm/e2e/phase115-public-archive-install.json`
- Phase 116 E2E: `.planning/e2e/2026-06-01-phase116-public-real-client-smoke.md`
- Phase 116 public real-client proof: `.ctxhelm/e2e/phase116-public-real-client-smoke.json`
- Phase 117 E2E: `.planning/e2e/2026-06-01-phase117-context-area-role-signals.md`
- Phase 117 context-area role signal proof: `.ctxhelm/e2e/phase117-context-area-role-signals.json`
- Phase 118 E2E: `.planning/e2e/2026-06-01-phase118-context-area-resource-scope.md`
- Phase 118 context-area resource scope proof: `.ctxhelm/e2e/phase118-context-area-resource-scope.json`
- Phase 119 E2E: `.planning/e2e/2026-06-01-phase119-index-env-lock-flake.md`
- Phase 119 index env-lock proof: `.ctxhelm/e2e/phase119-index-env-lock-proof.json`
- Phase 120 E2E: `.planning/e2e/2026-06-01-phase120-public-ci-release-gate.md`
- Phase 120 CI release-gate proof: `.ctxhelm/e2e/phase120-ci-release-gate-proof.json`
- Phase 121 E2E: `.planning/e2e/2026-06-01-phase121-ci-node24-runtime.md`
- Phase 121 CI Node 24 runtime proof: `.ctxhelm/e2e/phase121-ci-node24-runtime-proof.json`
- Phase 122 E2E: `.planning/e2e/2026-06-01-phase122-public-real-client-compat.md`
- Phase 122 public real-client compatibility proof: `.ctxhelm/e2e/phase122-public-real-client-compat-proof.json`
- Phase 123 E2E: `.planning/e2e/2026-06-01-phase123-context-area-coverage-profile.md`
- Phase 123 context-area coverage profile proof: `.ctxhelm/e2e/phase123-context-area-coverage-profile.json`
- Phase 124 E2E: `.planning/e2e/2026-06-01-phase124-context-area-inspection-strategy.md`
- Phase 124 context-area inspection strategy proof: `.ctxhelm/e2e/phase124-context-area-inspection-strategy.json`
- Phase 125 E2E: `.planning/e2e/2026-06-01-phase125-lexical-comparison-proof.md`
- Phase 125 lexical comparison proof: `.ctxhelm/e2e/phase125-lexical-comparison-proof.json`
- Phase 126 E2E: `.planning/e2e/2026-06-01-phase126-agent-evidence-lexical-comparison.md`
- Phase 126 agent-evidence lexical comparison proof: `.ctxhelm/e2e/phase126-agent-evidence-lexical-comparison.json`
- Phase 127 E2E: `.planning/e2e/2026-06-01-phase127-narrow-validation-test-reserve.md`
- Phase 127 narrow validation-test reserve proof: `.ctxhelm/e2e/phase127-narrow-validation-test-reserve.json`
- Phase 128 E2E: `.planning/e2e/2026-06-01-phase128-broad-operational-floor.md`
- Phase 128 broad operational floor proof: `.ctxhelm/e2e/phase128-broad-operational-floor.json`
- Phase 129 E2E: `.planning/e2e/2026-06-01-phase129-public-release-freshness.md`
- Phase 129 public release freshness proof: `.ctxhelm/e2e/phase129-public-release-freshness.json`
- Phase 130 E2E: `.planning/e2e/2026-06-01-phase130-public-v111-release.md`
- Phase 130 GitHub release metadata: `.ctxhelm/e2e/phase130-github-release.json`
- Phase 130 public release freshness proof: `.ctxhelm/e2e/phase130-public-release-freshness.json`
- Phase 130 public archive install proof: `.ctxhelm/e2e/phase130-public-archive-install.json`
- Phase 130 public real-client smoke proof: `.ctxhelm/e2e/phase130-public-real-client-smoke.json`
- Phase 131 E2E: `.planning/e2e/2026-06-01-phase131-product-aware-freshness-release.md`
- Phase 131 GitHub release metadata: `.ctxhelm/e2e/phase131-github-release.json`
- Phase 131 GitHub release verification: `.ctxhelm/e2e/phase131-github-release-verify.json`
- Phase 131 public release freshness proof: `.ctxhelm/e2e/phase131-public-release-freshness.json`
- Phase 131 public archive install proof: `.ctxhelm/e2e/phase131-public-archive-install.json`
- Phase 131 public real-client smoke proof: `.ctxhelm/e2e/phase131-public-real-client-smoke.json`
- Phase 132 E2E: `.planning/e2e/2026-06-01-phase132-claude-workflow-eval.md`
- Phase 132 Claude workflow proof: `.ctxhelm/e2e/phase132-claude-workflow-eval.json`
- Phase 133 E2E: `.planning/e2e/2026-06-01-phase133-product-readme-positioning.md`
- Phase 133 product README proof: `.ctxhelm/e2e/phase133-product-readme-positioning.json`
- Phase 134 E2E: `.planning/e2e/2026-06-01-phase134-public-v113-release.md`
- Phase 134 GitHub release metadata: `.ctxhelm/e2e/phase134-github-release.json`
- Phase 134 GitHub release verification: `.ctxhelm/e2e/phase134-github-release-verify.json`
- Phase 134 public release freshness proof: `.ctxhelm/e2e/phase134-public-release-freshness.json`
- Phase 134 public archive install proof: `.ctxhelm/e2e/phase134-public-archive-install.json`
- Phase 134 public real-client smoke proof: `.ctxhelm/e2e/phase134-public-real-client-smoke.json`
- Phase 135 E2E: `.planning/e2e/2026-06-01-phase135-distribution-readiness.md`
- Phase 135 distribution readiness proof: `.ctxhelm/e2e/phase135-distribution-readiness.json`
- Phase 136 E2E: `.planning/e2e/2026-06-01-phase136-public-v114-release.md`
- Phase 136 GitHub release metadata: `.ctxhelm/e2e/phase136-github-release.json`
- Phase 136 GitHub release verification: `.ctxhelm/e2e/phase136-github-release-verify.json`
- Phase 136 public release freshness proof: `.ctxhelm/e2e/phase136-public-release-freshness.json`
- Phase 136 public archive install proof: `.ctxhelm/e2e/phase136-public-archive-install.json`
- Phase 136 public real-client smoke proof: `.ctxhelm/e2e/phase136-public-real-client-smoke.json`
- Phase 136 distribution readiness proof: `.ctxhelm/e2e/phase136-distribution-readiness.json`
- Phase 137 E2E: `.planning/e2e/2026-06-01-phase137-homebrew-tap.md`
- Phase 137 public Homebrew tap proof: `.ctxhelm/e2e/phase137-homebrew-tap-proof.json`
- Phase 138 E2E: `.planning/e2e/2026-06-01-phase138-public-v115-release.md`
- Phase 138 GitHub release metadata: `.ctxhelm/e2e/phase138-github-release.json`
- Phase 138 GitHub release verification: `.ctxhelm/e2e/phase138-github-release-verify.json`
- Phase 138 public release freshness proof: `.ctxhelm/e2e/phase138-public-release-freshness.json`
- Phase 138 public archive install proof: `.ctxhelm/e2e/phase138-public-archive-install.json`
- Phase 138 Homebrew tap proof: `.ctxhelm/e2e/phase138-homebrew-tap-proof.json`
- Phase 138 public real-client smoke proof: `.ctxhelm/e2e/phase138-public-real-client-smoke.json`
- Phase 139 E2E: `.planning/e2e/2026-06-01-phase139-contextmason-brand.md`
- Phase 140 E2E: `.planning/e2e/2026-06-01-phase140-public-v116-release.md`
- Phase 140 GitHub release metadata: `.ctxhelm/e2e/phase140-github-release.json`
- Phase 140 GitHub release verification: `.ctxhelm/e2e/phase140-github-release-verify.json`
- Phase 140 public release freshness proof: `.ctxhelm/e2e/phase140-public-release-freshness.json`
- Phase 140 public archive install proof: `.ctxhelm/e2e/phase140-public-archive-install.json`
- Phase 140 Homebrew tap proof: `.ctxhelm/e2e/phase140-homebrew-tap-proof.json`
- Phase 140 public real-client smoke proof: `.ctxhelm/e2e/phase140-public-real-client-smoke.json`
- Phase 141 E2E: `.planning/e2e/2026-06-01-phase141-repowinnow-brand.md`
- Phase 142 E2E: `.planning/e2e/2026-06-01-phase142-repowinnow-public-release.md`
- Phase 142 public archive install proof: `.ctxhelm/e2e/phase142-public-archive-install.json`
- Phase 142 public real-client smoke proof: `.ctxhelm/e2e/phase142-public-real-client-smoke.json`
- Phase 142 Homebrew tap proof: `.ctxhelm/e2e/phase142-homebrew-tap.json`
- Phase 143 E2E: `.planning/e2e/2026-06-01-phase143-agent-run-outcome-harness.md`
- Phase 143 Claude paired-run proof: `.ctxhelm/e2e/phase143-agent-run-claude.json`
- Phase 152 E2E: `.planning/e2e/2026-06-02-phase152-native-agent-outcome-suite.md`
- Phase 153 E2E: `.planning/e2e/2026-06-02-phase153-bm25-symbol-lexical-index.md`
- Phase 154 E2E: `.planning/e2e/2026-06-02-phase154-bm25-legacy-comparison.md`
- Phase 155 E2E: `.planning/e2e/2026-06-02-phase155-bm25-corpus-comparison.md`
- Phase 156 E2E: `.planning/e2e/2026-06-02-phase156-lexical-backend-proof-integration.md`
- Phase 157 E2E: `.planning/e2e/2026-06-02-phase157-benchmark-corpus-health.md`
- Phase 158 E2E: `.planning/e2e/2026-06-02-phase158-bm25-exact-saturated-fast-path.md`
- Phase 159 E2E: `.planning/e2e/2026-06-02-phase159-lexical-runtime-accounting.md`
- Phase 160 E2E: `.planning/e2e/2026-06-02-phase160-bounded-semantic-status-search.md`
- Phase 161 E2E: `.planning/e2e/2026-06-02-phase161-semantic-gate-contribution-diagnostics.md`
- Phase 162 E2E: `.planning/e2e/2026-06-02-phase162-feature-enabled-local-fastembed-gate-proof.md`

**Current follow-up gaps:**

- Broader four-repo proof now promotes under the ceiling-aware gate: VeriSchema beats through effective validation recall (`1.0`) even though raw Test Recall@10 remains `0.7090`, and RefactoringMiner is accepted as a safe lexical-ceiling match.
- Protected retrieval-target miss-rate is now `0.0` across the clean four-repo fixture after Phase 128. Overall protected evidence misses remain diagnostic because non-target protected candidates can still fall below the standard budget.
- Parser/precision dependency misses remain a repeated source gap family, but Phase 84 reduces the measured VeriSchema source miss pressure by reserving dependency neighbors only for broad-scope tasks.
- Phase 85 improves broad-task agent guidance without claiming a recall lift: broad fixed-corpus quality metrics stayed flat, cold proofs still hit the existing RefactoringMiner runtime gate, and the warm-cache proof promotes.
- Phase 86 adds Python package re-export graph candidates, but broad fixed-corpus Recall@10 remains flat; the remaining VeriSchema gap is selection/budget pressure as much as parser coverage.
- Phase 87 fixes validation accounting: Java class-selector commands now count as validation coverage, and validation-covered tests no longer appear as unresolved test-mapping gap summaries. A broad source-area diversity selector was tested and rejected because it worsened VeriSchema file recall and protected target miss-rate.
- Phase 88 adds broad source-area candidates after graph/test seed selection. VeriSchema File Recall@10 improves from `0.17936651` to `0.18449473`, Source Recall@10 improves from `0.30409357` to `0.31067252`, and validation/protected-target metrics stay stable.
- Phase 89 replaces full source re-hashing on inventory cache-hit freshness checks with metadata manifest comparison. The pinned broader release proof promotes with Phase 88 quality metrics preserved.
- Phase 90 runs the complete release gate in a clean worktree with the broader benchmark proof enabled. The packaged archive, audit, extraction, extracted binary smokes, MCP protocol checks, Cursor/OpenCode setup checks, and broad product proof all pass; optional Codex/Claude real-client tool-call evidence remains skipped unless explicitly required.
- Phase 91 records `changedContextAreas`, `contextAreaHits`, and `broadContextAreaRecall` in historical eval. The hard VeriSchema lint/workflow commit surfaces 11 of 14 changed areas, and the broader release proof promotes with VeriSchema broad context-area recall `0.64708996`.
- Phase 92 classifies files inside surfaced context areas as `area_context_only` / `contextPlanning`, bumps the historical eval cache schema, and proves the clean four-repo fixture in warm-cache mode. Clean RefactoringMiner still exceeds the hard cold runtime ceiling without cached reports, making a real lexical/symbol index the next performance target.
- Phase 93 caches source-free symbol extraction and dependency edge reports. The force-refresh broad proof promotes, and the clean RefactoringMiner runtime drops under the hard cold ceiling at `4517ms`.
- Phase 94 rejects a top-10 area-diversity selector after proof regression and instead expands additive source-free context-area guidance. VeriSchema broad context-area recall improves from `0.64708996` to `0.71851856` while file/source/test/validation metrics stay stable.
- Phase 95 makes the broad `contextAreas` pack section operational: generated packs explain progressive native reads and list zero-selected areas with representative paths.
- Phase 96 makes broad `contextAreas` consumable through `ctxhelm://repo/context-areas` and `ctxhelm://repo/context-area/{encoded-area}` MCP resources while keeping the tool surface unchanged.
- Phase 97 expands broad governance/proof/eval classification. ctxhelm File Recall@10 improves from `0.44603175` to `0.47460318`, Source Recall@10 improves from `0.6333333` to `0.7166667`, and broad context-area recall improves from `0.0` to `1.0`.
- Phase 98 splits source-free broad context-area classification from target-file source-floor spending, covering archive/docs tasks without top-10 churn.
- Phase 99 deepens MCP context-area resources with source-free role buckets, path families, and next-read batches while preserving Phase 98 proof metrics.
- Phase 100 makes retrieval-gap summaries resource-backed with source-free context-area URIs and bounded next-read paths while preserving Phase 99 proof metrics.
- Phase 101 makes current reachable retrieval-gap summaries fail the product-proof checker when they lack context-area resource URIs or next-read paths.
- Phase 102 makes repo-scoped MCP resources follow the last explicit tool repo, so resource-backed gap URIs work in wrong-cwd real-client launch shapes.
- Phase 103 makes broad fixed-corpus metric floors part of the product-proof checker, blocking pinned-corpus regressions that still promote overall.
- Phase 104 makes ranked-below-budget source/docs pressure actionable by surfacing docs areas, `nextReadPaths`, and `unselectedCount` in `ContextArea` and generated packs; the three-repo proof promotes, while the full four-repo proof remains blocked by the local RefactoringMiner checkout's `git rev-list` timeout.
- Phase 105 keeps history-unavailable benchmark repos machine-checkable by embedding a zero-commit report with an `insufficient_evidence` proof verdict instead of producing `report: null`; transient degraded reports are not cached.
- Phase 106 makes Codex and Claude Code real-client proof artifacts auditable without persisting raw MCP traffic by recording source-free request hashes, request line counts, explicit repo tool-call counts, sanitized observed tool calls, and sanitized request-summary sidecars.
- Phase 107 makes the full four-repo proof hydrate RefactoringMiner, ctxhelm, ReAgent, and VeriSchema with embedded reports instead of hanging or returning missing evidence.
- Phase 108 bounds cold Git failure modes by moving parent caches outside source repos, removing archive extraction, using no-rename subject sampling, failing closed on stalled object-content batches, and refusing to reuse incomplete parent snapshot caches.
- Phase 109 makes cold proof blockers classify degraded Git history/object-store environments before treating insufficient evidence as retrieval-quality failure.
- Phase 110 promotes the clean cold four-repo fixture proof by evicting stale snapshot repo caches, bumping parent-snapshot schema, using source-free parent-history ranking, and skipping false whole-repo symbol extraction for commit-subject acronyms.
- Phase 111 wires the clean cold fixture proof into the packaged release gate with `CTXHELM_CLEAN_FIXTURE_CONFIG`, `CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF`, `CTXHELM_SKIP_CLEAN_FIXTURE_PROOF`, proof-summary fields, release docs, and release packaging contract coverage.
- Phase 112 runs the full packaged release gate from clean checkout `20c367dc7eafc1231559c9110901961c55645089` with `CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF=1`. The extracted archive binary passes all required checks and the proof summary records `cleanColdFixtureProductProof = passed`.
- Phase 113 creates source-free release-candidate status metadata where `ready` requires a passed Phase 112 proof summary, archive binary source, required clean fixture proof, and archive/binary checksums. It marks the local archive channel ready and defers Homebrew, crates.io, signed installers, and self-update.
- Phase 114 publishes the public `v1.1.0` GitHub archive release at `https://github.com/thromel/ctxhelm/releases/tag/v1.1.0`, targeting `68383cbfc2fff00c4f53fbd2b7bf90527ac4bd7e`, and verifies five uploaded asset digests against the local release artifacts.
- Phase 115 verifies the public archive install path from GitHub release assets, including checksum verification, clean archive verification, temporary-bin install, `--version`, `--help`, `doctor`, and first-pack smoke.
- Phase 116 refreshes optional real-client evidence against the public archive binary. Claude Code `2.1.158` passes with explicit-repo `prepare_task` and `get_pack`; Codex CLI `0.44.0` is recorded as an optional skip after failing to produce machine-checkable tool-call evidence.
- Phase 117 adds source-free role signals to broad context areas. Plan-level `ContextArea` entries now expose `roleCounts` and `selectedRoleCounts`, and generated packs render those counts so agents can distinguish source-heavy, validation-heavy, and docs-only areas before native reads.
- Phase 118 adds explicit source-free `resourceScope` metadata to context-area MCP resources, proving from a non-repo MCP server cwd that `ctxhelm://repo/context-areas` and `ctxhelm://repo/context-area/{encoded-area}` label role counts and read batches as safe-inventory scope rather than task-conditioned plan counts.
- Phase 119 removes an observed full-suite flake in `ctxhelm-index` by replacing per-module environment mutexes with one crate-wide `test_env_lock()` for `CTXHELM_HOME`; three consecutive parallel `ctxhelm-index --lib` runs passed after the fix.
- Phase 120 adds `.github/workflows/ci.yml`, release-doc and Rust contract coverage for that workflow, and local proof that the CI release-gate smoke passes with optional fixtures/real clients explicitly skipped.
- Phase 121 upgrades public CI to `actions/checkout@v5` and `actions/cache@v5`, keeps the Node 24 runtime opt-in visible, and verifies public CI run `26728393271` passed without Node 20 warning text.
- Phase 122 makes public archive real-client smoke compatible with the already-published `v1.1.0` protocol surface while keeping current resource-scope MCP assertions strict by default; Claude Code still passes and Codex CLI `0.44.0` remains an optional source-free skip instead of missing evidence.
- Phase 123 adds source-free `coverageProfile` metadata to context-area MCP resources, so agents can distinguish implementation, validation, docs, and mixed areas plus choose a recommended first read batch without changing ranking.
- Phase 124 adds source-free `inspectionStrategy` metadata to context-area MCP resources, so agents get a progressive read order, small path budget, and stop rule without changing ranking.
- Phase 125 makes lexical comparison claims machine-checkable: current proof trails lexical on all-file macro recall (`allFileClaim = trails_any_corpus`) while improving the context channel (`contextClaim = mixed`, average context delta `+0.23022664`).
- Phase 126 makes the production adoption claim more direct: `agentEvidenceClaim = mixed` with beat `3`, match `1`, trail `0`, average agent-evidence Recall@10 `0.64502084`, and average agent-evidence delta `+0.18792826` against lexical.
- Phase 127 removes the remaining target-file lexical trailing corpus with a narrow-plan validation-test reserve: `allFileBeatCount = 3`, `allFileMatchCount = 1`, `allFileTrailCount = 0`, average File Recall@10 `0.5927659` vs lexical `0.45709258`, and average file delta `+0.13567334`.
- Phase 128 clears the remaining protected target misses in the clean four-repo proof with broad governance/config/workflow reserves: protected target miss-rate is `0.0` on RefactoringMiner, ctxhelm, ReAgent, and VeriSchema; average File Recall@10 is `0.5986343` vs lexical `0.45709258`; average context delta is `+0.23717105`.
- Phase 178 separates raw all-file trails from explained and unexplained proof trails: current clean proof still promotes with `allFileClaim = mixed`, `allFileBeatCount = 3`, raw `allFileMatchCount = 0`, raw `allFileTrailCount = 1`, `allFileExplainedTrailCount = 1`, `allFileUnexplainedTrailCount = 0`, average File Recall@10 `0.61190045`, and average file delta `+0.15480787`.
- Phase 179 adds source-free graph edge profiles to historical eval reports. The warm four-repo proof still promotes with Phase 178 metrics, and the new profile shows VeriSchema `imports` edges produced `101` candidates, `19` selected@10, `28` retrieval targets, `6` target hits@10, and `22` target misses@10.
- Phase 180 adds conservative source-free graph edge ablations to historical eval reports. The four-repo proof promotes with average File Recall@10 `0.61190045` vs lexical `0.45709258`; disabling exclusive VeriSchema `imports` removes `2` selected paths and `1` target hit for Recall@K delta `-0.00512819`, while `python_reexport` has no exclusive top-10 lift on the fixture.
- Phase 181 converts those graph diagnostics into bounded source-dependency budget allocation: dependency floors now prefer `precision:*`, then `imports`, then other dependency labels, then `python_reexport`. Focused tests pass, a warm RefactoringMiner/ctxhelm/ReAgent proof promotes with average File Recall@10 `0.75555557` vs lexical `0.5687831`, current reachable VeriSchema promotes with lift `+0.14352942`, and the old pinned VeriSchema SHA is documented as unavailable from the remote.
- Phase 182 makes clean proof fixture freshness machine-checkable: fixture prep now writes source-free per-fixture readiness reports, blocks before checkout when a pinned revision is unavailable, and release-gate clean fixture proof now verifies each configured fixture has the requested commit checked out before treating it as valid proof evidence.
- Phase 183 refreshes the default clean fixture proof to the reachable VeriSchema revision and adds source-free per-repo proof runtime ceilings. The default remains `5000ms` per commit; only RefactoringMiner carries `proofRuntimeCeilingMillis: 15000`. Cold and warm refreshed four-repo proof both promote.
- Phase 184 adds source-free `signalCounts` to broad `contextAreas` and renders `Signals:` in context-area pack guidance. The measured dependency-reserve widening experiment produced no recall lift and was rejected; the accepted change keeps top-10 ranking stable while making progressive native reads explainable by signal family.
- Phase 185 carries those profiles into retrieval-gap summaries: profiled gaps now include source-free area signal counts, role counts, selected-role counts, and unselected counts. The fresh four-repo proof promotes with ctxhelm `7/7` gap summaries profiled and VeriSchema `9/10` profiled.
- Phase 186 deduplicates grouped gap area-profile counts: summaries still preserve file-level `missedCount`, examples, and next reads, but context-area signal/role/unselected counts are merged once per matching task-conditioned area profile. The fresh release-binary proof promotes with RefactoringMiner as a lexical-ceiling `match`.
- Phase 187 adds a corroborated source-history reserve: source co-change candidates need dependency, lexical, lexical-expansion, or symbol support before claiming bounded target-file budget. Fresh release-binary proof promotes; ctxhelm Source Recall@10 improves from `0.42857143` to `0.5`, VeriSchema Source Recall@10 improves from `0.32258064` to `0.38709676`, and the ctxhelm all-file Recall@10 docs tradeoff is explicitly recorded.
- Phase 188 adds selected-signal profiles to historical commit eval reports: all 16 commits in the fresh four-repo proof expose source-free selected top-10 counts by signal and role, plus selected retrieval-target counts, while retrieval metrics stay unchanged from Phase 187.
- Phase 189 balances broad governance docs against source-history reserve: broad root governance docs now run before broad source-history, source-history candidates prefer module entrypoints, the fresh proof promotes, ctxhelm File Recall@10 improves from `0.5587302` to `0.67777777`, and ctxhelm Source Recall@10 stays `0.55`.
- Phase 190 adds context-area coverage and inspection-pressure signals: plan-level context areas now expose `coveragePercent` and `inspectionPressure`, packs order zero-selected area guidance by pressure, and the fresh proof promotes with Phase 189 metrics unchanged across RefactoringMiner, ctxhelm, ReAgent, and VeriSchema.
- Phase 129 adds source-free public release freshness proof: current main descends from the published `v1.1.0` release target by 19 commits, so the public archive is verified but not current with latest post-release hardening.
- Phase 130 refreshes the public archive to `v1.1.1`: the GitHub release targets `6c93100fa0e4f5a5444fb7fd967c721cca49a401`, freshness status is `current` with `commitsAhead = 0`, public temporary install passes checksum/archive/version/help/doctor/first-pack proof, Claude Code `2.1.158` passes explicit-repo `prepare_task` and `get_pack`, and Codex CLI `0.44.0` remains an optional source-free skip.
- Phase 131 adds product-aware release freshness and publishes `v1.1.2`: the GitHub release targets `ac6dc97f04cd18b5f2c6c32f7a1eca49e3ef5587`, exact freshness and product freshness are `current`, `productCommitsAhead = 0`, public temporary install passes checksum/archive/version/help/doctor/first-pack proof, Claude Code `2.1.158` passes explicit-repo `prepare_task` and `get_pack`, and Codex CLI `0.44.0` remains an optional source-free skip.
- Phase 132 adds `scripts/e2e-claude-workflow.sh`, wires it into the release gate as optional/required maintainer proof, and records `.ctxhelm/e2e/phase132-claude-workflow-eval.json` from Claude Code `2.1.159`.
- Phase 133 makes the top-level README answer "why ctxhelm" before install commands, records the current proof snapshot, updates client evidence to Codex CLI `0.44.0` and Claude Code `2.1.159`, and release-gates those public strings.
- Phase 134 refreshes the public archive to `v1.1.3`: the GitHub release targets `f17bd4cb27f1989e696717ac706868808ff01151`, freshness status is `current` with `commitsAhead = 0`, public temporary install passes checksum/archive/version/help/doctor/first-pack proof, Claude Code `2.1.159` passes explicit-repo `prepare_task` and `get_pack`, and Codex CLI `0.44.0` remains an optional source-free skip.
- Phase 135 makes Homebrew formula rendering and crates package-boundary checks release-gated without publishing those channels.
- Phase 136 refreshes the public archive to `v1.1.4`: the GitHub release targets `186fbebc8a4e9131b09665809a426c021eb5f13b`, product freshness is `current`, public temporary install passes checksum/archive/version/help/doctor/first-pack proof, distribution-readiness proof matches the public archive digest, Claude Code `2.1.159` passes explicit-repo `prepare_task` and `get_pack`, and Codex CLI `0.44.0` remains an optional source-free skip.
- Phase 137 publishes the public Apple Silicon Homebrew install path: `thromel/homebrew-tap` commit `99af0b4ca7cb1b9756dec745810cc366e1d3c086` serves `thromel/tap/ctxhelm`, and proof verifies `brew tap`, strict audit, install, `brew test`, formula URL/SHA-256, arm64 constraint, and installed `ctxhelm 1.1.4`.
- Phase 138 refreshes the public archive and Homebrew tap to `v1.1.5`: the GitHub release targets `3efa8c18d9f186c7e6a91f19c4171c3c3224158d`, product freshness is `current`, public temporary install passes checksum/archive/version/help/doctor/first-pack proof, `thromel/homebrew-tap` commit `d49a94a48c0be46391ad92fd3d872e35f3a00378` installs `ctxhelm 1.1.5`, Claude Code `2.1.159` passes explicit-repo `prepare_task` and `get_pack`, and Codex CLI `0.44.0` remains an optional source-free skip.
- Phase 139 named the product ctxhelm while preserving `ctxhelm` as the CLI/package/install/MCP compatibility surface; Phase 141 supersedes that name with ctxhelm after availability review.
- Phase 140 refreshes the public archive and Homebrew tap to `v1.1.6`: the GitHub release targets `d1a602c6fbce9e69c2fd2e80e8e2b98a7a5dc8f6`, product freshness is `current`, public temporary install passes checksum/archive/version/help/doctor/first-pack proof, `thromel/homebrew-tap` commit `3c05f5e` installs `ctxhelm 1.1.6`, Claude Code `2.1.159` passes explicit-repo `prepare_task` and `get_pack`, and Codex CLI `0.44.0` remains an optional source-free skip.
- Phase 142 refreshes the public archive and Homebrew tap to `v1.1.7` for ctxhelm: the GitHub release targets `13d2b9536be23eda13fe56f2c01ac55ec7d79a36`, product freshness is `current`, public temporary install passes checksum/archive/version/help/doctor/first-pack proof, `thromel/homebrew-tap` commit `da7dc6f` installs `ctxhelm 1.1.7`, and optional real-client smoke evidence is source-free.
- Phase 143 adds paired real Claude Code process evidence: native baseline, `ctxhelm-plan`, and `ctxhelm-brief` lanes all hit both target files; `ctxhelm-brief` uses observed explicit-repo `prepare_task` and brief `get_pack`, reduces reads from 7 to 4, and reduces irrelevant reads from 5 to 2 without storing raw prompts, source text, raw transcripts, or raw MCP traffic.
- Phase 148 hardens Codex optional real-client skip evidence: Codex CLI `0.44.0` is now classified as `stream_disconnected` with source-free exit status, stderr hash/line count, and MCP method counts (`initialize`, `notifications/initialized`, `tools/list`), while Claude Code `2.1.159` still passes explicit-repo `prepare_task` and `get_pack` against the public `v1.1.10` archive.
- Phase 149 publishes/verifies the current public `v1.1.11` archive and Homebrew tap at commit `9e23b997c4cf8985767c1194245ab6d44491b19e`; public freshness reports `commitsAhead = 0`, archive install and Homebrew proof pass, Claude Code `2.1.159` passes explicit-repo `prepare_task`/`get_pack`, and Codex CLI `0.44.0` remains an optional source-free `stream_disconnected` skip.
- Phase 150 adds explicit-target packaging plus a non-publishing release-artifact workflow for Linux x64, macOS Intel, and macOS Apple Silicon archives; local packaging, release docs, release governance, workspace tests, and required Claude Code `2.1.159` workflow proof pass against the current tree.
- Phase 151 prepares `v1.1.12` public currentness by bumping the release identity, making version-tag pushes create/update the GitHub release and upload verified Linux x64, macOS Intel, and macOS Apple Silicon assets, and refreshing required Claude Code `2.1.159` workflow proof against `ctxhelm 1.1.12`; public freshness remains pending until the tag workflow succeeds.
- The broad corpora now have a pinned optional fixture that promotes in release/warm-cache mode, Phase 82 enforces warm-cache runtime thresholds, Phase 83 makes context-vs-all-file divergence auditable, Phase 84 records broad-scope tasks, Phase 85 exposes broad context-area hints, Phase 86 improves Python package graph coverage, Phase 87 keeps gap reports aligned with validation coverage, Phase 88 improves broad source candidate coverage, Phase 89 reduces broad-proof runtime overhead, Phase 90 proves the packaged release gate, Phase 91 makes broad area coverage measurable, Phase 92 makes broad-area gap taxonomy honest, Phase 93 removes the clean RefactoringMiner cold-runtime blocker when the checkout is usable, Phase 94 improves wide-task progressive area coverage, Phase 95 makes pack guidance actionable, Phase 96 makes area guidance resource-backed, Phase 97 improves broad governance classification, Phase 98 protects target-file budgets for archive/docs tasks, Phase 99 makes broad area resources more actionable, Phase 100 makes gap reports point to those resources, Phase 101 release-gates that shape, Phase 102 proves they can be consumed from explicit-repo MCP sessions, Phase 103 blocks broad metric regressions, Phase 104 gives agents concrete next-read paths, Phase 105 makes history outages explicit, Phase 106 makes real-client evidence request-auditable without storing raw traffic, Phase 107 keeps all four broad proof corpora machine-checkable, Phase 108 makes local object-store failures bounded and explicit, Phase 109 makes environment failures machine-readable, Phase 110 makes the clean cold fixture proof promote, Phase 111 makes it release-gate visible, Phase 112 proves the complete clean release-candidate gate, Phase 113 records the archive-first candidate decision, Phase 114 publishes it, Phase 115 proves the public install path, Phase 116 proves/reports optional real-client behavior against the public archive, Phase 117 makes broad area role mix explicit, Phase 118 makes context-area resource scope explicit, Phase 119 removes a release-validation flake, Phase 124 makes broad area resource consumption more bounded, Phase 125 makes lexical-proof claims explicit, Phase 126 proves the actual agent evidence set does not trail lexical on any corpus, Phase 127 proves raw target-file recall no longer trails lexical on any corpus, Phase 128 proves broad protected target misses are clear on the clean four-repo fixture, Phase 129 makes public archive freshness explicit, Phase 130 makes the refreshed archive current, Phase 131 makes release freshness product-aware while publishing v1.1.2, Phase 132 proves real Claude workflow calls without raw prompts or raw MCP traffic, Phase 133 keeps the public README aligned with that proof, Phase 134 makes that public-facing product state downloadable, and Phase 137 adds a normal Homebrew install channel. Remaining work is refreshed optional real-client evidence when client versions change, adding additional supported distribution channels when intentionally supported, and continued production hardening.

## Shipped

### v2.4 Production Semantic & Precision Backends

**Goal:** Convert semantic and precision retrieval from local scaffolding into measured, policy-gated retrieval-quality improvements without breaking ctxhelm's local-first and source-safe contract.

**Status:** Shipped locally: 2026-05-20.

**Phases completed:** Phases 56-60, 5 plans total

**Key accomplishments:**

- Added an optional production-local `local_fastembed` backend behind the `local-embeddings` feature, while keeping `local_hash` as deterministic scaffold behavior.
- Rebuilt semantic retrieval around source-free semantic documents enriched with safe inventory, symbols, dependency edges, related tests, docs/cards, and precision status.
- Added source-free query construction traces and hybrid fusion controls for task, commit, path, symbol, and error-like inputs.
- Added provider and reranker policy gates with cloud embeddings, source transfer, and cloud reranking disabled by default.
- Added semantic/precision evaluation gates and release proof boundaries so defaults are not promoted without measured lift.
- Verified Claude Code can pass semantic provider/model/dimension controls through MCP.
- Ran the fresh RefactoringMiner paired proof: default and `local_hash` are now at parity after the semantic seed fix, but both still trail lexical baseline.

**Artifacts:**

- Roadmap: `.planning/ROADMAP.md`
- Requirements: `.planning/REQUIREMENTS.md`
- Research: `.planning/research/`
- Audit: `.planning/milestones/v2.4-MILESTONE-AUDIT.md`
- Fresh E2E: `.planning/e2e/2026-05-22-refactoringminer-semantic-fusion-regression.md`
- Phases: `.planning/phases/56-production-local-semantic-backend/` through `.planning/phases/60-semantic-precision-evaluation-gates-and-release-proof/`

### v2.3 Evaluation Lab & Learned Retrieval Policy

**Goal:** Make ctxhelm's retrieval-quality claims repeatable across fixed corpora, large histories, policy variants, and source-free learned retrieval experiments.

**Status:** Shipped locally: 2026-05-19.

**Phases completed:** Phases 50-55, 6 plans total

**Key accomplishments:**

- Added fixed source-free benchmark corpus manifests and locked RefactoringMiner v2.3 baseline metadata.
- Added cached and deterministic parallel historical eval with runtime diagnostics.
- Added source-free candidate feature exports for learning, diagnostics, and paired analysis.
- Added paired baseline and ablation analysis with lexical lift/parity/regression verdicts.
- Added offline learned retrieval-policy proposals with thresholded application and rollback controls.
- Added v2.3 product proof summary, deterministic eval smoke, and release-gate proof boundary docs.

**Archive:**

- Roadmap: `.planning/milestones/v2.3-ROADMAP.md`
- Requirements: `.planning/milestones/v2.3-REQUIREMENTS.md`
- Audit: `.planning/milestones/v2.3-MILESTONE-AUDIT.md`
- Phases: `.planning/milestones/v2.3-phases/`

### v2.2 Release & Distribution Hardening

**Goal:** Turn the locally complete ctxhelm product into a reproducible, installable, public release with clean packaging, upgrade, adoption, and proof artifacts.

**Status:** Shipped locally: 2026-05-19.

**Phases completed:** Phases 45-49, 5 plans total

**Key accomplishments:**

- Added clean-checkout release gate and source-free release proof bundle.
- Added install, upgrade, setup-check, troubleshooting, and agent setup docs.
- Added public adoption docs, static demo artifacts, and first-pack guidance.
- Added distribution metadata, clean extraction verification, and signing/notarization gap docs.
- Added release governance, candidate lifecycle, and rollback documentation.

**Archive:**

- Roadmap: `.planning/milestones/v2.2-ROADMAP.md`
- Requirements: `.planning/milestones/v2.2-REQUIREMENTS.md`
- Audit: `.planning/milestones/v2.2-MILESTONE-AUDIT.md`
- Phases: `.planning/milestones/v2.2-phases/45-clean-release-gate-proof-bundle/` through `.planning/milestones/v2.2-phases/49-release-governance-candidate-lifecycle/`

### v2.1 Pack Inspector & GraphRAG Retrieval

**Goal:** Add an optional diagnostic pack inspector and measured GraphRAG/embedding retrieval improvements while keeping ctxhelm local-first, read-only, and agent-native.

**Status:** Shipped locally: 2026-05-18.

**Phases completed:** Phases 39-44, 6 plans total

**Key accomplishments:**

- Added source-free pack inspector contracts plus JSON, Markdown, and static HTML exports.
- Added a local read-only inspector UI with filters, responsive layout checks, and sentinel leak tests.
- Added retrieval-health reports for historical eval trends, feedback gaps, signal contribution, and token ROI.
- Added source-free graph neighborhood/community reports and policy experiment comparisons.
- Added semantic provider status and explicit cloud-disabled embedding/reranking controls.
- Added agent previews for Codex CLI, Claude Code, Cursor, OpenCode, and generic MCP clients.
- Wired inspector, health, graph, policy/embedding, and agent-preview smokes into release docs and release-gate contracts.

**Archive:**

- Roadmap: `.planning/milestones/v2.1-ROADMAP.md`
- Requirements: `.planning/milestones/v2.1-REQUIREMENTS.md`
- Audit: `.planning/milestones/v2.1-MILESTONE-AUDIT.md`
- Research: `.planning/milestones/v2.1-research/`
- Phases: `.planning/milestones/v2.1-phases/`

### v2.0 Workspace & Team Layer (Shipped locally: 2026-05-17)

**Delivered:** Multi-repo workspace manifests/status, workspace-aware context plans and packs, source-free shared artifact manifests, local team privacy policy templates, MCP workspace resources, docs, and release smokes.

**Phases completed:** Phases 35-38, 4 plans total

**Key accomplishments:**

- Added source-free workspace manifests and status aggregation across multiple local repositories.
- Added workspace-aware `prepare-task` and `get-pack` with repo-boundary-preserving `repoPacks`.
- Added shared artifact export, inspect, and import flows plus local team privacy policy reports.
- Added MCP resources for `ctxhelm://workspace/status` and `ctxhelm://workspace/shared-artifacts` without expanding the six-tool MCP surface.
- Added workspace/shared-artifact docs and release-gate smoke coverage.

**Archive:**

- Roadmap: `.planning/milestones/v2.0-ROADMAP.md`
- Requirements: `.planning/milestones/v2.0-REQUIREMENTS.md`
- Research: `.planning/milestones/v2.0-research/`
- Phases: `.planning/milestones/v2.0-phases/`

### v1.7 Adaptive Retrieval Policy & Feedback Loop (Shipped: 2026-05-17)

**Delivered:** Source-free feedback events, policy quality reports, adaptive policy profile controls, agent outcome comparison, feedback docs, and release-gate smoke coverage.

**Phases completed:** Phases 30-34, 5 plans total

**Key accomplishments:**

- Added source-free session feedback contracts and local JSONL ingestion/list/summary CLI.
- Added policy quality reports for context precision, read precision, edit recall proxy, validation coverage, repeated missing-file families, signal contribution, and token ROI.
- Added explicit local retrieval-policy profile controls for tune, list, apply, disable, and rollback.
- Added outcome comparison by plan-only, brief, standard, and deep pack budgets with low-evidence warnings.
- Added `docs/feedback.md`, `scripts/smoke-feedback.sh`, release-doc checks, and release-gate coverage.

**Archive:**

- Roadmap: `.planning/milestones/v1.7-ROADMAP.md`
- Requirements: `.planning/milestones/v1.7-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.7-MILESTONE-AUDIT.md`
- Phases: `.planning/milestones/v1.7-phases/`

### v1.6 Repo Memory & Experience Cards (Shipped: 2026-05-16)

**Delivered:** Source-free repo memory cards, experience cards, review controls, and selected-memory plan/pack output.

**Phases completed:** Phases 25-29, 5 plans total

**Key accomplishments:**

- Added source-free memory card contracts and SQLite `memory_cards` persistence.
- Generated freshness-aware domain cards from safe inventory, symbols, tests, docs, and dependency edges.
- Generated source-free experience cards from local eval traces and structured metadata.
- Selected memory into `prepare_task`, `get_pack`, and MCP resources under explicit evidence and token-budget caps.
- Added memory review/redaction/disable/regeneration commands, docs, and deterministic smoke coverage.

**Archive:**

- Roadmap: `.planning/milestones/v1.6-ROADMAP.md`
- Requirements: `.planning/milestones/v1.6-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.6-MILESTONE-AUDIT.md`
- Research: `.planning/milestones/v1.6-research/`
- Phases: `.planning/milestones/v1.6-phases/`

### v1.5 Parser/Semantic Precision (Shipped: 2026-05-16)

**Delivered:** Java/Kotlin parser precision plus source-free precision edge import for local SCIP/LSP bridge outputs.

**Phases completed:** Phases 21-24, 4 plans total

**Key accomplishments:**

- Added Java/Kotlin symbol extraction for safe inventoried source/test files.
- Added Java/Kotlin dependency graph inference for safe local package imports and common source-root layouts.
- Added `ctxhelm precision import` with source-free edge validation and `.ctxhelm/precision-edges.json` persistence.
- Added additive `precision:<edgeType>` dependency output without changing existing graph contracts.
- Added `docs/precision.md`, `scripts/smoke-precision.sh`, release-doc checks, and release-gate coverage.
- Verified parser precision on the RefactoringMiner repository.

**Archive:**

- Roadmap: `.planning/milestones/v1.5-ROADMAP.md`
- Requirements: `.planning/milestones/v1.5-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.5-MILESTONE-AUDIT.md`
- Phases: `.planning/milestones/v1.5-phases/`

### v1.4 Local Semantic Retrieval (Shipped: 2026-05-16)

**Delivered:** Optional local semantic retrieval as a measured, source-free, local-only signal inside the context compiler.

**Phases completed:** Phases 17-20, 16 plans total

**Key accomplishments:**

- Added typed semantic provider metadata with disabled-by-default invocation flags.
- Added schema v2 source-free semantic vector metadata with incremental reuse counts.
- Added local semantic search, `--semantic` CLI support, and additive MCP `semantic` arguments for existing workflows.
- Fused semantic candidates as a secondary retrieval signal behind exact path, active diff, symbol, lexical, graph, and test evidence.
- Added semantic-enabled historical eval metadata, `docs/semantic.md`, and deterministic semantic release-gate smoke coverage.

**Archive:**

- Roadmap: `.planning/milestones/v1.4-ROADMAP.md`
- Requirements: `.planning/milestones/v1.4-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.4-MILESTONE-AUDIT.md`
- Phases: `.planning/milestones/v1.4-phases/`

### v1.3 Production Storage (Shipped: 2026-05-14)

**Delivered:** Durable, source-free SQLite storage for repository intelligence, incremental inventory sync, pack/eval/proof metadata persistence, storage operations, docs, and release-gate smoke coverage.

**Phases completed:** Phases 13-16, 16 plans total

**Key accomplishments:**

- Added a versioned source-free SQLite schema with metadata, migration history, and privacy labels.
- Added `ctxhelm index --store` with reused/created/updated/deleted safe file record counts.
- Added source-free pack, historical eval, benchmark, retrieval-gap, and proof metadata persistence.
- Added `ctxhelm storage init/status/repair/vacuum/reset` with reset dry-run behavior.
- Added `docs/storage.md` and `scripts/smoke-storage.sh`, wired into release docs and release gate.

**Archive:**

- Roadmap: `.planning/milestones/v1.3-ROADMAP.md`
- Requirements: `.planning/milestones/v1.3-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.3-MILESTONE-AUDIT.md`
- Phases: `.planning/milestones/v1.3-phases/`
- Research: `.planning/milestones/v1.3-research/`

### v1.2 Retrieval Quality Proof (Shipped: 2026-05-14)

**Delivered:** Repeatable, source-free retrieval-quality proof with benchmark suites, fixed-budget metrics, baseline comparisons, gap taxonomy, trend comparison, and product proof generation.

**Phases completed:** Phases 9-12, 17 plans total

**Key accomplishments:**

- Added named benchmark suite contracts and bounded multi-repo historical evaluation with reproducibility and privacy metadata.
- Added fixed-budget file/test recall, lexical and no-context baselines, signal ablations, and token ROI reporting.
- Added source-free retrieval gap taxonomy, future-milestone recommendations, benchmark comparison, and regression thresholds.
- Added `ctxhelm eval proof` plus optional `CTXHELM_BENCHMARK_CONFIG` release-gate proof.
- Kept benchmark, comparison, and proof artifacts source-free and local-only by default.

**Archive:**

- Roadmap: `.planning/milestones/v1.2-ROADMAP.md`
- Requirements: `.planning/milestones/v1.2-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.2-MILESTONE-AUDIT.md`

### v1.1 Packaging & Adoption (Shipped: 2026-05-13)

**Delivered:** A packaged, documented, smoke-testable ctxhelm release path for agent-native adoption.

**Phases completed:** Phases 1-8, 32 plans total

**Key accomplishments:**

- Locked compatibility and source-free contract guardrails across CLI, MCP, and JSON outputs.
- Hardened safe inventory, diagnostics, context planning, packs, eval traces, and historical retrieval reports.
- Verified agent-native client durability through deterministic MCP proof and optional Codex/Claude real-client wrappers.
- Added v1.1.0 release identity, repeatable local binary archives, SHA-256 checksums, and artifact leakage audit.
- Added repo-local setup, `setup-check`, first-pack smoke, and thin guidance for Codex, Claude Code, Cursor, and OpenCode.
- Added docs and a release gate that verifies tests, docs, packaging, artifact audit, selected-binary behavior, MCP proof, and optional client wrappers.

**Archive:**

- Roadmap: `.planning/milestones/v1.1-ROADMAP.md`
- Requirements: `.planning/milestones/v1.1-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.1-MILESTONE-AUDIT.md`

## Planned Product Vision

### v2.5 Agent-Native Deep Integrations

**Goal:** Make ctxhelm feel native inside Codex, Claude Code, Cursor, OpenCode, and generic MCP clients without taking over editing or shell execution.

**Depends on:** v1.1 setup/adoption, v2.1 agent previews, v2.2 release installation path.

**Expected capabilities:**

- Stronger Codex and Claude Code real-client proof with request-log artifacts.
- Cursor and OpenCode proof paths where the clients expose machine-checkable evidence.
- Agent-specific prompts/hooks/rules that stay thin, dynamic, and repo-local.
- Cloud/disconnected fallback cards for agents that cannot reach local MCP.

### v2.5 Current Production-Readiness Follow-Up

**Latest evidence:** `.planning/e2e/2026-06-17-phase325-efficiency-status-verdict.md`

**Current R&D status:** Phase 324 closes the remaining Codex consumption
boundary where target paths were discovered but not read. The retry proof now
tracks discovered-only target counts before and after retry, and the strict
Phase 324 Codex breadth-suite proof passed with no evidence-only targets after
retry and no under-read targets. Efficiency is still not promoted: the
promotion-style irrelevant-read floor failed in the live run. Phase 325 makes
that claim boundary machine-readable through `efficiencyStatus`, classifying
the current real Codex proof as `reliability_improved_with_read_overhead`
rather than efficiency evidence. The next R&D slice should reduce retry/read
overhead without weakening target-consumption enforcement.

**Delivered through Phase 204:** Codex CLI and Claude Code real-client smoke evidence is now request-auditable without storing raw MCP traffic: evidence files keep compatibility booleans and add source-free request hashes, line counts, explicit repo tool-call counts, sanitized observed tool calls, and request-summary sidecars. History-unavailable benchmark repos keep embedded insufficient-evidence reports instead of returning `report: null`. The broad four-repo proof hydrates all configured repositories with embedded reports, the refreshed clean fixture proof now promotes with reachable RefactoringMiner, ctxhelm, ReAgent, and VeriSchema fixtures, broad context-area guidance exposes source-free signal profiles for progressive native reads, retrieval-gap summaries now include those area profiles when available, grouped gap summaries deduplicate area-profile counts without hiding per-file misses, corroborated source-history candidates now receive bounded target-file budget when co-change evidence has dependency/lexical/symbol support, historical commit evals now expose selected top-10 signal/role profiles for ranking-budget diagnosis, broad governance docs now regain budget before broad source-history while preserving the measured source recall gain, plan-level context areas now expose coverage/inspection-pressure signals so packs can prioritize high-pressure progressive native reads without target-file churn, pressure breakdowns now explain whether remaining unread area pressure comes from source-like, validation, or docs paths, eval/proof reports now aggregate that pressure per repository to expose the dominant broad-area bottleneck, next-read recovery summaries quantify how often progressive area guidance recovers files missed by top-10 context, next-read ordering now prefers stronger source-free local signals before weaker progressive reads, high-pressure areas now expose a larger bounded next-read budget without changing selected target-file metrics, broad plans now reserve selected validation areas plus package-mirrored test clusters so progressive reads can follow validation evidence instead of only source-tail pressure, product proof now separately reports how many selected-file misses are recoverable through the full agent evidence bundle, candidate coverage accounting now separates generated-but-unselected misses from true no-candidate gaps, candidate miss pressure profiles now summarize those generated-but-unselected misses by role, signal, and context area, broad tasks now reserve one nested README doc after config/workflow evidence to recover contextual documentation targets without source/test/validation regression, context-area next-read summaries now explain the remaining gap between progressive reads and the broader agent evidence bundle by role and area, markdown reports render those agent-evidence-only counts, roles, and areas for normal review, generated packs now surface selected related tests as source-free validation evidence with area/reason/confidence/command details so agents can consume tests even when they are not repeated in context-area next-read lists, and paired real-agent reports now surface forbidden shell/edit/write tool calls so read-only outcome proof cannot hide boundary violations.

**Remaining focus:** use candidate miss pressure to target VeriSchema source clusters in `schema_agent/agents` and validation routing for `tests/agents` / `tests/evaluation`; measure broad-area usefulness in real agent runs; deepen source-free area-resource diagnostics; and require retrieval improvements to show proof lift without top-10 churn or global runtime-threshold tuning.

### v2.6 Desktop Inspector & Local UX

**Goal:** Package the diagnostic inspector as a polished optional local UX for understanding and debugging ctxhelm decisions.

**Depends on:** v2.1 static inspector and agent preview, v2.2 release packaging.

**Expected capabilities:**

- Optional Tauri or native desktop shell around the local inspector.
- Graph visualization for source-free neighborhoods and communities.
- Onboarding/status checks for setup, storage, benchmark proof, and agent config.
- No daily coding UI: the desktop surface remains diagnostic and read-only.

### v2.7 Team Sync & Enterprise Controls

**Goal:** Add optional team-safe sharing and governance without weakening local-first defaults.

**Depends on:** v2.0 shared artifacts/team policy, v2.2 release trust, v2.4 provider policy controls.

**Expected capabilities:**

- Optional remote metadata sync for source-free cards, benchmark reports, policy profiles, and shared artifacts.
- Enterprise privacy/audit policy, SSO/admin controls, and explicit data-sharing review.
- Remote MCP endpoint for approved source-free or policy-allowed context.
- Clear local-only fallback with no hosted dependency.

### v3.0 Context Governor

**Goal:** Turn ctxhelm from a context compiler into an adaptive context governor for AI coding agents.

**Depends on:** v2.3 learned policy, v2.4 semantic/precision backends, v2.5 integrations, v2.7 governance.

**Expected capabilities:**

- Adaptive per-task budget, retrieval, memory, graph, semantic, and validation policy.
- Closed-loop learning from source-free agent sessions and eval outcomes.
- Policy rollout, rollback, and comparison across repos/teams.
- Context-quality decisions exposed clearly enough for maintainers to trust and tune.
