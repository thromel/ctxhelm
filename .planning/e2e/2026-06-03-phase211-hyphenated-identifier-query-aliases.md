# Phase 211 - Hyphenated Identifier Query Aliases

## Goal

Fix the first retrieval miss exposed by Phase 210's ctxhelm evidence
attribution.

## Observed Miss

Task:

```text
Improve agent-run report attribution
```

Targets:

```text
scripts/e2e-agent-run.sh
crates/ctxhelm/src/main.rs
```

Before the fix, `prepare-task` surfaced the harness script but did not put the
CLI report renderer in the top 10. The query used `agent` and `run` as ordinary
lexical terms, but did not create the code-identifier alias `agent_run`, so
symbol search could not match `render_agent_run_report`.

## Change

- Lexical query terms now add conservative hyphen-to-underscore aliases.
  Example: `agent-run` -> `agent_run`.
- Query construction now adds the same aliases as symbol facets when they look
  identifier-like.
- The lexical search cache key was versioned so old cached results do not hide
  the changed behavior.

## Local Evidence

Focused tests:

```bash
cargo test -p ctxhelm-index query_terms_add_hyphenated_identifier_aliases --locked -- --nocapture
cargo test -p ctxhelm-compiler hyphenated --locked -- --nocapture
cargo test -p ctxhelm-compiler prepare_plan_selects_renderer_for_hyphenated_agent_run_task --locked -- --nocapture
```

Real query probe after the fix:

```bash
cargo run -p ctxhelm --locked -- prepare-task \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --no-trace \
  "Improve agent-run report attribution"
```

Observed summary:

```text
symbol terms ['agent_run']
1 crates/ctxhelm-compiler/src/planning.rs lexical match
2 crates/ctxhelm/src/main.rs symbol match
3 scripts/e2e-agent-run.sh lexical match
```

Brief pack evidence after the fix:

```text
has script True
has main True
```

Real Claude Code probe after the fix:

```bash
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=90 \
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
  bash scripts/e2e-agent-run.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --task "Improve agent-run report attribution" \
  --target-file scripts/e2e-agent-run.sh \
  --target-file crates/ctxhelm/src/main.rs \
  --output /tmp/ctxhelm-phase211-real.json
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
baseline failed rate_limited read 0 ctxEvidenceHits 0 ctxEvidenceMisses 2
ctxhelm-plan failed rate_limited read 0 ctxEvidenceHits 2 ctxEvidenceMisses 0
ctxhelm-brief failed rate_limited read 0 ctxEvidenceHits 2 ctxEvidenceMisses 0
```

Interpretation: Claude Code is still unavailable due to rate limiting, so this
is not outcome-lift proof. It is retrieval/packing proof for the measured miss:
ctxhelm now surfaces both targets in both assisted lanes, and the remaining
gap is client availability/agent consumption.

## Next

- Rerun the paired suite when Claude Code is not rate-limited.
- Use future `ctxhelmEvidenceMissedTargets` entries as ranking/query-fix inputs.
- Use future `ctxhelmEvidenceOnlyTargets` entries as agent-guidance inputs.
