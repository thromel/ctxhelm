---
gsd_state_version: 1.0
milestone: v2.5
milestone_name: Production Retrieval Quality
status: active
last_updated: "2026-06-02T07:35:00Z"
last_activity: 2026-06-02 -- Phase 180 added conservative source-free graph edge ablations to historical eval, keeping the clean four-repo proof promoted while measuring edge-family top-10 lift for exclusive imports and Python re-export evidence
progress:
  total_phases: 94
  completed_phases: 94
  total_plans: 7
  completed_plans: 7
  percent: 100
---

# Project State

## Current Position

Phase: 180 - Graph Edge Ablations
Plan: 180-graph-edge-ablations
Status: Complete
Last activity: 2026-06-02 -- Phase 180 makes graph edge-family lift measurable through source-free `graphEdgeAblations` in historical eval reports. The clean four-repo proof at `/tmp/ctxhelm-rd/phase180-graph-edge-ablation-proof.json` promotes with average File Recall@10 `0.61190045` vs lexical `0.45709258`; disabling exclusive VeriSchema `imports` removes `2` selected paths and `1` target hit for Recall@K delta `-0.00512819`, while `python_reexport` has no exclusive top-10 lift on this fixture.

## Project Reference

See: `.planning/PROJECT.md`

**Core value:** Given a coding task, ctxhelm should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

**Current focus:** v2.5 Production Retrieval Quality.

## Active Milestone

v2.5 Production Retrieval Quality

Goal: Prove and improve retrieval quality across real repositories so production local embeddings, reranking, graph/test/history fixes, and learned fusion can beat lexical baseline while staying local-first and source-safe.

Planned phases:

- Phase 61: Multi-Repo Quality Baselines (complete)
- Phase 62: Production Local Embedding Quality (complete)
- Phase 63: Reranker And Fusion Promotion (complete)
- Phase 64: Gap-Family Retrieval Improvements (complete)
- Phase 65: v2.5 Product Proof And Release Gate (complete)
- Phase 66: Test Recall Evaluation Channel (complete follow-up)
- Phase 67: Retrievable Target Eval Denominator (complete follow-up)
- Phase 69: Channel-Aware Product Proof Gate (complete follow-up)
- Phase 70: Real-Client MCP Proof Refresh (complete follow-up)
- Phase 71: Archive Artifact Dampening (complete follow-up)
- Phase 72: Broader Repeated-Lift Validation (complete follow-up)
- Phase 73: Broader Fixed-Corpus Fixture (complete follow-up)
- Phase 74: Protected Evidence Diagnostics (complete follow-up)
- Phase 75: Parent-Bounded History And Test Reserve (complete follow-up)
- Phase 76: Parent-Bounded Validation History (complete follow-up)
- Phase 77: Validation Command Coverage (complete follow-up)
- Phase 78: Ceiling-Aware Broader Gate (complete follow-up)
- Phase 79: Protected Target Floors (complete follow-up)
- Phase 80: Unique Symbol Floor Accounting (complete follow-up)
- Phase 81: Warm Cache Latency Proof (complete follow-up)
- Phase 82: Warm Cache Release Gate (complete follow-up)
- Phase 83: Context Divergence Accounting (complete follow-up)
- Phase 84: Broad Scope Dependency Floors (complete follow-up)
- Phase 85: Broad Context Areas (complete follow-up)
- Phase 86: Python Package Re-Export Graph Coverage (complete follow-up)
- Phase 87: Validation Gap Accounting (complete follow-up)
- Phase 88: Broad Source-Area Candidates (complete follow-up)
- Phase 89: Fast Inventory Freshness (complete follow-up)
- Phase 90: Packaged Release Gate (complete follow-up)
- Phase 91: Broad Context-Area Eval (complete follow-up)
- Phase 92: Area-Aware Gap Taxonomy And Large-Repo Warm Proof (complete follow-up)
- Phase 93: Source-Free Index Cache (complete follow-up)
- Phase 94: Broad Context-Area Cap (complete follow-up)
- Phase 95: Progressive Area Pack Guidance (complete follow-up)
- Phase 96: Context Area MCP Resources (complete follow-up)
- Phase 97: Broad Governance Classification (complete follow-up)
- Phase 98: Progressive Broad Classification (complete follow-up)
- Phase 99: Context Area Read Batches (complete follow-up)
- Phase 100: Resource-Backed Gap Summaries (complete follow-up)
- Phase 101: Release-Gated Gap Summary Contract (complete follow-up)
- Phase 102: Explicit-Repo MCP Resource Consumption (complete follow-up)
- Phase 103: Broad Fixed-Corpus Floors (complete follow-up)
- Phase 104: Context Area Next-Read Paths (complete follow-up)
- Phase 105: History-Unavailable Embedded Reports (complete follow-up)
- Phase 106: Real-Client Request Evidence Hardening (complete follow-up)
- Phase 107: Hydrated Four-Repo Proof Path (complete follow-up)
- Phase 108: Cold Git Bounds (complete follow-up)
- Phase 109: Environment Health Verdicts (complete follow-up)
- Phase 110: Clean Cold Fixture Proof (complete follow-up)
- Phase 111: Clean Fixture Release Gate (complete follow-up)
- Phase 112: Clean Release Gate With Required Fixture Proof (complete follow-up)
- Phase 113: Release Candidate Status (complete follow-up)
- Phase 114: Public Archive Release (complete follow-up)
- Phase 115: Public Archive Install Verification (complete follow-up)
- Phase 116: Public Archive Real-Client Smoke (complete follow-up)
- Phase 117: Context Area Role Signals (complete follow-up)
- Phase 118: Context Area Resource Scope (complete follow-up)
- Phase 119: Index Test Environment Lock (complete follow-up)
- Phase 120: Public CI Release Gate (complete follow-up)
- Phase 121: CI Node 24 Runtime Guard (complete follow-up)
- Phase 122: Public Real-Client Protocol Compatibility (complete follow-up)
- Phase 123: Context Area Coverage Profile (complete follow-up)
- Phase 124: Context Area Inspection Strategy (complete follow-up)
- Phase 125: Lexical Comparison Proof Boundary (complete follow-up)
- Phase 126: Agent Evidence Lexical Comparison (complete follow-up)
- Phase 127: Narrow Validation-Test Reserve (complete follow-up)
- Phase 128: Broad Operational Floors (complete follow-up)
- Phase 129: Public Release Freshness Proof (complete follow-up)
- Phase 130: Public v1.1.1 Release Currentness (complete follow-up)
- Phase 131: Product-Aware Freshness Release (complete follow-up)
- Phase 132: Claude Workflow Eval (complete follow-up)
- Phase 133: Product README Positioning (complete follow-up)
- Phase 134: Public v1.1.3 Release Currentness (complete follow-up)
- Phase 135: Distribution Readiness (complete follow-up)
- Phase 136: Public v1.1.4 Release Currentness (complete follow-up)
- Phase 137: Public Homebrew Tap (complete follow-up)
- Phase 138: Public v1.1.5 Release Currentness (complete follow-up)
- Phase 139: ctxhelm Brand Identity (complete follow-up)
- Phase 140: Public v1.1.6 Release Currentness (complete follow-up)
- Phase 141: ctxhelm Brand Identity (complete follow-up)
- Phase 142: Public v1.1.7 Release Currentness (complete follow-up)
- Phase 143: Paired Agent-Run Outcome Harness (complete follow-up)
- Phase 144: Public v1.1.8 Post-Rename Currentness (complete follow-up)
- Phase 145: Public v1.1.9 Hardening Currentness (complete follow-up)
- Phase 146: Crates Publish-Order Readiness (complete follow-up)
- Phase 147: Public v1.1.10 Source Distribution Currentness (complete follow-up)
- Phase 148: Codex Real-Client Diagnostic Evidence (complete follow-up)
- Phase 149: Public v1.1.11 Currentness (complete follow-up)
- Phase 150: Multi-Platform Artifact Workflow (complete follow-up)
- Phase 151: Public v1.1.12 Multi-Platform Currentness (complete follow-up)
- Phase 152: Native-Agent Outcome Suite (complete follow-up)
- Phase 153: BM25 Symbol Lexical Index (complete follow-up)
- Phase 154: BM25 Legacy Comparison Report (complete follow-up)
- Phase 155: BM25 Corpus Comparison Report (complete follow-up)
- Phase 156: Lexical Backend Product Proof Integration (complete follow-up)
- Phase 157: Benchmark Corpus Health Guard (complete follow-up)
- Phase 158: BM25 Exact-Saturated Fast Path (complete follow-up)
- Phase 159: Lexical Runtime Accounting And Exact-Primary Policy (complete follow-up)
- Phase 160: Bounded Semantic Status And Search (complete follow-up)
- Phase 161: Semantic Gate Contribution Diagnostics (complete follow-up)
- Phase 174: Source Recall Release Proof Contract (complete follow-up)
- Phase 175: Semantic Missed Target Gap Families (complete follow-up)
- Phase 162: Feature-Enabled Local Fastembed Gate Proof (complete follow-up)
- Phase 163: Persisted Semantic Vector Reuse (complete follow-up)
- Phase 164: Global Semantic Vector Candidates And Write-Through (complete follow-up)
- Phase 165: Fastembed Default And Loud Index Errors (complete follow-up)
- Phase 166: Semantic Query Vector Reuse (complete follow-up)
- Phase 167: Pruned Generated Inventory Walk (complete follow-up)
- Phase 168: Semantic Alias And Noise Diagnostics (complete follow-up)
- Phase 169: Graph Ordering And Context Balance (complete follow-up)
- Phase 170: Auxiliary Source Priority (complete follow-up)
- Phase 171: Governance Doc Priority (complete follow-up)
- Phase 172: Benchmarking Governance Doc Priority (complete follow-up)
- Phase 173: Source Recall Promotion Guard (complete follow-up)

## Last Completed Milestone

v2.4 Production Semantic & Precision Backends

Goal: Convert semantic and precision retrieval from local scaffolding into measured, policy-gated retrieval-quality improvements without breaking ctxhelm's local-first and source-safe contract.

Planned phases:

- Phase 56: Production Local Semantic Backend (complete)
- Phase 57: Precision-Enriched Semantic Documents (complete)
- Phase 58: Query Construction And Hybrid Fusion Controls (complete)
- Phase 59: Provider And Reranker Policy Gates (complete)
- Phase 60: Semantic/Precision Evaluation Gates And Release Proof (complete)

## Next Step

Continue production-readiness work from remaining measured gaps: make Codex CLI real-client proof required only after the client can produce machine-checkable tool-call evidence, add crates.io only when ordered publication is ready, run research-backed world-class retrieval and integration gap planning, and continue hardening broad source-free ranking only when proof shows a concrete gap. Broad evals now distinguish files covered only by surfaced context areas from files with no candidate signal, clean warm full-fixture proof promotes across RefactoringMiner, ctxhelm, ReAgent, and VeriSchema, and product proof now explicitly reports `releaseGate.lexicalComparison`: current all-file claim `mixed` with beat `3`, raw match `0`, raw trail `1`, explained trail `1`, unexplained trail `0`, current agent-evidence claim `mixed` with zero trailing corpora, current context-channel claim `mixed`, all-file average delta `+0.15480787`, agent-evidence average delta `+0.2570628`, context average delta `+0.30652046`, and protected target miss-rate `0.0` on all four corpora. Historical eval now reports `graphEdgeProfiles` and `graphEdgeAblations`, showing both edge-family candidate pressure and conservative exclusive-edge top-10 lift for imports, Python re-exports, and future precision edges. Phase 180 proves imports produce some unique VeriSchema target lift while Python re-exports currently add no exclusive top-10 target lift on the fixed fixture, so the next GraphRAG work should focus on edge-family budget allocation rather than raw edge count. The full packaged release gate passes from a clean checkout with the clean fixture proof required, release-candidate metadata marks the local archive channel, tag-published multi-platform archive assets, and Apple Silicon Homebrew tap ready while deferring crates.io/signed installers/self-update, Phase 132 adds a committed source-free Claude workflow report proving real Claude Code explicit-repo `prepare_task` and `get_pack` calls through MCP, Phase 133 makes the public README state the product wedge and current proof snapshot while release-gating those claims against stale client-version drift, Phase 140 published the `v1.1.6` public archive for the superseded brand, Phase 141 finalizes ctxhelm as the active product name across CLI, package, MCP, and install surfaces, Phase 142 published and verified the current public `v1.1.7` archive plus refreshed Homebrew tap, Phase 143 adds paired Claude Code process proof showing `ctxhelm-brief` preserved target coverage while reducing irrelevant reads from 5 to 2, Phase 144 publishes/verifies the post-rename `v1.1.8` public release plus Homebrew tap and public-archive Claude Code proof, Phase 145 publishes/verifies `v1.1.9` so the public artifact includes the Git-history timeout hardening, Phase 146 makes crates source distribution publish-order ready by adding internal dependency versions, checking every crate package list, and documenting that dependent crate dry-runs remain blocked until internal crates are published in order, Phase 147 publishes/verifies `v1.1.10` so the public archive and Homebrew tap include that source-distribution hardening, Phase 148 makes Codex optional skips diagnostic instead of opaque by recording `stream_disconnected`, exit status, stderr hash/line count, and MCP method counts without raw stderr or raw MCP traffic, Phase 149 publishes/verifies `v1.1.11` so the public archive and Homebrew tap include the Phase 148 diagnostics hardening with fresh Claude Code and Codex evidence, Phase 150 adds explicit-target packaging plus a three-target release-artifact workflow with refreshed Claude Code workflow proof, and Phase 151 prepares `v1.1.12` so version-tag pushes publish verified Linux x64, macOS Intel, and macOS Apple Silicon release assets. Broad context-area guidance covers more wide-task implementation areas, packs tell agents concrete source/docs paths to inspect next, plan-level context areas expose source-free role signals, MCP context-area resources explicitly label their counts/read batches as safe-inventory scope and now expose source-free coverage profiles plus inspection strategies with progressive read order, path budget, and stop rules, ctxhelm-index release-validation tests now share one process-environment lock for `CTXHELM_HOME`, public CI now enforces formatting, clippy, locked tests, CLI help, release-doc consistency, and release-gate smoke on pushes/PRs using Node 24 action majors without Node 20 warning text, gap summaries point to area resources and bounded next-read paths, the product-proof checker gates those gap summaries and fails cleanly on missing embedded repo reports, history-unavailable repos now produce embedded insufficient-evidence reports instead of `null` reports, repo-scoped resources work after explicit-repo MCP tools even from a wrong server cwd, broad fixed-corpus metric floors block silent selection regressions, governance/proof tasks classify broad historical/eval language reliably, archive/docs broad tasks now receive context-area guidance without perturbing target-file source floors, broad operational tasks reserve governance/config/workflow evidence before lower-priority expansion, real-client smoke artifacts now carry source-free request metadata instead of boolean-only claims, and the full four-repo proof now hydrates all configured repositories instead of hanging or returning missing reports.

## Operator Next Steps

- Default local retrieval now passes the fixed two-repo product proof under the channel-aware release gate.
- The gate compares non-test context recall against lexical and checks validation-test recall separately.
- Latest required local proof: `.ctxhelm/e2e/phase77-validation-command-coverage-proof.json` with `releaseGate.decision = promote`.
- Latest broader probe: `.ctxhelm/e2e/phase78-ceiling-aware-broader-proof.json` with `releaseGate.decision = promote`; RefactoringMiner is accepted as a safe lexical-ceiling `match`, ctxhelm/ReAgent/VeriSchema are `beat`, and VeriSchema keeps Effective Validation Recall@10 `1.0`.
- Reproducible broader fixture: `.planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json`; latest Phase 78 run reports `releaseGate.decision = promote`.
- Latest protected-evidence diagnostic proof: `.ctxhelm/e2e/phase74-protected-evidence-diagnostics-proof.json` with target miss-rate separated from overall protected pressure.
- Latest broader fixed-corpus diagnostic proof: `.ctxhelm/e2e/phase74-broader-protected-evidence-diagnostics-proof.json`; RefactoringMiner and ReAgent have zero protected retrieval-target misses on the pinned probe, while ctxhelm and VeriSchema still have target misses.
- Latest parent-bounded validation-history proof: `.ctxhelm/e2e/phase76-parent-bounded-validation-history-proof.json`; required proof promotes with RefactoringMiner context Recall@10 `0.778` vs lexical `0.741`, ctxhelm context Recall@10 `0.444` vs lexical `0.361`, and Test Recall@10 `1.0` on both corpora.
- Latest broader parent-bounded validation-history proof: `.ctxhelm/e2e/phase76-broader-parent-bounded-validation-history-proof.json`; broader promotion still blocks, but VeriSchema Test Recall@10 improved from `0.661` to `0.709`.
- Latest validation-command coverage proof: `.ctxhelm/e2e/phase77-validation-command-coverage-proof.json`; required proof promotes with RefactoringMiner effective validation recall `1.0`.
- Latest broader validation-command coverage proof: `.ctxhelm/e2e/phase77-broader-validation-command-coverage-proof.json`; VeriSchema effective validation recall is `1.0`, resolving the previous validation-test floor failure through broad command coverage.
- Latest ceiling-aware broader proof: `.ctxhelm/e2e/phase78-ceiling-aware-broader-proof.json`; broader fixed-corpus promotion passes, but ctxhelm and VeriSchema still report non-zero protected retrieval-target misses.
- Latest protected-target floor proof: `.ctxhelm/e2e/phase79-protected-target-floors-proof.json`; required proof promotes, RefactoringMiner protected target miss-rate is `0.0`, and ctxhelm required protected target miss-rate is `0.0417`.
- Latest broader protected-target floor proof: `.ctxhelm/e2e/phase79-broader-protected-target-floors-proof.json`; broader proof promotes, VeriSchema protected target miss-rate is `0.0`, and ctxhelm broader protected target miss-rate remains `0.100`.
- Latest unique-symbol floor proof: `.ctxhelm/e2e/phase80-unique-symbol-floor-proof.json`; required proof promotes and required protected target miss-rates are `0.0` across measured corpora.
- Latest broader unique-symbol floor proof: `.ctxhelm/e2e/phase80-broader-unique-symbol-floor-proof.json`; broader proof promotes and protected target miss-rates are `0.0` across measured corpora.
- Latest warm-cache latency proof config: `.planning/e2e/2026-05-30-phase81-warm-cache-proof-config.json`.
- Latest warm-cache cold proof: `.ctxhelm/e2e/phase81-warm-cache-cold-proof.json`; cold proof promotes and populates source-free eval caches.
- Latest warm-cache warm proof: `.ctxhelm/e2e/phase81-warm-cache-warm-proof.json`; warm proof promotes with cache hits on all four corpora and `1ms` reported runtime for each cached repo on this run.
- Latest warm-cache release-gate proof: `.planning/e2e/2026-05-30-phase82-warm-cache-gate.md`, `.ctxhelm/e2e/phase82-warm-cache-gate-cold-proof.json`, and `.ctxhelm/e2e/phase82-warm-cache-gate-warm-proof.json`; warm-cache regressions now block product-proof promotion.
- Latest context-divergence accounting proof: `.planning/e2e/2026-05-30-phase83-context-divergence-accounting.md` and `.ctxhelm/e2e/phase83-context-divergence-proof.json`; product-proof verdicts now expose context-vs-all-file deltas and require all-file lexical deficits to be explained by non-regressed context plus validation channels.
- Latest broad-scope dependency proof: `.planning/e2e/2026-05-31-phase84-broad-scope-dependency-floors.md` and `.ctxhelm/e2e/phase84-broad-scope-dependency-proof.json`; broad multi-area tasks are now counted separately and get a bounded dependency source floor.
- Latest broad context-area proof: `.planning/e2e/2026-05-31-phase85-broad-context-areas.md` and `.ctxhelm/e2e/phase85-context-areas-warm-proof.json`; broad multi-area prepare-task plans and packs now include source-free `contextAreas` that do not perturb target-file recall.
- Latest Python package re-export graph proof: `.planning/e2e/2026-05-31-phase86-python-package-reexports.md`; focused dependency tests pass and broad proof metrics stay flat/non-regressing, proving the remaining VeriSchema gap is selection/budget pressure rather than only missing Python package edges.
- Latest validation gap accounting proof: `.planning/e2e/2026-05-31-phase87-validation-gap-accounting.md` and `.ctxhelm/e2e/phase87-validation-gap-accounting-proof.json`; RefactoringMiner validation-command recall improves from `0.0` to `1.0`, validation-covered tests no longer appear as unresolved test-mapping gap summaries, and the rejected broad source-area diversity experiment is documented.
- Latest broad source-area candidate proof: `.planning/e2e/2026-05-31-phase88-broad-source-area-candidates.md` and `.ctxhelm/e2e/phase88-broad-source-area-candidates-proof.json`; VeriSchema File Recall@10 improves from `0.17936651` to `0.18449473` and Source Recall@10 improves from `0.30409357` to `0.31067252` without validation or protected-target regression.
- Latest fast inventory freshness proof: `.planning/e2e/2026-05-31-phase89-fast-inventory-freshness.md`, `.ctxhelm/e2e/phase89-fast-inventory-freshness-proof.json`, and `.ctxhelm/e2e/phase89-fast-inventory-freshness-release-proof.json`; release-mode broader proof promotes while preserving Phase 88 quality metrics.
- Latest packaged release-gate proof: `.planning/e2e/2026-05-31-phase90-packaged-release-gate.md`; clean worktree release gate passed with broad benchmark proof enabled and optional Codex/Claude real-client tool-call evidence skipped.
- Latest broad context-area eval proof: `.planning/e2e/2026-05-31-phase91-broad-context-area-eval.md` and `.ctxhelm/e2e/phase91-broad-context-area-release-proof.json`; release-mode broader proof promotes, existing quality metrics stay stable, and VeriSchema broad context-area recall is now measured at `0.64708996`.
- Latest area-aware gap taxonomy proof: `.planning/e2e/2026-05-31-phase92-area-aware-gap-taxonomy.md`, `.ctxhelm/e2e/phase92-area-aware-gap-taxonomy-clean-force-proof.json`, and `.ctxhelm/e2e/phase92-area-aware-gap-taxonomy-warm-proof.json`; warm-cache broader proof promotes, VeriSchema broad context-area recall remains `0.64708996`, and unresolved broad-area misses are classified as `area_context_only` / `contextPlanning`.
- Latest source-free index cache proof: `.planning/e2e/2026-05-31-phase93-source-free-index-cache.md` and `.ctxhelm/e2e/phase93-index-cache-cold-proof.json`; force-refresh broad proof promotes, RefactoringMiner runtime is `4517ms`, and VeriSchema broad context-area recall remains `0.64708996`.
- Latest broad context-area cap proof: `.planning/e2e/2026-05-31-phase94-broad-context-area-cap.md` and `.ctxhelm/e2e/phase94-context-area-cap-proof.json`; force-refresh broad proof promotes, VeriSchema broad context-area recall improves to `0.71851856`, and target-file/test/validation metrics stay stable.
- Latest progressive area pack proof: `.planning/e2e/2026-05-31-phase95-progressive-area-pack-guidance.md` and `.ctxhelm/e2e/phase95-progressive-area-pack-proof.json`; force-refresh broad proof promotes, packs now identify zero-selected areas for next native reads, and VeriSchema broad context-area recall remains `0.71851856`.
- Latest context area MCP resource proof: `.planning/e2e/2026-05-31-phase96-context-area-resources.md` and `.ctxhelm/e2e/phase96-context-area-resources-proof.json`; force-refresh broad proof promotes, `contextAreas` now carry source-free `resourceUri` values, and MCP serves `ctxhelm://repo/context-areas` plus `ctxhelm://repo/context-area/{encoded-area}` without adding tools.
- Latest broad governance classification proof: `.planning/e2e/2026-05-31-phase97-broad-governance-classification.md` and `.ctxhelm/e2e/phase97-broad-governance-classification-proof.json`; force-refresh broad proof promotes, ctxhelm File Recall@10 improves to `0.47460318`, Source Recall@10 improves to `0.7166667`, and broad context-area recall improves to `1.0`.
- Latest progressive broad classification proof: `.planning/e2e/2026-05-31-phase98-progressive-broad-classification.md` and `.ctxhelm/e2e/phase98-broader-broad-task-classification-proof.json`; force-refresh broad proof promotes, archive/docs retrieval tasks now get broad context-area guidance, and ctxhelm File Recall@10 `0.47460318` / Source Recall@10 `0.7166667` stay stable against Phase 97.
- Latest context-area read batch proof: `.planning/e2e/2026-05-31-phase99-context-area-read-batches.md` and `.ctxhelm/e2e/phase99-context-area-read-batches-proof.json`; force-refresh broad proof promotes, context-area resources now expose source-free `roleBuckets`, `pathFamilies`, and `nextReadBatches`, and Phase 98 retrieval metrics stay unchanged.
- Latest resource-backed gap summary proof: `.planning/e2e/2026-05-31-phase100-resource-backed-gap-summaries.md` and `.ctxhelm/e2e/phase100-resource-backed-gap-summaries-proof.json`; force-refresh broad proof promotes, retrieval gap summaries now include source-free `contextAreaResourceUri` and `nextReadPaths`, and Phase 99 retrieval metrics stay unchanged.
- Latest release-gated gap summary contract proof: `.planning/e2e/2026-05-31-phase101-release-gated-gap-summary-contract.md`; the product-proof checker now fails current reachable retrieval-gap summaries that are not resource-backed, and the Phase 100 four-repo proof passes the stricter release-gate check.
- Latest explicit-repo MCP resource proof: `.planning/e2e/2026-05-31-phase102-explicit-repo-mcp-resource-consumption.md`; deterministic MCP protocol smoke now reads context-area resources after an explicit-repo `prepare_task` from a non-repo server cwd, and Cursor setup smoke records deterministic context-area resource-read evidence.
- Latest broad fixed-corpus floor proof: `.planning/e2e/2026-05-31-phase103-broad-fixed-corpus-floors.md`; the product-proof checker now enforces pinned four-repo metric floors and rejects the broad dependency-priority ranking experiment that regressed VeriSchema File Recall@10 to `0.17936651`.
- Latest context-area next-read proof: `.planning/e2e/2026-05-31-phase104-context-area-next-read-paths.md` and `.ctxhelm/e2e/phase104-context-area-next-read-paths-no-refminer-proof.json`; broad context areas now include docs, `nextReadPaths`, and `unselectedCount`, packs render explicit `Next reads`, the three-repo proof promotes, and the four-repo proof is blocked only by the local RefactoringMiner `git rev-list` timeout/no embedded report.
- Latest history-unavailable report proof: `.planning/e2e/2026-05-31-phase105-history-unavailable-report.md` and `.ctxhelm/e2e/phase105-history-unavailable-proof.json`; a repo without git history now returns `report: {...}`, `evaluatedCommits: 0`, a source-free history-unavailable error, and an `insufficient_evidence` product-proof verdict instead of `report: null`.
- Latest real-client proof: `.planning/e2e/2026-05-31-phase106-real-client-request-evidence.md`; Codex and Claude smoke artifacts now include source-free request-log hashes, line counts, explicit repo tool-call counts, sanitized observed tool calls, and request-summary sidecars while preserving the Phase 70 real-client proof boundary.
- Latest hydrated four-repo proof: `.planning/e2e/2026-05-31-phase107-hydrated-four-repo-proof.md`, `.ctxhelm/e2e/phase107-hydrated-four-repo-cold-proof.json`, and `.ctxhelm/e2e/phase107-hydrated-four-repo-warm-proof.json`; the cold proof hydrates all four repositories and blocks only on ctxhelm runtime, while the warm proof promotes with four hydrated corpus verdicts.
- RefactoringMiner and ReAgent still trail lexical on all-file recall in the broader proof, but those deficits are now machine-checkable as explained by the context/validation split instead of only prose notes.
- Phase 84 improves VeriSchema Source Recall@10 from `0.249` to `0.304` on the broader fixed corpus while keeping RefactoringMiner, ctxhelm, and ReAgent stable.
- Phase 85 intentionally keeps broad fixed-corpus quality metrics flat while improving agent guidance through non-displacing context-area hints; cold proofs block on the existing RefactoringMiner runtime threshold, while the warm-cache proof promotes.
- Phase 86 adds Python package re-export graph candidates, but VeriSchema Recall@10 remains flat because those candidates do not enter the constrained top-10.
- Phase 87 fixes validation accounting rather than target ranking: Java class-selector commands now count as validation coverage, and validation-covered tests no longer inflate test-mapping gap summaries.
- Phase 88 improves broad VeriSchema source/file recall by adding bounded source-area candidates after graph/test seed selection. An earlier variant was rejected because it perturbed raw test recall.
- Phase 89 reduces repeated inventory freshness overhead. Debug broad proof now blocks only on the RefactoringMiner single-commit cold-start diagnostic, while release-mode broad proof promotes and passes `scripts/check-product-proof.py`.
- Phase 90 proves the packaged release path: the archive audit, clean extraction, extracted binary smokes, MCP protocol checks, Cursor/OpenCode setup checks, and packaged broad benchmark proof all passed from a clean worktree.
- Phase 91 adds the broad context-area eval channel. It does not alter top-10 target-file ranking, but it proves broad plans expose implementation area coverage when a commit is too wide for the initial file budget.
- Phase 92 fixes area-aware gap taxonomy and prevents stale historical eval caches from hiding new eval fields. It also reduces large-repo sampling/freshness overhead, but clean RefactoringMiner still exceeds the hard cold runtime ceiling without cached historical reports.
- Phase 93 adds source-free symbol/dependency index caches. Clean RefactoringMiner force-refresh proof now promotes under the hard runtime ceiling without release-threshold tuning.
- Phase 94 expands source-free broad context-area guidance after rejecting a top-10 area-diversity selector that regressed VeriSchema file/source recall.
- Phase 95 makes broad context-area packs operational by highlighting zero-selected areas and representative paths for progressive native reads.
- Phase 96 makes broad context-area guidance consumable through MCP resources while preserving target-file/test/validation metrics.
- Phase 97 improves governance/proof task classification so historical evaluation and product-proof phrasing gets source-free planning docs and broad source-area signals.
- Phase 98 separates source-free broad context-area classification from target-file source-floor spending, covering archive/docs tasks without reintroducing top-10 source/doc churn.
- Phase 99 makes broad context-area resources more actionable for native reads by adding source-free role buckets, path families, and next-read batches without changing top-10 retrieval metrics.
- Phase 100 makes remaining gap summaries resource-backed so broad misses point to the matching context-area MCP resource and bounded next-read paths.
- Phase 101 makes that resource-backed gap shape part of the release proof checker, so future broad proof reports cannot silently drop the progressive read contract.
- Phase 102 makes the resource URIs actually consumable from explicit-repo agent sessions where the MCP server is launched outside the workspace.
- Phase 103 makes broad fixed-corpus regressions release-checkable before more ranking experiments; the rejected dependency-priority reorder proves focused tests are not enough when selection pressure shifts across a wide corpus.
- Phase 104 makes ranked-below-budget source/docs pressure actionable without top-10 churn by surfacing concrete next-read paths and unselected counts in the existing source-free context-area channel.
- Phase 105 makes history-unavailable proof failures explicit and cache-safe instead of schema-shaped missing-report failures.
- Phase 106 makes Codex and Claude Code real-client proof artifacts auditable by recording source-free request hashes, line counts, explicit repo tool-call counts, and sanitized observed tool-call summaries.
- Phase 107 fixes the hydrated full four-repo proof path by bounding parent snapshot git calls, falling back from slow rename detection, and preserving full corpus verdicts in cold and warm proof artifacts.
- Phase 108 bounds cold Git failures: parent caches live outside source repos, archive extraction is removed, subject sampling uses no-rename diffs, stalled object-content batches fail closed instead of recursively hanging, and source-free parent snapshot manifests prevent incomplete caches from being reused as valid warm-cache evidence.
- Latest cold proof: `.ctxhelm/e2e/phase108-cold-git-bounded-proof.json` blocks because RefactoringMiner, ctxhelm, and ReAgent still have insufficient evidence under local cold object-store conditions; VeriSchema remains `beat` at `7463ms`.
- Phase 109 adds source-free `environmentHealth` metadata to benchmark repositories and product-proof corpus verdicts. The latest refreshed cold proof, `.ctxhelm/e2e/phase109-environment-health-proof.json`, blocks because environment health is degraded before retrieval quality can be proven: RefactoringMiner is `git_history_unavailable`, ctxhelm and ReAgent are `git_history_timeout`, and VeriSchema is `healthy` / `beat`.
- Phase 110 adds a clean full-fixture proof path plus parent-snapshot cache invalidation and symbol-facet gating. Latest proof: `.planning/e2e/2026-05-31-phase110-clean-cold-fixture-proof.md` and `.ctxhelm/e2e/phase110-clean-fixture-proof.json`; `releaseGate.decision = promote`, RefactoringMiner is a lexical-ceiling `match`, and ctxhelm/ReAgent/VeriSchema are `beat`.
- Phase 111 wires that clean fixture proof into `scripts/release-gate.sh` with `CTXHELM_CLEAN_FIXTURE_CONFIG`, `CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF`, `CTXHELM_SKIP_CLEAN_FIXTURE_PROOF`, proof-summary fields, release docs, and release packaging contract coverage. Latest proof notes: `.planning/e2e/2026-06-01-phase111-clean-fixture-release-gate.md`.
- Phase 112 proves the complete packaged release gate from clean checkout `/Users/romel/Documents/GitHub/ctxhelm-release-gate-clean-20260601` at `20c367dc7eafc1231559c9110901961c55645089` with `CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF=1`. Latest proof notes: `.planning/e2e/2026-06-01-phase112-clean-release-gate-required.md`; durable summary: `.ctxhelm/e2e/phase112-clean-release-gate-summary.json`.
- Phase 113 creates source-free release-candidate status metadata tied to the Phase 112 proof summary. Latest proof notes: `.planning/e2e/2026-06-01-phase113-release-candidate-status.md`; durable status artifact: `.ctxhelm/e2e/phase113-release-candidate-status.json`.
- Phase 114 publishes and verifies the public `v1.1.0` archive release at `https://github.com/thromel/ctxhelm/releases/tag/v1.1.0`. Latest proof notes: `.planning/e2e/2026-06-01-phase114-public-archive-release.md`; durable source-free release metadata: `.ctxhelm/e2e/phase114-release-proof-summary.json`, `.ctxhelm/e2e/phase114-release-candidate-status.json`, and `.ctxhelm/e2e/phase114-github-release.json`.
- Phase 115 verifies the public archive install path from GitHub release assets. Latest proof notes: `.planning/e2e/2026-06-01-phase115-public-archive-install.md`; durable source-free install proof: `.ctxhelm/e2e/phase115-public-archive-install.json`.
- Phase 116 refreshes optional real-client proof against the public archive binary. Latest proof notes: `.planning/e2e/2026-06-01-phase116-public-real-client-smoke.md`; durable source-free summary: `.ctxhelm/e2e/phase116-public-real-client-smoke.json`; Claude Code `2.1.158` passed with explicit-repo `prepare_task` and `get_pack`, while Codex CLI `0.44.0` was recorded as an optional skip after failing to produce machine-checkable tool-call evidence.
- Phase 117 adds source-free role signals to broad context areas. Latest proof notes: `.planning/e2e/2026-06-01-phase117-context-area-role-signals.md`; durable source-free summary: `.ctxhelm/e2e/phase117-context-area-role-signals.json`; generated packs render `Role counts:` and `Selected roles:` for broad context areas.
- Phase 118 adds explicit source-free scope metadata to context-area MCP resources. Latest proof notes: `.planning/e2e/2026-06-01-phase118-context-area-resource-scope.md`; durable source-free summary: `.ctxhelm/e2e/phase118-context-area-resource-scope.json`; `ctxhelm://repo/context-areas` and `ctxhelm://repo/context-area/{encoded-area}` now report `resourceScope.kind = safeInventoryArea`, `taskConditioned = false`, `countsSource = safeInventory`, and `pathSource = safeInventory`.
- Phase 119 removes an observed `ctxhelm-index` release-validation flake by using one crate-wide test lock for all tests that mutate `CTXHELM_HOME`. Latest proof notes: `.planning/e2e/2026-06-01-phase119-index-env-lock-flake.md`; durable source-free summary: `.ctxhelm/e2e/phase119-index-env-lock-proof.json`; three consecutive parallel `ctxhelm-index --lib` test runs passed after the change.
- Phase 123 adds source-free coverage profiles to context-area MCP resources. Latest proof notes: `.planning/e2e/2026-06-01-phase123-context-area-coverage-profile.md`; durable source-free summary: `.ctxhelm/e2e/phase123-context-area-coverage-profile.json`; resources now expose `coverageProfile.profile`, `dominantRole`, `recommendedFirstBatch`, and source-free role-family counts.
- Phase 124 adds source-free inspection strategies to context-area MCP resources. Latest proof notes: `.planning/e2e/2026-06-01-phase124-context-area-inspection-strategy.md`; durable source-free summary: `.ctxhelm/e2e/phase124-context-area-inspection-strategy.json`; resources now expose `inspectionStrategy.initialBatch`, `preferredOrder`, `pathBudget`, and `stopRule` without changing ranking.
- Phase 125 adds source-free lexical comparison summaries to product proof. Latest proof notes: `.planning/e2e/2026-06-01-phase125-lexical-comparison-proof.md`; durable source-free summary: `.ctxhelm/e2e/phase125-lexical-comparison-proof.json`; `releaseGate.lexicalComparison` now makes all-file and context-channel lexical claims explicit.
- Phase 126 adds source-free agent-evidence lexical comparison. Latest proof notes: `.planning/e2e/2026-06-01-phase126-agent-evidence-lexical-comparison.md`; durable source-free summary: `.ctxhelm/e2e/phase126-agent-evidence-lexical-comparison.json`; `agentEvidenceClaim = mixed` with beat `3`, match `1`, trail `0`, and average agent-evidence delta `+0.18792826`.
- Phase 127 adds narrow-plan validation-test reservation in context ranking. Latest proof notes: `.planning/e2e/2026-06-01-phase127-narrow-validation-test-reserve.md`; durable source-free summary: `.ctxhelm/e2e/phase127-narrow-validation-test-reserve.json`; `allFileClaim = mixed` with beat `3`, match `1`, trail `0`, and average file delta `+0.13567334`.
- Phase 128 adds broad operational floors for root governance docs, exact config evidence, and workflow lifecycle scripts. Latest proof notes: `.planning/e2e/2026-06-01-phase128-broad-operational-floor.md`; durable source-free summary: `.ctxhelm/e2e/phase128-broad-operational-floor.json`; all four corpora have protected target miss-rate `0.0`, average file delta improves to `+0.14154172`, and average context delta improves to `+0.23717105`.
- Phase 178 separates explained raw all-file trails from unexplained proof regressions. Latest proof notes: `.planning/e2e/2026-06-02-phase178-explained-all-file-trails.md`; fresh source-free summary: `/tmp/ctxhelm-rd/phase178-clean-fixture-explained-trails.json`; `allFileClaim = mixed`, beat `3`, raw match `0`, raw trail `1`, explained trail `1`, unexplained trail `0`, average file delta `+0.15480787`, agent-evidence delta `+0.2570628`, and context delta `+0.30652046`.
- Phase 179 adds source-free graph edge profiles to historical eval. Latest proof notes: `.planning/e2e/2026-06-02-phase179-graph-edge-profiles.md`; fresh source-free summary: `/tmp/ctxhelm-rd/phase179-graph-edge-profiles-warm-proof.json`; the warm proof promotes and VeriSchema now exposes `imports` target hits/misses as `6`/`22`, making graph edge-family ranking pressure measurable.
- Phase 180 adds source-free graph edge ablations to historical eval. Latest proof notes: `.planning/e2e/2026-06-02-phase180-graph-edge-ablations.md`; fresh source-free summary: `/tmp/ctxhelm-rd/phase180-graph-edge-ablation-proof.json`; the four-repo proof promotes and VeriSchema exclusive `imports` ablation removes `1` target hit for Recall@K delta `-0.00512819`, while `python_reexport` has no exclusive top-10 target lift.
- Phase 129 adds public release freshness proof. Latest proof notes: `.planning/e2e/2026-06-01-phase129-public-release-freshness.md`; durable source-free summary: `.ctxhelm/e2e/phase129-public-release-freshness.json`; public `v1.1.0` targets `68383cbfc2fff00c4f53fbd2b7bf90527ac4bd7e`, current main is `a07e6be31a0605af1810e79cb18b34245fa7def0`, `gitRelation = current_descends_from_release`, `commitsAhead = 19`, and `status = outdated`.
- Phase 130 publishes and verifies the current public `v1.1.1` archive. Latest proof notes: `.planning/e2e/2026-06-01-phase130-public-v111-release.md`; durable source-free summaries: `.ctxhelm/e2e/phase130-github-release.json`, `.ctxhelm/e2e/phase130-public-release-freshness.json`, `.ctxhelm/e2e/phase130-public-archive-install.json`, and `.ctxhelm/e2e/phase130-public-real-client-smoke.json`; public `v1.1.1` targets `6c93100fa0e4f5a5444fb7fd967c721cca49a401`, `gitRelation = same`, `commitsAhead = 0`, temporary public archive install passes version/help/doctor/first-pack checks, Claude Code `2.1.158` passes explicit-repo `prepare_task` and `get_pack`, and Codex CLI `0.44.0` remains an optional source-free skip.
- Phase 131 publishes and verifies the product-aware `v1.1.2` archive. Latest proof notes: `.planning/e2e/2026-06-01-phase131-product-aware-freshness-release.md`; durable source-free summaries: `.ctxhelm/e2e/phase131-github-release.json`, `.ctxhelm/e2e/phase131-github-release-verify.json`, `.ctxhelm/e2e/phase131-public-release-freshness.json`, `.ctxhelm/e2e/phase131-public-archive-install.json`, and `.ctxhelm/e2e/phase131-public-real-client-smoke.json`; public `v1.1.2` targets `ac6dc97f04cd18b5f2c6c32f7a1eca49e3ef5587`, `productStatus = current`, `productCommitsAhead = 0`, temporary public archive install passes version/help/doctor/first-pack checks, Claude Code `2.1.158` passes explicit-repo `prepare_task` and `get_pack`, and Codex CLI `0.44.0` remains an optional source-free skip.
- Phase 132 adds and verifies a deeper Claude Code workflow eval. Latest proof notes: `.planning/e2e/2026-06-01-phase132-claude-workflow-eval.md`; durable source-free summary: `.ctxhelm/e2e/phase132-claude-workflow-eval.json`; Claude Code `2.1.159` produced explicit-repo `prepare_task` and `get_pack` calls through MCP, with only hashes and sanitized request metadata committed.
- Phase 133 makes the top-level README answer why agents should use ctxhelm before the command tour, records the current lexical/protected-target/Claude proof snapshot, updates agent setup docs to Codex CLI `0.44.0` and Claude Code `2.1.159`, and release-gates those public claims through `scripts/check-release-docs.sh`. Latest proof notes: `.planning/e2e/2026-06-01-phase133-product-readme-positioning.md`; durable source-free summary: `.ctxhelm/e2e/phase133-product-readme-positioning.json`.
- Phase 134 publishes and verifies the current public `v1.1.3` archive. Latest proof notes: `.planning/e2e/2026-06-01-phase134-public-v113-release.md`; durable source-free summaries: `.ctxhelm/e2e/phase134-github-release.json`, `.ctxhelm/e2e/phase134-github-release-verify.json`, `.ctxhelm/e2e/phase134-public-release-freshness.json`, `.ctxhelm/e2e/phase134-public-archive-install.json`, and `.ctxhelm/e2e/phase134-public-real-client-smoke.json`; public `v1.1.3` targets `f17bd4cb27f1989e696717ac706868808ff01151`, `productStatus = current`, `productCommitsAhead = 0`, temporary public archive install passes checksum/archive/version/help/doctor/first-pack checks, Claude Code `2.1.159` passes explicit-repo `prepare_task` and `get_pack`, and Codex CLI `0.44.0` remains an optional source-free skip.
- Next work should use optional real-client evidence freshness, public release freshness, and any new proof-backed broad protected miss rates as the production-readiness scoreboard.
