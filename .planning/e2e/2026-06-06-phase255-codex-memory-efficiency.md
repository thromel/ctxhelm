# Phase 255 - Codex Memory Efficiency

## Goal

Turn the Phase 253 memory target-consumption proof into a stronger memory
outcome proof by reducing extra reads in the memory lane without losing target
consumption.

## Change

Tightened only the Codex `ctxhelm-memory` lane prompt in
`scripts/e2e-agent-run-codex.sh`.

The memory lane now behaves as a memory-efficiency probe:

- at most 4 shell commands after ctxhelm calls;
- choose the smallest current-file set that memory points to;
- prefer paths present in both memory evidence and current target/high-confidence
  pack evidence;
- consume at most 2 memory-backed current files first;
- stop early when those reads answer the task;
- only read extra files when the memory path is missing, stale, or needs one
  immediate neighbor.

The baseline, plan, brief, and standard lane prompts were not changed.

## Evidence

Source-free artifact:

- `.ctxhelm/e2e/phase255-codex-memory-efficiency-suite.json`

Run scope:

- Codex CLI `0.137.0`
- ctxhelm `1.1.12`
- 3 repositories: VeriSchema, ReAgent, RefactoringMiner
- 1 repeated-file memory pair per repository
- 3 comparison-eligible real-client pairs

Aggregate before/after:

| Metric | Phase 253 | Phase 255 |
| --- | ---: | ---: |
| Improved pairs | `3` | `3` |
| Memory target-read improved pairs | `3` | `3` |
| Memory target-read matched-or-improved pairs | `3` | `3` |
| Memory irrelevant-read improved pairs | `0` | `3` |
| Evidence miss pairs | `0` | `0` |
| Under-read pairs | `0` | `0` |
| Client failure / rate-limit pairs | `0` | `0` |
| Forbidden command pairs | `0` | `0` |
| Missing/invalid required ctxhelm call pairs | `0` | `0` |

Per-repo memory-lane read behavior:

| Repo | Target | Target read | Memory reads | Irrelevant reads |
| --- | --- | ---: | ---: | ---: |
| VeriSchema | `schema_agent/agents/entity_discovery.py` | `1.0 -> 1.0` | `6 -> 2` | `5 -> 1` |
| ReAgent | `ccia/types.py` | `1.0 -> 1.0` | `6 -> 4` | `5 -> 3` |
| RefactoringMiner | `src/main/java/org/refactoringminer/astDiff/matchers/wrappers/MethodMatcher.java` | `1.0 -> 1.0` | `5 -> 2` | `4 -> 1` |

Privacy flags remain local-only with no source text, raw task text, raw prompts,
raw transcripts, raw MCP traffic, raw command output, remote embeddings, or
remote reranking.

## Interpretation

This closes the current measured memory-efficiency R&D gap for the three-repo
Codex memory outcome slice. The new prompt preserves target consumption and
reduces irrelevant reads in every measured pair.

This is still not a universal memory-efficiency proof. Larger pair counts and
additional repos remain useful regression tests, but the current bottleneck is
no longer "memory adds target reads but always adds exploratory noise" on the
measured real-client slice.
