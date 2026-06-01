# v2.0 Research: Architecture

## Milestone

v2.0 Workspace & Team Layer

## Proposed Architecture

```text
Workspace Manifest
  ├── Repo roots
  ├── Repo IDs
  ├── Package/service labels
  ├── Team policy template
  └── Shareable artifact settings
        │
        ▼
Workspace Index Layer
  ├── Per-repo safe inventory
  ├── Per-repo storage status
  ├── Per-repo cards/memory/evals
  └── Source-free aggregate metadata
        │
        ▼
Workspace Context Planner
  ├── Task-to-repo routing
  ├── Cross-repo candidate fusion
  ├── Budgeted repo-boundary expansion
  ├── Workspace validation commands
  └── Source-free provenance
        │
        ▼
Agent-Native Output
  ├── CLI JSON/Markdown
  ├── MCP resources
  ├── AGENTS.md guidance
  └── Adapter docs/rules
```

## Data Model Direction

Add source-free contracts such as:

- `WorkspaceManifest`
- `WorkspaceRepo`
- `WorkspaceInventoryReport`
- `WorkspaceContextPlan`
- `SharedArtifactManifest`
- `TeamPrivacyPolicy`
- `PolicyTemplateReport`

Do not store raw source in workspace-level tables. Workspace rows should reference repo IDs, path labels, card IDs, benchmark IDs, policy IDs, timestamps, hashes, and privacy status.

## Retrieval Direction

Workspace planning should happen in two stages:

1. route task to likely repos using repo labels, package names, manifests, cards, current diff, and explicit paths
2. run normal per-repo retrieval only inside selected repos, then fuse source-free candidate metadata with repo-boundary labels

This avoids the failure mode where cross-repo support becomes "search every file in every repo."

## Storage Direction

Use the existing local storage model:

- per-repo stores remain authoritative for repo-local state
- workspace metadata is a thin aggregation layer
- shared artifacts are exported as source-free files
- policy templates are explicit local files

## Agent Surface Direction

Additive changes are preferred:

- `ctxhelm workspace ...` for setup/status/export
- workspace-aware `prepare-task` and `get-pack` arguments
- MCP resources for workspace summary and shared artifact manifests
- no large new MCP tool list unless release smokes prove it is necessary

## Build Order

1. workspace manifest and source-free inventory aggregation
2. workspace-aware planning and pack provenance
3. source-free shared artifact/policy exports
4. docs, adapter guidance, and release gates
