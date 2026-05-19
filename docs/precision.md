# Parser and Precision Edges

ctxpack now extracts local symbols with Tree-sitter for TypeScript, JavaScript,
Python, Rust, Go, Java, and Kotlin, then falls back to the lightweight extractor
only when a parser is unavailable or produces no symbols. This gives the context
compiler AST-backed classes, functions, methods, constants, modules, line
ranges, and signatures while preserving the same safe-read policy.

ctxpack can also generate a local source-free precision overlay from those
Tree-sitter symbols:

```bash
ctxpack precision discover --repo /path/to/repo
ctxpack dependencies src/auth/middleware.ts --repo /path/to/repo
```

Discovery scans only safe inventoried source/test files, records metadata edges
such as `precision:calls` and `precision:references`, and writes only paths,
symbol names, edge labels, confidence, and source-free reasons to
`.ctxpack/precision-edges.json`.

For repositories that already run SCIP, LSP, or another code-intelligence indexer, ctxpack can also import a source-free precision edge overlay:

```bash
ctxpack precision import --repo /path/to/repo --input /path/to/precision-edges.json
ctxpack dependencies src/auth/middleware.ts --repo /path/to/repo
```

The import format is deliberately small and source-free:

```json
{
  "schemaVersion": 1,
  "provider": "scip-json",
  "edges": [
    {
      "sourcePath": "src/auth/middleware.ts",
      "targetPath": "src/routes/login.ts",
      "edgeType": "calls",
      "symbol": "loginRoute",
      "confidence": 0.98,
      "reason": "local SCIP edge"
    }
  ]
}
```

The overlay is written to `.ctxpack/precision-edges.json`. ctxpack validates every edge against the safe local inventory before storing it, rejects sensitive/generated/ignored paths, and does not store source snippets, prompt text, secrets, or cloud payloads.

Discovered and imported precision edges appear in dependency results with kinds
such as `precision:calls` or `precision:references`. They are additive:
inferred imports still work when no overlay exists, and invalid or unreadable
overlays produce non-fatal diagnostics instead of breaking context planning.

Semantic status also reports precision availability:

```bash
ctxpack semantic status --repo /path/to/repo --format json
```

The `precisionStatus` field reports whether the local overlay is `unavailable`, `available`, `invalid`, or `degraded`, along with provider, edge count, rejected edge count, overlay path, and diagnostics. Missing or invalid overlays do not disable lexical, graph, test, or semantic-document retrieval. Valid overlays enrich semantic documents as `precision` facets and dependency edges as `precision:*` relations.

Use precision overlays when exact reference/call information is available from a trusted local indexer. Avoid them when the edge export contains raw source snippets or private payloads; strip those fields before importing.

Default promotion is evaluated by `ctxpack eval gate`. If no fresh precision
overlay is available, precision-specific variants are reported as `skipped`
rather than silently omitted. This keeps release proof honest about whether
precision retrieval was actually exercised.
