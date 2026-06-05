# Phase 239: Codex Capped Target Consumption

## Goal

Fix the Phase 237 Codex consumption gap without making ctxhelm-assisted agents
read the entire returned target list. The desired behavior is progressive:
native-read the top targets first, treat docs/config/schema/script targets as
real target files, and stop when those reads answer the task.

## What Changed

- MCP `prepare_task` text now tells agents to start with the first up to 5
  `targetFiles`, not the whole list.
- Pack and adapter guidance now says docs, config, schema, and script entries in
  the initial target set are first-class targets.
- Adapter guidance now tells Codex/Cursor/Claude/OpenCode to stop after the
  initial native reads if they answer the task.
- `scripts/e2e-agent-run-codex.sh` now measures this capped-consumption behavior
  in all ctxhelm lanes.
- Tests now lock the capped target-consumption rule across MCP text, pack text,
  generated adapter guidance, and release packaging.

## Debugging Note

Phase 238 proved the first fix was too blunt. It eliminated evidence-only target
misses, but the prompt said to read returned `targetFiles` first without a cap.
For the task `Improve paired agent-run lane matrix`, `prepare_task` returned 10
target files and 3 related tests, so Codex read too broadly.

Phase 239 changed the guidance to the intended progressive strategy: read the
first up to 5 target files, include docs/config/schema/script files in that set,
and stop if the evidence is sufficient.

## Evidence

Artifacts:

- `.ctxhelm/e2e/phase238-agent-run-codex-consumption.json`
- `.ctxhelm/e2e/phase239-agent-run-codex-capped-consumption.json`

Rendered report:

```bash
ctxhelm eval agent-run --report .ctxhelm/e2e/phase239-agent-run-codex-capped-consumption.json
```

Summary:

- `status = passed`
- `client = codex`
- `client.version = codex-cli 0.137.0`
- `comparisonEligible = true`
- `comparableCtxhelmLaneCount = 4`
- `ctxhelmToolCallsObserved = true`
- `missingRequiredCtxhelmCallsObserved = false`
- `invalidRequiredCtxhelmCallsObserved = false`
- `clientFailuresObserved = false`
- `rateLimitsObserved = false`
- `forbiddenCommandsObserved = false`
- `ctxhelmEvidenceMissesObserved = false`
- `outcomeClaim = ctxhelm_improved`

Best lane:

- `bestLane = ctxhelm-brief`
- Target coverage delta: `0.00`
- Target read coverage delta: `0.00`
- Irrelevant read delta: `2`
- Command execution delta: `10`
- Read file delta: `2`

Lane details:

- Baseline read all three target files, used `16` commands, read `7` files, and
  had `4` non-target reads.
- All ctxhelm lanes read all three target files with no discovered-only targets,
  no missed targets, and no ctxhelm evidence-only target misses.
- `ctxhelm-plan` used `5` commands, read `5` files, and had `2` non-target
  reads.
- `ctxhelm-brief`, `ctxhelm-standard`, and `ctxhelm-memory` each used `6`
  commands, read `5` files, and had `2` non-target reads.

## Validation

Commands:

```bash
bash -n scripts/e2e-agent-run-codex.sh
cargo fmt --check
cargo test -p ctxhelm-mcp --lib prepare_task_call_returns_structured_context_plan --locked
cargo test -p ctxhelm-core --lib adapter_guidance_explains_progressive_native_reads_and_session_scope --locked
cargo test -p ctxhelm-compiler --lib compile_context_pack_renders_context_areas --locked
cargo test -p ctxhelm --test release_packaging codex_agent_run_e2e_script_contract --locked
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm CTXHELM_RUN_REAL_CLIENT=1 CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=180 bash scripts/e2e-agent-run-codex.sh --repo /Users/romel/Documents/GitHub/ctxhelm --task "Improve paired agent-run lane matrix" --target-file scripts/e2e-agent-run.sh --target-file docs/feedback.md --target-file docs/agent-setup.md --output .ctxhelm/e2e/phase239-agent-run-codex-capped-consumption.json
```

## Interpretation

This closes the Phase 237 follow-up. The result is still task-specific and
real-agent runs remain noisy, but the progression is clean:

- Phase 237: ctxhelm improved coverage/efficiency, but some Codex lanes surfaced
  docs without reading them.
- Phase 238: Codex read all surfaced expected targets, but read too broadly.
- Phase 239: Codex read all expected targets and improved efficiency over the
  baseline with fewer commands and fewer file reads.
