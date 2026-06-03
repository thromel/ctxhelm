# Phase 210 - Agent-Run Evidence Attribution

## Goal

Distinguish real-agent outcome gaps caused by ctxhelm not surfacing a target from
gaps caused by the agent not reading a surfaced target.

## Change

- `scripts/e2e-agent-run.sh` now computes source-free ctxhelm evidence coverage
  for assisted lanes.
- `ctxhelm-plan` evidence is collected from `prepare-task`.
- `ctxhelm-brief` evidence is collected from `get-pack --budget brief --format
  json`.
- The persisted report stores only path labels and counts:
  - `ctxhelmEvidenceFiles`
  - `ctxhelmEvidenceTargetHits`
  - `ctxhelmEvidenceOnlyTargets`
  - `ctxhelmEvidenceMissedTargets`
  - matching count fields in lane metrics
  - comparison and suite booleans for evidence misses and evidence-only targets
- Raw pack JSON, snippets, prompts, transcripts, MCP traffic, terminal logs, and
  source text are not persisted.

## Local Evidence

Focused checks:

```bash
bash -n scripts/e2e-agent-run.sh
cargo test -p ctxhelm eval_agent_run --locked -- --nocapture
cargo test -p ctxhelm agent_run_e2e_script_contract --locked -- --nocapture
```

Skipped contract probe:

```bash
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
  bash scripts/e2e-agent-run.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --task "Improve agent-run report attribution" \
  --target-file scripts/e2e-agent-run.sh \
  --target-file crates/ctxhelm/src/main.rs \
  --output /tmp/ctxhelm-phase210-skipped.json
```

Observed summary:

```text
status skipped
outcome insufficient_comparable_lanes
ctxhelmEvidenceMissesObserved false
```

Real Claude Code probe:

```bash
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=90 \
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
  bash scripts/e2e-agent-run.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --task "Improve agent-run report attribution" \
  --target-file scripts/e2e-agent-run.sh \
  --target-file crates/ctxhelm/src/main.rs \
  --output /tmp/ctxhelm-phase210-real.json
```

Observed summary:

```text
status skipped
outcome insufficient_comparable_lanes
comparisonEligible false
clientFailuresObserved true
rateLimitsObserved true
ctxhelmEvidenceMissesObserved true
ctxhelmEvidenceOnlyTargetsObserved true
baseline failed rate_limited read 0 ctxEvidenceHits 0 ctxEvidenceMisses 2
ctxhelm-plan failed rate_limited read 0 ctxEvidenceHits 1 ctxEvidenceMisses 1
ctxhelm-brief failed rate_limited read 0 ctxEvidenceHits 1 ctxEvidenceMisses 1
```

Interpretation: the client remains unavailable due to Claude Code rate limits,
so this is not outcome-lift proof. The report now still provides useful R&D
attribution: `scripts/e2e-agent-run.sh` was surfaced by ctxhelm evidence but not
read because the client failed, while `crates/ctxhelm/src/main.rs` was not
surfaced by ctxhelm evidence for this task.

## Next

- Rerun a real Claude Code paired suite after the rate limit clears.
- Use `ctxhelmEvidenceMissedTargets` to decide whether to improve retrieval and
  packing.
- Use `ctxhelmEvidenceOnlyTargets` to decide whether to improve agent guidance
  and pack consumption instructions.
