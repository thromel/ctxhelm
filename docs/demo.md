# Public Demo Artifacts

ctxhelm's public demo artifacts are static, source-free examples that show what
the product gives an existing coding agent: target files, related tests,
validation commands, retrieval-health metadata, graph neighborhoods, policy and
embedding status, and agent-specific preview guidance.

Generate the demo set:

```bash
bash scripts/generate-demo-artifacts.sh
```

Validate the demo set:

```bash
bash scripts/smoke-demo-artifacts.sh
```

The generator writes to `docs/demo-artifacts/` by default. Set
`CTXHELM_DEMO_DIR=/absolute/path` to write elsewhere.

## What The Demo Shows

- `pack-inspector.json` and `pack-inspector.html`: source-free target-file and
  validation metadata.
- `retrieval-health.json` and `retrieval-health.md`: source-free retrieval
  metrics, signal contributions, and gap families.
- `graph-neighborhood.json` and `graph-neighborhood.md`: safe file/test/import
  graph metadata.
- `policy-embedding.json` and `policy-embedding.md`: local semantic provider
  status and cloud-disabled policy posture.
- `agent-preview.json` and `agent-preview.md`: how Codex, Claude Code, Cursor,
  OpenCode, and generic MCP clients should consume ctxhelm.

## Privacy Contract

Demo artifacts must not contain raw source, prompts, terminal logs, secrets,
machine-local paths, or real user repository paths. They are examples of
metadata shape, not a copied repository pack.

## Product Wedge

ctxhelm is not another code editor or autonomous coding agent. It is a local,
read-only context compiler that helps the agents developers already use choose
better files, tests, examples, constraints, and validation commands.
