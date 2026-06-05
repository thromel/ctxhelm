# Repo Memory

ctxhelm memory is a local, source-free layer for durable repo lessons. It is
selected per task and budget; it is not injected into every prompt.

## Generate Domain Memory

```bash
ctxhelm cards generate --repo "$REPO"
```

This writes `.ctxhelm/cards/*.md` and stores matching source-free memory-card
metadata in local SQLite. Domain cards include IDs, titles, summaries, source
links, input hashes, freshness, review status, confidence, and privacy status.
They summarize safe inventory, tests, dependency edges, and path domains. They
do not store raw file contents, source snippets, secrets, or raw prompts.

## Generate Disconnected Fallback Cards

```bash
ctxhelm cards fallback --repo "$REPO" --target-agent codex
```

This regenerates source-free context cards and writes a guide under
`.ctxhelm/fallback/<agent>-context.md`. Use it for cloud, copied, or
disconnected agent contexts where local MCP is unavailable. The guide points
agents to `AGENTS.md` and the source-free cards, but it still tells them to
inspect current files with native tools before editing.

## Generate Experience Memory

```bash
ctxhelm memory generate-experience --repo "$REPO" --limit 20
ctxhelm memory list --repo "$REPO"
ctxhelm memory show experience:<task-hash> --repo "$REPO"
ctxhelm memory regenerate --repo "$REPO"
```

Experience cards are derived from local eval trace metadata only: task hashes,
task type, target agent, budget, recommended files, recommended tests, and
recommended commands. They do not store raw prompts, terminal logs, source text,
or model transcripts. Repo memory does not store raw prompts.

Experience cards default to `pending` review. Pending, rejected, disabled,
stale, or degraded cards are blocked from pack inclusion by default.

## Review Controls

```bash
ctxhelm memory approve experience:<task-hash> --repo "$REPO"
ctxhelm memory reject experience:<task-hash> --repo "$REPO"
ctxhelm memory disable experience:<task-hash> --repo "$REPO"
```

`approve` makes a fresh card eligible for task-conditioned selection. `reject`
and `disable` keep the card record for auditability but prevent pack inclusion.

## Plan And Pack Selection

`prepare-task` returns selected memory under `selectedMemory` when a fresh,
approved or deterministic card overlaps the current task or selected target
files. `get-pack` renders the result in a separate `Selected memory` section so
memory cannot crowd out target files, tests, or constraints.

MCP exposes the same data additively through existing plan/pack flows and the
resource URI:

```text
ctxhelm://repo/memory
```

## Release Coverage

The release gate runs `scripts/smoke-memory.sh`,
`scripts/smoke-memory-reuse.sh`, `scripts/smoke-memory-history-lift.sh`,
`scripts/smoke-memory-parent-snapshot-lift.sh`, and
`scripts/smoke-memory-benchmark-lift.sh`.
The memory smoke proves local-only storage, source-free persistence,
stale/pending review blocking, selected-memory pack output, and approve/disable
review controls. The memory reuse smoke proves that pending experience cards are
blocked, approved experience memory can promote a source-linked target on a
related task, and storage does not persist source sentinels, raw prompts,
transcripts, or MCP traffic. The memory history-lift smoke runs `eval history`
before and after approval and proves the historical report records a unique
memory target hit beyond lexical, with `evaluate_memory_reuse_lift` as the
source-free R&D action. The memory parent-snapshot lift smoke proves the same
memory path still works for non-root historical commits, where `eval history`
builds a parent-revision snapshot with a different local storage identity from
the source checkout. The memory benchmark-lift smoke raises that proof to the
benchmark/product-proof layer by running two local repositories through
`eval proof` and requiring both embedded reports to show memory-only target hits
beyond lexical plus product-level `evaluate_memory_reuse_lift` routing. These
are still controlled release smokes, not proof of broad memory generalization.

For broader R&D measurement, use:

```bash
scripts/measure-memory-generalization.sh \
  --repo ../ctxhelm-proof-fixtures/RefactoringMiner \
  --pairs 2 \
  --scan-commits 200 \
  --output .ctxhelm/e2e/phase219-refactoringminer-memory-generalization.json
```

That harness scans a real local repository for repeated-file historical pairs,
seeds one approved source-free experience card from the older task, evaluates
the newer task before and after approval, and aggregates memory lift, lexical
coverage, unique non-target noise, and runtime. It is intentionally a
measurement report rather than a release-gate pass/fail smoke: weak or noisy
memory lift should be recorded honestly so the next ranking work is grounded in
evidence.

Experience cards preserve the original recommendation order from the eval trace:
recommended files first, then recommended tests, with duplicates removed without
sorting. Memory source-link candidates are capped to a small source-like context
set per card and do not inject tests into the ranked context budget. Tests can
still appear in selected memory source links and normal validation channels, but
old experience memory cannot flood top-10 context with every prior test or
auxiliary file.

For multi-repo R&D, run:

```bash
scripts/measure-memory-generalization-suite.sh \
  --repo ../ctxhelm-proof-fixtures/RefactoringMiner \
  --repo ../ctxhelm-proof-fixtures/VeriSchema \
  --repo ../ctxhelm-proof-fixtures/ReAgent \
  --repo ../ctxhelm-proof-fixtures/ctxhelm \
  --pairs 1 \
  --scan-commits 120 \
  --output .ctxhelm/e2e/phase221-memory-generalization-suite.json
```

The suite wrapper runs the single-repo harness for each checkout and aggregates
only source-free labels, hashes, booleans, counters, and runtime. It should be
used before claiming memory lift generalizes across repositories.

To compare memory against local semantic and graph signals in the same repeated
history tasks, enable the local semantic lane:

```bash
scripts/measure-memory-generalization-suite.sh \
  --repo ../ctxhelm-proof-fixtures/RefactoringMiner \
  --repo ../ctxhelm-proof-fixtures/VeriSchema \
  --repo ../ctxhelm-proof-fixtures/ReAgent \
  --repo ../ctxhelm-proof-fixtures/ctxhelm \
  --pairs 1 \
  --scan-commits 120 \
  --semantic \
  --semantic-provider local_hash \
  --output .ctxhelm/e2e/phase222-memory-signal-ablation-suite.json
```

The v2 report adds `semantic`, `signalComparison`, graph-edge ablation counters,
semantic selected-target pairs, and memory corroboration bounds. These are
source-free aggregate diagnostics, not path-level overlap claims: the report
stores upper/lower bounds because it does not persist raw candidate paths or
source text. Treat `memoryNeedsCorroboration = true` as evidence that future
ranking should demote memory candidates without target, graph, semantic, or
other correlated support.
