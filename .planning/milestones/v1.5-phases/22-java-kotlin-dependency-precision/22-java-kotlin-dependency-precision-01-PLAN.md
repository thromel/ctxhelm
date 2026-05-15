# Phase 22 Plan: Java/Kotlin Dependency Precision

1. Include Java/Kotlin source and test files in safe dependency graph inference.
2. Parse Java and Kotlin package import statements.
3. Resolve package paths to safe `.java`, `.kt`, and `.kts` files by exact and source-root suffix matching.
4. Add focused tests for Java-to-Java and Kotlin-to-Java local edges.

