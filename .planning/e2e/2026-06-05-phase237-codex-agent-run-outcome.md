# Phase 237: Codex Agent-Run Outcome Matrix

## Goal

Move from Codex MCP availability proof to real Codex outcome evidence: does a
ctxhelm-assisted Codex run inspect better files, consume ctxhelm output, and
stay inside the read-only proof boundary?

## What Changed

- Added `scripts/e2e-agent-run-codex.sh`.
- The script runs five Codex CLI lanes:
  - native baseline
  - `ctxhelm-plan`
  - `ctxhelm-brief`
  - `ctxhelm-standard`
  - `ctxhelm-memory`
- The script measures Codex `command_execution` events rather than Claude
  `tool_use` events.
- The report stores command hashes, command verbs, exit statuses, path labels,
  read/discovery counts, ctxhelm request summaries, and forbidden-command
  counts.
- The report does not store raw command output, raw prompts, raw transcripts,
  raw MCP traffic, source text, or user project test output.
- `ctxhelm eval agent-run` now renders Codex command-count deltas and falls back
  from Claude-only forbidden-tool fields to Codex forbidden-command fields.

## Debugging Note

The first Codex matrix attempt failed every lane with `stream_disconnected`.
The root cause was the harness forcing a temporary `CODEX_HOME`, which removed
Codex authentication. The fixed harness follows the existing Codex MCP smoke
pattern: use `--ignore-user-config` and `--ephemeral` when available while
preserving the normal auth home.

## Evidence

Artifact:

- `.ctxhelm/e2e/phase237-agent-run-codex.json`

Rendered report:

```bash
ctxhelm eval agent-run --report .ctxhelm/e2e/phase237-agent-run-codex.json
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

- `bestLane = ctxhelm-memory`
- Target coverage delta: `+0.33`
- Target read coverage delta: `0.00`
- Irrelevant read delta: `2`
- Command execution delta: `14`
- Read file delta: `2`

Lane details:

- Baseline read two of three target files and missed `docs/agent-setup.md`.
- `ctxhelm-memory` hit all three targets, read two of three target files,
  reduced irrelevant reads from `7` to `5`, reduced read files from `9` to `7`,
  and reduced command executions from `24` to `10`.
- `ctxhelm-plan` and `ctxhelm-standard` exposed a remaining consumption gap:
  ctxhelm surfaced `docs/agent-setup.md` and `docs/feedback.md`, but Codex did
  not read those docs in those weaker lanes.

Recommended R&D actions:

- `improve_agent_consumption_guidance`
- `inspect_pack_ordering_and_native_read_instruction`

## Validation

Commands:

```bash
bash -n scripts/e2e-agent-run-codex.sh
cargo test -p ctxhelm --test release_packaging codex_agent_run_e2e_script_contract --locked
cargo test -p ctxhelm --test cli_compat eval_agent_run_renders_source_free_report --locked
cargo build -p ctxhelm --locked
target/debug/ctxhelm eval agent-run --report .ctxhelm/e2e/phase237-agent-run-codex.json
git diff --check
```

## Interpretation

This is the first clean current real-client outcome lift after the recent
availability blockers. It is not a universal claim across all tasks or clients;
it is a source-free, task-specific Codex result. The important next step is to
make Codex consume doc targets more consistently when ctxhelm already surfaced
them.
