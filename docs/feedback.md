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
read count, target-read coverage, discovered-only targets, missed-target counts,
source-free read-role counts, tool-call count, ctxhelm tool-call count, and
forbidden tool-call count. The ctxhelm-assisted lanes also record required,
observed, and missing ctxhelm calls: `ctxhelm-plan` must call `prepare_task`,
and `ctxhelm-brief` must call both `prepare_task` and `get_pack`. Required calls
are argument-validated: `prepare_task` must include the explicit repo and task,
while `get_pack` must include the explicit repo, task, `budget = "brief"`,
`format = "json"`, and `recordTrace = false`. Reports mark a lane as
comparison-eligible only when the client passed and those required calls were
observed with valid arguments, so a failed, skipped, wrong-repo, or malformed
ctxhelm lane cannot be mistaken for weak ctxhelm output. Observed
shell/edit/write tools are surfaced as forbidden calls
and degrade the report status. Client failures are classified source-free as
well: rate limits, API errors, timeouts, and generic non-zero exits are exposed
through `clientFailureKind`, `clientApiErrorStatus`, and rate-limit booleans
without storing raw Claude output. Target coverage can include files
found through search results; target-read coverage requires an actual native
file read, which is the stronger signal for whether an agent consumed the
evidence. Assisted lanes also record source-free ctxhelm evidence attribution:
`ctxhelmEvidenceTargetHits` are targets surfaced by the plan or brief pack,
`ctxhelmEvidenceOnlyTargets` are surfaced targets the agent did not read, and
`ctxhelmEvidenceMissedTargets` are targets not surfaced by ctxhelm evidence for
that lane. This separates retrieval/packing gaps from agent-consumption gaps.
The report stores path labels, hashes, and sanitized MCP request summaries only;
it does not store raw prompts, raw model transcripts, raw MCP traffic, source
snippets, terminal logs, or project test output.

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
summaries: task count, average target-coverage delta, average target-read
coverage delta, total irrelevant-read delta, per-lane target-read/missed-target
counts, read-role counts, tool counts, required-call compliance, eligible-lane
counts, ctxhelm evidence hit/miss/evidence-only counts, and the aggregate
outcome claim. If no baseline plus ctxhelm-assisted lane is comparable, the
outcome claim is `insufficient_comparable_lanes`. This is the preferred path
for comparing native agent search against ctxhelm-assisted exploration across
repeated tasks.

## Release Coverage

The release gate runs `scripts/smoke-feedback.sh`. The smoke proves source-free
feedback ingestion, policy report generation, candidate profile tuning,
profile apply/rollback metadata, and outcome comparison.

Repo feedback does not store raw prompts.
