# R&D Completion Audit - 2026-06-06

## Scope

Audit the current ctxhelm R&D roadmap against the latest source-free proof
artifacts after Phase 283. This audit is not a release claim by itself; it
separates implemented/measured work from remaining research risk.

## Current Proof Summary

| Area | Current evidence | Status |
| --- | --- | --- |
| Codex real-agent outcome | `.ctxhelm/e2e/phase245-agent-run-codex-suite-real-bounded-final.json`, `.ctxhelm/e2e/phase250-agent-run-codex-governor-rd-after.json`, and `.ctxhelm/e2e/phase251-agent-run-codex-rd-breadth-suite.json` | Strong for the measured failure classes. Phase 245 proves a two-task suite where every ctxhelm lane reaches `1.00` target-read coverage vs baseline `0.75`. Phase 250 proves a governor/release R&D task improves from matched baseline with evidence misses to `ctxhelm_improved`. Phase 251 expands this to four R&D task families with 16 comparable ctxhelm lanes and no evidence misses, under-read targets, forbidden commands, or client failures. |
| Claude workflow | `.ctxhelm/e2e/phase250-claude-workflow-refresh.json`, `.ctxhelm/e2e/phase252-agent-run-claude-rd-breadth-suite.json`, `.ctxhelm/e2e/phase254-agent-run-claude-rd-breadth-suite.json`, `.ctxhelm/e2e/phase264-agent-client-availability.json` | Current workflow proof passes with explicit-repo MCP usage and local-only privacy. Phase 254 retried the fresh paired Claude R&D suite with Claude Code `2.1.163`, but all four current preflights reported rate limiting, so the report is correctly `degraded` and not used as retrieval-quality evidence. Phase 264 rechecked client availability: Claude Code `2.1.163` is still rate-limited, while Codex CLI `0.137.0` remains available with deterministic explicit-repo evidence. |
| Experience memory | `.ctxhelm/e2e/phase234-memory-pack-impact-suite.json`, `.ctxhelm/e2e/phase244-agent-run-codex-memory-consumption.json`, `.ctxhelm/e2e/phase253-codex-memory-outcome-suite.json`, `.ctxhelm/e2e/phase255-codex-memory-efficiency-suite.json`, `.planning/e2e/2026-06-06-phase255-codex-memory-efficiency.md` | Local memory selection is source-free and bounded. Six-repo local evidence shows final-pack memory does not add non-targets; Phase 244 proves Codex consumes selected-memory targets on one repo while reducing read-file count from 18 to 5 in the best lane. Phase 253 extends real Codex memory outcome evidence across VeriSchema, ReAgent, and RefactoringMiner: 3/3 comparison-eligible pairs improved target-read coverage in the memory lane. Phase 255 tightens the memory lane and reruns the same three-repo proof: target-read improvement remains 3/3, while memory irrelevant-read improvement moves from 0/3 to 3/3. |
| GraphRAG | `.planning/e2e/2026-06-02-phase179-graph-edge-profiles.md`, `.planning/e2e/2026-06-02-phase180-graph-edge-ablations.md`, `.planning/e2e/2026-06-03-phase181-graph-edge-budget-allocation.md` | Implemented and measured. Edge-family profiles and ablations expose import and Python re-export pressure; Phase 181 applies bounded edge-family budget ordering and promotes on reachable fixtures. |
| Semantic/reranker retrieval | `.planning/e2e/2026-06-02-phase160-bounded-semantic-status-search.md`, `.planning/e2e/2026-06-02-phase161-semantic-gate-contribution-diagnostics.md`, `.planning/e2e/2026-06-06-phase252-semantic-gate-classification.md`, `.planning/e2e/2026-06-06-phase256-semantic-rich-document-rejection.md`, `.planning/e2e/2026-06-06-phase257-safe-local-metadata-reranker.md`, `.planning/e2e/2026-06-06-phase258-reranker-contribution-diagnostics.md`, `.planning/e2e/2026-06-06-phase259-runtime-reranker-protected-floor.md`, `.planning/e2e/2026-06-06-phase260-query-family-reranker-routing.md`, `.planning/e2e/2026-06-06-phase261-routed-reranker-policy.md`, `.planning/e2e/2026-06-06-phase262-routed-reranker-broader-proof.md`, `.planning/e2e/2026-06-06-phase263-routed-reranker-runtime-policy.md`, `.planning/e2e/2026-06-06-phase264-semantic-family-diagnostics.md`, `.planning/e2e/2026-06-06-phase265-semantic-family-stability.md`, `.planning/e2e/2026-06-06-phase266-semantic-support-profiles.md`, `.planning/e2e/2026-06-06-phase267-semantic-support-profile-routing.md`, `.planning/e2e/2026-06-06-phase268-support-profile-routed-semantic.md`, `.planning/e2e/2026-06-06-phase269-semantic-candidate-generation.md`, `.planning/e2e/2026-06-06-phase270-semantic-corroborated-reranker.md`, `.planning/e2e/2026-06-06-phase271-semantic-corroborated-family-diagnostics.md`, `.planning/e2e/2026-06-06-phase272-semantic-corroborated-path-family-diagnostics.md`, `.planning/e2e/2026-06-06-phase273-verischema-semantic-prefilter-cap.md`, `.planning/e2e/2026-06-06-phase274-verischema-python-module-facets.md`, `.planning/e2e/2026-06-06-phase275-semantic-shape-diagnostics.md`, `.planning/e2e/2026-06-06-phase276-reranker-displacement-diagnostics.md`, `.planning/e2e/2026-06-06-phase277-family-budget-semantic-reranker.md`, `.planning/e2e/2026-06-06-phase278-learned-profile-semantic-reranker.md`, `.planning/e2e/2026-06-07-phase280-learned-semantic-policy-artifact.md`, `.planning/e2e/2026-06-07-phase281-learned-semantic-policy-holdout.md`, `.planning/e2e/2026-06-07-phase282-learned-policy-holdout-limit40.md`, `.planning/e2e/2026-06-07-phase283-ranged-semantic-gate.md` | Integrated and measured as optional local signals. Phase 252 fixes gate classification so eval-only reranker regressions no longer make semantic look blocked. Phase 256 rejects richer symbol/dependency semantic search documents. Phase 257 makes the eval reranker preserve protected source evidence and validation-test reserves. Phase 258 adds contribution diagnostics showing RefactoringMiner is reranker-neutral while ctxhelm has strong target-hit lift plus explicit target churn. Phase 259 aligns policy-enabled runtime reranking with the same protected-source floor. Phase 260 adds query-family routing diagnostics. Phase 261 adds an eval-only routed variant. Phase 262 broadens that proof and rejects `symbol_identifier` routing after ReAgent regression, narrowing routing to `commit_clue` only. Phase 263 exposes that narrower route as opt-in runtime policy through `enableQueryFamilyRoutedReranker`, while defaults stay disabled. Phase 264 adds semantic query-family contribution diagnostics and shows selective semantic candidates. Phase 265 broadens those semantic-family checks to `limit 20`, adds commit-level stability counters, and rejects coarse semantic query-family routing. Phase 266 adds semantic-only support profiles and rejects generic "semantic plus any other signal" routing. Phase 267 classifies support profiles as route candidates, mixed holds, or noise holds. Phase 268 implements the resulting eval-only support-profile-routed variant and rejects it as neutral across four repos. Phase 269 adds candidate-generation versus fusion diagnostics. Phase 270 tests semantic-corroborated reranking and rejects global promotion after strong RefactoringMiner lift but regressions on ctxhelm, ReAgent, and VeriSchema. Phase 271 adds contribution diagnostics for that variant and rejects query-family-only routing because RefactoringMiner's clean route candidates are unsafe on other repos. Phase 272 adds path-family diagnostics and rejects broad path-family routing because clean RefactoringMiner docs/config/Java lift does not generalize to ctxhelm, ReAgent, or VeriSchema. Phase 273 rejects a larger local-fastembed prefilter cap as a VeriSchema fix because all semantic metrics remain unchanged at cap `256`. Phase 274 rejects path-derived Python package/module facets because they lower semantic-corroborated recall and increase semantic noise. Phase 275 adds query/path shape diagnostics and rejects a temporary shape-routed insertion variant because it loses RefactoringMiner and ReAgent target tests. Phase 276 adds displacement diagnostics showing which reranked-only non-target docs/planning/scripts/paper files enter top-K while default target tests/source files are lost. Phase 277 adds an eval-only path-family budget variant; it has clean lift on RefactoringMiner and ReAgent but regresses ctxhelm and VeriSchema, so it stays diagnostic only and is rejected for promotion. Phase 278 adds a leave-one-out learned-profile semantic variant that recovers clean RefactoringMiner lift (`0.41857147 -> 0.51857144`, `+2`) while staying exactly neutral on ctxhelm, ReAgent, and VeriSchema. Phase 280 adds `learnedSemanticPolicy`, a deterministic source-free artifact with staleness, holdout, and `minimumSupportCommitCount = 2`; only RefactoringMiner exports one eligible durable profile and every artifact remains `runtimePromotable = false`. Phase 281 applies that support threshold in a source-free held-out variant; it has zero regressions but `appliedCommitCount = 0` on all four repos, so the decision is `insufficient_eligible_holdout_profiles` rather than promotion. Phase 282 repeats the learned-policy holdout at `limit 40`; it still has zero held-out applications on all four repos, proving same-gate widening is insufficient. Phase 283 adds stable ranged gates and shows older/recent slices still do not repeat the learned-policy signal across time. |
| Task-mentioned commit anchoring | `.planning/e2e/2026-06-06-phase279-mentioned-commit-path-anchors.md`, `prepare_plan_promotes_task_mentioned_commit_changed_paths` | Implemented as a local-only planner signal. Prepare-task now promotes safe changed paths from explicitly mentioned commit SHAs into path facets, lexical terms, graph seeds, related-test terms, and target anchors. The signal is inventory-filtered, bounded, diagnostic-backed, and does not expose commit subjects, patches, or source text. |
| Agent-native integrations | `.planning/e2e/2026-06-06-phase247-cursor-opencode-real-client-proof.md`, `.planning/e2e/2026-06-06-phase246-agent-native-fallback-proof.md`, `docs/agent-setup.md` | Codex, Claude, OpenCode, Cursor setup/fallback surfaces exist. OpenCode real-client proof passes. Cursor real-client proof is available but local auth-blocked. |
| Local inspector/governor UX | `.planning/e2e/2026-06-06-phase248-local-inspector-shell.md`, `.planning/e2e/2026-06-06-phase249-context-governor.md` | Implemented with source-sentinel smokes and release-gate wiring. Phase 250 fixed the first observed real-agent retrieval gap for governor/release R&D tasks. |

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
   signal across time. Promotion needs a true train-on-range/apply-on-disjoint
   evaluator or cross-repo profile aggregation where exported profiles can
   apply outside the training snapshot.
2. Claude has current workflow proof but not a fresh paired outcome suite on the
   same level as Codex Phase 245/250. Phase 264 confirms this is still a
   client-availability problem: Claude Code is rate-limited, while Codex remains
   available. Reports should keep rate limits separate from retrieval quality.
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
   policy applications. The next semantic step is cross-repo profile
   aggregation or a true train-on-range/apply-on-disjoint-test-range evaluator,
   not runtime promotion. Increasing the local-fastembed document cap, adding
   broad Python path metadata, adding another hand-written route, and merely
   widening or splitting the same gate are already rejected paths.

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
the remaining semantic R&D path is cross-repo learned-policy aggregation or a
true train-on-range/apply-on-disjoint-test-range evaluator, plus separate
VeriSchema query/document-construction work.
Phase 253 reduces memory risk by
proving cross-repo Codex target-consumption lift. Phase 255 further reduces
memory risk by preserving that target consumption while improving irrelevant
reads in all three measured memory pairs.
