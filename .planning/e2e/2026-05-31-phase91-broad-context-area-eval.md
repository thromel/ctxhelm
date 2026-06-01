# Phase 91: Broad Context-Area Eval

## Goal

Make broad multi-area historical evals honest about the context plan's
progressive-disclosure behavior. A top-10 file list cannot fully cover commits
that touch dozens of source files, but the plan should still expose the
implementation areas the agent should inspect next.

## Changes

- Historical eval commits now record:
  - `changedContextAreas`
  - `contextAreaHits`
  - `contextAreas`
- Historical eval reports now include `broadContextAreaRecall`, averaged only
  over broad-scope commits with changed context areas.
- Broad context-area ordering now prefers implementation/config/schema areas
  before test/example areas because related tests already have a separate
  recommendation channel.
- The context-area cap increased from 8 to 12. This is source-free metadata,
  not source snippets, and it does not displace target files or tests.

## Focused VeriSchema Probe

Command:

```bash
cargo run -p ctxhelm -- eval history \
  --repo /Users/romel/Documents/GitHub/VeriSchema \
  --base b5cfb2a551d026514f505c45863db31277bcd1ad^ \
  --head b5cfb2a551d026514f505c45863db31277bcd1ad \
  --limit 1 \
  --budget 10 \
  --mode bug-fix \
  --target-agent generic \
  --force \
  --format json
```

Result for the hard broad lint/workflow commit:

| Metric | Before | After |
| --- | ---: | ---: |
| File Recall@10 | `0.15384616` | `0.15384616` |
| Source Recall@10 | `0.13157895` | `0.13157895` |
| Broad context-area recall | not measured | `0.78571427` |
| Context areas surfaced | `8` before priority/cap change | `12` |

The change does not pretend the top-10 file list covers the whole 39-file patch.
It proves the source-free context map now surfaces 11 of 14 changed areas for
the broad commit.

## Broad Fixed-Corpus Proof

Debug command:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json
```

Debug result:

- Existing quality metrics stayed stable.
- VeriSchema `broadContextAreaRecall = 0.64708996` across 4 broad-scope commits.
- Gate blocked only on the known debug runtime threshold.

Release command:

```bash
cargo run --release -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json
```

Release proof artifact:

```text
.ctxhelm/e2e/phase91-broad-context-area-release-proof.json
```

Release result:

| Corpus | Status | File Recall@10 | Context Recall@10 | Test Recall@10 | Protected Target Miss-Rate@10 | Runtime |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | match | `0.6` | `1.0` | `1.0` | `0.0` | `8415ms` |
| ctxhelm | beat | `0.44603175` | `0.44444445` | `0.0` | `0.0` | `8752ms` |
| ReAgent | beat | `0.5` | `1.0` | `1.0` | `0.0` | `4831ms` |
| VeriSchema | beat | `0.18449473` | `0.23287672` | `0.7089947` | `0.2857143` | `7305ms` |

`releaseGate.decision = promote`.

## Result

This phase improves production readiness by adding a measured source-free
broad-task map channel. File-level ranking is unchanged, but broad evals now
show whether ctxhelm gives agents the right implementation areas to inspect
after the initial target list.
