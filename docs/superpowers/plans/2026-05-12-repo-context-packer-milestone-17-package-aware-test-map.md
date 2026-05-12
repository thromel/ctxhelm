# Milestone 17: Package-Aware MCP Test Map

## Goal

Make `ctxpack://repo/test-map` use the same targeted command inference as `related_tests`, so agents receive accurate validation commands from both tool and resource paths.

## Scope

- Add a shared index-level `test_map` API over safe inventoried tests.
- Reuse package-manager and test-script detection for test-map commands.
- Replace the MCP resource's hard-coded `pnpm test <path>` fallback.
- Keep the MCP tool surface unchanged.

## Non-Goals

- No command execution by ctxpack.
- No coverage collection.
- No shell probing beyond existing local file/config inspection.
- No cloud telemetry.

## Verification

- Index test proves `test_map` filters safe tests and infers `pnpm vitest run <test>`.
- MCP test proves `ctxpack://repo/test-map` returns package-aware commands.
- Full workspace tests, clippy, and CLI help smoke.
