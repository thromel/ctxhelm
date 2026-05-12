# Milestone 23: Deep Pack Resources

## Goal

Align `prepare_task` pack discovery with the implemented `get_pack` budgets and documented MCP resources.

## Scope

- Add a deep `PackOption` to every `ContextPlan`.
- Cache `ctxpack://pack/<task-id>/deep` and `.json` resources during `prepare_task`.
- Keep progressive disclosure intact: agents still start from the tiny plan and only load deep packs explicitly.
- Update README wording for brief, standard, and deep session resources.

## Non-goals

- Do not make deep packs the default.
- Do not change pack budget sizes.
- Do not add new MCP tools or autonomous behavior.

## Verification

- Compiler test for all three pack options.
- MCP test that the deep JSON resource is available after `prepare_task`.
- Full workspace tests, clippy, CLI help smoke, and live stdio MCP smoke.
