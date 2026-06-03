# Phase 205 - Agent Consumption Diagnostics

## Goal

Make the real-agent consumption gap from Phase 204 measurable without storing
raw prompts, raw model transcripts, raw MCP traffic, source text, terminal logs,
or project test output.

Phase 204 proved that forbidden tool calls are now visible, but the hardened
Claude Code run still showed no ctxhelm lift on the validation-evidence task:
the native baseline covered `2 / 3` targets, while `ctxhelm-plan` and
`ctxhelm-brief` each covered `1 / 3`. The next question is whether ctxhelm lanes
are discovering files without reading them, missing target roles, or reading a
too-narrow mix of source/docs/tests.

## Change

`scripts/e2e-agent-run.sh` now records source-free consumption diagnostics:

- per-lane `metrics.targetReadCoverage`
- per-lane `metrics.targetReadCount`
- per-lane `metrics.targetDiscoveredOnlyCount`
- per-lane `metrics.missedTargetCount`
- per-lane `targetReads`
- per-lane `discoveredOnlyTargets`
- per-lane `missedTargets`
- per-lane `readRoleCounts`
- per-lane `missedTargetRoleCounts`
- comparison-level `targetReadCoverageDelta`
- comparison-level `ctxhelmUnderReadTargetsObserved`
- suite-level `targetReadCoverageDeltaAverage`
- suite lane target-read/missed-target totals and role-count totals

`ctxhelm eval agent-run` renders the new fields in markdown. This keeps the
report source-free while distinguishing target discovery from actual native file
reads.

## Validation

Focused commands:

```bash
bash -n scripts/e2e-agent-run.sh
cargo fmt --all -- --check
cargo test -p ctxhelm --test release_packaging \
  agent_run_e2e_script_contract --locked -- --nocapture
cargo test -p ctxhelm --test cli_compat \
  eval_agent_run_renders_source_free --locked -- --nocapture
cargo test -p ctxhelm --test cli_compat \
  eval_agent_run_renders_source_free_suite_report --locked -- --nocapture
bash scripts/check-release-docs.sh
```

Expected result: pass.

Real Claude Code command:

```bash
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=120 \
bash scripts/e2e-agent-run.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --task "Identify files relevant to improving paired agent-run consumption diagnostics in ctxhelm" \
  --target-file scripts/e2e-agent-run.sh \
  --target-file crates/ctxhelm/src/main.rs \
  --target-file docs/feedback.md \
  --output /tmp/ctxhelm-rd/phase205-agent-consumption-real.json
```

Result:

- Report status: `passed`
- Client: Claude Code `2.1.159`
- Forbidden calls observed: `false`
- ctxhelm under-read targets observed: `true`
- Outcome claim: `ctxhelm_improved`
- Native baseline: target coverage `0.67`, target-read coverage `0.67`,
  read files `5`, irrelevant reads `3`, target reads `2`,
  discovered-only targets `0`, missed targets `1`, read roles `docs=1,
  source=4`, missed target roles `docs=1`
- `ctxhelm-plan`: target coverage `0.67`, target-read coverage `0.33`,
  read files `4`, irrelevant reads `3`, target reads `1`,
  discovered-only targets `1`, missed targets `1`, ctxhelm calls `1`,
  read roles `docs=1, source=3`, missed target roles `docs=1`
- `ctxhelm-brief`: target coverage `0.67`, target-read coverage `0.67`,
  read files `4`, irrelevant reads `2`, target reads `2`,
  discovered-only targets `0`, missed targets `1`, ctxhelm calls `2`,
  read roles `docs=1, source=3`, missed target roles `docs=1`

The new fields expose a nuance the Phase 204 report could not show: `ctxhelm`
brief matched the native baseline on actual target reads while reducing one
irrelevant read, but the plan-only lane discovered one target without reading it.

## Interpretation

This phase is diagnostic. It does not claim a retrieval-quality lift or a new
general real-agent outcome win. It gives future Claude Code paired runs stronger
evidence about whether ctxhelm-assisted lanes under-consume target files, miss
specific source-free roles, or merely reduce irrelevant reads while preserving
actual target consumption.
