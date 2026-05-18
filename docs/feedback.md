# Feedback And Policy Learning

ctxpack feedback is a local, source-free event stream for comparing context
recommendations with what agents actually read, edited, tested, and validated.
It is the foundation for adaptive retrieval policy work.

## Record Feedback

```bash
ctxpack eval feedback record --repo "$REPO" \
  --task-hash "<trace-or-pack-task-hash>" \
  --mode bug-fix \
  --target-agent codex \
  --budget brief \
  --outcome passed \
  --recommended-file src/auth/session.ts \
  --read-file src/auth/session.ts \
  --edited-file src/auth/session.ts \
  --tested-file tests/auth/session.test.ts \
  --tested-command "pnpm test tests/auth/session.test.ts"
```

Feedback events store task hashes, task type, target agent, pack budget,
outcome, path labels, command labels, and tags. They do not store raw prompts,
terminal logs, source snippets, secrets, cloud payloads, or model transcripts.

## Inspect Feedback

```bash
ctxpack eval feedback list --repo "$REPO"
ctxpack eval feedback summary --repo "$REPO"
```

These commands read local `feedback.jsonl` metadata from `CTXPACK_HOME` and
render Markdown or JSON. `sourceTextLogged` must stay `false`.

## Policy Reports

```bash
ctxpack eval policy report --repo "$REPO"
```

Policy reports compare recommended files and tests with read, edited, tested,
and corrected files. Reports include context precision, read precision, edit
recall proxy, validation coverage, correction rate, repeated missing-file
families, signal contributions, and token ROI by pack budget when enough
metadata exists.

Low-sample reports include a warning. Treat them as directional evidence, not
as proof that retrieval improved.

## Policy Profiles

```bash
ctxpack eval policy tune --repo "$REPO"
ctxpack eval policy list --repo "$REPO"
ctxpack eval policy apply <profile-id> --repo "$REPO"
ctxpack eval policy disable <profile-id> --repo "$REPO"
ctxpack eval policy rollback --repo "$REPO"
```

`tune` writes a candidate local retrieval-policy profile from source-free
feedback evidence. Profiles are inspectable and disabled by default until a
maintainer applies one. Profiles include rationale, weights, safety floors,
regression warnings, sample counts, and rollback metadata.

Safety floors keep exact anchors, lexical identifiers, and validation context
from being demoted below conservative minimums.

## Outcome Comparison

```bash
ctxpack eval outcome compare --repo "$REPO"
```

Outcome comparison groups feedback by plan-only, brief, standard, and deep pack
budgets. It reports pass rate, blocked rate, correction rate, validation
coverage, average recommended context size, and useful target files per 1k
estimated tokens.

Changed-sample and low-information warnings are explicit. Do not claim lift
from a small or shifting sample.

## Release Coverage

The release gate runs `scripts/smoke-feedback.sh`. The smoke proves source-free
feedback ingestion, policy report generation, candidate profile tuning,
profile apply/rollback metadata, and outcome comparison.

Repo feedback does not store raw prompts.
