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

For Codex CLI, use the Codex-specific harness:

```bash
CTXHELM_RUN_REAL_CLIENT=1 bash scripts/e2e-agent-run-codex.sh \
  --repo "$REPO" \
  --task "Identify files relevant to improving the paired agent-run matrix" \
  --target-file scripts/e2e-agent-run.sh \
  --target-file docs/feedback.md \
  --output .ctxhelm/e2e/agent-run-codex.json

ctxhelm eval agent-run --report .ctxhelm/e2e/agent-run-codex.json
```

The Claude script runs five read-only Claude Code lanes: native baseline,
`prepare_task`, `prepare_task` plus a brief `get_pack`, `prepare_task` plus a
standard `get_pack`, and a memory-guided standard-pack lane. It records
one source-free `clientPreflight` result before the lane matrix by default
(`CTXHELM_AGENT_RUN_PREFLIGHT=0` disables it). If preflight detects a rate
limit, API error, timeout, or non-zero client exit, the report records that
client failure once and skips live lane execution while still collecting
ctxhelm evidence for assisted lanes. It records source-free lane metrics such as
target coverage, read-file count, irrelevant
read count, target-read coverage, discovered-only targets, missed-target counts,
source-free read-role counts, tool-call count, ctxhelm tool-call count, and
forbidden tool-call count. The ctxhelm-assisted lanes also record required,
observed, and missing ctxhelm calls: `ctxhelm-plan` must call `prepare_task`,
while `ctxhelm-brief`, `ctxhelm-standard`, and `ctxhelm-memory` must call both
`prepare_task` and `get_pack`. Required calls are argument-validated:
`prepare_task` must include the explicit repo and task, while `get_pack` must
include the explicit repo, task, expected budget, `format = "json"`, and
`recordTrace = false`. Reports mark a lane as comparison-eligible only when the
client passed and those required calls were observed with valid arguments, so a
failed, skipped, wrong-repo, or malformed ctxhelm lane cannot be mistaken for
weak ctxhelm output. Observed
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
Reports also expose `recommendedResearchActions`, a source-free routing field
for the next R&D loop. Client failures and rate limits route to retrying the
real client, skipped contract-only reports route to collecting real-client
evidence, evidence misses route to retrieval/query fixes, evidence-only targets
route to consumption guidance, malformed observed ctxhelm calls route to
required-call guidance, and comparable no-lift results route to native-baseline
analysis.

The Codex script runs the same five lane modes, but measures native consumption
through Codex `command_execution` events. It stores command hashes, command
verbs, exit statuses, path labels, read/discovery counts, and forbidden-command
counts; it does not store raw command output, raw prompts, raw transcripts, raw
MCP traffic, or source text. On current Codex CLI versions, the script uses
`--ignore-user-config`/`--ephemeral` when available while preserving the normal
auth home. Do not force a temporary `CODEX_HOME` unless the installed client
requires it; doing so can remove auth and create false stream-disconnect
failures.

Historical eval and product-proof reports use the same
`recommendedResearchActions` shape for report-level R&D routing. Historical
reports can recommend candidate generation, ranking or budget allocation,
progressive-read alignment, validation-test mapping, memory reuse evidence
collection or selection work, and graph edge-budget work. Historical reports
also expose `memoryReuseSummary`, which counts memory candidates, selected
memory evidence, memory target hits/misses, and unique target or non-target
memory contributions beyond lexical baseline evidence. Product proof reports can
recommend fixture/history refresh, runtime work, protected-evidence
preservation, retrieval/ranking fixes, native-baseline analysis, BM25 evidence
collection, memory reuse proof, or preserving the current contract when no
source-free bottleneck is present.
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
counts, read-role counts, tool counts, required-call compliance, preflight
failure/rate-limit counts, eligible-lane
counts, ctxhelm evidence hit/miss/evidence-only counts, and the aggregate
outcome claim. If no baseline plus ctxhelm-assisted lane is comparable, the
outcome claim is `insufficient_comparable_lanes`. This is the preferred path
for comparing native agent search against ctxhelm-assisted exploration across
repeated tasks.

Persisted suite reports can be checked without rerunning a live agent:

```bash
python3 scripts/check-agent-run-proof.py \
  .ctxhelm/e2e/agent-run-suite-codex.json \
  --workflow suite \
  --require-outcome ctxhelm_improved \
  --expected-ctxhelm-version "ctxhelm 2.4.3" \
  --expected-client-name codex \
  --expected-client-version "codex-cli 0.137.0" \
  --min-task-count 4 \
  --min-comparison-eligible 4 \
  --min-comparable-ctxhelm-lanes 16 \
  --min-ctxhelm-target-read-coverage 1.0 \
  --max-extra-read-delta 2 \
  --min-irrelevant-read-delta 0 \
  --require-retry-cost \
  --require-runner-fingerprint \
  --current-runner-script scripts/e2e-agent-run-codex.sh \
  --current-suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json \
  --format json \
  --output .ctxhelm/e2e/agent-run-proof-check.json
```

The checker enforces the source-free privacy contract, runner fingerprint
metadata, expected client identity, expected ctxhelm version, comparable-lane
counts, current-suite task hashes and target lists, per-task lane metrics,
target-read coverage floors, retry-cost fields, aggregate consistency against
derived task comparisons, lane-summary metrics derived from
`tasks[*].lanes[*].metrics`, aggregate retry-cost metrics derived from
per-task retry costs, read-efficiency metrics derived from lane summaries, and
top-level comparison aggregates derived from `tasks[*].comparison`, including
coverage deltas, read deltas, command deltas, and ctxhelm tool-call observation.
The JSON audit summary publishes those comparison aggregates as
`metrics.targetReadCoverageDeltaAverage` and
`metrics.targetCoverageDeltaAverage`, plus
`metrics.commandExecutionDeltaSum` and `metrics.ctxhelmToolCallsObserved`,
matching the validated aggregate field names.
For the run workflow, the JSON audit summary also publishes
`metrics.commandExecutionDelta` and `metrics.ctxhelmToolCallsObserved`, giving
single accepted reports the same source-free command/tool-call audit surface.
The checker also enforces suite status checks by deriving `suite.taskCount` and
`report.status` from the nested task records, comparison eligibility, and
boundary state, so stale suite envelopes cannot support proof claims.
The checker derives `aggregate.outcomeClaim` and
`aggregate.recommendedResearchActions` from those source-free comparison
aggregates and boundary flags, and also enforces strict absence of client
failures, rate limits, forbidden commands, evidence
misses, evidence-only targets, and under-read targets. Use it to gate claims
about a specific saved report; it is not a substitute for collecting a fresh
real-client report when the client version, ctxhelm version, runner, prompt
contract, or task suite changes. JSON output uses
`ctxhelm-agent-run-proof-check-v1` and records the saved report filename,
report SHA-256, thresholds, factual `privacyStatus`, source-free
`privacyChecks`, runner metadata,
identity checks, current runner script freshness, current suite freshness,
suite-task checks, suite status checks, aggregate metrics, aggregate consistency checks,
lane-summary metric checks, retry-cost consistency checks, read-efficiency consistency checks,
comparison aggregate checks, outcome routing checks, task-lane checks, factual
`boundaryStatus`, boundary checks, and lane quality summaries. `privacyStatus`
and `boundaryStatus` preserve the report facts, while `privacyChecks` and
`boundaryChecks` record the corresponding pass/fail proof checks.

## Release Coverage

The release gate runs `scripts/smoke-feedback.sh`. The smoke proves source-free
feedback ingestion, policy report generation, candidate profile tuning,
profile apply/rollback metadata, and outcome comparison.

Repo feedback does not store raw prompts.
