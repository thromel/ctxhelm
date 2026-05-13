# Roadmap: Repo Context Packer

## Milestones

- ✅ **v1.1 Packaging & Adoption** — shipped 2026-05-13. Full roadmap archived at `.planning/milestones/v1.1-ROADMAP.md`; audit archived at `.planning/milestones/v1.1-MILESTONE-AUDIT.md`.

## Current State

ctxpack v1.1 is shipped as a local-first, read-only context broker for existing coding agents. The completed work covers:

- compatibility guardrails, trust diagnostics, measured retrieval, and agent-native durability;
- versioned local binary packaging and release artifact audit;
- repo-local agent setup for Codex CLI, Claude Code, Cursor, and OpenCode;
- installed-binary first-pack onboarding, setup validation, and troubleshooting docs;
- release gates that verify docs, packaging, artifact audit, selected-binary behavior, deterministic MCP proof, and optional Codex/Claude wrapper paths.

## Next Milestone Candidates

Future milestones should start from fresh requirements with `$gsd-new-milestone`.

- **v1.2 Retrieval Quality Proof** — broaden real-repo historical evals, benchmark reports, token ROI, retrieval deltas, failure modes, and regression trends.
- **v1.3 Production Storage** — add durable local storage and faster incremental indexing when measured performance justifies it.
- **v1.4 Local Semantic Retrieval** — add optional local embeddings/vector retrieval with explicit privacy controls.
- **v1.5 Parser/Semantic Precision** — expand parser-backed coverage or optional SCIP/LSP precision where eval gaps justify it.
- **v2.0 Workspace & Team Layer** — support multi-repo workspaces and source-free shared context cards.
- **v2.1 UI / Pack Inspector** — add optional diagnostics UI without changing agents as the daily surface.
