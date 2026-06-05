# R&D Completion Audit - 2026-06-06

## Scope

Audit the current ctxhelm R&D roadmap against the latest source-free proof
artifacts after Phase 253. This audit is not a release claim by itself; it
separates implemented/measured work from remaining research risk.

## Current Proof Summary

| Area | Current evidence | Status |
| --- | --- | --- |
| Codex real-agent outcome | `.ctxhelm/e2e/phase245-agent-run-codex-suite-real-bounded-final.json`, `.ctxhelm/e2e/phase250-agent-run-codex-governor-rd-after.json`, and `.ctxhelm/e2e/phase251-agent-run-codex-rd-breadth-suite.json` | Strong for the measured failure classes. Phase 245 proves a two-task suite where every ctxhelm lane reaches `1.00` target-read coverage vs baseline `0.75`. Phase 250 proves a governor/release R&D task improves from matched baseline with evidence misses to `ctxhelm_improved`. Phase 251 expands this to four R&D task families with 16 comparable ctxhelm lanes and no evidence misses, under-read targets, forbidden commands, or client failures. |
| Claude workflow | `.ctxhelm/e2e/phase250-claude-workflow-refresh.json`, `.ctxhelm/e2e/phase252-agent-run-claude-rd-breadth-suite.json` | Current workflow proof passes with explicit-repo MCP usage and local-only privacy. Phase 252 attempted a fresh paired Claude R&D suite with Claude Code `2.1.163`, but client preflight observed rate limiting, so the report is correctly `degraded` and not used as retrieval-quality evidence. |
| Experience memory | `.ctxhelm/e2e/phase234-memory-pack-impact-suite.json`, `.ctxhelm/e2e/phase244-agent-run-codex-memory-consumption.json`, `.ctxhelm/e2e/phase253-codex-memory-outcome-suite.json`, `.planning/e2e/2026-06-06-phase253-codex-memory-outcome-diversity.md` | Local memory selection is source-free and bounded. Six-repo local evidence shows final-pack memory does not add non-targets; Phase 244 proves Codex consumes selected-memory targets on one repo while reducing read-file count from 18 to 5 in the best lane. Phase 253 extends real Codex memory outcome evidence across VeriSchema, ReAgent, and RefactoringMiner: 3/3 comparison-eligible pairs improved target-read coverage in the memory lane, with no evidence misses, under-read targets, client failures, rate limits, forbidden commands, or malformed required ctxhelm calls. |
| GraphRAG | `.planning/e2e/2026-06-02-phase179-graph-edge-profiles.md`, `.planning/e2e/2026-06-02-phase180-graph-edge-ablations.md`, `.planning/e2e/2026-06-03-phase181-graph-edge-budget-allocation.md` | Implemented and measured. Edge-family profiles and ablations expose import and Python re-export pressure; Phase 181 applies bounded edge-family budget ordering and promotes on reachable fixtures. |
| Semantic retrieval | `.planning/e2e/2026-06-02-phase160-bounded-semantic-status-search.md`, `.planning/e2e/2026-06-02-phase161-semantic-gate-contribution-diagnostics.md`, `.planning/e2e/2026-06-06-phase252-semantic-gate-classification.md` | Integrated and measured as an optional local signal. Phase 252 fixes gate classification so eval-only reranker regressions no longer make semantic look blocked. Current `local_fastembed` gates correctly `hold`: RefactoringMiner shows small lift, ctxhelm is neutral, and default promotion remains unjustified. |
| Agent-native integrations | `.planning/e2e/2026-06-06-phase247-cursor-opencode-real-client-proof.md`, `.planning/e2e/2026-06-06-phase246-agent-native-fallback-proof.md`, `docs/agent-setup.md` | Codex, Claude, OpenCode, Cursor setup/fallback surfaces exist. OpenCode real-client proof passes. Cursor real-client proof is available but local auth-blocked. |
| Local inspector/governor UX | `.planning/e2e/2026-06-06-phase248-local-inspector-shell.md`, `.planning/e2e/2026-06-06-phase249-context-governor.md` | Implemented with source-sentinel smokes and release-gate wiring. Phase 250 fixed the first observed real-agent retrieval gap for governor/release R&D tasks. |

## Remaining R&D Risk

1. Semantic is not yet a proven default-lift channel. It is useful and bounded,
   and Phase 252 shows `local_fastembed` is available and source-free, with a
   small RefactoringMiner lift but neutral ctxhelm recall. The right next work
   is stronger local embedding/cross-query-family evaluation, not unconditional
   promotion.
2. Claude has current workflow proof but not a fresh paired outcome suite on the
   same level as Codex Phase 245/250. This depends partly on client/API
   availability, so reports should keep rate limits separate from retrieval
   quality.
3. Memory now has credible local, single-repo Codex-consumption, and three-repo
   Codex outcome evidence. Phase 253 closes the earlier "single-repo only"
   memory outcome gap for target consumption. The remaining memory question is
   efficiency: the memory lane improved target reads in all three pairs but did
   not reduce irrelevant reads.
4. GraphRAG is measured and improved, but still has budget-allocation questions
   for large source clusters. The current system avoids adding more graph edges
   blindly; future work should target edge-family ranking where ablations show
   exclusive target lift.
5. Cursor proof remains auth-blocked locally. The wrapper behavior is correct,
   but it is not a successful current real-client proof until a logged-in Cursor
   Agent environment is available.

## Next Correct Order

1. Keep the current Phase 250 artifact promotion and governor proof as the
   active regression guard.
2. Preserve the Phase 253 memory target-consumption contract and optimize
   memory-lane read efficiency separately.
3. Refresh Claude paired outcome evidence when the client is available.
4. Promote semantic only if the semantic contribution gate shows repeated
   measurable query-family lift without source/privacy regressions. Phase 252
   establishes the current honest gate state as `hold`, not `block`.

## Conclusion

ctxhelm is past MVP and has real agent-outcome proof for multiple failure
classes. It is not yet R&D complete in the world-class sense because semantic
default lift, memory-lane efficiency, and fresh paired Claude outcome proof
remain open research questions. Phase 252 reduces semantic risk by fixing a
misleading `block` classification, and Phase 253 reduces memory risk by proving
cross-repo Codex target-consumption lift, but neither result justifies
unconditional semantic promotion or a memory efficiency claim.
