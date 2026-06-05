# Phase 225 - Expanded Paired-Agent Lane Matrix

Date: 2026-06-05

## Goal

Close the gap between the original R&D plan and the paired real-agent harness.
The plan called for native baseline, ctxhelm plan, brief pack, standard pack,
and memory-card lanes. Before this phase, `scripts/e2e-agent-run.sh` only ran
baseline, `ctxhelm-plan`, and `ctxhelm-brief`.

## Changes

- Added `ctxhelm-standard` lane.
- Added `ctxhelm-memory` lane.
- `ctxhelm-standard` and `ctxhelm-memory` require valid explicit-repo
  `prepare_task` plus `get_pack` with:
  - `budget = "standard"`
  - `format = "json"`
  - `recordTrace = false`
- The memory lane tells Claude to use selected memory or experience-card
  evidence as guidance, while still requiring native reads of current files.
- Suite aggregation automatically includes the new lane summaries.
- Documentation now describes the five-lane matrix.

## Deterministic Shape Proof

```bash
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
  bash scripts/e2e-agent-run.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --task "Improve paired agent-run lane matrix" \
  --target-file scripts/e2e-agent-run.sh \
  --target-file docs/feedback.md \
  --output /tmp/ctxhelm-phase225-agent-run-skipped.json
```

Observed lanes:

- `baseline`
- `ctxhelm-plan`
- `ctxhelm-brief`
- `ctxhelm-standard`
- `ctxhelm-memory`

The skipped report preserves the contract and reports
`insufficient_comparable_lanes` rather than pretending outcome proof exists.

## Real-Client Attempt

```bash
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=45 \
  bash scripts/e2e-agent-run.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --task "Improve paired agent-run lane matrix" \
  --target-file scripts/e2e-agent-run.sh \
  --target-file docs/feedback.md \
  --output .ctxhelm/e2e/phase225-agent-run-lane-matrix.json
```

Artifact: `.ctxhelm/e2e/phase225-agent-run-lane-matrix.json`

Result:

- Claude Code version: `2.1.163`
- Status: `skipped`
- Outcome claim: `insufficient_comparable_lanes`
- Every lane failed with `clientFailureKind = rate_limited`
- `rateLimitsObserved = true`
- `ctxhelmToolCallsObserved = false`
- `recommendedResearchActions` includes `retry_real_client_when_available`

The report also records a secondary ctxhelm-evidence miss for `docs/feedback.md`
in the pack lanes. Because the client could not execute any comparable lane,
this should be revisited after a successful real-client run instead of being
treated as proven outcome regression.

## Validation

- `bash -n scripts/e2e-agent-run.sh`
- `cargo test -p ctxhelm --test release_packaging agent_run_e2e_script_contract --locked`
