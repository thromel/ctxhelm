# Repo Context Packer Milestone 10: Local Evaluation Traces

## Goal

Record local, source-free context-decision metadata so ctxpack can later evaluate whether its plans and packs helped agents choose the right files and tests.

## Scope

- Add a typed `EvalTrace` contract.
- Store traces as JSONL under the existing private local repo state path.
- Record traces from `prepare-task`, `get-pack`, and MCP `prepare_task`/`get_pack`.
- Add `ctxpack eval traces` to inspect recent local traces.
- Add `ctxpack eval checklist` to generate a manual dogfood comparison artifact.
- Do not store source snippets or task text.

## Trace Fields

- repo id
- task hash
- task type
- optional pack id
- target agent label
- optional pack budget
- recommended files
- recommended tests
- recommended commands
- created timestamp
- `sourceTextLogged: false`

## Verification

- unit coverage for trace serialization without source text
- unit coverage for append/list JSONL trace storage
- unit coverage for pack trace construction
- unit coverage for dogfood checklist rendering
- CLI smoke for `prepare-task` plus `eval traces`
- CLI smoke for `eval checklist`
- full workspace test, clippy, and CLI help before closing the milestone
