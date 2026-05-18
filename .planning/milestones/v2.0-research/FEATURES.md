# v2.0 Research: Features

## Milestone

v2.0 Workspace & Team Layer

## Table Stakes

### Multi-repo workspace inventory

Users can define a workspace made of multiple local repositories. ctxpack can inspect each repo independently, report freshness and privacy status, and return a source-free workspace summary.

### Cross-repo task context planning

For tasks that mention a package, service, repo, or cross-repo behavior, ctxpack can route retrieval to the right repository or repositories and return evidence-labeled targets without flattening the workspace into one source blob.

### Source-free shared artifacts

Teams can share context cards, benchmark reports, policy profiles, and privacy templates without committing source snippets, raw prompts, terminal logs, or model transcripts.

### Team privacy templates

Maintainers can define default local policies for what may be indexed, exported, shared, or used in optional cloud features. Templates should be inspectable and enforceable before packs or reports are produced.

### Agent-native workspace guidance

Codex, Claude Code, Cursor, OpenCode, and other MCP-aware agents should continue using ctxpack through existing agent-native surfaces. Workspace support should improve `prepare_task`, `get_pack`, and resources without making a separate daily UI mandatory.

## Differentiators

### Workspace routing with provenance

The useful part is not merely "multiple repos"; it is explaining why a task belongs to repo A, repo B, or both, then preserving repo boundaries in the context plan.

### Source-free team memory

Teams can share lessons and policy choices safely when all artifacts are derived from metadata, cards, eval summaries, and feedback signals rather than raw source text.

### Policy-first collaboration

Before teams share anything, ctxpack should show whether the artifact is local-only, source-free, redacted, generated, stale, or policy-blocked.

## Anti-Features

- Shared source snippets by default.
- Hosted repo indexing as a requirement.
- Syncing raw local feedback events to a server.
- New editor UI as the main workflow.
- Silent cross-repo context expansion without budget or provenance.

## Recommended v2.0 Scope

Keep v2.0 focused on local workspace and team-safe artifact contracts:

1. workspace manifest and inventory aggregation
2. workspace-aware planning and pack output
3. shareable source-free cards, reports, policies
4. docs, adapter guidance, and release smokes
