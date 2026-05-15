# Phase 23 Plan: Source-Free Precision Edge Overlay

1. Define a small local JSON edge schema for SCIP/LSP bridge outputs.
2. Implement `ctxpack precision import --input <file>`.
3. Persist accepted edges in `.ctxpack/precision-edges.json`.
4. Merge accepted edges into dependency graph output and reject unsafe paths with source-free diagnostics.

