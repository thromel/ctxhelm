# Phase 324: Codex Discovered-Only Retry Enforcement

## Goal

Close the remaining Codex consumption gap where a ctxhelm-assisted lane could
discover a target path but never consume it with a native read command.

## Change

- `scripts/e2e-agent-run-codex.sh` now retries eligible ctxhelm lanes when
  either `ctxhelmEvidenceOnlyTargetCount > 0` or `targetDiscoveredOnlyCount > 0`.
- Per-lane retry metadata records discovered-only target counts before and
  after retry.
- Suite and single-run retry-cost summaries include
  `discoveredOnlyTargetsBeforeRetry` and
  `discoveredOnlyTargetsAfterRetry`.
- `scripts/check-agent-run-proof.py` rejects retry-cost proof when
  discovered-only targets remain after retry.
- CLI and proof-inspector renderers expose discovered-only retry counts in
  source-free summaries.

## Validation

Local engineering validation passed:

```bash
cargo fmt --all -- --check
bash scripts/check-release-docs.sh
cargo check -p ctxhelm --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Real Codex breadth-suite validation passed the strict source-free proof checker
for identity, current runner/suite fingerprints, aggregate consistency, retry
cost, and read-only/privacy boundaries:

```bash
CTXHELM_BIN="$PWD/target/debug/ctxhelm" \
  CTXHELM_RUN_REAL_CLIENT=1 \
  CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=180 \
  bash scripts/e2e-agent-run-codex.sh \
  --repo "$PWD" \
  --suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json \
  --suite-work-dir .ctxhelm/e2e/phase324-codex-v247-discovered-retry-checkpoints \
  --output .ctxhelm/e2e/phase324-agent-run-codex-v247-discovered-retry.json

python3 scripts/check-agent-run-proof.py \
  .ctxhelm/e2e/phase324-agent-run-codex-v247-discovered-retry.json \
  --expected-ctxhelm-version "ctxhelm 2.4.7" \
  --expected-client-name codex \
  --expected-client-version "codex-cli 0.137.0" \
  --current-runner-script scripts/e2e-agent-run-codex.sh \
  --current-suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json \
  --require-retry-cost \
  --format json
```

Key source-free outcome metrics from the run:

- Status: `passed`
- Outcome claim: `ctxhelm_improved`
- Comparison-eligible tasks: `4`
- Comparable ctxhelm lanes: `16`
- `ctxhelmUnderReadTargetsObserved`: `false`
- `evidenceOnlyTargetsBeforeRetry`: `8`
- `evidenceOnlyTargetsAfterRetry`: `0`
- `discoveredOnlyTargetsBeforeRetry`: `0`
- `discoveredOnlyTargetsAfterRetry`: `0`
- Target-read coverage before retry: `0.6547619047619048`
- Target-read coverage after retry: `1.0`

## Research Finding

This phase strengthens the reliability claim, not the efficiency claim.

Allowed claim:

```text
ctxhelm retry enforcement closed evidence-only and discovered-only target
consumption gaps in the Phase 324 Codex breadth-suite run.
```

Not allowed yet:

```text
ctxhelm retry improves read efficiency.
```

The promotion-style efficiency gate still failed:

```text
aggregate.irrelevantReadDeltaSum 0 < 2
```

The best ctxhelm lane recovered target-read coverage to `1.0`, but in this run
it required `5` extra reads and `3` extra irrelevant reads versus baseline. The
next R&D slice should focus on reducing retry/read overhead without weakening
the target-consumption boundary.
