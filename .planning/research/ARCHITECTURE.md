# Architecture Research: v2.2 Release & Distribution Hardening

## Integration Points

- `scripts/release-gate.sh`
- `scripts/release-package.sh`
- `scripts/audit-release-artifact.sh`
- `scripts/check-release-docs.sh`
- `crates/ctxpack/tests/release_packaging.rs`
- README and release docs
- setup-check and agent setup docs

## Build Order

1. Harden release gate and proof bundle.
2. Add install/upgrade verification and troubleshooting.
3. Add public demo artifacts and adoption docs.
4. Add package metadata and clean extraction verification.
5. Add release governance and candidate lifecycle.

## Boundaries

- Release validation remains local and non-publishing.
- Package-manager metadata is preparatory, not mandatory.
- Source-free artifacts remain source-free.
