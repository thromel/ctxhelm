# v2.0 Research: Pitfalls

## Milestone

v2.0 Workspace & Team Layer

## Pitfalls And Prevention

### Pitfall: Turning local workspace support into hosted sync

**Risk:** Team features can pull the product toward server-side source indexing.

**Prevention:** Keep v2.0 local-first. Share only source-free artifacts and policy files. Defer hosted sync.

### Pitfall: Losing repo boundaries

**Risk:** A workspace pack that merges all repos into one undifferentiated candidate set will confuse agents and inflate context.

**Prevention:** Every workspace candidate must carry repo ID, repo label, path label, and selection reason.

### Pitfall: Sharing source-bearing memory by accident

**Risk:** Team cards, feedback, or benchmark reports can leak prompts, source snippets, terminal logs, or private paths if not validated.

**Prevention:** Reuse the v1.6/v1.7 source-free validators and add artifact smoke tests with sentinel strings.

### Pitfall: Over-expanding cross-repo context

**Risk:** Cross-repo retrieval can become expensive and noisy if every task searches every repo.

**Prevention:** Route first, retrieve second. Default to one or two repos unless the task or graph evidence justifies more.

### Pitfall: Adding too many MCP tools

**Risk:** Workspace support could bloat the MCP schema and hurt the agents it is meant to help.

**Prevention:** Prefer existing tools with workspace arguments and resources. Add new tools only when a repeated workflow cannot be expressed cleanly.

### Pitfall: Policy profiles silently changing behavior

**Risk:** Team policy profiles can create confusing retrieval differences across machines.

**Prevention:** Require explicit active profile IDs, report policy provenance, and include rollback/disabled state in all policy status output.

### Pitfall: Claims without fixed samples

**Risk:** Team or workspace proof can overclaim retrieval lift based on shifting repos or small samples.

**Prevention:** Keep benchmark reports source-free and fixed-sample. Use RefactoringMiner or another stable large-history corpus before claiming external lift.
