# Phase 212 - Agent-Run R&D Action Routing

## Goal

Turn the paired real-agent report from a passive diagnostic into an actionable
R&D router.

Phase 210 separated ctxhelm evidence misses from agent consumption misses, and
Phase 211 fixed the first concrete retrieval miss exposed by that split. The
next gap was operational: a report could show rate limits, missing required
calls, evidence misses, evidence-only targets, or no-lift outcomes, but it did
not state which R&D loop should run next.

## Change

Single-run comparisons and suite aggregates now include
`recommendedResearchActions`.

The actions are source-free and path-free:

```text
retry_real_client_when_available
collect_real_client_evidence
fix_retrieval_or_query_construction
improve_agent_consumption_guidance
harden_required_ctxhelm_call_guidance
inspect_pack_ordering_and_native_read_instruction
analyze_native_baseline_gap
preserve_current_agent_contract
```

The routing is intentionally conservative:

- Client failures or rate limits route to retrying the real client.
- Skipped non-real reports route to collecting real-client evidence.
- Missing/invalid required calls route to guidance hardening only when a
  ctxhelm tool call was actually observed.
- Evidence misses route to retrieval/query fixes even when the client is not
  available.
- Evidence-only and under-read targets route to consumption guidance only when
  the client was available enough to make consumption meaningful.
- Comparable no-lift outcomes route to native-baseline analysis.
- Stable comparable improved/matched outcomes route to preserving the current
  contract.

## Local Evidence

Focused validation:

```bash
cargo fmt --check
bash -n scripts/e2e-agent-run.sh
cargo test -p ctxhelm --test cli_compat eval_agent_run --locked -- --nocapture
cargo test -p ctxhelm --test release_packaging agent_run_e2e_script_contract --locked -- --nocapture
```

Observed result:

```text
cli_compat eval_agent_run: 3 passed
release_packaging agent_run_e2e_script_contract: 1 passed
```

Skipped contract probe:

```bash
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
  bash scripts/e2e-agent-run.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --task "Improve agent-run report attribution" \
  --target-file scripts/e2e-agent-run.sh \
  --target-file crates/ctxhelm/src/main.rs \
  --output /tmp/ctxhelm-phase212-skip.json
```

Observed summary:

```text
status skipped
outcome insufficient_comparable_lanes
recommendedResearchActions collect_real_client_evidence
```

Real Claude Code probe:

```bash
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=90 \
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
  bash scripts/e2e-agent-run.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --task "Improve agent-run report attribution" \
  --target-file scripts/e2e-agent-run.sh \
  --target-file crates/ctxhelm/src/main.rs \
  --output /tmp/ctxhelm-phase212-real.json
```

Observed summary:

```text
status skipped
outcome insufficient_comparable_lanes
comparisonEligible false
clientFailuresObserved true
rateLimitsObserved true
ctxhelmEvidenceMissesObserved false
ctxhelmEvidenceOnlyTargetsObserved true
recommendedResearchActions retry_real_client_when_available
```

Interpretation: Claude Code remains unavailable due to rate limiting, so this
is not outcome-lift proof. The accepted improvement is that the report no
longer misroutes rate-limited missing calls or evidence-only targets into
guidance work. It correctly says the next outcome-proof step is a retry when the
client is available.

## Remaining R&D

- Rerun the paired suite after the Claude Code rate limit clears.
- Use `fix_retrieval_or_query_construction` actions as direct ranking/query
  work inputs.
- Use `improve_agent_consumption_guidance` actions as pack/MCP guidance inputs.
- Use `analyze_native_baseline_gap` actions to study cases where native search
  already covers the target set.
