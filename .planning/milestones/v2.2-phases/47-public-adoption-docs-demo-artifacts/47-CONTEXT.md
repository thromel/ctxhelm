# Phase 47 Context: Public Adoption Docs & Demo Artifacts

Milestone: v2.2 Release & Distribution Hardening

Goal: make ctxhelm understandable to a public first-time user without leaking
source or overstating unsupported distribution claims.

## Inputs

- Phase 45 release proof bundle and package audit are complete.
- Phase 46 install doctor and troubleshooting docs are complete.
- Existing docs already cover quickstart, release, agent setup, architecture,
  inspector, retrieval health, graph, policy/embedding, and troubleshooting.

## Constraints

- Demo artifacts must be static and source-free.
- Public copy must describe ctxhelm as a read-only context compiler for existing
  coding agents, not as an autonomous coding agent.
- Release gates must validate demo artifacts before publication.
- Unsupported claims remain excluded: hosted sync, self-update, signed
  installers, package-manager publication, global agent config mutation, and
  cloud telemetry.

