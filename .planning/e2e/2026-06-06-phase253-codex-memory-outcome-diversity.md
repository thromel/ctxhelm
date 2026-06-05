# Phase 253 - Codex Memory Outcome Diversity

## Goal

Move the experience-memory R&D lane beyond local ranking metrics and single-repo
agent proof by measuring real Codex CLI behavior across multiple repositories
with approved memory available.

## Harness

Added `scripts/e2e-codex-memory-outcome-suite.sh`.

For each requested repository, the harness:

1. discovers repeated-file historical pairs;
2. creates an isolated `CTXHELM_HOME`;
3. seeds one approved source-free experience card from the older pair task;
4. runs the existing paired read-only Codex harness on the newer pair task;
5. writes a source-free aggregate report.

The report stores repo labels, commit prefixes, path labels, hashes, counters,
and booleans only. It does not store raw task text, prompts, transcripts, MCP
traffic, command output, source snippets, or raw repository paths.

## Evidence

Source-free artifact:

- `.ctxhelm/e2e/phase253-codex-memory-outcome-suite.json`

Run scope:

- Codex CLI `0.137.0`
- ctxhelm `1.1.12`
- 3 repositories: VeriSchema, ReAgent, RefactoringMiner
- 1 repeated-file memory pair per repository
- 3 comparison-eligible real-client pairs

Aggregate result:

| Metric | Value |
| --- | ---: |
| Status | `passed` |
| Improved pairs | `3` |
| Memory target-read improved pairs | `3` |
| Memory target-read matched-or-improved pairs | `3` |
| Memory irrelevant-read improved pairs | `0` |
| Evidence miss pairs | `0` |
| Under-read pairs | `0` |
| Client failure / rate-limit pairs | `0` |
| Forbidden command pairs | `0` |
| Missing/invalid required ctxhelm call pairs | `0` |

Per-repo memory-lane target-read coverage:

| Repo | Target | Baseline | Memory lane |
| --- | --- | ---: | ---: |
| VeriSchema | `schema_agent/agents/entity_discovery.py` | `0.0` | `1.0` |
| ReAgent | `ccia/types.py` | `0.0` | `1.0` |
| RefactoringMiner | `src/main/java/org/refactoringminer/astDiff/matchers/wrappers/MethodMatcher.java` | `0.0` | `1.0` |

Privacy flags remain local-only with no source text, raw task text, raw prompts,
raw transcripts, raw MCP traffic, raw command output, remote embeddings, or
remote reranking.

## Interpretation

This phase closes the current "single-repo only" real-agent memory outcome gap:
approved memory-backed ctxhelm evidence caused Codex to consume the target file
across three different repositories.

This is not an efficiency claim. In this rerun, the memory lane improved target
consumption but did not reduce irrelevant reads. Future memory R&D should keep
target-consumption coverage as the regression guard and separately optimize
read-count efficiency.
