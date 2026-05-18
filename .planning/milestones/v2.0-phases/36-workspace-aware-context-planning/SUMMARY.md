# Phase 36 Summary: Workspace-Aware Context Planning

## Completed

- Added workspace context pack contracts that keep repository boundaries
  explicit through `WorkspaceRepoPack` and `WorkspaceContextPack`.
- Added compiler support for `compile_workspace_context_pack`.
- Added CLI support for `ctxpack workspace get-pack`.
- Extended workspace docs and release docs for workspace pack behavior.
- Updated workspace smoke coverage to verify `repoPacks`, target agent
  preservation, and sentinel non-leakage.

## Result

ctxpack can now route a task across a local workspace and return a
repo-boundary-aware pack instead of flattening snippets from different
repositories into one context blob.

