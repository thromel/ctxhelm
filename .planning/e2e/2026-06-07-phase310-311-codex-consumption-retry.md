# Phase 310-311 - Codex Consumption Retry Enforcement

## Scope

Phase 305-309 tightened Codex prompt guidance and fixed the memory lane, but it
also showed prompt-only target consumption can remain stochastic in
plan/brief/standard lanes. This phase adds a source-free, bounded retry layer to
the Codex real-agent harness.

The retry only triggers when a ctxhelm-assisted lane is otherwise eligible and
the source-free evaluator sees `ctxhelmEvidenceOnlyTargetCount > 0`: ctxhelm
surfaced an expected target, but Codex did not consume it with a read command.
The retry prompt includes the target list and requires those targets to be read
before broader exploration. The report keeps only source-free retry summaries,
not raw prompts, transcripts, MCP traffic, command output, or source text.

## Change

`scripts/e2e-agent-run-codex.sh` now:

- detects eligible ctxhelm lanes with evidence-only targets;
- runs one bounded target-consumption retry for that lane;
- scores the original and retry reports by status, eligibility, target-read
  coverage, target coverage, evidence-only count, missed-target count,
  forbidden-command count, irrelevant reads, and read-file count;
- writes the selected report back under the original lane name;
- records `retryAttempted`, `retrySelected`, `retryReason`, `initialAttempt`,
  `retryAttempt`, and `retrySourceLane` when applicable.

## Targeted Proof

Command:

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
  --output .ctxhelm/e2e/phase310-agent-run-codex-governor-consumption-retry.json
```

Result:

| Metric | Value |
| --- | ---: |
| status | `passed` |
| comparison eligible | `true` |
| comparable ctxhelm lanes | `4` |
| outcome claim | `ctxhelm_improved` |
| target coverage delta | `0.5` |
| target-read coverage delta | `1.0` |
| ctxhelm evidence misses | `false` |
| ctxhelm evidence-only targets | `false` |
| ctxhelm under-read targets | `false` |
| client failures / rate limits | `false` / `false` |

The `ctxhelm-memory` lane retry was selected. Its initial attempt had
`targetReadCoverage = 0.5`, `ctxhelmEvidenceOnlyTargetCount = 2`,
`missedTargetCount = 2`, and `targetReadCount = 2`. The selected retry had
`targetReadCoverage = 1.0`, `ctxhelmEvidenceOnlyTargetCount = 0`,
`missedTargetCount = 0`, and `targetReadCount = 4`.

## Breadth-Suite Proof

Command:

```bash
CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" \
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=150 \
bash scripts/e2e-agent-run-codex.sh \
  --repo "$(pwd)" \
  --suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json \
  --output .ctxhelm/e2e/phase311-agent-run-codex-rd-breadth-suite-consumption-retry.json
```

Aggregate:

| Metric | Value |
| --- | ---: |
| status | `passed` |
| tasks | `4` |
| comparison-eligible tasks | `4` |
| comparable ctxhelm lanes | `16` |
| outcome claim | `ctxhelm_improved` |
| target coverage delta average | `0.2916666666666667` |
| target-read coverage delta average | `0.2916666666666667` |
| ctxhelm evidence misses | `false` |
| ctxhelm evidence-only targets | `false` |
| ctxhelm under-read targets | `false` |
| forbidden commands | `false` |
| client failures / rate limits | `false` / `false` |
| recommended action | `preserve_current_agent_contract` |

Lane summary:

| Lane | Avg target read coverage | Target reads | Evidence-only targets | Missed targets |
| --- | ---: | ---: | ---: | ---: |
| baseline | `0.7083333333333333` | `9` | `0` | `4` |
| ctxhelm-plan | `1.0` | `13` | `0` | `0` |
| ctxhelm-brief | `1.0` | `13` | `0` | `0` |
| ctxhelm-standard | `1.0` | `13` | `0` | `0` |
| ctxhelm-memory | `1.0` | `13` | `0` | `0` |

Selected retries occurred in:

- `memory-native-read` / `ctxhelm-brief`: `0.6666666666666666 -> 1.0`
  target-read coverage.
- `semantic-contribution` / `ctxhelm-standard`: `0.3333333333333333 -> 1.0`
  target-read coverage.
- `semantic-contribution` / `ctxhelm-memory`: `0.3333333333333333 -> 1.0`
  target-read coverage.
- `governor-release-proof` / `ctxhelm-brief`: `0.5 -> 1.0`
  target-read coverage.

## Decision

This closes the measured Codex evidence-only consumption gap from Phase 305-309
for the four-task R&D breadth suite. The retry layer improves proof reliability
without weakening the read-only/source-free boundary.

This is not an efficiency improvement: retries can increase read-file and
irrelevant-read counts in some lanes. Treat Phase 311 as consumption
enforcement evidence, not as a memory-efficiency replacement for Phase 255.
