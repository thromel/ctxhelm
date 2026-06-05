# Milestones

## Active

### v2.5 Production Retrieval Quality

**Goal:** Prove and improve retrieval quality across real repositories so production local embeddings, reranking, graph/test/history fixes, and learned fusion can beat lexical baseline while staying local-first and source-safe.

**Status:** Active: 2026-05-22. Phases 61-67 complete; Phase 69 promoted default local retrieval under the channel-aware product-proof gate; Phase 70 refreshed Codex CLI and Claude Code real-client MCP proof; Phase 71 reduced ctxhelm archive-artifact retrieval noise; Phase 72 broadened repeated-lift validation and improved validation-test recall seeding; Phase 73 committed a pinned broader fixed-corpus probe; Phase 74 separated protected retrieval-target misses from non-target protected pressure; Phase 75 restored parent-bounded eval history and reserved co-changed validation tests; Phase 76 split historical parent history into validation-only mode and enriched co-changed test commands; Phase 77 added broad validation fallback commands and effective validation-command coverage metrics; Phase 78 made the broader proof gate ceiling-aware and promoted the fixed four-repo corpus; Phase 79 added protected target floors; Phase 80 fixed symbol-floor duplicate accounting and cleared protected target misses; Phase 81 fixed warm-cache runtime reporting; Phase 82 made warm-cache runtime enforceable; Phase 83 made context-vs-all-file divergence machine-checkable; Phase 84 added broad-scope accounting and scoped dependency source floors; Phase 85 added source-free context-area hints for broad prepare-task plans and packs; Phase 86 added bounded Python package re-export graph coverage; Phase 87 fixed validation gap accounting; Phase 88 added broad source-area candidates; Phase 89 reduced inventory freshness overhead and promoted the broader release proof; Phase 90 proved the packaged release gate from a clean worktree with the broad benchmark enabled; Phase 91 added broad context-area recall metrics and implementation-first area ordering; Phase 92 added area-aware gap taxonomy, clean large-repo warm proof, historical eval cache invalidation, and large-repo eval runtime caching; Phase 93 added source-free symbol/dependency index caches and promoted clean cold large-repo proof; Phase 94 increased broad context-area guidance while preserving target-file/test/validation metrics; Phase 95 made broad context-area pack guidance progressive and actionable; Phase 96 exposed broad context areas as source-free MCP resources; Phase 97 improved broad governance/proof task classification; Phase 98 split broad context-area classification from target-file source floors for archive/docs tasks; Phase 99 added source-free role buckets, path families, and next-read batches to context-area resources; Phase 100 made retrieval-gap summaries resource-backed; Phase 101 made the resource-backed gap shape release-gated; Phase 102 made explicit-repo MCP resources consumable from non-repo server cwd; Phase 103 added broad fixed-corpus floors; Phase 104 added source-free next-read paths and unselected counts to broad context areas; Phase 105 made history-unavailable benchmark repos produce embedded insufficient-evidence reports; Phase 106 hardened Codex/Claude real-client proof artifacts with source-free request metadata; Phase 107 fixed the hydrated full four-repo proof path; Phase 108 bounded cold Git failures; Phase 109 made environment-health blockers machine-readable; Phase 110 promoted the clean cold full-fixture proof; Phase 111 wired that proof into the packaged release gate; Phase 112 passed the clean packaged release gate with the clean fixture proof required; Phase 113 recorded source-free release-candidate status and archive-first distribution decisions; Phase 114 published and verified the public v1.1.0 archive release; Phase 115 verified the public archive install path from GitHub release assets; Phase 116 refreshed optional real-client evidence against the public archive binary; Phase 117 added source-free role signals to broad context-area guidance; Phase 118 added safe-inventory scope metadata to context-area MCP resources; Phase 119 removed an observed `CTXHELM_HOME` test-environment race from `ctxhelm-index`; Phase 120 added public CI release-gate enforcement; Phase 121 moved public CI JavaScript actions to Node 24 and verified no Node 20 warning text; Phase 122 fixed public archive real-client smoke compatibility with post-release MCP protocol assertions; Phase 123 added source-free context-area coverage profiles; Phase 124 added source-free context-area inspection strategies; Phase 125 added source-free lexical comparison summaries to product proof; Phase 126 added agent-evidence lexical comparison for the actual context/test/validation evidence set; Phase 127 added a narrow-plan validation-test reserve that eliminates target-file lexical trailing corpora without regressing broad context-area plans; Phase 128 added broad operational floors that clear protected target misses across all four measured corpora; Phase 129 added public release freshness proof; Phase 130 published and verified a current public v1.1.1 archive release; Phase 131 added product-aware public release freshness and published/verified v1.1.2 with Claude Code evidence; Phase 132 added and verified a source-free Claude Code workflow eval with real-client explicit-repo `prepare_task` and `get_pack` evidence; Phase 133 sharpened README positioning, current proof snapshot, and release-doc drift checks; Phase 134 published and verified the current public v1.1.3 archive release; Phase 135 added Homebrew/crates readiness checks without publishing those channels; Phase 136 published and verified the current public v1.1.4 archive release; Phase 137 published and verified the public Homebrew tap; Phase 138 published and verified the current public v1.1.5 archive and refreshed Homebrew tap; Phase 139 introduced a short-lived interim product name while preserving the `ctxhelm` compatibility surface; Phase 140 published and verified the public v1.1.6 archive and Homebrew tap; Phase 141 finalized ctxhelm after availability review and bumped the release line to v1.1.7; Phase 142 published and verified the ctxhelm v1.1.7 archive and Homebrew tap; Phase 143 added paired Claude Code agent-run outcome proof showing `ctxhelm-brief` preserved target coverage while reducing irrelevant reads.

Latest follow-up: Phase 249 adds `ctxhelm governor decide`, a source-free context-governor report for task-conditioned retrieval, budget, memory, validation, semantic, and policy-profile decisions. `scripts/smoke-governor.sh` proves selected/omitted evidence, rollout controls, active learned-profile visibility, policy apply/rollback reflection, release-gate wiring, and source sentinel rejection. Phase 248 adds `ctxhelm inspector serve`, a localhost-only, read-only diagnostic shell for pack inspector, graph neighborhood, setup status, and shell health routes. `scripts/smoke-inspector.sh` now starts the shell, fetches `/`, `/pack-inspector.json`, `/graph.html`, `/graph.json`, `/setup-status.json`, and `/health.json`, and rejects source sentinel leakage. Phase 247 added optional Cursor Agent CLI and OpenCode real-client smoke wrappers with source-free server-side request evidence. `scripts/smoke-opencode-real-client.sh` passes locally with OpenCode `1.14.25` and records explicit-repo `prepare_task` plus `get_pack` tool calls; `scripts/smoke-cursor-real-client.sh` has the same proof contract and uses an isolated temporary Cursor workspace, but current local required proof is auth-blocked because Cursor Agent CLI `3.6.21` reports not logged in. The release gate now records Cursor/OpenCode optional proof status and can require OpenCode proof without requiring Cursor auth. Phase 246 added a release-gated agent-native fallback smoke for thin repo-local guidance and disconnected source-free cards. `scripts/smoke-agent-native-fallback.sh` proves `ctxhelm init --cursor --claude --opencode`, `setup-check`, and `cards fallback --target-agent codex` produce repo-local, bounded guidance without broad static source injection or source sentinel leakage; the release gate and release docs contract now require that smoke. Phase 245 added a strict multi-task Codex real-client suite and fixed the harness/guidance first-read gap it exposed. Earlier suite evidence showed ctxhelm could surface `scripts/e2e-agent-run-codex.sh` for `Improve Codex agent-run harness`, but the `ctxhelm-plan` lane did not read it because the script ranked outside Codex's first native-read batch. Phase 245 adds `--suite` aggregation, promotes bounded agent-native guidance implementation surfaces, treats named shell harnesses as source-like for that promotion, isolates spawned Codex runs from Desktop thread environment leakage, forbids bootstrap/setup/superpowers commands, and caps read-only exploration. The source-free artifact `.ctxhelm/e2e/phase245-agent-run-codex-suite-real-bounded-final.json` passes with Codex CLI `0.137.0`, two comparison-eligible tasks, eight comparable ctxhelm lanes, no forbidden commands, no client failures, no ctxhelm evidence misses, no evidence-only targets, no ctxhelm under-read targets, and outcome claim `ctxhelm_improved`: baseline average target-read coverage is `0.75`, while every ctxhelm lane reaches `1.00`.

**Phases planned:** Phases 61-65 plus Phase 66-246 production-readiness follow-ups, 7 planned phase files plus measured proof follow-up artifacts

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

**Latest evidence:** `.planning/e2e/2026-06-03-phase204-agent-run-forbidden-tool-accounting.md`

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
