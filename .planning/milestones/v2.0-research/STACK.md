# v2.0 Research: Stack

## Milestone

v2.0 Workspace & Team Layer

## Stack Additions

The existing Rust workspace remains the right foundation. v2.0 should add workspace and team metadata on top of the current local storage, compiler, CLI, and MCP surfaces rather than introducing a hosted backend or replacing the agent-native architecture.

## Recommended Additions

### Workspace metadata

- Add typed workspace contracts in `ctxhelm-core`.
- Store source-free workspace state in the existing local storage pattern.
- Treat each repository as a bounded local root with its own repo ID, privacy status, and freshness metadata.
- Prefer TOML/JSON config files for team-shareable policy inputs.

### Multi-repo indexing

- Reuse existing safe inventory, storage, memory, semantic, precision, feedback, and eval modules per repo.
- Add workspace-level aggregation instead of merging source-bearing indexes.
- Keep cross-repo links source-free: repo ID, path label, card ID, benchmark ID, policy ID, and diagnostic metadata only.

### Team artifacts

- Use source-free files for shareable outputs:
  - workspace manifest
  - shared cards
  - benchmark summaries
  - policy profiles
  - privacy templates
- Avoid source-bearing shared caches.

### Agent-native surface

- Extend existing CLI and MCP commands additively.
- Keep the MCP tool surface small.
- Prefer workspace-aware arguments on existing workflows before adding new tools.

## Non-Goals

- No hosted source indexing.
- No background daemon required.
- No remote sync service.
- No autonomous editing.
- No source-bearing team memory.

## Implementation Guidance

Start with local manifests and source-free aggregation. This keeps v2.0 compatible with the trust model established by v1.1 through v1.7 and avoids turning ctxhelm into an enterprise service before the local product is proven.
