# Phase 21: Java/Kotlin Symbol Precision - Context

**Gathered:** 2026-05-16
**Status:** Ready for planning
**Mode:** Autonomous lifecycle

<domain>
## Phase Boundary

Inventory already recognizes Java and Kotlin, but symbol extraction only covered TypeScript/JavaScript, Python, Rust, and Go. RefactoringMiner is Java-heavy, so ctxhelm needed useful Java/Kotlin symbol records before precision evals could become meaningful.
</domain>

<decisions>
## Implementation Decisions

- Keep extraction lightweight and local; do not introduce parser/runtime dependencies in this phase.
- Preserve the existing `CodeSymbol` public contract.
- Treat this as precision improvement for retrieval, not language-complete AST parsing.
</decisions>

