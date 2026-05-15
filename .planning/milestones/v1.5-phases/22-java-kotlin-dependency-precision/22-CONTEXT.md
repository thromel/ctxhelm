# Phase 22: Java/Kotlin Dependency Precision - Context

**Gathered:** 2026-05-16
**Status:** Ready for planning
**Mode:** Autonomous lifecycle

<domain>
## Phase Boundary

Dependency graph inference did not use Java/Kotlin imports even though inventory classified those languages. Java package imports also require source-root suffix resolution because local files usually live below `src/main/java` or similar roots.
</domain>

<decisions>
## Implementation Decisions

- Infer only safe local package imports.
- Ignore wildcard package imports and external packages.
- Resolve package paths by exact candidate first, then safe-path suffix.
</decisions>

