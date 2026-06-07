# R&D Completion Audit - 2026-06-06

## Scope

Audit the current ctxhelm R&D roadmap against the latest source-free proof
artifacts after Phase 289. This audit is not a release claim by itself; it
separates implemented/measured work from remaining research risk.

## Current Proof Summary

| Area | Current evidence | Status |
| --- | --- | --- |
| Codex real-agent outcome | `.ctxhelm/e2e/phase245-agent-run-codex-suite-real-bounded-final.json`, `.ctxhelm/e2e/phase250-agent-run-codex-governor-rd-after.json`, and `.ctxhelm/e2e/phase251-agent-run-codex-rd-breadth-suite.json` | Strong for the measured failure classes. Phase 245 proves a two-task suite where every ctxhelm lane reaches `1.00` target-read coverage vs baseline `0.75`. Phase 250 proves a governor/release R&D task improves from matched baseline with evidence misses to `ctxhelm_improved`. Phase 251 expands this to four R&D task families with 16 comparable ctxhelm lanes and no evidence misses, under-read targets, forbidden commands, or client failures. |
| Claude workflow | `.ctxhelm/e2e/phase250-claude-workflow-refresh.json`, `.ctxhelm/e2e/phase252-agent-run-claude-rd-breadth-suite.json`, `.ctxhelm/e2e/phase254-agent-run-claude-rd-breadth-suite.json`, `.ctxhelm/e2e/phase264-agent-client-availability.json`, `.ctxhelm/e2e/phase304-agent-client-availability.json`, `.ctxhelm/e2e/phase304-agent-run-claude-rd-breadth-suite.json`, `.ctxhelm/e2e/phase312-agent-client-availability.json`, `.ctxhelm/e2e/phase312-agent-run-claude-rd-breadth-suite-preflight-disabled.json` | Current workflow proof passes with explicit-repo MCP usage and local-only privacy. Phase 312 refreshes the fresh paired Claude R&D suite after Codex retry enforcement: Claude Code `2.1.163` can answer a tiny manual Opus probe, but source-free availability still records `rate_limited`, the preflight-disabled four-task suite is `degraded`, and there are zero comparison-eligible tasks or comparable ctxhelm lanes. Some ctxhelm calls and memory-lane forbidden `Bash` calls were observed, but every lane is non-comparable because client failures and rate limits are present. Codex CLI `0.137.0` remains available with deterministic explicit-repo MCP evidence, so this is still client availability rather than retrieval-quality evidence. |
| Experience memory | `.ctxhelm/e2e/phase234-memory-pack-impact-suite.json`, `.ctxhelm/e2e/phase244-agent-run-codex-memory-consumption.json`, `.ctxhelm/e2e/phase253-codex-memory-outcome-suite.json`, `.ctxhelm/e2e/phase255-codex-memory-efficiency-suite.json`, `.planning/e2e/2026-06-06-phase255-codex-memory-efficiency.md` | Local memory selection is source-free and bounded. Six-repo local evidence shows final-pack memory does not add non-targets; Phase 244 proves Codex consumes selected-memory targets on one repo while reducing read-file count from 18 to 5 in the best lane. Phase 253 extends real Codex memory outcome evidence across VeriSchema, ReAgent, and RefactoringMiner: 3/3 comparison-eligible pairs improved target-read coverage in the memory lane. Phase 255 tightens the memory lane and reruns the same three-repo proof: target-read improvement remains 3/3, while memory irrelevant-read improvement moves from 0/3 to 3/3. |
| GraphRAG | `.planning/e2e/2026-06-02-phase179-graph-edge-profiles.md`, `.planning/e2e/2026-06-02-phase180-graph-edge-ablations.md`, `.planning/e2e/2026-06-03-phase181-graph-edge-budget-allocation.md` | Implemented and measured. Edge-family profiles and ablations expose import and Python re-export pressure; Phase 181 applies bounded edge-family budget ordering and promotes on reachable fixtures. |
| Semantic/reranker retrieval | `.planning/e2e/2026-06-02-phase160-bounded-semantic-status-search.md`, `.planning/e2e/2026-06-02-phase161-semantic-gate-contribution-diagnostics.md`, `.planning/e2e/2026-06-06-phase252-semantic-gate-classification.md`, `.planning/e2e/2026-06-06-phase256-semantic-rich-document-rejection.md`, `.planning/e2e/2026-06-06-phase257-safe-local-metadata-reranker.md`, `.planning/e2e/2026-06-06-phase258-reranker-contribution-diagnostics.md`, `.planning/e2e/2026-06-06-phase259-runtime-reranker-protected-floor.md`, `.planning/e2e/2026-06-06-phase260-query-family-reranker-routing.md`, `.planning/e2e/2026-06-06-phase261-routed-reranker-policy.md`, `.planning/e2e/2026-06-06-phase262-routed-reranker-broader-proof.md`, `.planning/e2e/2026-06-06-phase263-routed-reranker-runtime-policy.md`, `.planning/e2e/2026-06-06-phase264-semantic-family-diagnostics.md`, `.planning/e2e/2026-06-06-phase265-semantic-family-stability.md`, `.planning/e2e/2026-06-06-phase266-semantic-support-profiles.md`, `.planning/e2e/2026-06-06-phase267-semantic-support-profile-routing.md`, `.planning/e2e/2026-06-06-phase268-support-profile-routed-semantic.md`, `.planning/e2e/2026-06-06-phase269-semantic-candidate-generation.md`, `.planning/e2e/2026-06-06-phase270-semantic-corroborated-reranker.md`, `.planning/e2e/2026-06-06-phase271-semantic-corroborated-family-diagnostics.md`, `.planning/e2e/2026-06-06-phase272-semantic-corroborated-path-family-diagnostics.md`, `.planning/e2e/2026-06-06-phase273-verischema-semantic-prefilter-cap.md`, `.planning/e2e/2026-06-06-phase274-verischema-python-module-facets.md`, `.planning/e2e/2026-06-06-phase275-semantic-shape-diagnostics.md`, `.planning/e2e/2026-06-06-phase276-reranker-displacement-diagnostics.md`, `.planning/e2e/2026-06-06-phase277-family-budget-semantic-reranker.md`, `.planning/e2e/2026-06-06-phase278-learned-profile-semantic-reranker.md`, `.planning/e2e/2026-06-07-phase280-learned-semantic-policy-artifact.md`, `.planning/e2e/2026-06-07-phase281-learned-semantic-policy-holdout.md`, `.planning/e2e/2026-06-07-phase282-learned-policy-holdout-limit40.md`, `.planning/e2e/2026-06-07-phase283-ranged-semantic-gate.md`, `.planning/e2e/2026-06-07-phase284-learned-policy-train-test.md`, `.planning/e2e/2026-06-07-phase293-jina-code-semantic-model.md`, `.planning/e2e/2026-06-07-phase294-semantic-next-read.md`, `.planning/e2e/2026-06-07-phase295-allmini12-model-probe.md`, `.planning/e2e/2026-06-07-phase296-semantic-candidate-missed-support.md`, `.planning/e2e/2026-06-07-phase297-supported-candidate-tail-slot-oracle.md`, `.planning/e2e/2026-06-07-phase298-supported-shape-tail-slot-reranker.md`, `.planning/e2e/2026-06-07-phase299-supported-shape-broader-validation.md`, `.planning/e2e/2026-06-07-phase300-supported-profile-base-rate-diagnostic.md`, `.planning/e2e/2026-06-07-phase301-semantic-candidate-retention.md`, `.planning/e2e/2026-06-07-phase302-retention-separator-train-test.md`, `.planning/e2e/2026-06-07-phase303-retention-separator-margin.md` | Integrated and measured as optional local signals. Phase 252 fixes gate classification so eval-only reranker regressions no longer make semantic look blocked. Phase 256 rejects richer symbol/dependency semantic search documents. Phase 257 makes the eval reranker preserve protected source evidence and validation-test reserves. Phase 258 adds contribution diagnostics showing RefactoringMiner is reranker-neutral while ctxhelm has strong target-hit lift plus explicit target churn. Phase 259 aligns policy-enabled runtime reranking with the same protected-source floor. Phase 260 adds query-family routing diagnostics. Phase 261 adds an eval-only routed variant. Phase 262 broadens that proof and rejects `symbol_identifier` routing after ReAgent regression, narrowing routing to `commit_clue` only. Phase 263 exposes that narrower route as opt-in runtime policy through `enableQueryFamilyRoutedReranker`, while defaults stay disabled. Phase 264 adds semantic query-family contribution diagnostics and shows selective semantic candidates. Phase 265 broadens those semantic-family checks to `limit 20`, adds commit-level stability counters, and rejects coarse semantic query-family routing. Phase 266 adds semantic-only support profiles and rejects generic "semantic plus any other signal" routing. Phase 267 classifies support profiles as route candidates, mixed holds, or noise holds. Phase 268 implements the resulting eval-only support-profile-routed variant and rejects it as neutral across four repos. Phase 269 adds candidate-generation versus fusion diagnostics. Phase 270 tests semantic-corroborated reranking and rejects global promotion after strong RefactoringMiner lift but regressions on ctxhelm, ReAgent, and VeriSchema. Phase 271 adds contribution diagnostics for that variant and rejects query-family-only routing because RefactoringMiner's clean route candidates are unsafe on other repos. Phase 272 adds path-family diagnostics and rejects broad path-family routing because clean RefactoringMiner docs/config/Java lift does not generalize to ctxhelm, ReAgent, or VeriSchema. Phase 273 rejects a larger local-fastembed prefilter cap as a VeriSchema fix because all semantic metrics remain unchanged at cap `256`. Phase 274 rejects path-derived Python package/module facets because they lower semantic-corroborated recall and increase semantic noise. Phase 275 adds query/path shape diagnostics and rejects a temporary shape-routed insertion variant because it loses RefactoringMiner and ReAgent target tests. Phase 276 adds displacement diagnostics showing which reranked-only non-target docs/planning/scripts/paper files enter top-K while default target tests/source files are lost. Phase 277 adds an eval-only path-family budget variant; it has clean lift on RefactoringMiner and ReAgent but regresses ctxhelm and VeriSchema, so it stays diagnostic only and is rejected for promotion. Phase 278 adds a leave-one-out learned-profile semantic variant that recovers clean RefactoringMiner lift (`0.41857147 -> 0.51857144`, `+2`) while staying exactly neutral on ctxhelm, ReAgent, and VeriSchema. Phase 280 adds `learnedSemanticPolicy`, a deterministic source-free artifact with staleness, holdout, and `minimumSupportCommitCount = 2`; only RefactoringMiner exports one eligible durable profile and every artifact remains `runtimePromotable = false`. Phase 281 applies that support threshold in a source-free held-out variant; it has zero regressions but `appliedCommitCount = 0` on all four repos, so the decision is `insufficient_eligible_holdout_profiles` rather than promotion. Phase 282 repeats the learned-policy holdout at `limit 40`; it still has zero held-out applications on all four repos, proving same-gate widening is insufficient. Phase 283 adds stable ranged gates and shows older/recent slices still do not repeat the learned-policy signal across time. Phase 284 adds a true train-on-range/apply-on-disjoint-test-range evaluator and records a negative four-repo result: no repo has eligible-profile test overlap or policy applications outside training, so promotion remains blocked. Phase 293 fixes the explicit Jina code-model dimension/text contract and shows better VeriSchema candidate quality, but no recall lift or safe semantic fusion. Phase 294 adds bounded semantic next-read contribution diagnostics and rejects next-read promotion because the only appended post-top-K path is a non-target. Phase 295 rejects `AllMiniLML12V2Q` and `AllMiniLML12V2` as simple model swaps because both are noisier than corrected Jina on the targeted slice. Phase 296 adds candidate-missed support profiles and shows the remaining Jina candidate miss has `dependency_co_change` support. Phase 297 adds a gold-backed supported-candidate tail-slot oracle that cleanly recovers that target, proving an upper bound for source-free predictor work without promoting the oracle. Phase 298 replaces the oracle dependency with a source-free supported-shape variant that matches the clean lift on the targeted slice. Phase 299 broadens the proof across eight stable slices and finds zero regressions/default-only churn but only the original VeriSchema older lift, so the variant stays eval-only. Phase 300/301 add supported-profile base-rate and retained/dropped candidate diagnostics, showing useful separator input but no runtime/default promotion evidence. Phase 302 tests strict held-out retention separation and finds zero eligible train families across four repos. Phase 303 shows that hidden retention margins are still structurally weak: only 8 of 117 families have positive margin, and relaxed rows recover targets only with much larger non-target insertion. |
| Task-mentioned commit anchoring | `.planning/e2e/2026-06-06-phase279-mentioned-commit-path-anchors.md`, `prepare_plan_promotes_task_mentioned_commit_changed_paths` | Implemented as a local-only planner signal. Prepare-task now promotes safe changed paths from explicitly mentioned commit SHAs into path facets, lexical terms, graph seeds, related-test terms, and target anchors. The signal is inventory-filtered, bounded, diagnostic-backed, and does not expose commit subjects, patches, or source text. |
| Agent-native integrations | `.planning/e2e/2026-06-06-phase247-cursor-opencode-real-client-proof.md`, `.planning/e2e/2026-06-06-phase246-agent-native-fallback-proof.md`, `docs/agent-setup.md` | Codex, Claude, OpenCode, Cursor setup/fallback surfaces exist. OpenCode real-client proof passes. Cursor real-client proof is available but local auth-blocked. |
| Local inspector/governor UX | `.planning/e2e/2026-06-06-phase248-local-inspector-shell.md`, `.planning/e2e/2026-06-06-phase249-context-governor.md` | Implemented with source-sentinel smokes and release-gate wiring. Phase 250 fixed the first observed real-agent retrieval gap for governor/release R&D tasks. |

Phase 285 addendum: `.planning/e2e/2026-06-07-phase285-broader-train-test-surface.md`
reruns the Phase 284 train/test evaluator at `--train-limit 40 --test-limit
40`. The result is still no-signal: every measured repo has
`trainTestEligibleProfileOverlapCount = 0` and `appliedCommitCount = 0`, so the
next semantic step should not be another same-range doubling.

Phase 286 addendum: `.planning/e2e/2026-06-07-phase286-profile-key-backoff.md`
adds an eval-only path-family backoff key to the train/test evaluator and
rejects it. Coarse train/test overlap increases, but all path-family aggregates
are blocked by inserted semantic non-targets, missing target hits, or lost
default targets before application.

Phase 287 addendum: `.planning/e2e/2026-06-07-phase287-semantic-source-role-query.md`
adds an eval-only `--semantic-query-mode source-role-hints` probe and rejects
generic source-role query expansion on the targeted VeriSchema older-range
slice. Candidate target hits rise only `13 -> 14`, while candidate misses rise,
selected semantic target hits fall, semantic-only target hits drop to zero, and
the semantic-corroborated regression remains.

Phase 288 addendum: `.planning/e2e/2026-06-07-phase288-cross-repo-learned-policy.md`
adds `ctxhelm eval learned-policy-cross-repo` and rejects the current
source-free cross-repo learned-policy aggregation. Strict Phase 285 inputs have
`28` candidate profiles and `0` eligible profiles; Phase 286 backoff inputs
have `9` candidate profiles and `0` eligible profiles. The repeated keys are
blocked by inserted semantic non-targets, lost default targets, or missing
inserted target hits.

Phase 289 addendum: `.planning/e2e/2026-06-07-phase289-semantic-candidate-path-query.md`
adds an eval-only `--semantic-query-mode candidate-path-hints` probe. It
improves the targeted VeriSchema semantic candidate-quality counters
(`candidateMisses = 3`, selected semantic targets `11`, semantic-only targets
`2`, semantic-only non-targets `16`) but does not improve `local_semantic`
recall or remove the semantic-corroborated regression, so it remains
diagnostic-only.

Phase 290 addendum:
`.planning/e2e/2026-06-07-phase290-semantic-tail-slot-reranker.md` adds the
eval-only `semantic_tail_slot_reranked` variant. It preserves the top
`ceil(0.8 * K)` default context files and lets semantic-corroborated candidates
compete only for tail slots. On the same VeriSchema older-range
candidate-path-hints proof, it removes the known semantic-corroborated
regression (`targetHitDelta = 0`, `regressedCommitCount = 0`,
`defaultOnlyTargetHitCount = 0`) but adds no target hits, so it is safety
evidence rather than promotion evidence.

Phase 291 addendum:
`.planning/e2e/2026-06-07-phase291-semantic-candidate-sibling-query.md` adds
an eval-only `--semantic-query-mode candidate-sibling-path-hints` probe for
same-directory and mirrored-test aliases around top lexical candidates. The
targeted VeriSchema proof rejects it: candidate misses worsen from Phase 289
`3 -> 4`, selected semantic targets fall `11 -> 10`, the semantic-corroborated
regression remains, and the tail-slot variant remains neutral.

Phase 292 addendum:
`.planning/e2e/2026-06-07-phase292-semantic-path-concept-query.md` adds an
eval-only `--semantic-query-mode candidate-path-concept-hints` probe for generic
software-domain concept terms derived from source-free task and candidate-path
terms. The targeted VeriSchema proof rejects it harder: semantic candidate
targets fall `14 -> 13`, selected semantic targets fall `11 -> 9`,
semantic-only non-targets rise `16 -> 19`, and even the conservative tail-slot
variant regresses.

Phase 293 addendum:
`.planning/e2e/2026-06-07-phase293-jina-code-semantic-model.md` fixes and tests
the explicit `JinaEmbeddingsV2BaseCode` local-fastembed path. The first model
probe exposed zero semantic candidates because Jina was normalized to `384`
dimensions while the model emits `768`-dimension vectors. The fix normalizes
Jina to `768`, renders provider-specific `query:` / `passage:` text, and
includes that text contract in semantic document vector hashes. The corrected
targeted VeriSchema proof improves candidate quality over Phase 289 AllMini
candidate-path hints (`candidate misses 3 -> 1`, selected semantic targets
`11 -> 12`, semantic-only non-targets `16 -> 12`) but remains held because
`local_semantic` recall is unchanged, `semantic_corroborated_reranked` still
has `targetHitDelta = -1`, tail-slot reranking is neutral, and runtime ratio is
`6.01x` with recall delta `+0.000`.

Phase 294 addendum:
`.planning/e2e/2026-06-07-phase294-semantic-next-read.md` adds
`semanticNextReadContribution`, a source-free eval diagnostic for bounded
semantic next reads after preserving the full default top K. On the targeted
VeriSchema older-range Jina candidate-path proof, semantic appends only one
post-top-K next-read path and it is a non-target:
`tests/core/test_state_validator.py`. The report records
`appendedTargetHitCount = 0`, `appendedNonTargetCount = 1`, and diagnostic
`semantic_next_read_noise_hold`. This closes the current "semantic as next-read
guidance" hypothesis as negative for the corrected Jina candidate-path setup.

Phase 295 addendum:
`.planning/e2e/2026-06-07-phase295-allmini12-model-probe.md` tests the
documented `AllMiniLML12V2Q` and `AllMiniLML12V2` local-fastembed models on the
same VeriSchema older-range `candidate-path-hints` proof after a feature-enabled
provider preflight. Both variants are worse than corrected Jina: candidate
misses worsen `1 -> 3`, semantic-only non-targets rise `12 -> 18/19`, bounded
next-read noise rises `1 -> 3` non-target appends with no target appends,
`local_semantic` Recall@10 stays `0.29122806`, the known
semantic-corroborated `targetHitDelta = -1` remains, runtime ratio is
`6.75x/7.34x`, and tail-slot reranking remains neutral. This rejects MiniLM12
as a simple model-swap branch.

Phase 296 addendum:
`.planning/e2e/2026-06-07-phase296-semantic-candidate-missed-support.md` adds
candidate-missed support profiles to `semanticContribution`. On the same
feature-enabled Jina candidate-path proof, the only semantic-generated missed
target is `schema_agent/core/state.py`, and it has `dependency_co_change`
support in the missed-candidate profile. The gate remains held with recall
delta `+0.000`, but the diagnostic is now
`semantic_candidate_fusion_supported_gap`, so the next semantic slice should
target fusion/top-K ordering for supported semantic candidates before broader
document construction or model swaps.

Phase 297 addendum:
`.planning/e2e/2026-06-07-phase297-supported-candidate-tail-slot-oracle.md`
adds the eval-only `semantic_supported_candidate_tail_slot_oracle` variant and
`supportedCandidateTailSlotRerankerContribution`. On the same feature-enabled
Jina proof, the oracle recovers `schema_agent/core/state.py`, improves target
hits `13 -> 14`, raises file Recall@10 to `0.3438596`, and has zero
default-only target churn. This is an upper bound for source-free predictor
work, not runtime policy, because it consumes eval target-miss profiles.

Phase 298 addendum:
`.planning/e2e/2026-06-07-phase298-supported-shape-tail-slot-reranker.md`
adds the eval-only source-free `semantic_supported_shape_tail_slot_reranked`
variant, `supportedSemanticCandidateProfilesAt10`, and
`supportedShapeTailSlotSemanticRerankerContribution`. On the same
feature-enabled Jina proof, the predictor uses the measured
`symbol_identifier` / `python_source` / `dependency_co_change` supported
candidate shape, recovers `schema_agent/core/state.py`, matches the oracle's
target-hit lift `13 -> 14`, raises file Recall@10 to `0.3438596`, and has zero
regressions/default-only target churn. This replaces the oracle dependency for
the targeted slice but remains eval-only pending broader proof.

Phase 299 addendum:
`.planning/e2e/2026-06-07-phase299-supported-shape-broader-validation.md`
broadens the supported-shape predictor across RefactoringMiner, ctxhelm,
ReAgent, and VeriSchema older/recent ranges after a clean feature-enabled Jina
rebuild. Across 159 evaluated commits, the predictor has target-hit delta `+1`,
one improved commit, zero regressions, and zero default-only target hits. The
only lift remains the original VeriSchema older `schema_agent/core/state.py`
case, so the variant is safe but too sparse for runtime/default promotion.

Phase 300 addendum:
`.planning/e2e/2026-06-07-phase300-supported-profile-base-rate-diagnostic.md`
adds `supportedSemanticCandidateProfileSummary` to history and gate reports.
Across the eight Phase 299 slices, supported semantic candidate profiles have
787 rows, 22 targets, 765 non-targets, and `0.027954` target precision. The
shape surface has 171 per-slice rows, 90 thin cells, and only one
repeated-target shape, which is docs/planning noise. The VeriSchema source lift
remains a singleton thin cell, so this branch is diagnostic-only.

Phase 301 addendum:
`.planning/e2e/2026-06-07-phase301-semantic-candidate-retention.md` adds
`semanticCandidateRetentionSummary` to history and gate reports. Across the
same eight slices, supported semantic candidate retention has 1071 profiles,
52 retained targets, 22 dropped targets, 232 retained non-targets, and 765
dropped non-targets. The 12 recoverable dropped-target family rows are useful
for held-out separator work, but retained non-targets still block
runtime/default promotion.

Phase 302 addendum:
`.planning/e2e/2026-06-07-phase302-retention-separator-train-test.md` adds
`ctxhelm eval retention-separator-train-test` and
`SemanticRetentionSeparatorTrainTestReport`. Across the four-repo
recent-to-older proof, strict retained-target / retained-non-target training
families produce 117 train families, 0 eligible train families, 377 test
dropped profiles, 53 train/test family overlaps, 0 eligible overlaps, and 0
applications. Every repo decides `insufficient_train_families`, so this remains
diagnostic-only and the next branch should not relax the same proof merely to
force applications.

Phase 303 addendum:
`.planning/e2e/2026-06-07-phase303-retention-separator-margin.md` adds margin
and threshold-sweep diagnostics to the same train/test report after Claude Code
recommended checking whether the zero-eligible result was a threshold artifact.
Across the same four repos, only 8 of 117 train families have positive margin
(`0.068376`), below the pre-registered `15%` continuation bar. Relaxed
threshold rows recover targets only with larger non-target insertion: the best
aggregate recovery row recovers 3 targets and inserts 21 non-targets, while the
cleanest relaxed target-recovery row still recovers 1 target and inserts 4
non-targets.

Phase 304 addendum:
`.planning/e2e/2026-06-07-phase304-claude-outcome-availability-refresh.md`
refreshes real-client outcome availability after Phase 303 redirected the next
R&D branch toward agent outcome proof. `.ctxhelm/e2e/phase304-agent-client-availability.json`
shows Claude Code `2.1.163` is still `rate_limited` while Codex CLI `0.137.0`
is available with deterministic explicit-repo MCP evidence. The same four-task
Claude R&D breadth suite remains `degraded`: 4/4 preflights are rate-limited,
with 0 comparison-eligible tasks, 0 comparable ctxhelm lanes, no ctxhelm tool
calls, and `insufficient_comparable_lanes`.

Phase 312 addendum:
`.planning/e2e/2026-06-07-phase312-claude-cross-agent-refresh.md` refreshes
cross-agent proof after Phase 310/311 closed the measured Codex
target-consumption gap. A tiny manual Claude Opus probe returned `OK`, but
`.ctxhelm/e2e/phase312-agent-client-availability.json` still records Claude
Code `2.1.163` as `rate_limited` while Codex CLI `0.137.0` is available. The
preflight-disabled four-task Claude suite remains `degraded` with 0/4
comparison-eligible tasks, 0 comparable ctxhelm lanes, outcome claim
`insufficient_comparable_lanes`, client failures and rate limits observed, and
recommended action `retry_real_client_when_available`. It observed ctxhelm tool
calls and memory-lane forbidden `Bash` calls, but this is boundary/availability
evidence rather than retrieval-quality evidence because every lane is
non-comparable.

## Remaining R&D Risk

1. Semantic is not yet a proven default-lift channel. It is useful and bounded,
   and Phase 252 shows `local_fastembed` is available and source-free, with a
   small RefactoringMiner lift but neutral ctxhelm recall. Phase 256 shows that
   adding symbol/dependency facets to semantic search documents is the wrong
   next move. Phase 257/258 reduce reranker risk by making the local metadata
   reranker safe to evaluate and diagnosable, but they also show corpus-specific
   behavior and target churn. Phase 259 reduces runtime risk by giving
   policy-enabled reranking the same protected-source floor. Phase 260 adds the
   first query-family routing view, Phase 261 evaluates a conservative routed
   policy, and Phase 262 broadens that policy across four repos. The broader
   proof rejected `symbol_identifier` routing and kept only `commit_clue`,
   removing routed regressions and default-only churn across all four measured
   repos while preserving a smaller clean ctxhelm lift. Phase 263 adds this
   narrower route to provider policy as opt-in runtime behavior. This is still
   not unconditional semantic or full-reranker promotion. Phase 264 adds
   semantic query-family diagnostics and finds possible route candidates
   (`domain_phrase` on VeriSchema and `broad_scope` on ctxhelm), but Phase 265
   broadens that check and shows the candidates are unstable at `limit 20`: each
   clean target-only family also has mixed or noise-only commits. Semantic
   remains opt-in. Phase 266 tests the next stricter hypothesis and shows that
   generic non-semantic corroboration also supports semantic-only non-targets.
   The next semantic step needs support-family-specific constraints or better
   local query/document construction, not a broad "semantic plus support"
   policy. Phase 267 finds support-profile-level candidates, but they are sparse
   and surrounded by mixed/noise holds. Phase 268 runs the resulting eval-only
   support-profile-routed variant and finds zero regressions but also zero wins
   across the four-repo proof. Phase 269 then separates candidate-generation
   failures from fusion failures: ctxhelm, ReAgent, and RefactoringMiner have
   semantic candidate target hits that final top-K selection drops, while
   VeriSchema has no generated-but-unselected semantic target candidates and
   still needs candidate-generation/query coverage work. Semantic R&D should now
   test fusion/budget constraints for the first three repos and separate local
   query/model/document coverage experiments for VeriSchema before any route or
   runtime exposure. Phase 270 tests the first such source-free fusion rule and
   proves it is not globally safe: it works strongly on RefactoringMiner but
   regresses the other three measured repos. Phase 271 then tests the obvious
   narrower route and rejects query-family-only routing: `domain_phrase` and
   `symbol_identifier` lift RefactoringMiner but regress or churn other repos.
   Phase 272 adds path-family diagnostics and shows the RefactoringMiner lift is
   concentrated in docs/config/Java source/test paths, while ctxhelm churns
   docs/planning/Rust source, ReAgent loses planning/scripts, and VeriSchema
   loses Python source. Broad path-family routing is also rejected. Phase 273
   then raises the local-fastembed prefilter cap to `256` for VeriSchema and
   shows no metric movement, so the Python-source gap is not simply prefilter
   starvation. Phase 274 tests path-derived Python module/package facets and
   rejects them after semantic-corroborated recall falls to `0.32294118`.
   Phase 275 adds query/path shape diagnostics and rejects a temporary
   shape-routed insertion route because it loses RefactoringMiner and ReAgent
   target tests. Phase 276 adds displacement diagnostics and shows the concrete
   budget-pressure mechanism: non-target docs/planning/scripts/paper files can
   enter top-K while default target tests/source files are lost. Phase 277 tests
   a path-family budget constraint and rejects promotion because ctxhelm and
   VeriSchema still regress. Phase 278 tests a leave-one-out source-free
   learned profile and produces the first post-Phase-270 semantic reranker
   proof with four-repo no-regress behavior: RefactoringMiner improves while
   ctxhelm, ReAgent, and VeriSchema remain exactly neutral. Phase 280 closes the
   immediate artifact-shape gap by exporting `learnedSemanticPolicy` with
   deterministic profile rows, staleness status, holdout status, and
   `minimumSupportCommitCount = 2`. This is promising R&D evidence, not
   runtime/default completion. Phase 281 then applies that repeated-support
   policy in a leave-one-out holdout and finds no regressions but zero applied
   held-out commits across all four repos. Phase 282 widens the same gate to
   `limit 40` and still sees zero held-out applications, so same-gate
   broadening is not enough. Phase 283 adds stable `--base`/`--head` gate
   ranges and shows older/recent slices still do not repeat the learned-policy
   signal across time. Phase 284 implements the true
   train-on-range/apply-on-disjoint evaluator and finds no eligible-profile
   test overlap or applications in the four-repo recent-to-older proof. Phase
   285 broadens those same ranges and still finds no eligible-profile overlap
   or applications, even on VeriSchema's deeper `38`/`39` split. Phase 286
   tests path-family backoff and rejects it because the coarser profiles expose
   semantic non-targets or missing target hits instead of safe applications.
   Phase 287 rejects generic source-role query hints, and Phase 288 rejects the
   current cross-repo learned-policy aggregation because repeated strict and
   backoff keys both aggregate to zero eligible profiles. Phase 289 improves
   VeriSchema semantic candidate quality with bounded candidate-path hints but
   still does not improve recall or remove the semantic-corroborated
   regression. Phase 290 removes that regression with a conservative tail-slot
   selector but adds no target hits. Phase 291 rejects same-directory and
   mirrored-test sibling path aliases because candidate misses worsen and
   selected semantic targets fall. Phase 292 rejects generic software-domain
   concept terms because target candidates and selected targets fall while
   semantic noise rises. Phase 293 fixes the explicit Jina code-model contract
   and improves VeriSchema candidate quality, but still shows no recall lift,
   no safe semantic-corroborated fusion, and a `6.01x` runtime ratio. Phase 294
   then tests whether semantic can help as bounded next-read guidance after the
   full default top K is protected; it appends only one non-target, so that path
   is also held as diagnostic-only. Phase 295 tests the documented MiniLM12
   variants and rejects them because both are noisier than corrected Jina.
   Phase 296 then shows the remaining corrected-Jina candidate miss is a
   semantic-generated `schema_agent/core/state.py` candidate with
   `dependency_co_change` support. Phase 297 proves a gold-backed protected
   tail-slot oracle can recover that target without churn. Phase 298 replaces
   the oracle dependency with a source-free supported-shape predictor on the
   targeted slice. Phase 299 broadens that predictor across eight stable
   slices and finds zero churn but only one lifted commit. Phase 300 records the
   broader supported-profile base rate directly and shows 787 profiles with
   only 22 targets, 90 thin per-slice cells, and no repeated source shape.
   Phase 301 adds retained/dropped supported-candidate cells and finds 22
   dropped targets but also 232 retained non-targets. Phase 302 evaluates the
   strict held-out separator over those retention cells and finds zero eligible
   train families across four repos, despite 53 train/test family overlaps.
   Phase 303 then checks the hidden margin distribution and finds only 8
   positive-margin families out of 117, with relaxed thresholds recovering
   targets only by inserting many more non-targets. The next semantic work
   should treat corrected Jina as a diagnostic backend and pivot away from
   retention-separator relaxation toward fresh agent outcome proof or a
   materially different source-free feature family, not more single-query
   expansion, MiniLM model swaps, next-read promotion, broad document
   expansion, supported-shape promotion, same-proof relaxation, or default model
   promotion.
2. Claude has current workflow proof but not a fresh paired outcome suite on the
   same level as Codex Phase 245/250/311. Phase 312 confirms this is still a
   client-availability problem: Claude Code can answer a tiny manual Opus probe,
   but the source-free availability check and full paired suite still emit
   rate-limit/client-failure flags, while Codex remains available. Reports
   should keep rate limits separate from retrieval quality.
3. Memory now has credible local, single-repo Codex-consumption, three-repo
   Codex target-consumption evidence, and three-repo Codex read-efficiency
   evidence. Phase 255 closes the current measured memory-efficiency gap for
   the same three-repo slice. Future memory R&D should treat this as the
   regression guard and broaden pair counts only when looking for new
   counterexamples.
4. GraphRAG is measured and improved, but still has budget-allocation questions
   for large source clusters. The current system avoids adding more graph edges
   blindly; future work should target edge-family ranking where ablations show
   exclusive target lift.
5. Cursor proof remains auth-blocked locally. The wrapper behavior is correct,
   but it is not a successful current real-client proof until a logged-in Cursor
   Agent environment is available.
6. Task-mentioned commit anchoring is implemented for prepare-task and covered
   by a focused local Git fixture. It should be treated as a narrow planner
   improvement, not as broad semantic/RAG completion evidence.

## Next Correct Order

1. Keep the current Phase 250 artifact promotion and governor proof as the
   active regression guard.
2. Preserve the Phase 255 memory target-consumption plus read-efficiency
   contract while broadening only if new counterexamples appear.
3. Refresh Claude paired outcome evidence when the client is available.
4. Move semantic R&D from routing existing clean-looking support profiles to a
   split plan: fusion/budget experiments where Phase 269 proves semantic
   candidate targets exist, and query/model/document coverage experiments where
   Phase 269 proves semantic never generated the missed target candidates.
   Phase 270 rejects the first global fusion rule, Phase 271 rejects the
   query-family-only route, Phase 272 rejects broad path-family routing, Phase
   275 rejects simple query/path-shape insertion, and Phase 277 rejects
   path-family budget allocation as a general rule. Phase 276 identifies the
   displacement families that any next experiment must guard against. Phase 278
   shows a stricter leave-one-out learned profile can preserve the default on
   three repos while lifting RefactoringMiner, and Phase 280 gives that path a
   durable source-free policy artifact with staleness and support-threshold
   metadata. Phase 281 tests that artifact in a held-out setting and shows the
   current slice is too sparse for repeated-support application. Phase 282
   rules out the simplest same-gate widening fix at `limit 40`. Phase 283 adds
   ranged semantic gates and shows older/recent slices still have zero held-out
   policy applications. Phase 284 then adds the true train/test evaluator and
   shows the current four-repo slices still have zero external applications:
   the repeated profiles are either unsafe or do not overlap the test candidate
   profiles. Phase 285 rules out one more same-range doubling because broader
   `40`/`40` limits still produce zero eligible overlap and zero applications.
   Phase 286 rules out the simplest path-family backoff because the coarser
   aggregates remain unsafe or target-empty. Phase 287 rules out generic
   source-role query hints on the targeted VeriSchema slice because they remove
   semantic-only target hits while preserving the same semantic-corroborated
   regression. Phase 288 rules out the current cross-repo learned-policy
   aggregation because strict and backoff profiles both produce zero eligible
   cross-repo keys. Phase 289 shows candidate-path hints improve candidate
   quality but not final recall or regression status. Phase 290 proves a
   conservative tail-slot selector can remove the regression but not create
   lift. Phase 291 rejects candidate-sibling path aliases because they worsen
   candidate misses and selected semantic target hits. Phase 292 rejects generic
   path-concept terms because they worsen target candidates, selected targets,
   semantic-only noise, and tail-slot safety. Phase 294 rejects the immediate
   semantic-next-read hypothesis because preserving default top-K leaves only a
   non-target appended path. Phase 295 rejects the documented MiniLM12 model
   branch after both variants increase candidate misses and semantic noise.
   Phase 296 shows the remaining corrected-Jina candidate miss is a supported
   `schema_agent/core/state.py` fusion/top-K ordering gap. Phase 297 confirms a
   gold-backed tail-slot oracle can recover that target without churn. Phase 298
   confirms the same target can be recovered with a source-free
   `symbol_identifier` / `python_source` / `dependency_co_change` supported
   shape predictor on the targeted slice. Phase 299 broadens the predictor and
   finds it safe but sparse: one lift, no regressions, no default-only target
   churn across eight slices. Phase 300 turns that branch into a base-rate
   diagnostic and shows supported profiles have only `0.027954` target
   precision across the same slices. Phase 301 adds the candidate-retention
   view and exposes 12 recoverable dropped-target family rows. Phase 302 tests
   the strict train/test separator over those rows and finds zero eligible
   train families, zero eligible test overlap, and zero applications. Phase 303
   shows this is not just a threshold artifact: positive-margin families are
   only `0.068376` of the train surface, and relaxed recovery rows insert far
   more non-targets than targets. The next semantic step should be outcome
   proof or a new feature family, not retention-separator relaxation or runtime
   promotion.
   Increasing the local-fastembed document cap, adding broad Python path metadata, adding
   generic source-role query words, adding another hand-written route, merely
   widening or splitting the same gate, relaxing profile keys, current
   cross-repo aggregation, candidate-path hints as a runtime/default policy, and
   candidate-sibling path aliases, generic path-concept query terms, semantic
   next-read promotion from the current proof, MiniLM12 as a simple model swap,
   document expansion before supported fusion is resolved, and Jina as the
   default local-fastembed model are already rejected paths. Corrected Jina
   remains useful diagnostic evidence for candidate quality and supported
   fusion-gap diagnosis.

## Conclusion

ctxhelm is past MVP and has real agent-outcome proof for multiple failure
classes. It is not yet R&D complete in the world-class sense because semantic
default lift and fresh paired Claude outcome proof remain open research
questions. Phase 264 reduces semantic risk by exposing where semantic is a
candidate versus a noise source, Phase 265 further reduces risk by proving the
coarse candidate families are not stable enough for runtime routing, and Phase
266 rejects generic corroboration as a sufficient separator. Phase 267 narrows
the remaining semantic path to eval-only support-profile routing, Phase 268
rejects that route as safe but neutral, Phase 269 shows the next semantic
work must split fusion/budget work from candidate-generation work by repo, and
Phase 270 rejects a global semantic-corroborated fusion rule after measured
cross-repo regressions. Phase 271 rejects query-family-only routing for that
rule, Phase 272 rejects broad path-family routing, Phase 275 rejects simple
query/path-shape insertion after it loses default target tests, Phase 276 shows
the displacement pressure behind those losses, Phase 277 rejects the path-family
budget rule as a general solution, Phase 278 shows a more credible
learned-profile direction with no regressions across the four measured repos,
and Phase 280 turns that direction into a source-free learned-policy artifact
with support, staleness, and holdout status. Phase 281 then shows the exported
policy has insufficient held-out support on the current four-repo slice:
`appliedCommitCount = 0` everywhere with no regressions. Phase 282 confirms the
same result at `limit 40`. Phase 283 adds stable ranged gates and finds no
older/recent slice repetition beyond the same recent RefactoringMiner row, so
Phase 284 adds the train-on-range/apply-on-disjoint-test-range evaluator and
finds the same negative result outside the training snapshot, with
support/overlap diagnostics showing no eligible-profile test overlap. Phase 285
reruns broader `40`/`40` limits on the same stable ranges and remains empty, so
Phase 286 tests and rejects path-family backoff as the simplest key relaxation.
Phase 287 then tests and rejects generic source-role semantic query hints on
the targeted VeriSchema older-range slice. Phase 288 tests and rejects the
current source-free cross-repo learned-policy aggregation because neither
strict query/path profiles nor path-family backoff profiles have an eligible
cross-repo zero-harm key. Phase 289 tests bounded candidate-path semantic query
hints; it improves semantic candidate quality but not recall or the
semantic-corroborated regression. Phase 290 tests conservative tail-slot
selection; it removes the regression but adds no target lift. Phase 291 tests
candidate-sibling path aliases; candidate misses worsen and selected semantic
targets fall, so more path aliases are rejected. Phase 292 tests generic
path-concept terms; target candidates and selected targets fall, semantic noise
rises, and tail-slot safety regresses. Phase 293 fixes the explicit Jina code
model dimension/text contract and improves VeriSchema candidate quality, but
does not improve recall, does not make semantic-corroborated fusion safe, and
remains too slow for default promotion. Phase 294 adds a protected-top-K
semantic next-read diagnostic and rejects promotion after the only appended
path is a non-target. Phase 295 rejects documented MiniLM12 model swaps after
both are noisier than corrected Jina. Phase 296 adds candidate-missed support
profiles and shows the remaining Jina candidate miss is
`schema_agent/core/state.py` with `dependency_co_change` support. Phase 297
adds a gold-backed supported-candidate tail-slot oracle that recovers
that target without churn. Phase 298 replaces the oracle dependency with a
source-free supported-shape predictor on the targeted slice. Phase 299 shows
that predictor is safe but too sparse across eight stable slices. Phase 300
turns the supported-profile base rate into a durable source-free summary and
confirms the surface is mostly non-target singleton evidence. Phase 301 records
retained/dropped candidate cells and shows there is separability signal to
study, but not enough for a default policy. Phase 302 runs the strict held-out
retention separator and finds zero eligible train families across four repos.
Phase 303 confirms the retention proxy is structurally weak rather than merely
miscalibrated: only 8 of 117 families have positive margin, and relaxed rows
recover targets only with larger non-target insertion. The remaining semantic
R&D path should move to fresh agent outcome proof or a different source-free
feature family, not more same-range doubling, not coarse profile-key relaxation
inside the same proof, not generic role words, not current cross-repo
aggregation, not candidate-path hints, candidate-sibling hints, generic
concept-term hints, tail-slot reranking, semantic next-read promotion, MiniLM12
model swaps, supported-shape promotion, or Jina as runtime/default policy, and
not runtime/default promotion.
Phase 253 reduces memory risk by
proving cross-repo Codex target-consumption lift. Phase 255 further reduces
memory risk by preserving that target consumption while improving irrelevant
reads in all three measured memory pairs.
Phase 304 refreshes Claude outcome evidence and keeps the gap correctly scoped
to client rate limiting. Phase 305 then refreshes current-head Codex outcome
evidence and finds the memory-efficiency prompt had become too aggressive for
multi-target R&D tasks. Phase 307/309 fix that memory-lane under-read while
preserving real Codex lift, but they also show prompt-only compliance is still
stochastic in non-memory plan/brief/standard lanes. The remaining agent-outcome
R&D should either add stronger consumption enforcement/retry logic or keep this
as residual guidance work; it should not weaken the source-free reporting
boundary or hide evidence-only targets.
Phase 310/311 add that stronger source-free retry layer. Eligible
ctxhelm-assisted lanes with evidence-only targets get one bounded
target-consumption retry, and the selected report records source-free retry
metadata. The targeted governor proof and the full four-task Codex R&D breadth
suite both clear evidence misses, evidence-only targets, under-read targets,
forbidden commands, client failures, and rate limits. This closes the measured
Codex evidence-only consumption gap, while leaving Claude availability and
semantic default-lift questions separate.
Phase 312 retests the Claude side after that Codex closure and keeps the
cross-agent gap scoped correctly: a manual tiny Opus prompt is not enough to
make the paired suite comparable, and the source-free Claude run still reports
`insufficient_comparable_lanes` because every lane records rate-limit/client
failure evidence.
