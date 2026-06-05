# Phase 251 - Codex R&D Breadth Suite

## Scope

Expand real Codex outcome proof beyond the earlier two-task suite and the
single governor regression task. The suite covers four R&D task families:

- selected-memory native-read behavior
- semantic provider/search/contribution diagnostics
- GraphRAG edge profiles, ablations, and edge-family budget allocation
- context-governor and release-gate proof artifacts

Suite manifest:

```text
.planning/e2e/2026-06-06-phase251-codex-rd-suite.json
```

Final source-free report:

```text
.ctxhelm/e2e/phase251-agent-run-codex-rd-breadth-suite.json
```

## Finding

The first dry `prepare-task` probe exposed a real GraphRAG retrieval gap:
graph-edge budget tasks surfaced `crates/ctxhelm-compiler/src/eval.rs` but did
not place `crates/ctxhelm-compiler/src/ranking.rs` or
`crates/ctxhelm-index/src/dependencies.rs` in the top targets.

The first real Codex run after that retrieval fix exposed a second issue:
some lanes used `rg` as if it were file consumption. The evaluator correctly
classified those as discovered-only targets, not native reads.

The second real Codex run cleared under-read but exposed forbidden `awk` plus
shell-redirection usage in two lanes. That was a harness prompt boundary gap.

## Fixes

- GraphRAG/graph-edge R&D tasks now promote bounded implementation surfaces:
  - `crates/ctxhelm-compiler/src/eval.rs`
  - `crates/ctxhelm-compiler/src/ranking.rs`
  - `crates/ctxhelm-index/src/dependencies.rs`
- The Codex real-agent harness now distinguishes discovery from consumption:
  `rg`, `grep`, `find`, `ls`, and `wc` may locate/count evidence, but only
  `sed`, `cat`, `nl`, `head`, or `tail` count as consuming a target file.
- The Codex real-agent harness now explicitly forbids `awk` and shell
  redirection in the read-only prompts.

## Final Result

Command:

```bash
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=150 \
bash scripts/e2e-agent-run-codex.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json \
  --output .ctxhelm/e2e/phase251-agent-run-codex-rd-breadth-suite.json
```

Observed:

- Status: `passed`
- Client: Codex CLI `0.137.0`
- Tasks: `4`
- Comparison-eligible tasks: `4`
- Comparable ctxhelm lanes: `16`
- Outcome claim: `ctxhelm_improved`
- Target coverage delta average: `+0.38`
- Target read coverage delta average: `+0.54`
- Read-file delta sum: `-8`
- ctxhelm evidence misses observed: `false`
- ctxhelm evidence-only targets observed: `false`
- ctxhelm under-read targets observed: `false`
- Forbidden commands observed: `false`
- Missing required ctxhelm calls observed: `false`
- Invalid required ctxhelm calls observed: `false`
- Client failures observed: `false`
- Rate limits observed: `false`
- Privacy: local-only, no raw prompts, raw transcripts, raw MCP traffic, raw
  command output, source text, remote embeddings, or remote reranking stored.

Lane summary:

- Baseline average target-read coverage: `0.46`
- `ctxhelm-plan` average target-read coverage: `1.00`
- `ctxhelm-brief` average target-read coverage: `1.00`
- `ctxhelm-standard` average target-read coverage: `1.00`
- `ctxhelm-memory` average target-read coverage: `1.00`

## Interpretation

Phase 251 closes the current single-repo larger Codex suite gap from the R&D
audit. It also turns the GraphRAG task-family miss into a focused retrieval
fix and tightens the real-agent harness so discovery is not misreported as
file consumption.

This does not close all R&D. Semantic retrieval is still policy-gated until it
shows semantic-only or query-family lift, and fresh paired Claude outcome proof
still depends on client availability.
