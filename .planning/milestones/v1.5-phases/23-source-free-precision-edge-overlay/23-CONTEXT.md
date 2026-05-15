# Phase 23: Source-Free Precision Edge Overlay - Context

**Gathered:** 2026-05-16
**Status:** Ready for planning
**Mode:** Autonomous lifecycle

<domain>
## Phase Boundary

The original product vision called for optional SCIP/LSP precision. Direct SCIP protobuf parsing and project-specific LSP startup are larger than this milestone. A source-free precision edge overlay gives a local bridge format that can consume SCIP/LSP-derived edges without making language tooling mandatory.
</domain>

<decisions>
## Implementation Decisions

- Import only source-free edge metadata.
- Validate every imported source and target path against safe inventory.
- Add precision edges to dependency output with explicit `precision:<edgeType>` provenance.
</decisions>

