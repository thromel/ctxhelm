# Roadmap: ctxhelm

## Overview

This roadmap tracks v2.5 Production Retrieval Quality and its immediate production-readiness follow-ups. v2.4 made semantic, precision, provider, and reranker paths source-safe and policy-gated, then the fresh RefactoringMiner proof fixed a semantic fusion regression. The current fixed two-repo product proof promotes default local retrieval under a channel-aware gate: non-test context recall beats lexical on both corpora, while validation-test recall is measured separately through `recommended_tests`.

v2.5 therefore focuses on measured retrieval quality, not more surface area. The milestone must prove whether production local embeddings, reranking, graph/test/history fixes, and learned fusion can beat lexical baseline on real repositories while staying local-first and source-safe. Phase 66 fixed the false zero-test-recall signal by measuring `recommended_tests` as its own validation channel. Phase 67 fixed the denominator for historical retrieval metrics by separating all safe changed files from parent-snapshot `retrievalTargetFiles`. Phase 69 promoted default local retrieval under the channel-aware proof, Phase 70 refreshed real-client MCP evidence for Codex CLI and Claude Code, Phase 71 reduced archive-artifact retrieval noise in ctxhelm's own history, Phase 72 broadened repeated-lift validation while improving validation-test recall seeding, Phase 73 pinned a broader optional fixed-corpus probe, Phase 76 split partial-snapshot history into validation-only mode for historical eval, Phase 77 added broad validation-command coverage for multi-area smoke/eval tasks, Phase 78 made the broader proof gate lexical-ceiling aware, Phase 79 added protected target floors, Phase 80 fixed symbol-floor duplicate accounting, Phase 81 made warm-cache runtime proof trustworthy, Phase 82 made warm-cache runtime enforceable, Phase 83 made context-vs-all-file divergence machine-checkable, Phase 84 added broad-scope task accounting plus scoped dependency source floors, Phase 85 added source-free context-area hints for broad prepare-task plans and packs, Phase 86 added bounded Python package re-export graph coverage, Phase 87 fixed validation-covered test gap accounting, Phase 88 added bounded broad source-area candidates, Phase 89 reduced repeated inventory freshness overhead so the pinned broader release proof promotes, Phase 90 proved the packaged release gate from a clean worktree with the broad benchmark enabled, Phase 91 made broad context-area coverage measurable in historical eval, Phase 92 made broad-area gap taxonomy area-aware, Phase 93 added source-free symbol/dependency index caches so clean cold large-repo proof promotes, Phase 94 improved wide-task progressive area coverage without target-file churn, Phase 95 made generated broad packs tell agents which zero-selected areas to read next, Phase 96 made those broad areas consumable as source-free MCP resources, Phase 97 improved broad governance/proof/eval task classification, Phase 98 split source-free broad context-area classification from target-file source floors for archive/docs tasks, Phase 99 added source-free read batches to context-area MCP resources, Phase 100 made retrieval-gap summaries resource-backed, Phase 101 made that resource-backed shape part of the product-proof checker, Phase 102 made repo-scoped MCP resources consumable after explicit-repo tool calls from a non-repo server cwd, Phase 103 added pinned broad fixed-corpus floors, Phase 104 added source-free next-read paths and unselected counts to broad context areas, Phase 105 made history-unavailable benchmark repos produce embedded insufficient-evidence reports instead of missing reports, Phase 106 hardened Codex/Claude real-client evidence with source-free request metadata, Phase 107 fixed the hydrated full four-repo proof path, Phase 108 bounded cold Git failure modes, Phase 109 made cold proof environment-health blockers machine-readable, Phase 110 promoted a clean cold full-fixture proof by fixing stale parent-snapshot inventory reuse and false symbol extraction, Phase 111 wired that clean fixture proof into the packaged release gate, Phase 112 passed the clean packaged release gate with the clean fixture proof required, Phase 113 recorded source-free release-candidate status that binds `ready` to the archive binary proof and archive-first distribution decisions, Phase 114 published plus verified the public `v1.1.0` GitHub archive release, Phase 115 verified the public archive install path from GitHub release assets, Phase 116 refreshed optional real-client evidence against the public archive binary, Phase 117 added source-free role signals to broad context-area guidance, Phase 118 added explicit safe-inventory resource-scope metadata to context-area MCP resources, Phase 119 removed an observed `CTXHELM_HOME` test-environment race from `ctxhelm-index` release validation, Phase 120 added public GitHub Actions CI release-gate enforcement, Phase 121 moved public CI JavaScript actions to Node 24 and verified the public run passed without Node 20 warning text, Phase 122 fixed public archive real-client smoke compatibility with post-release MCP protocol assertions, Phase 123 added source-free coverage profiles to context-area MCP resources, Phase 124 added source-free inspection strategies to context-area resources, Phase 125 added explicit source-free lexical comparison summaries to product proof output, Phase 126 added agent-evidence lexical comparison for the actual context/test/validation evidence set ctxhelm gives agents, Phase 127 added narrow-plan validation-test reservation so raw target-file recall no longer trails lexical on any measured corpus, Phase 128 added broad operational floors so protected target miss-rate is zero across all four measured corpora, Phase 129 added public release freshness proof so publication claims cannot silently imply the already-published archive contains the latest post-release hardening, Phase 130 published plus verified a current public `v1.1.1` archive with `commitsAhead = 0`, public install proof, and refreshed optional real-client evidence, Phase 131 added product-aware release freshness plus published and verified the public `v1.1.2` archive with Claude Code integration evidence, Phase 132 added source-free Claude Code workflow eval proof with explicit-repo `prepare_task` and `get_pack` calls, Phase 133 aligned the public README with the product wedge and current proof snapshot, Phase 134 published plus verified the current public `v1.1.3` archive, Phase 135 added Homebrew/crates readiness gates without publishing package-manager artifacts, Phase 136 published plus verified the current public `v1.1.4` archive, Phase 137 published plus verified the public Apple Silicon Homebrew tap, Phase 138 published plus verified the current public `v1.1.5` archive and refreshed Homebrew tap, Phase 139 introduced a short-lived interim brand while preserving the `ctxhelm` compatibility surface, Phase 140 published plus verified the public `v1.1.6` archive and Homebrew tap, Phase 141 finalized ctxhelm after availability review, Phase 142 published plus verified the public `v1.1.7` ctxhelm archive and Homebrew tap, Phase 143 added paired Claude Code process proof, Phase 144 published plus verified the post-rename public `v1.1.8` archive and Homebrew tap with public-archive Claude Code evidence, Phase 145 published plus verified `v1.1.9` so the public artifact includes the Git-history timeout hardening, Phase 146 made crates source distribution publish-order ready without claiming crates.io publication, Phase 147 published plus verified `v1.1.10` so the public archive and Homebrew tap include that source-distribution hardening, Phase 148 made Codex real-client optional skips source-free and diagnosable by classifying the current client failure as `stream_disconnected` while preserving Claude Code machine-checkable proof, and Phase 149 published plus verified `v1.1.11` so the public archive and Homebrew tap include Phase 148 diagnostics with fresh public-archive Claude Code and Codex evidence.

## v2.5 Production Retrieval Quality

## Phases

**Phase Numbering:**

- Integer phases (61, 62, 63, 64, 65): Planned v2.5 work
- Phases 66-169: Production-readiness follow-ups from the original blocked proof and the channel-aware promotion path
- Decimal phases (61.1, 62.1): Urgent insertions if needed

- [x] **Phase 61: Multi-Repo Quality Baselines** - Maintainers can run source-free paired baselines across RefactoringMiner and a second real repository with stable comparison artifacts.
- [x] **Phase 62: Production Local Embedding Quality** - Maintainers can evaluate production local embeddings against lexical/default baselines with bounded local cache and provider metadata.
- [x] **Phase 63: Reranker And Fusion Promotion** - Maintainers can compare reranker/fusion variants under promotion gates that protect anchors and exact evidence.
- [x] **Phase 64: Gap-Family Retrieval Improvements** - Maintainers can convert repeated gap families into targeted retrieval fixes with before/after proof.
- [x] **Phase 65: v2.5 Product Proof And Release Gate** - Maintainers can ship or hold v2.5 variants using multi-repo proof, docs, and release gates.
- [x] **Phase 66: Test Recall Evaluation Channel** - Maintainers can measure validation-test recall through the dedicated `recommended_tests` output without degrading target-file recall.
- [x] **Phase 67: Retrievable Target Eval Denominator** - Maintainers can distinguish all safe changed files from files that existed in the parent snapshot and could be retrieved as context.
- [x] **Phase 69: Channel-Aware Product Proof Gate** - Maintainers can promote default local retrieval when context recall beats lexical while validation-test recall is proven separately.
- [x] **Phase 70: Real-Client MCP Proof Refresh** - Maintainers can verify Codex CLI and Claude Code still invoke `prepare_task` and `get_pack` through actual MCP client paths after promotion.
- [x] **Phase 71: Archive Artifact Dampening** - Maintainers can reduce ctxhelm planning-archive retrieval noise without excluding archived evidence from search.
- [x] **Phase 72: Broader Repeated-Lift Validation** - Maintainers can probe more local repositories, improve validation-test recall seeding, and keep broader promotion gaps explicit.
- [x] **Phase 73: Broader Fixed-Corpus Fixture** - Maintainers can rerun the broader probe from a pinned committed config instead of a temporary manifest.
- [x] **Phase 74: Protected Evidence Diagnostics** - Maintainers can separate protected retrieval-target misses from non-target exact/symbol pressure.
- [x] **Phase 75: Parent-Bounded History And Test Reserve** - Maintainers can preserve source-free parent-bounded history and reserve co-changed validation tests.
- [x] **Phase 76: Parent-Bounded Validation History** - Maintainers can use parent-bounded history for historical eval validation tests without perturbing non-test target ranking from partial snapshots.
- [x] **Phase 77: Validation Command Coverage** - Maintainers can represent broad multi-area validation tasks with suite-level fallback commands and effective validation recall.
- [x] **Phase 78: Ceiling-Aware Broader Gate** - Maintainers can promote broader proof when a corpus reaches a safe lexical ceiling instead of falsely requiring impossible recall lift.
- [x] **Phase 79: Protected Target Floors** - Maintainers can reduce protected retrieval-target misses by reserving bounded source/config/governance evidence and deferring archive artifacts.
- [x] **Phase 80: Unique Symbol Floor Accounting** - Maintainers can keep symbol-only source evidence inside budget when duplicate already-selected symbol files appear first.
- [x] **Phase 81: Warm Cache Latency Proof** - Maintainers can prove cache-hit eval runtime with source-free cold/warm product proof artifacts.
- [x] **Phase 82: Warm Cache Release Gate** - Maintainers can block product-proof promotion when cached runtime evidence is stale or too slow.
- [x] **Phase 83: Context Divergence Accounting** - Maintainers can distinguish useful context-channel lift from all-file lexical deficits caused by validation targets.
- [x] **Phase 84: Broad Scope Dependency Floors** - Maintainers can identify broad multi-area tasks and preserve bounded dependency source evidence for them.
- [x] **Phase 85: Broad Context Areas** - Agents can inspect source-free area hints for broad multi-area tasks without displacing target files or validation channels.
- [x] **Phase 86: Python Package Re-Export Graph Coverage** - Python package `__init__.py` re-exports can contribute bounded graph candidates without enabling general depth-2 expansion.
- [x] **Phase 87: Validation Gap Accounting** - Proof reports stop classifying validation-covered tests as unresolved retrieval gaps, and Java class-selector commands count toward validation coverage.
- [x] **Phase 88: Broad Source-Area Candidates** - Broad multi-area tasks get bounded source-area inventory candidates after graph/test seed selection, improving VeriSchema source recall without validation regression.
- [x] **Phase 89: Fast Inventory Freshness** - Inventory cache-hit freshness avoids full source re-hashing, reducing broad-proof runtime while preserving quality and privacy gates.
- [x] **Phase 90: Packaged Release Gate** - The clean packaged release gate passes with broad benchmark proof enabled.
- [x] **Phase 91: Broad Context-Area Eval** - Historical eval records changed context areas, surfaced area hits, and broad context-area recall for wide tasks that exceed the target-file budget.
- [x] **Phase 92: Area-Aware Gap Taxonomy And Large-Repo Warm Proof** - Gap reports distinguish surfaced context-area coverage from true no-candidate retrieval failures.
- [x] **Phase 93: Source-Free Index Cache** - Cold large-repo planner proof reuses source-free symbol/dependency indexes and promotes without release-threshold tuning.
- [x] **Phase 94: Broad Context-Area Cap** - Wide tasks expose more source-free context areas after proof rejects top-10 area-diversity churn.
- [x] **Phase 95: Progressive Area Pack Guidance** - Generated packs tell agents which zero-selected broad areas to inspect next.
- [x] **Phase 96: Context Area MCP Resources** - Broad source-free context areas expose MCP resource URIs for progressive agent reads without expanding the tool surface.
- [x] **Phase 97: Broad Governance Classification** - Historical eval, product proof, metric, and promotion phrasing gets source-free planning docs and broad source-area signals.
- [x] **Phase 98: Progressive Broad Classification** - Archive/docs broad tasks get source-free context-area guidance without spending target-file budget on broad source floors.
- [x] **Phase 99: Context Area Read Batches** - Dynamic context-area resources expose source-free role buckets, path families, and next-read batches for progressive native reads.
- [x] **Phase 100: Resource-Backed Gap Summaries** - Historical eval gap summaries include source-free context-area resource URIs and next-read paths.
- [x] **Phase 101: Release-Gated Gap Summary Contract** - The product-proof checker fails current reachable gap summaries that drop context-area resource URIs or next-read paths.
- [x] **Phase 102: Explicit-Repo MCP Resource Consumption** - Repo-scoped MCP resources resolve against the last explicit tool repo when the server cwd is outside the workspace.
- [x] **Phase 103: Broad Fixed-Corpus Floors** - The product-proof checker rejects broad pinned-corpus metric regressions even when the aggregate proof still promotes.
- [x] **Phase 104: Context Area Next-Read Paths** - Broad context areas expose docs, unselected counts, and concrete source-free next-read paths for agents without changing top-10 ranking.
- [x] **Phase 105: History-Unavailable Embedded Reports** - Benchmark repos with unavailable git history keep embedded reports and block as insufficient evidence instead of producing `report: null`.
- [x] **Phase 106: Real-Client Request Evidence Hardening** - Codex CLI and Claude Code smoke evidence records source-free request hashes, line counts, explicit repo tool-call counts, and sanitized observed tool-call summaries.
- [x] **Phase 107: Hydrated Four-Repo Proof Path** - The broad proof hydrates RefactoringMiner, ctxhelm, ReAgent, and VeriSchema with embedded reports instead of hanging or returning missing evidence.
- [x] **Phase 108: Cold Git Bounds** - Cold parent snapshots use external caches, no-rename subject sampling, bounded object-batch reads, and completion manifests; the proof now fails fast on local unreadable Git objects without poisoning later caches.
- [x] **Phase 109: Environment Health Verdicts** - Cold proofs now classify degraded Git history/object-store environments before treating insufficient evidence as retrieval-quality failure.
- [x] **Phase 110: Clean Cold Fixture Proof** - Clean detached full fixtures promote across RefactoringMiner, ctxhelm, ReAgent, and VeriSchema after parent-snapshot cache invalidation and symbol-facet gating.
- [x] **Phase 111: Clean Fixture Release Gate** - The packaged release gate runs or requires the clean cold fixture product proof and records source-free proof-summary status.
- [x] **Phase 112: Clean Release Gate With Required Fixture Proof** - A clean release-candidate checkout passes the full packaged release gate with `CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF=1`.
- [x] **Phase 113: Release Candidate Status** - Source-free release-candidate metadata records archive/binary proof, required clean fixture proof, and archive-first distribution decisions.
- [x] **Phase 114: Public Archive Release** - The `v1.1.0` GitHub archive release is published and verified against source-free asset digests.
- [x] **Phase 115: Public Archive Install Verification** - The public archive install path downloads GitHub release assets, verifies checksums, installs temporarily, and passes version/help/doctor/first-pack checks.
- [x] **Phase 116: Public Archive Real-Client Smoke** - Optional Codex CLI and Claude Code real-client evidence is refreshed against the public archive binary with source-free pass/skip artifacts.
- [x] **Phase 117: Context Area Role Signals** - Broad plan-level context areas expose source-free role counts and selected-role counts, and packs render those signals for progressive native reads.
- [x] **Phase 118: Context Area Resource Scope** - Context-area MCP resources expose source-free `resourceScope` metadata so agents can distinguish inventory-wide resource counts from task-conditioned plan counts.
- [x] **Phase 119: Index Test Environment Lock** - `ctxhelm-index` tests that mutate `CTXHELM_HOME` share one crate-wide lock, removing an observed release-validation flake.
- [x] **Phase 120: Public CI Release Gate** - GitHub Actions enforce formatting, clippy, locked tests, CLI help, release-doc consistency, and release-gate smoke on public pushes and pull requests.
- [x] **Phase 121: CI Node 24 Runtime Guard** - Public GitHub Actions use Node 24 action majors and pass without Node 20 deprecation warning text.
- [x] **Phase 122: Public Real-Client Protocol Compatibility** - Public archive real-client smoke stays compatible with `v1.1.0` while current MCP resource-scope checks remain strict.
- [x] **Phase 123: Context Area Coverage Profile** - Context-area MCP resources expose source-free coverage profiles and recommended first read batches for progressive agent reads.
- [x] **Phase 124: Context Area Inspection Strategy** - Context-area MCP resources expose source-free progressive read order, path budgets, and stop rules for broad agent reads.
- [x] **Phase 125: Lexical Comparison Proof Boundary** - Product proof exposes source-free all-file and context-channel lexical comparison claims.
- [x] **Phase 126: Agent Evidence Lexical Comparison** - Product proof exposes source-free lexical comparison for the actual agent evidence set: context files, related tests, and validation commands.
- [x] **Phase 127: Narrow Validation-Test Reserve** - Narrow plans reserve validation-test slots in context ranking while broad context-area plans stay file-first.
- [x] **Phase 128: Broad Operational Floors** - Broad tasks reserve root governance docs, exact config evidence, and workflow lifecycle scripts before lower-priority expansion.
- [x] **Phase 129: Public Release Freshness Proof** - Public release metadata records whether the published archive target commit matches current main before claiming latest production hardening is downloadable.
- [x] **Phase 130: Public v1.1.1 Release Currentness** - The refreshed public archive release is current with main, installable from GitHub assets, and covered by refreshed optional real-client smoke evidence.
- [x] **Phase 131: Product-Aware Freshness Release** - Public freshness checks distinguish proof-only commits from product-impacting drift, and the public v1.1.2 archive is verified with Claude Code integration evidence.
- [x] **Phase 132: Claude Workflow Eval** - Maintainers can require a source-free Claude Code workflow eval proving real explicit-repo `prepare_task` and `get_pack` calls without storing raw prompts or raw MCP traffic.
- [x] **Phase 133: Product README Positioning** - Public docs explain why ctxhelm is useful beyond native agent search, show the current proof snapshot, and gate current client-evidence strings.
- [x] **Phase 134: Public v1.1.3 Release Currentness** - The current public archive release targets the latest product commit, installs from GitHub assets, and carries refreshed optional real-client proof.
- [x] **Phase 135: Distribution Readiness** - Deferred Homebrew and crates.io install channels are mechanically checked through formula rendering and package-boundary smoke proof without publishing them.
- [x] **Phase 136: Public v1.1.4 Release Currentness** - The current public archive release includes the distribution-readiness gate, installs from GitHub assets, and carries refreshed optional real-client proof.
- [x] **Phase 137: Public Homebrew Tap** - The public `thromel/tap` Homebrew install path passes strict audit, install, formula test, and installed-version proof for Apple Silicon macOS.
- [x] **Phase 138: Public v1.1.5 Release Currentness** - The current public archive and public Homebrew tap both target the latest product commit and carry refreshed archive, tap, freshness, and real-client proof.
- [x] **Phase 139: ctxhelm Brand Identity** - Public product surfaces used a short-lived interim name while `ctxhelm` remained the CLI, package, Homebrew formula, and MCP namespace.
- [x] **Phase 140: Public v1.1.6 Release Currentness** - The ctxhelm-branded public archive and Homebrew tap both targeted the latest product commit and carried refreshed archive, tap, freshness, and real-client proof.
- [x] **Phase 141: ctxhelm Brand Identity** - Public product surfaces use ctxhelm after availability review rejected the interim name as too close to an adjacent Mason MCP/code-context product.
- [x] **Phase 142: Public v1.1.7 Release Currentness** - The ctxhelm-branded public archive and Homebrew tap both target the latest product commit and carry refreshed archive, tap, freshness, and real-client proof.
- [x] **Phase 143: Paired Agent-Run Outcome Harness** - Claude Code paired-run evidence compares native baseline, `prepare_task`, and brief-pack lanes; the brief-pack lane preserves target coverage while reducing irrelevant reads in a source-free report.
- [x] **Phase 144: Public v1.1.8 Post-Rename Currentness** - The post-rename public archive and Homebrew tap both target the current `thromel/ctxhelm` commit, install cleanly, and carry refreshed public-archive Claude Code proof.
- [x] **Phase 145: Public v1.1.9 Hardening Currentness** - The current public archive and Homebrew tap include the bounded Git-history timeout hardening, install cleanly, pass public freshness checks, and carry refreshed public-archive Claude Code proof.
- [x] **Phase 146: Crates Publish-Order Readiness** - Source distribution checks cover every workspace crate, internal path dependencies carry explicit versions, and docs record the required crates.io publish order while publication remains deferred.
- [x] **Phase 147: Public v1.1.10 Source Distribution Currentness** - The current public archive and Homebrew tap target the latest product commit, include crates source-distribution readiness, pass public freshness checks, and carry refreshed public-archive Claude Code proof.
- [x] **Phase 148: Codex Real-Client Diagnostic Evidence** - Codex optional-skip proof now records source-free client failure classification, exit status, stderr hash/line count, and MCP method counts while Claude Code still passes explicit-repo `prepare_task` and `get_pack`.
- [x] **Phase 149: Public v1.1.11 Currentness** - The current public archive and Homebrew tap target the latest product commit, include Codex diagnostic hardening, pass public freshness/install/tap checks, and carry refreshed public-archive Claude Code and Codex evidence.
- [x] **Phase 152: Native-Agent Outcome Suite** - The paired Claude Code agent-run harness now accepts source-free task suites, aggregates native baseline versus ctxhelm-assisted lane metrics, and renders suite reports through `ctxhelm eval agent-run`.
- [x] **Phase 153: BM25 Symbol Lexical Index** - Lexical search now uses a query-time Tantivy/BM25 fielded index with symbol facets and exact-match bonuses while keeping source-derived inverted index data in memory.
- [x] **Phase 154: BM25 Legacy Comparison Report** - `ctxhelm eval lexical compare` now produces a source-free report comparing active BM25 lexical ranking against the legacy heuristic scanner, including overlap, top-path changes, backend-only paths, and privacy metadata.
- [x] **Phase 155: BM25 Corpus Comparison Report** - `ctxhelm eval lexical corpus` now compares active BM25 and legacy lexical ranking across historical commit tasks and parent snapshots, reporting recall, MRR, overlap, top-path churn, win/tie counts, and backend runtime.
- [x] **Phase 156: Lexical Backend Product Proof Integration** - Benchmark suites can opt into the BM25-vs-legacy corpus comparison, and product proof aggregates successful source-free backend reports under `releaseGate.lexicalBackendComparison`.
- [x] **Phase 157: Benchmark Corpus Health Guard** - Large-history proof fixtures can be prepared as clean detached worktrees with source-free readiness reports; the fresh RefactoringMiner proof exposes that current query-time BM25 trails the legacy scanner on the 20-commit sample.
- [x] **Phase 158: BM25 Exact-Saturated Fast Path** - Active lexical ranking keeps exact evidence primary, skips fielded BM25 indexing when exact candidates fill the budget, and reaches RefactoringMiner parity with legacy while reducing cold backend time versus Phase 157.
- [x] **Phase 159: Lexical Runtime Accounting And Exact-Primary Policy** - Lexical backend proof now separates shared inventory warmup, uses source-safe inventory fingerprints for cache keys, reuses in-process fielded indexes, and stops early for exact-dominant or single-identifier queries; clean RefactoringMiner active backend time reaches parity with legacy while preserving recall parity.
- [x] **Phase 160: Bounded Semantic Status And Search** - Direct semantic status/search now use bounded source-free document samples, avoid eager symbol/dependency/test/vector work on status paths, and rank exact path/identifier matches through source-free path metadata plus an exact metadata boost; the clean RefactoringMiner `TypeScriptVisitor` semantic probe now returns the target file first.
- [x] **Phase 161: Semantic Gate Contribution Diagnostics** - The semantic/precision gate now reports source-free semantic contribution counts and named semantic-only target hits, accepts provider selection for `local_hash` versus `local_fastembed`, and proves the feature-gated production-local embedding path still compiles.
- [x] **Phase 162: Feature-Enabled Local Fastembed Gate Proof** - A feature-enabled `local_fastembed` gate run on clean RefactoringMiner now proves the production-local backend works end-to-end, but remains held because it adds no semantic-only target hits and is still slower than default; the gate emits a source-free diagnostic for that condition.
- [x] **Phase 163: Persisted Semantic Vector Reuse** - Fresh CLI/MCP processes can reuse persisted source-free semantic document vectors instead of recomputing every candidate vector.
- [x] **Phase 164: Global Semantic Vector Candidates And Write-Through** - Semantic search can include persisted vector candidates outside the lexical prefilter and write through newly embedded candidate misses.
- [x] **Phase 165: Fastembed Default And Loud Index Errors** - `local_fastembed` defaults to `AllMiniLML6V2Q`, documented model ids resolve explicitly, and semantic indexing fails loudly instead of reporting successful zero-vector stores.
- [x] **Phase 166: Semantic Query Vector Reuse** - Repeated fresh-process `local_fastembed` searches reuse source-free query vectors and single-pass stored-candidate expansion, reducing RefactoringMiner steady-state search latency while preserving the known top result.
- [x] **Phase 167: Pruned Generated Inventory Walk** - Inventory and freshness walks now prune generated fixture/cache/build trees before descending, cutting clean RefactoringMiner first lexical setup to `3.70s`, semantic seed to `5.18s`, and second fresh-process semantic search to `0.11s` while preserving the known top result.
- [x] **Phase 168: Semantic Alias And Noise Diagnostics** - Semantic document/query text now includes source-free identifier aliases and versioned vector hashes; the gate reports semantic-only non-targets, proving the current RefactoringMiner semantic lift failure is noise/coupled-source context rather than a promotable embedding win.
- [x] **Phase 169: Graph Ordering And Context Balance** - Related dependency retrieval now prefers outgoing seed imports, source-free identifier affinity, dependency-order preservation, bounded standard dependency reserve, and a narrower validation-test reserve; the exact RefactoringMiner two-commit proof improves File/Source Recall@10 from `0.5833334` to `0.75`.

## Phase Details

### Phase 61: Multi-Repo Quality Baselines

**Goal**: Maintainers can run source-free paired baselines across RefactoringMiner and a second real repository with stable comparison artifacts.

**Depends on**: v2.3 fixed-corpus eval, v2.4 semantic proof

**Requirements**: BASE-01, BASE-02, BASE-03, BASE-04

**Success Criteria**:

1. Baseline command/report covers at least RefactoringMiner and one second real repo.
2. Reports include stable corpus identity, revision range, provider status, runtime, cache, privacy, and named gap families.
3. Default, lexical, graph, semantic, reranked, and learned-policy variants can be compared deterministically.
4. Reports remain source-free.

**Plans**: 1 plan

Plans:

- [x] 61-multi-repo-quality-baselines-01-PLAN.md - Build the multi-repo baseline manifest/report path and prove it on real repos.

### Phase 62: Production Local Embedding Quality

**Goal**: Maintainers can evaluate production local embeddings against lexical/default baselines with bounded local cache and provider metadata.

**Depends on**: Phase 61, v2.4 local provider policy

**Requirements**: EMBED-01, EMBED-02, EMBED-03, EMBED-04

**Success Criteria**:

1. Production local embedding backend runs from CLI/eval without cloud transfer.
2. Cache behavior is bounded, ignored by inventory, and visible in status reports.
3. Quality is measured against lexical/default before promotion.
4. `local_hash` remains scaffold-labeled.

**Plans**: 1 plan

Plans:

- [x] 62-production-local-embedding-quality-01-PLAN.md - Harden local embedding quality, cache behavior, and provider evidence.

### Phase 63: Reranker And Fusion Promotion

**Goal**: Maintainers can compare reranker/fusion variants under promotion gates that protect anchors and exact evidence.

**Depends on**: Phases 61-62

**Requirements**: RANK-01, RANK-02, RANK-03, RANK-04

**Success Criteria**:

1. Reranker/fusion variants are source-safe and do not expand MCP tool surface.
2. Anchors, current diff, exact lexical, and strong symbols are protected.
3. Promotion gates compare quality, runtime, token ROI, and privacy.
4. Named regressions block promotion.

**Plans**: 1 plan

Plans:

- [x] 63-reranker-and-fusion-promotion-01-PLAN.md - Implement and evaluate promotion-safe reranking/fusion variants.

### Phase 64: Gap-Family Retrieval Improvements

**Goal**: Maintainers can convert repeated gap families into targeted retrieval fixes with before/after proof.

**Depends on**: Phase 61

**Requirements**: GAP-01, GAP-02, GAP-03, GAP-04

**Success Criteria**:

1. Gap families are grouped into actionable tasks.
2. At least one high-impact RefactoringMiner gap family improves with measured before/after proof.
3. Test recommendation quality is evaluated separately from target-file recall.
4. Graph expansion stays budgeted and seed-safe.

**Plans**: 1 plan

Plans:

- [x] 64-gap-family-retrieval-improvements-01-PLAN.md - Fix one or more measured gap families and prove improvement.

### Phase 65: v2.5 Product Proof And Release Gate

**Goal**: Maintainers can ship or hold v2.5 variants using multi-repo proof, docs, and release gates.

**Depends on**: Phases 61-64

**Requirements**: PROOF-01, PROOF-02, PROOF-03, PROOF-04

**Success Criteria**:

1. Product proof states beat/match/trail status per corpus and variant.
2. Release gate blocks neutral, mixed, unsafe, or too-expensive defaults.
3. Docs tell users which retrieval mode to use today.
4. Workspace validation and source-free E2E proof pass.

**Plans**: 1 plan

Plans:

- [x] 65-v25-product-proof-release-gate-01-PLAN.md - Finalize measured proof, docs, and release gates.

### Phase 66: Test Recall Evaluation Channel

**Goal**: Maintainers can measure validation-test recall through the dedicated related-tests channel instead of treating tests as absent when the target-file ranking is full.

**Depends on**: Phase 65

**Requirements**: GAP-03, PROOF-01

**Success Criteria**:

1. Test Recall@10 is measured from `recommended_tests`.
2. Target-file ranking behavior is not degraded by forced test-slot reservation.
3. Related-test ordering preserves raw score differences hidden by capped public confidence.
4. The product proof still blocks default promotion until every corpus beats lexical.

**Plans**: 1 plan

Plans:

- [x] 66-test-recall-eval-channel-01-PLAN.md - Correct the validation-test evaluation channel and prove the result.

### Phase 67: Retrievable Target Eval Denominator

**Goal**: Maintainers can evaluate retrieval quality against files that existed in the parent snapshot while preserving the full source-free patch audit list.

**Depends on**: Phase 66

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. Historical eval emits `retrievalTargetFiles`.
2. `safeChangedFiles` continues to show the full safe patch surface.
3. Recall, MRR, token ROI, role recall, missing-file, and gap metrics use retrievable parent-snapshot targets.
4. Product proof remains blocked unless every corpus beats lexical.

**Plans**: 1 plan

Plans:

- [x] 67-retrievable-target-eval-denominator-01-PLAN.md - Make retrieval metrics use parent-snapshot retrievable targets.

### Phase 69: Channel-Aware Product Proof Gate

**Goal**: Maintainers can promote default local retrieval when context recall beats lexical and validation-test recall is proven through the dedicated test channel.

**Depends on**: Phase 67

**Requirements**: PROOF-01, PROOF-02, GAP-03

**Success Criteria**:

1. Product proof separates context Recall@10 from validation-test Recall@10.
2. Release gate promotes only when required corpora beat lexical on context recall and meet the test-recall floor.
3. Proof notes preserve all-file recall transparency without treating tests as both context targets and validation commands.
4. Source-free JSON proof reports `releaseGate.decision = "promote"`.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase69-channel-aware-product-proof.md`
- [x] `.ctxhelm/e2e/phase69-channel-scoped-governance-proof.json`

### Phase 70: Real-Client MCP Proof Refresh

**Goal**: Maintainers can verify that Codex CLI and Claude Code still invoke ctxhelm through actual MCP client paths after the Phase 69 promotion.

**Depends on**: Phase 69

**Requirements**: AGENT-01 follow-up evidence

**Success Criteria**:

1. Codex CLI real-client wrapper passes deterministic protocol proof first.
2. Claude Code real-client wrapper passes deterministic protocol proof first.
3. Both wrappers record server-side `prepare_task` and `get_pack` evidence with an explicit repo path.
4. Docs keep Cursor/OpenCode real-client tool-call proof out of scope until machine-checkable client proof exists.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase70-real-client-mcp-proof.md`

### Phase 71: Archive Artifact Dampening

**Goal**: Maintainers can reduce ctxhelm planning-archive retrieval noise without excluding archived evidence from search.

**Depends on**: Phase 69

**Requirements**: GAP-01, GAP-02, RANK-02

**Success Criteria**:

1. `.planning/milestones/**` and `.planning/e2e/**/*.json` stay searchable but no longer dominate generic lexical retrieval.
2. Symbol budget reserve activates only when archive lexical artifacts are present.
3. The fixed two-repo proof still promotes default local retrieval.
4. ctxhelm protected evidence miss-rate improves on the current-history proof without changing RefactoringMiner.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase71-archive-artifact-dampening.md`
- [x] `.ctxhelm/e2e/phase71-archive-artifact-dampening-proof.json`

### Phase 72: Broader Repeated-Lift Validation

**Goal**: Maintainers can probe more local repositories, improve validation-test recall seeding, and keep broader promotion gaps explicit.

**Depends on**: Phase 69, Phase 71

**Requirements**: PROOF-01, PROOF-02, GAP-03

**Success Criteria**:

1. The required fixed two-repo proof still promotes after test-selection changes.
2. Related-test selection can return up to 10 tests to align with Test Recall@10.
3. Related-test discovery uses co-changed and dependency-neighbor source files as additional seeds.
4. Broader probe results are documented honestly when they block promotion.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase72-broader-repeated-lift-validation.md`

### Phase 73: Broader Fixed-Corpus Fixture

**Goal**: Maintainers can rerun the broader probe from a pinned committed config instead of a temporary manifest.

**Depends on**: Phase 72

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. The broader probe config is committed under `.planning/e2e`.
2. External repository paths are relative to the config file.
3. Repository heads are pinned so ctxhelm development commits do not silently change the probe.
4. Docs state that the broader fixture is optional and currently blocks broader promotion.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json`
- [x] `.planning/e2e/2026-05-30-phase73-broader-fixed-corpus-fixture.md`

### Phase 74: Protected Evidence Diagnostics

**Goal**: Maintainers can tell whether protected-evidence misses are actual
retrievable target misses or non-target exact/symbol pressure.

**Depends on**: Phase 73

**Requirements**: PROOF-01, PROOF-02, RANK-02

**Success Criteria**:

1. Protected evidence files include source-free retrieval-target status and file role.
2. Protected evidence summaries report total, retrieval-target, and non-target miss counts.
3. Product-proof corpus verdicts expose protected retrieval-target miss-rate.
4. Required and broader proofs document the remaining target misses without changing retrieval ranking.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase74-protected-evidence-diagnostics.md`
- [x] `.ctxhelm/e2e/phase74-protected-evidence-diagnostics-proof.json`
- [x] `.ctxhelm/e2e/phase74-broader-protected-evidence-diagnostics-proof.json`

### Phase 75: Parent-Bounded History And Test Reserve

**Goal**: Historical eval snapshots preserve bounded prior co-change history,
and validation-test selection protects tests that co-change with target files.

**Depends on**: Phase 74

**Requirements**: PROOF-01, PROOF-02, GAP-03

**Success Criteria**:

1. Parent snapshots can use source-free commit/path history without a full Git checkout.
2. The eval-history sidecar is excluded from inventory and context selection.
3. Co-changed validation tests get a reserved selection pass before generic matches.
4. Required proof still promotes and broader proof records whether VeriSchema improves.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase75-parent-history-test-reserve.md`
- [x] `.ctxhelm/e2e/phase75-parent-history-test-reserve-proof.json`
- [x] `.ctxhelm/e2e/phase75-broader-parent-history-test-reserve-proof.json`

### Phase 76: Parent-Bounded Validation History

**Goal**: Historical eval snapshots use source-free parent-bounded history for
validation-test discovery without using partial snapshot history as a general
non-test target ranking signal.

**Depends on**: Phase 75

**Requirements**: PROOF-01, PROOF-02, GAP-03

**Success Criteria**:

1. Historical eval distinguishes full history from validation-only history.
2. Parent snapshots use sidecar history for related-test discovery and command generation.
3. Partial snapshot history does not perturb non-test target context ranking.
4. Required proof still promotes and broader proof records VeriSchema validation-test movement.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase76-parent-bounded-validation-history.md`
- [x] `.ctxhelm/e2e/phase76-parent-bounded-validation-history-proof.json`
- [x] `.ctxhelm/e2e/phase76-broader-parent-bounded-validation-history-proof.json`

### Phase 77: Validation Command Coverage

**Goal**: Broad multi-area smoke/eval tasks can recommend suite-level fallback
commands and prove effective validation coverage without hiding raw top-10 test
recall.

**Depends on**: Phase 76

**Requirements**: PROOF-01, PROOF-02, GAP-03

**Success Criteria**:

1. Broad validation tasks add fallback commands after targeted test commands.
2. Historical eval reports raw Test Recall@10 and command-backed effective validation recall separately.
3. Product proof uses effective validation recall for validation floors while preserving raw test recall diagnostics.
4. Required proof still promotes and broader proof records remaining corpus blockers.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase77-validation-command-coverage.md`
- [x] `.ctxhelm/e2e/phase77-validation-command-coverage-proof.json`
- [x] `.ctxhelm/e2e/phase77-broader-validation-command-coverage-proof.json`

### Phase 78: Ceiling-Aware Broader Gate

**Goal**: Safe perfect lexical-ceiling matches should not block broader
production-readiness proof when validation is healthy and protected target
misses are zero.

**Depends on**: Phase 77

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. Ordinary non-ceiling `match` verdicts still block promotion.
2. Perfect context-channel ceiling matches with zero protected target misses can promote.
3. Product-proof checker accepts only `beat` or safe perfect-ceiling `match` verdicts.
4. Broader fixed-corpus proof promotes while preserving protected-miss diagnostics.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase78-ceiling-aware-broader-gate.md`
- [x] `.ctxhelm/e2e/phase78-ceiling-aware-broader-proof.json`

### Phase 79: Protected Target Floors

**Goal**: Protected source/config/governance evidence should survive standard
budget selection more reliably, while archive artifacts remain available as
fallback context.

**Depends on**: Phase 78

**Requirements**: PROOF-01, PROOF-02, RANK-02

**Success Criteria**:

1. Source lexical and source symbol candidates receive bounded floors under no-active-context tasks.
2. Exact config candidates and agent setup governance docs receive bounded floors.
3. Archive artifacts from `.planning/e2e`, `.planning/phases`, and `.planning/milestones` are deferred during fill.
4. Required and broader proofs still promote and record protected target miss movement.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase79-protected-target-floors.md`
- [x] `.ctxhelm/e2e/phase79-protected-target-floors-proof.json`
- [x] `.ctxhelm/e2e/phase79-broader-protected-target-floors-proof.json`

### Phase 80: Unique Symbol Floor Accounting

**Goal**: Symbol floor limits should count unique newly selected files, not
duplicate attempts for files already selected by lexical floors.

**Depends on**: Phase 79

**Requirements**: PROOF-01, PROOF-02, RANK-02

**Success Criteria**:

1. Source-symbol floor runs before governance/doc fill for no-active-context tasks.
2. Source-symbol and general symbol floors count only newly selected files against their limits.
3. Required and broader proofs promote with protected retrieval-target miss-rate `0.0` in measured corpora.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase80-unique-symbol-floor.md`
- [x] `.ctxhelm/e2e/phase80-unique-symbol-floor-proof.json`
- [x] `.ctxhelm/e2e/phase80-broader-unique-symbol-floor-proof.json`

### Phase 81: Warm Cache Latency Proof

**Goal**: Cached historical eval reports should report warm lookup runtime,
not stale cold-run timings.

**Depends on**: Phase 80

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. Cache-hit reports show warm lookup runtime with zero commit-loop, ranking,
   pack/compiler, git sample, and slow-commit timings.
2. Cold and warm product-proof artifacts promote on the four-repo fixed corpus.
3. Warm proof records cache hits and no cache misses for every measured corpus.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase81-warm-cache-latency.md`
- [x] `.planning/e2e/2026-05-30-phase81-warm-cache-proof-config.json`
- [x] `.ctxhelm/e2e/phase81-warm-cache-cold-proof.json`
- [x] `.ctxhelm/e2e/phase81-warm-cache-warm-proof.json`

### Phase 82: Warm Cache Release Gate

**Goal**: Warm-cache product proof should be an enforceable release gate, not
only a diagnostic artifact.

**Depends on**: Phase 81

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. Cache-hit product proofs block if warm runtime carries stale cold timings.
2. Cache-hit product proofs block if warm lookup runtime exceeds `1000ms`.
3. Clean cold/warm proof replay still promotes and records warm-cache notes.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase82-warm-cache-gate.md`
- [x] `.ctxhelm/e2e/phase82-warm-cache-gate-cold-proof.json`
- [x] `.ctxhelm/e2e/phase82-warm-cache-gate-warm-proof.json`

### Phase 83: Context Divergence Accounting

**Goal**: Context-vs-all-file corpus divergence should be machine-checkable,
not only explained in prose notes.

**Depends on**: Phase 69, Phase 77, Phase 82

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. Product-proof corpus verdicts expose context-vs-all-file deltas for ctxhelm
   and lexical.
2. Product-proof promotion blocks unexplained all-file lexical deficits.
3. The source-free product-proof checker fails if divergence fields are missing
   or if an all-file deficit is not explained.
4. The broader four-repo proof still promotes with explained RefactoringMiner
   and ReAgent all-file deficits.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase83-context-divergence-accounting.md`
- [x] `.ctxhelm/e2e/phase83-context-divergence-proof.json`

### Phase 84: Broad Scope Dependency Floors

**Goal**: Broad workflow/eval/lint tasks should be visible in eval output and
should not lose dependency source evidence to unrelated context when the task
spans many files.

**Depends on**: Phase 77, Phase 80, Phase 83

**Requirements**: GAP-02, GAP-04, PROOF-01

**Success Criteria**:

1. Prepare-task emits `multi_area_task` diagnostics for broad workflow/eval/lint
   prompts.
2. Historical eval JSON includes `broadScopeCommitCount` and per-commit
   `broadScopeTask`.
3. Dependency source floors activate only for broad-scope tasks, avoiding the
   RefactoringMiner regression seen with an unconditional floor.
4. The broader four-repo proof still promotes and improves VeriSchema source
   recall.

**Evidence**:

- [x] `.planning/e2e/2026-05-31-phase84-broad-scope-dependency-floors.md`
- [x] `.ctxhelm/e2e/phase84-broad-scope-dependency-proof.json`

### Phase 85: Broad Context Areas

**Goal**: Broad multi-area prepare-task plans and packs should expose adjacent
repository areas as source-free guidance without changing protected target-file,
test, or validation budgets.

**Depends on**: Phase 84

**Requirements**: GAP-02, GAP-04, PROOF-01

**Success Criteria**:

1. `ContextPlan` exposes typed additive `contextAreas`.
2. `prepare-task` populates `contextAreas` only for broad multi-area tasks.
3. Packs render context areas as inspection hints after risk flags.
4. Focused tests cover the public JSON shape, source-free contract, and
   multi-area diagnostics.
5. Proof documents that broad fixed-corpus quality metrics are unchanged and
   the warm-cache proof still promotes.

**Evidence**:

- [x] `.planning/e2e/2026-05-31-phase85-broad-context-areas.md`
- [x] `.ctxhelm/e2e/phase85-context-areas-warm-proof.json`

### Phase 86: Python Package Re-Export Graph Coverage

**Goal**: Python package re-exports should appear in graph retrieval so
`from package import Symbol` tasks can surface concrete module files without
general recursive graph expansion.

**Depends on**: Phase 84, Phase 85

**Requirements**: GAP-02, GAP-04

**Success Criteria**:

1. Python import extraction records imported submodule paths for
   `from module import member`.
2. Dependency resolution recognizes `package/__init__.py`.
3. Related dependency expansion adds bounded `python_reexport` edges from an
   anchor through package `__init__.py` to re-exported modules.
4. Focused dependency tests cover absolute submodule imports, relative
   `from . import module`, and package re-export expansion.
5. Broader proof documents no protected-target regression even if top-10 recall
   remains flat.

**Evidence**:

- [x] `.planning/e2e/2026-05-31-phase86-python-package-reexports.md`

### Phase 103: Broad Fixed-Corpus Floors

**Goal**: Product proofs should fail broad pinned-corpus regressions even when
the aggregate release verdict still promotes.

**Depends on**: Phase 92, Phase 101

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. `scripts/check-product-proof.py` enforces recorded floors for the pinned
   `phase92-area-aware-gap-taxonomy-2026-05-31` four-repo corpus.
2. A fixture below the VeriSchema broad file-recall floor fails the checker.
3. The known-good broad proof passes the stricter checker.

**Evidence**:

- [x] `.planning/e2e/2026-05-31-phase103-broad-fixed-corpus-floors.md`

### Phase 104: Context Area Next-Read Paths

**Goal**: Broad context areas should give agents concrete source/docs paths to
read next when candidate pressure remains below the selected target-file
budget.

**Depends on**: Phase 95, Phase 100, Phase 103

**Requirements**: GAP-02, GAP-04, PROOF-01

**Success Criteria**:

1. `ContextArea` includes additive source-free `nextReadPaths` and
   `unselectedCount`.
2. Broad context areas include docs candidates without spending target-file
   ranking budget.
3. Packs render `Next reads` for surfaced areas and zero-selected areas.
4. The product-proof checker fails cleanly when an embedded repository report
   is missing.
5. Focused tests and the available three-repo proof pass while the blocked
   RefactoringMiner checkout is documented honestly.

**Evidence**:

- [x] `.planning/e2e/2026-05-31-phase104-context-area-next-read-paths.md`
- [x] `.ctxhelm/e2e/phase104-context-area-next-read-paths-no-refminer-proof.json`

### Phase 105: History-Unavailable Embedded Reports

**Goal**: Benchmark and product-proof output should stay machine-checkable when
git history sampling is unavailable or times out.

**Depends on**: Phase 101, Phase 103, Phase 104

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. Historical eval degrades git history sampling failures to an embedded
   zero-commit report instead of returning a repository-level error.
2. Benchmark repo reports keep `report: {...}` and add a source-free
   history-unavailable error when no commits were evaluated.
3. Product-proof verdicts mark that corpus as `insufficient_evidence` and
   block promotion.
4. Degraded zero-commit reports are not cached as valid historical eval
   results.
5. Focused tests and a CLI proof fixture cover the behavior.

**Evidence**:

- [x] `.planning/e2e/2026-05-31-phase105-history-unavailable-report.md`
- [x] `.ctxhelm/e2e/phase105-history-unavailable-proof.json`

### Phase 106: Real-Client Request Evidence Hardening

**Goal**: Codex CLI and Claude Code real-client proof artifacts should show
what was observed through the real client path without storing raw MCP traffic
or source-bearing prompt data.

**Depends on**: Phase 70, Phase 102, Phase 105

**Requirements**: AGENT-01, PROOF-01

**Success Criteria**:

1. Codex and Claude real-client smoke evidence keeps the existing client,
   ctxhelm, repo, `prepare_task`, and `get_pack` fields for compatibility.
2. Evidence adds a request evidence schema version, request-log SHA-256, request
   line count, explicit repo tool-call count, and sanitized observed tool-call
   metadata.
3. Evidence-directory runs also write sanitized request-summary JSON sidecars
   without raw request logs, task text, prompt text, or source snippets.
4. Claude semantic smoke evidence records whether observed tool calls matched
   semantic provider/model/dimension requirements when semantic smoke mode is
   enabled.
5. Focused script-contract tests and deterministic wrapper smokes cover the
   hardened artifact shape.

**Evidence**:

- [x] `.planning/e2e/2026-05-31-phase106-real-client-request-evidence.md`

### Phase 107: Hydrated Four-Repo Proof Path

**Goal**: The broad proof should hydrate all configured repositories with
embedded, source-free reports instead of hanging, returning `report: null`, or
downgrading the proof to a smaller corpus.

**Depends on**: Phase 92, Phase 93, Phase 105

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. Historical commit collection falls back when rename detection is too slow on
   a large repository.
2. Parent snapshot hydration uses bounded path-limited git operations instead
   of full-tree scans or whole-revision archive extraction.
3. Bad single parent-snapshot paths are skipped without blocking the whole
   repository proof.
4. The cold four-repo proof embeds reports for RefactoringMiner, ctxhelm,
   ReAgent, and VeriSchema, even if it still blocks on runtime.
5. The warm/cache four-repo proof promotes with all four corpus verdicts.

**Evidence**:

- [x] `.planning/e2e/2026-05-31-phase107-hydrated-four-repo-proof.md`
- [x] `.ctxhelm/e2e/phase107-hydrated-four-repo-cold-proof.json`
- [x] `.ctxhelm/e2e/phase107-hydrated-four-repo-warm-proof.json`

## Requirement Coverage

| Requirement | Phase |
|-------------|-------|
| BASE-01 | Phase 61 |
| BASE-02 | Phase 61 |
| BASE-03 | Phase 61 |
| BASE-04 | Phase 61 |
| EMBED-01 | Phase 62 |
| EMBED-02 | Phase 62 |
| EMBED-03 | Phase 62 |
| EMBED-04 | Phase 62 |
| RANK-01 | Phase 63 |
| RANK-02 | Phase 63 |
| RANK-03 | Phase 63 |
| RANK-04 | Phase 63 |
| GAP-01 | Phase 64 |
| GAP-02 | Phase 64 |
| GAP-03 | Phase 64 |
| GAP-04 | Phase 64 |
| PROOF-01 | Phase 65 |
| PROOF-02 | Phase 65 |
| PROOF-03 | Phase 65 |
| PROOF-04 | Phase 65 |
| GAP-03 | Phase 66 |
| PROOF-01 | Phase 66 |
| PROOF-01 | Phase 67 |
| PROOF-02 | Phase 67 |
| PROOF-01 | Phase 69 |
| PROOF-02 | Phase 69 |
| GAP-03 | Phase 69 |
| AGENT-01 | Phase 70 |
| GAP-01 | Phase 71 |
| GAP-02 | Phase 71 |
| RANK-02 | Phase 71 |
| PROOF-01 | Phase 72 |
| PROOF-02 | Phase 72 |
| GAP-03 | Phase 72 |
| PROOF-01 | Phase 73 |
| PROOF-02 | Phase 73 |
| PROOF-01 | Phase 74 |
| PROOF-02 | Phase 74 |
| RANK-02 | Phase 74 |
| PROOF-01 | Phase 75 |
| PROOF-02 | Phase 75 |
| GAP-03 | Phase 75 |
| PROOF-01 | Phase 76 |
| PROOF-02 | Phase 76 |
| GAP-03 | Phase 76 |
| PROOF-01 | Phase 77 |
| PROOF-02 | Phase 77 |
| GAP-03 | Phase 77 |
| PROOF-01 | Phase 78 |
| PROOF-02 | Phase 78 |
| PROOF-01 | Phase 79 |
| PROOF-02 | Phase 79 |
| RANK-02 | Phase 79 |
| PROOF-01 | Phase 80 |
| PROOF-02 | Phase 80 |
| RANK-02 | Phase 80 |
| PROOF-01 | Phase 81 |
| PROOF-02 | Phase 81 |
| PROOF-01 | Phase 82 |
| PROOF-02 | Phase 82 |
| PROOF-01 | Phase 83 |
| PROOF-02 | Phase 83 |
| GAP-02 | Phase 84 |
| GAP-04 | Phase 84 |
| PROOF-01 | Phase 84 |
| GAP-02 | Phase 85 |
| GAP-04 | Phase 85 |
| PROOF-01 | Phase 85 |
| GAP-02 | Phase 86 |
| GAP-04 | Phase 86 |
| GAP-02 | Phase 92 |
| GAP-04 | Phase 92 |
| PROOF-01 | Phase 92 |
| PROOF-02 | Phase 92 |
| GAP-02 | Phase 93 |
| GAP-04 | Phase 93 |
| PROOF-01 | Phase 93 |
| PROOF-02 | Phase 93 |
| GAP-02 | Phase 94 |
| GAP-04 | Phase 94 |
| PROOF-01 | Phase 94 |
| PROOF-02 | Phase 94 |
| GAP-04 | Phase 95 |
| PROOF-01 | Phase 95 |
| PROOF-02 | Phase 95 |
| PROOF-01 | Phase 103 |
| PROOF-02 | Phase 103 |
| GAP-02 | Phase 104 |
| GAP-04 | Phase 104 |
| PROOF-01 | Phase 104 |
| PROOF-01 | Phase 105 |
| PROOF-02 | Phase 105 |
| AGENT-01 | Phase 106 |
| PROOF-01 | Phase 106 |
| PROOF-01 | Phase 107 |
| PROOF-02 | Phase 107 |

**Coverage:** 20/20 v2.5 requirements mapped, with Phases 66-107 as measured follow-ups for proof/eval correctness gaps, real-client evidence, archive-noise reduction, broader validation, fixed-corpus reproducibility, protected-evidence diagnostics, parent-bounded history/test reservation, validation-only historical eval history, validation-command coverage, lexical-ceiling proof semantics, protected target floors, symbol-floor accounting, warm-cache runtime proof, warm-cache release gating, context-vs-all-file divergence accounting, broad-scope dependency source floors, broad context-area hints, Python package re-export graph coverage, validation gap accounting, broad source-area candidates, fast inventory freshness, packaged release-gate proof, broad context-area eval coverage, area-aware gap taxonomy with clean large-repo warm proof, source-free index caching for cold large-repo planner runtime, wider context-area guidance for broad tasks, progressive area guidance in generated packs, MCP context-area resources, broad governance task classification, progressive broad classification, source-free context-area read batches, resource-backed gap summaries, release-gated gap summary contracts, explicit-repo MCP resource consumption, broad fixed-corpus floor gates, context-area next-read paths, history-unavailable embedded reports, source-free real-client request evidence, and hydrated four-repo proof evidence. No orphaned v2.5 requirements.

## Progress

**Execution Order:**
Phases execute in numeric order: 61 -> 62 -> 63 -> 64 -> 65 -> 66 -> 67 -> 69 -> 70 -> 71 -> 72 -> 73 -> 74 -> 75 -> 76 -> 77 -> 78 -> 79 -> 80 -> 81 -> 82 -> 83 -> 84 -> 85 -> 86 -> 87 -> 88 -> 89 -> 90 -> 91 -> 92 -> 93 -> 94 -> 95 -> 96 -> 97 -> 98 -> 99 -> 100 -> 101 -> 102 -> 103 -> 104 -> 105 -> 106 -> 107

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 61. Multi-Repo Quality Baselines | 1/1 | Complete | 2026-05-22 |
| 62. Production Local Embedding Quality | 1/1 | Complete | 2026-05-30 |
| 63. Reranker And Fusion Promotion | 1/1 | Complete | 2026-05-30 |
| 64. Gap-Family Retrieval Improvements | 1/1 | Complete | 2026-05-30 |
| 65. v2.5 Product Proof And Release Gate | 1/1 | Complete | 2026-05-30 |
| 66. Test Recall Evaluation Channel | 1/1 | Complete | 2026-05-30 |
| 67. Retrievable Target Eval Denominator | 1/1 | Complete | 2026-05-30 |
| 69. Channel-Aware Product Proof Gate | Evidence artifact | Complete | 2026-05-30 |
| 70. Real-Client MCP Proof Refresh | Evidence artifact | Complete | 2026-05-30 |
| 71. Archive Artifact Dampening | Evidence artifact | Complete | 2026-05-30 |
| 72. Broader Repeated-Lift Validation | Evidence artifact | Complete | 2026-05-30 |
| 73. Broader Fixed-Corpus Fixture | Evidence artifact | Complete | 2026-05-30 |
| 74. Protected Evidence Diagnostics | Evidence artifact | Complete | 2026-05-30 |
| 75. Parent-Bounded History And Test Reserve | Evidence artifact | Complete | 2026-05-30 |
| 76. Parent-Bounded Validation History | Evidence artifact | Complete | 2026-05-30 |
| 77. Validation Command Coverage | Evidence artifact | Complete | 2026-05-30 |
| 78. Ceiling-Aware Broader Gate | Evidence artifact | Complete | 2026-05-30 |
| 79. Protected Target Floors | Evidence artifact | Complete | 2026-05-30 |
| 80. Unique Symbol Floor Accounting | Evidence artifact | Complete | 2026-05-30 |
| 81. Warm Cache Latency Proof | Evidence artifact | Complete | 2026-05-30 |
| 82. Warm Cache Release Gate | Evidence artifact | Complete | 2026-05-30 |
| 83. Context Divergence Accounting | Evidence artifact | Complete | 2026-05-30 |
| 84. Broad Scope Dependency Floors | Evidence artifact | Complete | 2026-05-31 |
| 85. Broad Context Areas | Evidence artifact | Complete | 2026-05-31 |
| 86. Python Package Re-Export Graph Coverage | Evidence artifact | Complete | 2026-05-31 |
| 87. Validation Gap Accounting | Evidence artifact | Complete | 2026-05-31 |
| 88. Broad Source-Area Candidates | Evidence artifact | Complete | 2026-05-31 |
| 89. Fast Inventory Freshness | Evidence artifact | Complete | 2026-05-31 |
| 90. Packaged Release Gate | Evidence artifact | Complete | 2026-05-31 |
| 91. Broad Context-Area Eval | Evidence artifact | Complete | 2026-05-31 |
| 92. Area-Aware Gap Taxonomy And Large-Repo Warm Proof | Evidence artifact | Complete | 2026-05-31 |
| 93. Source-Free Index Cache | Evidence artifact | Complete | 2026-05-31 |
| 94. Broad Context-Area Cap | Evidence artifact | Complete | 2026-05-31 |
| 95. Progressive Area Pack Guidance | Evidence artifact | Complete | 2026-05-31 |
| 96. Context Area MCP Resources | Evidence artifact | Complete | 2026-05-31 |
| 97. Broad Governance Classification | Evidence artifact | Complete | 2026-05-31 |
| 98. Progressive Broad Classification | Evidence artifact | Complete | 2026-05-31 |
| 99. Context Area Read Batches | Evidence artifact | Complete | 2026-05-31 |
| 100. Resource-Backed Gap Summaries | Evidence artifact | Complete | 2026-05-31 |
| 101. Release-Gated Gap Summary Contract | Evidence artifact | Complete | 2026-05-31 |
| 102. Explicit-Repo MCP Resource Consumption | Evidence artifact | Complete | 2026-05-31 |
| 103. Broad Fixed-Corpus Floors | Evidence artifact | Complete | 2026-05-31 |
| 104. Context Area Next-Read Paths | Evidence artifact | Complete | 2026-05-31 |
| 105. History-Unavailable Embedded Reports | Evidence artifact | Complete | 2026-05-31 |
| 106. Real-Client Request Evidence Hardening | Evidence artifact | Complete | 2026-05-31 |
| 107. Hydrated Four-Repo Proof Path | Evidence artifact | Complete | 2026-05-31 |
| 108. Cold Git Bounds | Evidence artifact | Complete | 2026-05-31 |
| 109. Environment Health Verdicts | Evidence artifact | Complete | 2026-05-31 |

---
*Roadmap created: 2026-05-22*
