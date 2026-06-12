# Phase 321: Codex Target-First Efficiency Guidance

## Goal

Reduce noisy reads in the Codex plan, brief, and standard lanes without
weakening the target-consumption proof from Phase 318. Phase 319 showed that
`ctxhelm-memory` was the efficient lane, while plan/brief/standard reached
`1.0` target-read coverage with much lower read precision and higher
irrelevant-read rate.

## Baseline Evidence

Phase 318 aggregate:

```text
baseline:
  target-read coverage: 0.625
  reads: 15
  irrelevant reads: 7
  read precision: 0.5333333333333333

ctxhelm-plan:
  target-read coverage: 1.0
  reads: 36
  irrelevant reads: 23
  read precision: 0.3611111111111111

ctxhelm-brief:
  target-read coverage: 1.0
  reads: 35
  irrelevant reads: 22
  read precision: 0.37142857142857144

ctxhelm-standard:
  target-read coverage: 1.0
  reads: 35
  irrelevant reads: 22
  read precision: 0.37142857142857144

ctxhelm-memory:
  target-read coverage: 1.0
  reads: 22
  irrelevant reads: 9
  read precision: 0.5909090909090909
```

The four-task breadth suite has three target files in the first three tasks
and four target files in the governor/release task. A six-command shell budget
after ctxhelm calls still leaves enough room to read every target in the suite.

## Change

The `ctxhelm-plan`, `ctxhelm-brief`, and `ctxhelm-standard` prompts now use a
target-first efficiency rule:

```text
This is a target-first efficiency probe.
Use at most 6 shell commands total after ctxhelm calls.
Read no more than 6 current files total.
Do not batch-read broad context-area, pack-neighbor, or planning/doc lists.
Stop immediately after target-backed reads are enough to name the key files.
Do not read extra non-target files just to fill the command budget.
```

The existing target-read requirements remain:

```text
consume targetFiles first
read docs/config/schema/script targets as first-class targets
read selectedMemory evidence when present
use get_pack progressively for brief/standard lanes
do not edit, run tests, mutate git, or write files
```

## Claim

Allowed:

```text
The Codex plan, brief, and standard lanes now have the same target-first
efficiency stop condition that made the memory lane the efficient Phase 319
lane.
```

Not allowed until a fresh real Codex run:

```text
The prompt change improved real Codex read efficiency.
```

Phase 320 runner fingerprints intentionally invalidate old checkpoints after
this script change, so stale Phase 318/319 task reports cannot prove the new
prompt.

## Validation

```bash
bash -n scripts/e2e-agent-run-codex.sh
cargo test -p ctxhelm --test release_packaging codex_agent_run_e2e_script_contract --locked -- --nocapture
```

Fresh real-client acceptance target:

```text
ctxhelm lanes preserve 1.0 target-read coverage
plan/brief/standard reduce readFileCount or irrelevantReadCount
no evidence misses
no evidence-only targets after retry
no under-read targets
no forbidden commands
no client failures
no rate limits
```

## Focused Probe

A focused governor/release task was run after the first command-budget-only
prompt change. It preserved target-read coverage and improved plan/brief, but
showed that command caps alone are incomplete because `ctxhelm-standard` can
still read many files in a small number of commands by batching pack/context
paths:

```text
baseline:        target-read 0.75, reads 5,  irrelevant 2
ctxhelm-plan:    target-read 1.00, reads 6,  irrelevant 2
ctxhelm-brief:   target-read 1.00, reads 6,  irrelevant 2
ctxhelm-standard target-read 1.00, reads 11, irrelevant 7
ctxhelm-memory:  target-read 1.00, reads 5,  irrelevant 1
```

The prompt now includes an explicit current-file read cap and batch-read guard
for plan/brief/standard. The next real run should check whether that closes the
standard-lane batching gap.

## Final Focused Proof

After adding the current-file cap and batch-read guard, the same focused
governor/release task was rerun with Codex CLI `0.137.0`:

```text
status: passed
outcome: ctxhelm_improved
comparison eligible: true
target-read coverage delta: +0.25
forbidden commands: false
client failures: false
rate limits: false
ctxhelm evidence misses: false
evidence-only targets: false
under-read targets: false
```

Lane metrics:

```text
baseline:         target-read 0.75, reads 5, irrelevant 2, precision 0.60
ctxhelm-plan:     target-read 1.00, reads 6, irrelevant 2, precision 0.6667
ctxhelm-brief:    target-read 1.00, reads 6, irrelevant 2, precision 0.6667
ctxhelm-standard: target-read 1.00, reads 6, irrelevant 2, precision 0.6667
ctxhelm-memory:   target-read 1.00, reads 6, irrelevant 2, precision 0.6667
```

Compared with the command-budget-only probe, `ctxhelm-standard` improved from
`11` reads and `7` irrelevant reads to `6` reads and `2` irrelevant reads while
preserving full target-read coverage. The focused task still costs one extra
read versus baseline to recover the missed `docs/context-governor.md` target, so
this is focused evidence for closing the standard-lane batching gap, not a
broad-suite claim.
