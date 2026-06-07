# Phase 305-309 - Codex Outcome Refresh And Target Consumption

## Scope

Phase 304 showed Claude Code is still rate-limited, while Codex CLI is
available. This phase refreshes the four-task Codex R&D breadth-suite outcome
on current `main` and follows the measured target-consumption regression in
the Codex memory lane.

## Commands

```bash
CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" \
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=150 \
bash scripts/e2e-agent-run-codex.sh \
  --repo "$(pwd)" \
  --suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json \
  --output .ctxhelm/e2e/phase305-agent-run-codex-rd-breadth-suite.json
```

After the memory-lane prompt fix:

```bash
CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" \
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=150 \
bash scripts/e2e-agent-run-codex.sh \
  --repo "$(pwd)" \
  --suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json \
  --output .ctxhelm/e2e/phase307-agent-run-codex-rd-breadth-suite.json
```

Targeted governor retry:

```bash
CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" \
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=150 \
bash scripts/e2e-agent-run-codex.sh \
  --repo "$(pwd)" \
  --task "Identify the files relevant to improving ctxhelm context governor decision reports and release-gate proof. Do not edit files." \
  --target-file crates/ctxhelm/src/main.rs \
  --target-file crates/ctxhelm-core/src/contracts.rs \
  --target-file scripts/smoke-governor.sh \
  --target-file docs/context-governor.md \
  --output .ctxhelm/e2e/phase309-agent-run-codex-governor-target-consumption.json
```

## Finding

The first current-head refresh passed and still claimed `ctxhelm_improved`, but
exposed a real regression from the earlier memory-efficiency prompt:

| Metric | Phase 305 |
| --- | ---: |
| tasks | `4` |
| comparison-eligible tasks | `4` |
| comparable ctxhelm lanes | `16` |
| outcome claim | `ctxhelm_improved` |
| target-read delta average | `0.3125` |
| ctxhelm evidence misses | `false` |
| ctxhelm evidence-only targets | `true` |
| ctxhelm under-read targets | `true` |
| client failures / rate limits | `false` / `false` |
| memory-lane target-read coverage | `0.5625` |
| memory-lane discovered-only targets | `5` |
| memory-lane missed targets | `1` |

The prompt let the memory lane stop after two memory-backed current-file reads,
which was efficient but too weak for multi-target R&D tasks.

## Fix

`scripts/e2e-agent-run-codex.sh` now makes memory efficiency subordinate to
target consumption:

- memory lane has a bounded six-command shell budget after ctxhelm calls;
- targetFiles and high-confidence target paths are consumed first, up to five
  current files;
- small target sets must be read before broader exploration or answering;
- docs/config/schema/script targets are explicitly treated as first-class
  reads, including in plan/brief/standard lanes.

## Validation

Phase 307 reran the same four-task suite after the prompt change:

| Metric | Phase 307 |
| --- | ---: |
| status | `degraded` |
| tasks | `4` |
| comparison-eligible tasks | `4` |
| comparable ctxhelm lanes | `15` |
| outcome claim | `ctxhelm_improved` |
| target-read delta average | `0.22916666666666669` |
| ctxhelm evidence misses | `false` |
| ctxhelm evidence-only targets | `true` |
| ctxhelm under-read targets | `false` |
| client failures / rate limits | `true` / `false` |
| memory-lane target-read coverage | `1.0` |
| memory-lane evidence-only targets | `0` |
| memory-lane missed targets | `0` |

The degraded status came from a single Codex timeout in the
`semantic-contribution` `ctxhelm-brief` lane, not from a ctxhelm retrieval miss.

Phase 309 targeted the governor docs target that remained evidence-only in
some non-memory lanes:

| Lane | Target read coverage | Evidence-only targets | Missed targets |
| --- | ---: | ---: | ---: |
| baseline | `0.25` | `0` | `1` |
| ctxhelm-plan | `1.0` | `0` | `0` |
| ctxhelm-brief | `0.75` | `1` | `1` |
| ctxhelm-standard | `0.5` | `2` | `2` |
| ctxhelm-memory | `1.0` | `0` | `0` |

## Decision

The Codex outcome refresh still supports the current R&D claim that ctxhelm can
improve real Codex target reads on the four-task breadth suite. The memory-lane
regression found in Phase 305 is fixed by Phase 307/309 evidence.

Do not overclaim deterministic prompt compliance for every non-memory
ctxhelm-assisted lane. Even with stricter prompt wording, Codex sometimes skips
surfaced docs or source targets in plan/brief/standard lanes. Keep
`improve_agent_consumption_guidance` as a residual R&D action unless the harness
adds a stronger machine-checkable retry/enforcement layer.
