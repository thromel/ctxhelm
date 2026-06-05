# ctxhelm Context Governor

The context governor is a source-free inspection layer over ctxhelm's planner,
memory, feedback, policy-profile, and validation signals. It does not edit
files, run user project tests, or replace a coding agent. It explains which
context policy ctxhelm would use for a task and why.

```bash
ctxhelm governor decide "fix requireSession redirect" \
  --repo /path/to/repo \
  --mode bug-fix \
  --path src/auth/session.ts \
  --format json
```

Use `--semantic` to include the configured local semantic provider in the
decision path:

```bash
ctxhelm governor decide "where do payment webhooks validate signatures" \
  --repo /path/to/repo \
  --mode explain \
  --semantic \
  --semantic-provider local_fastembed
```

## What It Reports

`ctxhelm governor decide` returns a `ctxhelm-context-governor-report-v1`
contract with:

- task hash, task type, repo ID, and recommended budget
- active learned policy profile, profile count, and feedback event count
- decision explanations for retrieval, budget, memory, validation, semantic,
  and policy-profile state
- selected evidence, such as target files, related tests, validation commands,
  and selected source-free memory cards
- omitted evidence, such as candidates not selected under the current budget,
  context areas with unselected pressure, alternate budget options, and missing
  information questions
- rollout controls for `eval policy learn`, `eval policy experiments`,
  `eval policy apply`, and `eval policy rollback`
- source-free privacy metadata

The report stores and prints paths, counts, confidence scores, policy IDs,
reason codes, and commands. It does not include source snippets or raw prompts.

## Inputs

The governor reuses existing ctxhelm components:

- task classifier and query trace from `prepare-task`
- lexical/symbol, graph, history, test, memory, and optional local semantic
  candidates from the context planner
- source-free feedback events from `ctxhelm eval feedback`
- source-free policy quality reports from `ctxhelm eval policy report`
- active learned policy profiles from `ctxhelm eval policy apply`
- local provider policy from `docs/semantic.md`

This means governor decisions are tied to the same context plan that agents
consume through MCP, native rules, and packs.

## Rollout Flow

Use source-free feedback/eval outcomes to tune a candidate policy:

```bash
ctxhelm eval policy tune --repo /path/to/repo --format json
ctxhelm eval policy experiments --repo /path/to/repo
ctxhelm eval policy apply <profile-id> --repo /path/to/repo
ctxhelm governor decide "fix auth redirect" --repo /path/to/repo --mode bug-fix
ctxhelm eval policy rollback --repo /path/to/repo
```

The governor report makes the active policy profile visible before and after
apply/rollback, so maintainers can compare task-conditioned behavior across
repos without inspecting private source.

## Release Coverage

The release gate runs `scripts/smoke-governor.sh`. The smoke proves:

- `ctxhelm governor decide` emits the source-free governor contract
- decision areas cover retrieval, budget, memory, validation, semantic, and
  policy-profile state
- selected evidence includes the anchored task file
- omitted evidence and rollout controls are machine-readable
- policy apply and rollback are reflected in subsequent governor reports
- source sentinel text is absent from command output and ctxhelm local state
