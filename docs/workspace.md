# Workspace Manifests

Workspace support is the v2.0 foundation for local multi-repo context planning.
It defines local workspaces, reports source-free inventory status, routes tasks
to likely repositories, and compiles repo-boundary-aware packs.

## Manifest

Create a local manifest:

```bash
ctxpack workspace init --repo /path/to/main-repo --member /path/to/related-repo
```

The manifest is written to:

```text
.ctxpack/workspace.json
```

Shape:

```json
{
  "schemaVersion": 1,
  "workspaceId": null,
  "repos": [
    {
      "id": "stable-repo-id",
      "path": ".",
      "label": "main-repo",
      "tags": []
    }
  ]
}
```

Repository paths may be absolute or relative to the workspace root. Repository
IDs are optional; when omitted, ctxpack derives the same stable local repo ID it
uses for single-repo inventory and storage.

## Status

Inspect source-free workspace status:

```bash
ctxpack workspace status --repo /path/to/main-repo --format json
```

The report includes:

- workspace root and manifest path
- repository IDs, labels, tags, and path labels
- per-repo file, generated, sensitive, and ignored counts
- storage compatibility and memory card counts when local storage exists
- diagnostics for missing, duplicate, inaccessible, non-git, generated-looking,
  sensitive-looking, or invalid entries
- `sourceTextLogged: false`

Status output does not include raw file contents, prompts, terminal logs, model
transcripts, snippets, or secrets.

## Workspace-Aware Planning

Route a task across the local workspace:

```bash
ctxpack workspace prepare-task "fix login redirect" \
  --repo /path/to/main-repo \
  --mode bug-fix \
  --format json
```

The command:

1. loads `.ctxpack/workspace.json`,
2. plans inside each available member repository,
3. ranks repositories by plan confidence and evidence volume,
4. returns the top repo-boundary-aware plans.

The returned `WorkspaceContextPlan` preserves:

- workspace root and manifest path
- selected repo count
- repo IDs, labels, tags, and path labels
- per-repo `ContextPlan` objects
- source-free diagnostics
- `sourceTextLogged: false`

## Workspace Packs

Compile a repo-boundary-aware workspace pack:

```bash
ctxpack workspace get-pack "fix login redirect" \
  --repo /path/to/main-repo \
  --mode bug-fix \
  --budget brief \
  --target-agent codex
```

The returned `WorkspaceContextPack` contains `repoPacks`. Each repo pack keeps
the repository ID, label, path label, reason, confidence, and nested
single-repo `ContextPack`. ctxpack does not flatten snippets from different
repositories into one undifferentiated section.

Anchors can be absolute, workspace-relative, repo-prefixed, or repo-relative:

```bash
ctxpack workspace prepare-task "fix auth" \
  --repo /path/to/main-repo \
  --path web/src/auth/session.ts \
  --path /absolute/path/to/api/src/auth/session.ts
```

## Privacy Boundary

Workspace metadata is local-first and source-free. `workspace status` may read
safe inventory metadata for each member repository, but it only emits paths,
counts, compatibility labels, diagnostics, and privacy flags.

No workspace command uploads source code, calls cloud embeddings, calls cloud
rerankers, mutates global agent config, or edits source files.

## Shared Team Artifacts

Workspace teams can exchange source-free artifact manifests without exchanging
source files:

```bash
ctxpack workspace artifacts export --repo /path/to/main-repo --format json
ctxpack workspace artifacts inspect .ctxpack/shared-artifacts.json --format json
ctxpack workspace artifacts import shared-artifacts.json --repo /path/to/main-repo
```

The manifest tracks context cards, benchmark reports, policy profiles, feedback
summaries, proof reports, workspace manifests, and team policy files by path
label, status, size, hash, diagnostics, and privacy metadata.

Team privacy policy templates are also local and source-free:

```bash
ctxpack workspace policy init --repo /path/to/main-repo --format json
ctxpack workspace policy status --repo /path/to/main-repo --format json
```

See [shared artifacts](shared-artifacts.md) for the manifest and policy
boundary.

## Current Limits

Workspace support currently has no new MCP workspace resources and does not
sync shared artifacts through a remote service. Artifact import stores only the
manifest at `.ctxpack/imported-shared-artifacts.json`; it does not hydrate
source code or overwrite local generated cards.

## Smoke

The release gate runs:

```bash
scripts/smoke-workspace.sh
scripts/smoke-shared-artifacts.sh
```

The smoke creates two temporary git repositories, writes a source sentinel,
initializes a workspace manifest, runs JSON status, and verifies the sentinel is
absent from workspace output and local ctxpack state.
