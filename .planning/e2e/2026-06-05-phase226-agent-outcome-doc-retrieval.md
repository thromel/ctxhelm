# Phase 226 Agent Outcome Doc Retrieval

## Goal

Fix the secondary evidence miss discovered in Phase 225: ctxhelm-assisted lanes for `Improve paired agent-run lane matrix` did not surface `docs/feedback.md`, even though that document defines paired agent-run feedback, outcome comparison, required-call accounting, and R&D routing semantics.

## Change

- Added a bounded agent-outcome task detector for prompts involving agent runs, paired lanes, matrices, outcomes, Claude/Codex clients, or real-client comparison.
- Injected only `docs/feedback.md` and `docs/agent-setup.md` for those tasks.
- Kept generic project-governance tasks unchanged so `docs/feedback.md` is not pulled into unrelated planning/release/eval prompts.
- Added `docs/feedback.md` to root governance doc ordering so selected agent-output docs render predictably in packs.

## Proof

Artifact: `.ctxhelm/e2e/phase226-agent-outcome-doc-retrieval.json`

Commands:

```bash
target/debug/ctxhelm prepare-task --repo /Users/romel/Documents/GitHub/ctxhelm --no-trace "Improve paired agent-run lane matrix"
target/debug/ctxhelm get-pack --repo /Users/romel/Documents/GitHub/ctxhelm --budget brief --format json --no-trace "Improve paired agent-run lane matrix"
target/debug/ctxhelm get-pack --repo /Users/romel/Documents/GitHub/ctxhelm --budget standard --format json --no-trace "Improve paired agent-run lane matrix"
cargo test -p ctxhelm-compiler agent_outcome_tasks_add_feedback_docs_as_candidates --locked
cargo test -p ctxhelm-compiler prepare_plan_selects_renderer_for_hyphenated_agent_run_task --locked
cargo test -p ctxhelm-compiler governance_tasks_add_root_planning_docs_as_candidates --locked
```

Result:

- `prepare-task` target files include `docs/feedback.md` and `docs/agent-setup.md`.
- Brief and standard packs include `docs/feedback.md` in high-confidence target files.
- Standard pack includes the `docs/feedback.md` snippet.
- Generic governance docs still do not pull in `docs/feedback.md`.

## Boundary

This is a retrieval and packing fix. It does not prove real-agent outcome lift. The Phase 225 Claude Code rerun was rate-limited across all five lanes, so the next real-agent step remains a fresh non-rate-limited five-lane run.
