#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"
out_dir="${CTXPACK_DEMO_DIR:-"$repo_root/docs/demo-artifacts"}"

rm -rf "$out_dir"
mkdir -p "$out_dir"

cat >"$out_dir/README.md" <<'EOF'
# ctxpack Demo Artifacts

These artifacts are static, source-free examples for public adoption docs. They
show the shape of ctxpack outputs without copying repository source code,
prompts, terminal logs, secrets, or machine-local paths.

Artifacts:

- `pack-inspector.json` and `pack-inspector.html`
- `retrieval-health.json` and `retrieval-health.md`
- `graph-neighborhood.json` and `graph-neighborhood.md`
- `policy-embedding.json` and `policy-embedding.md`
- `agent-preview.json` and `agent-preview.md`
EOF

cat >"$out_dir/pack-inspector.json" <<'EOF'
{
  "schemaVersion": 1,
  "taskHash": "demo-task-auth-redirect",
  "budget": "brief",
  "targetAgent": "generic",
  "sourceTextLogged": false,
  "privacyStatus": {
    "localOnly": true,
    "remoteEmbeddingsUsed": false,
    "remoteRerankingUsed": false,
    "sourceTextLogged": false
  },
  "targetFiles": [
    {
      "path": "src/auth/middleware.ts",
      "reason": "Owns unauthenticated redirect behavior",
      "confidence": 0.88
    },
    {
      "path": "tests/auth/redirect.test.ts",
      "reason": "Likely focused validation target",
      "confidence": 0.91
    }
  ],
  "validationCommands": [
    "pnpm vitest run tests/auth/redirect.test.ts",
    "pnpm typecheck"
  ],
  "sections": [
    {
      "title": "Target Files",
      "tokenEstimate": 480,
      "sourceBearing": false
    },
    {
      "title": "Validation",
      "tokenEstimate": 120,
      "sourceBearing": false
    }
  ]
}
EOF

cat >"$out_dir/pack-inspector.html" <<'EOF'
<!doctype html>
<html lang="en">
<meta charset="utf-8">
<title>ctxpack Demo Pack Inspector</title>
<style>
body{font-family:system-ui,-apple-system,sans-serif;margin:2rem;line-height:1.45;color:#1f2933;background:#fbfbfa}
main{max-width:860px;margin:auto}
table{border-collapse:collapse;width:100%;background:white}
th,td{border:1px solid #d9dde3;padding:.65rem;text-align:left}
code{background:#eef2f6;padding:.1rem .25rem;border-radius:4px}
.ok{color:#0f766e;font-weight:700}
</style>
<main>
<h1>ctxpack Demo Pack Inspector</h1>
<p class="ok">Source-free demo: no raw source text, prompts, logs, or secrets.</p>
<table>
<tr><th>Path</th><th>Reason</th><th>Confidence</th></tr>
<tr><td><code>src/auth/middleware.ts</code></td><td>Owns unauthenticated redirect behavior</td><td>0.88</td></tr>
<tr><td><code>tests/auth/redirect.test.ts</code></td><td>Likely focused validation target</td><td>0.91</td></tr>
</table>
</main>
</html>
EOF

cat >"$out_dir/retrieval-health.json" <<'EOF'
{
  "schemaVersion": 1,
  "reportKind": "retrieval_health",
  "sourceTextLogged": false,
  "privacyStatus": {
    "localOnly": true,
    "sourceTextLogged": false
  },
  "metrics": {
    "targetFileRecallAt5": 0.8,
    "testRecallAt5": 0.75,
    "tokenEfficiency": 2.4
  },
  "signalContributions": [
    {"signal": "lexical", "share": 0.31},
    {"signal": "graph", "share": 0.27},
    {"signal": "tests", "share": 0.22},
    {"signal": "history", "share": 0.2}
  ],
  "gapFamilies": ["ambiguous_task", "missing_test_edge"]
}
EOF

cat >"$out_dir/retrieval-health.md" <<'EOF'
# Demo Retrieval Health

- Local-only: true
- Source text logged: false
- Target File Recall@5: 0.80
- Test Recall@5: 0.75
- Token efficiency: 2.4 useful items / 1k tokens

Top gaps: ambiguous task wording, missing test edge.
EOF

cat >"$out_dir/graph-neighborhood.json" <<'EOF'
{
  "schemaVersion": 1,
  "reportKind": "graph_neighborhood",
  "sourceTextLogged": false,
  "privacyStatus": {"localOnly": true, "sourceTextLogged": false},
  "anchor": "src/auth/middleware.ts",
  "nodes": [
    {"id": "file:src/auth/middleware.ts", "kind": "file", "role": "source"},
    {"id": "file:src/auth/session.ts", "kind": "file", "role": "source"},
    {"id": "file:tests/auth/redirect.test.ts", "kind": "test", "role": "test"}
  ],
  "edges": [
    {"from": "file:tests/auth/redirect.test.ts", "to": "file:src/auth/middleware.ts", "type": "tests"},
    {"from": "file:src/auth/middleware.ts", "to": "file:src/auth/session.ts", "type": "imports"}
  ],
  "communities": [{"id": "auth", "nodeCount": 3}]
}
EOF

cat >"$out_dir/graph-neighborhood.md" <<'EOF'
# Demo Graph Neighborhood

Anchor: `src/auth/middleware.ts`

- `tests/auth/redirect.test.ts` tests `src/auth/middleware.ts`
- `src/auth/middleware.ts` imports `src/auth/session.ts`
- Community: `auth`, 3 nodes

This report is metadata-only and source-free.
EOF

cat >"$out_dir/policy-embedding.json" <<'EOF'
{
  "schemaVersion": 1,
  "reportKind": "policy_embedding",
  "semanticProvider": "local_hash",
  "cloudEmbeddingsAllowed": false,
  "cloudRerankingAllowed": false,
  "sourceTextLogged": false,
  "privacyStatus": {
    "localOnly": true,
    "remoteEmbeddingsUsed": false,
    "remoteRerankingUsed": false,
    "sourceTextLogged": false
  },
  "policyRows": [
    {"profile": "default", "lexicalWeight": 0.3, "graphWeight": 0.25, "semanticWeight": 0.1}
  ]
}
EOF

cat >"$out_dir/policy-embedding.md" <<'EOF'
# Demo Policy And Embedding Controls

- Semantic provider: `local_hash`
- Cloud embeddings allowed: false
- Cloud reranking allowed: false
- Source text logged: false

The default profile keeps exact lexical, graph, tests, and history above local
semantic candidates.
EOF

cat >"$out_dir/agent-preview.json" <<'EOF'
{
  "schemaVersion": 1,
  "reportKind": "agent_preview",
  "sourceTextLogged": false,
  "privacyStatus": {"localOnly": true, "sourceTextLogged": false},
  "agents": [
    {"agent": "codex", "tools": ["prepare_task", "get_pack"], "ownsEdits": true},
    {"agent": "claude-code", "tools": ["prepare_task", "get_pack"], "ownsEdits": true},
    {"agent": "cursor", "tools": ["prepare_task", "get_pack"], "ownsEdits": true},
    {"agent": "opencode", "tools": ["prepare_task", "get_pack"], "ownsEdits": true}
  ],
  "ctxpackBoundary": "ctxpack recommends context; agents read, edit, and run commands."
}
EOF

cat >"$out_dir/agent-preview.md" <<'EOF'
# Demo Agent Preview

ctxpack exposes `prepare_task` and `get_pack` to Codex, Claude Code, Cursor,
OpenCode, and generic MCP clients.

Boundary: ctxpack recommends files, tests, packs, and constraints. The coding
agent owns file reads, edits, shell commands, and approvals.
EOF

echo "demo artifacts written: $out_dir"
