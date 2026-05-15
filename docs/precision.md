# Parser and Precision Edges

ctxpack now extracts local symbols and import edges for TypeScript, JavaScript, Python, Rust, Go, Java, and Kotlin. Java/Kotlin support is intentionally lightweight: it reads safe inventoried files, recognizes common classes, interfaces, records, methods, constants, and package imports, and degrades to the existing lexical/graph signals when a file cannot be read safely.

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

Imported precision edges appear in dependency results with kinds such as `precision:calls` or `precision:references`. They are additive: inferred imports still work when no overlay exists, and invalid or unreadable overlays produce non-fatal diagnostics instead of breaking context planning.

Use precision overlays when exact reference/call information is available from a trusted local indexer. Avoid them when the edge export contains raw source snippets or private payloads; strip those fields before importing.
