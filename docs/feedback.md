# Feedback And Policy Learning

ctxhelm feedback is a local, source-free event stream for comparing context
recommendations with what agents actually read, edited, tested, and validated.
It is the foundation for adaptive retrieval policy work.

## Record Feedback

```bash
ctxhelm eval feedback record --repo "$REPO" \
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
ctxhelm eval feedback list --repo "$REPO"
ctxhelm eval feedback summary --repo "$REPO"
```

These commands read local `feedback.jsonl` metadata from `CTXHELM_HOME` and
render Markdown or JSON. `sourceTextLogged` must stay `false`.

## Policy Reports

```bash
ctxhelm eval policy report --repo "$REPO"
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
ctxhelm eval policy tune --repo "$REPO"
ctxhelm eval policy learn --repo "$REPO"
ctxhelm eval policy list --repo "$REPO"
ctxhelm eval policy apply <profile-id> --repo "$REPO"
ctxhelm eval policy disable <profile-id> --repo "$REPO"
ctxhelm eval policy rollback --repo "$REPO"
```

`tune` writes a candidate local retrieval-policy profile from source-free
feedback evidence. Profiles are inspectable and disabled by default until a
maintainer applies one. Profiles include rationale, weights, safety floors,
regression warnings, sample counts, and rollback metadata.

`learn` writes a candidate offline learned profile from local source-free
candidate feature exports, historical labels carried by those feature rows, and
feedback/outcome traces. Learned profiles include schema version, training
corpus ID, training sources, metric summary, baseline thresholds, and a
`defaultEligible` flag. `apply` refuses profiles whose thresholds did not pass.

Safety floors keep exact anchors, lexical identifiers, and validation context
from being demoted below conservative minimums.

## Outcome Comparison

```bash
ctxhelm eval outcome compare --repo "$REPO"
```

Outcome comparison groups feedback by plan-only, brief, standard, and deep pack
budgets. It reports pass rate, blocked rate, correction rate, validation
coverage, average recommended context size, and useful target files per 1k
estimated tokens.

Changed-sample and low-information warnings are explicit. Do not claim lift
from a small or shifting sample.

## Paired Agent Runs

Use the paired agent-run harness when you need real-client process evidence,
not just feedback entered after a session:

```bash
CTXHELM_RUN_REAL_CLIENT=1 bash scripts/e2e-agent-run.sh \
  --repo "$REPO" \
  --task "Identify files relevant to improving the Claude workflow eval harness" \
  --target-file scripts/e2e-claude-workflow.sh \
  --target-file scripts/smoke-claude-mcp.sh \
  --output .ctxhelm/e2e/agent-run-claude.json

ctxhelm eval agent-run --report .ctxhelm/e2e/agent-run-claude.json
```

The script runs three read-only Claude Code lanes: native baseline,
`prepare_task`, and `prepare_task` plus a brief `get_pack`. It records
source-free lane metrics such as target coverage, read-file count, irrelevant
read count, tool-call count, and ctxhelm tool-call count. It stores path labels,
hashes, and sanitized MCP request summaries only; it does not store raw prompts,
raw model transcripts, raw MCP traffic, source snippets, terminal logs, or
project test output.

Without `CTXHELM_RUN_REAL_CLIENT=1`, the script writes a skipped report so CI
can verify the contract without pretending a real agent ran. Treat a passing
agent-run report as process evidence for that task and client version, not as a
general retrieval-quality proof.

For benchmark-style evidence, run the same harness against a suite:

```json
{
  "schemaVersion": "ctxhelm-native-agent-suite-v1",
  "tasks": [
    {
      "id": "release-docs",
      "task": "Identify files relevant to release-proof documentation drift",
      "targetFiles": ["README.md", "docs/release.md"]
    }
  ]
}
```

```bash
CTXHELM_RUN_REAL_CLIENT=1 bash scripts/e2e-agent-run.sh \
  --repo "$REPO" \
  --suite .ctxhelm/outcomes/tasks.json \
  --output .ctxhelm/e2e/agent-run-suite-claude.json

ctxhelm eval agent-run --report .ctxhelm/e2e/agent-run-suite-claude.json
```

Suite reports keep the same source-free contract and add aggregate lane
summaries: task count, average target-coverage delta, total irrelevant-read
delta, per-lane read/tool counts, and the aggregate outcome claim. This is the
preferred path for comparing native agent search against ctxhelm-assisted
exploration across repeated tasks.

## Release Coverage

The release gate runs `scripts/smoke-feedback.sh`. The smoke proves source-free
feedback ingestion, policy report generation, candidate profile tuning,
profile apply/rollback metadata, and outcome comparison.

Repo feedback does not store raw prompts.
