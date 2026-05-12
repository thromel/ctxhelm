# Milestone 22: Related Diff Expansion

## Goal

Complete the MCP `related` anchor model so agents can expand from a path, symbol, or the current safe local diff.

## Scope

- Add `includeCurrentDiff` to the `related` MCP schema.
- Reuse the shared source-free current diff summary.
- Keep existing path and symbol expansion behavior unchanged.
- Return the same related tests, dependency edges, and co-change hints around resolved diff paths.

## Non-goals

- Do not return patch hunks or source text.
- Do not add a new MCP tool.
- Do not make diff expansion automatic; agents must request it.

## Verification

- MCP schema test for the new flag.
- MCP behavior test for `related` with only `includeCurrentDiff`.
- Full workspace tests, clippy, CLI help smoke, and live stdio MCP smoke.
