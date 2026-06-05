# R&D Completion Audit - 2026-06-06

## Scope

Audit the current ctxhelm R&D roadmap against the latest source-free proof
artifacts after Phase 250. This audit is not a release claim by itself; it
separates implemented/measured work from remaining research risk.

## Current Proof Summary

| Area | Current evidence | Status |
| --- | --- | --- |
| Codex real-agent outcome | `.ctxhelm/e2e/phase245-agent-run-codex-suite-real-bounded-final.json` and `.ctxhelm/e2e/phase250-agent-run-codex-governor-rd-after.json` | Strong for the measured failure classes. Phase 245 proves a two-task suite where every ctxhelm lane reaches `1.00` target-read coverage vs baseline `0.75`. Phase 250 proves a governor/release R&D task improves from matched baseline with evidence misses to `ctxhelm_improved`. |
| Claude workflow | `.ctxhelm/e2e/phase250-claude-workflow-refresh.json` | Current workflow proof passes with explicit-repo MCP usage and local-only privacy. Paired Claude outcome proof remains older and client-availability dependent. |
| Experience memory | `.ctxhelm/e2e/phase234-memory-pack-impact-suite.json`, `.ctxhelm/e2e/phase244-agent-run-codex-memory-consumption.json` | Local memory selection is source-free and bounded. Six-repo local evidence shows final-pack memory does not add non-targets; Phase 244 proves Codex consumes selected-memory targets while reducing read-file count from 18 to 5 in the best lane. |
| GraphRAG | `.planning/e2e/2026-06-02-phase179-graph-edge-profiles.md`, `.planning/e2e/2026-06-02-phase180-graph-edge-ablations.md`, `.planning/e2e/2026-06-03-phase181-graph-edge-budget-allocation.md` | Implemented and measured. Edge-family profiles and ablations expose import and Python re-export pressure; Phase 181 applies bounded edge-family budget ordering and promotes on reachable fixtures. |
| Semantic retrieval | `.planning/e2e/2026-06-02-phase160-bounded-semantic-status-search.md`, `.planning/e2e/2026-06-02-phase161-semantic-gate-contribution-diagnostics.md` | Integrated and measured as an optional local signal. Direct semantic search/status is bounded and source-free; contribution gates show overlap vs lexical and correctly hold promotion when semantic-only lift is not proven. |
| Agent-native integrations | `.planning/e2e/2026-06-06-phase247-cursor-opencode-real-client-proof.md`, `.planning/e2e/2026-06-06-phase246-agent-native-fallback-proof.md`, `docs/agent-setup.md` | Codex, Claude, OpenCode, Cursor setup/fallback surfaces exist. OpenCode real-client proof passes. Cursor real-client proof is available but local auth-blocked. |
| Local inspector/governor UX | `.planning/e2e/2026-06-06-phase248-local-inspector-shell.md`, `.planning/e2e/2026-06-06-phase249-context-governor.md` | Implemented with source-sentinel smokes and release-gate wiring. Phase 250 fixed the first observed real-agent retrieval gap for governor/release R&D tasks. |

## Remaining R&D Risk

1. Semantic is not yet a proven default-lift channel. It is useful and bounded,
   but the current gate evidence says semantic mostly overlaps lexical on the
   sampled RefactoringMiner proof. The right next work is stronger local
   embedding/cross-query-family evaluation, not unconditional promotion.
2. Claude has current workflow proof but not a fresh paired outcome suite on the
   same level as Codex Phase 245/250. This depends partly on client/API
   availability, so reports should keep rate limits separate from retrieval
   quality.
3. Memory has credible local and Codex-consumption evidence, but broad
   real-agent memory outcome lift is still smaller than the local memory
   measurement suite. Future suites should include memory-specific tasks where
   baseline and ctxhelm are comparison-eligible.
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
2. Build a larger paired-agent suite that includes one memory-driven task, one
   semantic/conceptual task, one graph-neighborhood task, and one governance
   artifact task.
3. Run Codex first because it is currently available and machine-checkable.
4. Refresh Claude paired outcome evidence when the client is available.
5. Promote semantic only if the semantic contribution gate shows semantic-only
   target hits or measurable query-family lift without source/privacy
   regressions.

## Conclusion

ctxhelm is past MVP and has real agent-outcome proof for multiple failure
classes. It is not yet R&D complete in the world-class sense because semantic
default lift, broad memory outcome lift, fresh Claude paired outcome proof, and
larger agent-suite coverage remain open research questions.
