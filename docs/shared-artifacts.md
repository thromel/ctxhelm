# Shared Artifacts

Shared artifacts are source-free manifests that let a team inspect or exchange
ctxhelm metadata without sharing source code.

They are the v2.0 team layer boundary: teams can compare which generated
ctxhelm artifacts exist, whether they are compatible, and whether local policy
allows export before any future sync layer exists.

## Export

```bash
ctxhelm workspace artifacts export --repo /path/to/repo --format json
```

The command writes:

```text
.ctxhelm/shared-artifacts.json
```

The manifest can describe:

- context cards
- benchmark reports
- policy profiles
- feedback summaries
- proof reports
- workspace manifests
- team policies

Each entry contains path labels, size, hash, status, timestamp, diagnostics, and
privacy metadata. It does not contain raw source, prompts, terminal logs,
secrets, or model transcripts.

## Inspect

```bash
ctxhelm workspace artifacts inspect .ctxhelm/shared-artifacts.json --format json
```

Inspection validates schema compatibility and the source-free privacy flag.

## Import

```bash
ctxhelm workspace artifacts import shared-artifacts.json --repo /path/to/repo
```

Compatible manifests are copied to:

```text
.ctxhelm/imported-shared-artifacts.json
```

Importing a manifest does not hydrate source code, overwrite local cards, or
enable cloud features. It only stores the source-free manifest for later
workspace/team workflows.

## Team Policy

Initialize a local source-free team privacy policy template:

```bash
ctxhelm workspace policy init --repo /path/to/repo --format json
```

Inspect the effective policy:

```bash
ctxhelm workspace policy status --repo /path/to/repo --format json
```

The default policy:

- allows local workspace indexing
- allows source-free artifact export
- disables cloud embeddings
- disables cloud reranking
- disallows source snippets in shared artifacts
- enables secret redaction
- reports `sourceTextLogged: false`

Policy reports list allowed, blocked, degraded, and redacted artifact classes.
They do not include source code, prompts, terminal logs, or model transcripts.

## Release Smoke

The release gate runs:

```bash
scripts/smoke-shared-artifacts.sh
```

The smoke creates a temporary git repository, writes a secret sentinel into
ignored/sensitive locations, exports and inspects a shared artifact manifest,
imports the compatible manifest, initializes and inspects team policy, and
fails if the sentinel appears in outputs or ctxhelm local state.
