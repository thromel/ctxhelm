# crates.io preparation

This directory records future crates.io preparation notes. The current release
path is a prebuilt local archive; crates.io publication is not part of v1.1.7.

Before any future registry publication:

- Confirm `crates/ctxhelm/Cargo.toml` metadata, license, repository, README, and
  binary package boundaries.
- Run `cargo package --manifest-path crates/ctxhelm/Cargo.toml --locked` from a
  clean checkout.
- Run `bash scripts/smoke-distribution-metadata.sh` from a checkout with the
  current release archive available through `CTXHELM_DIST_DIR`; it checks the
  `cargo package --list` boundary without publishing.
- Verify package contents do not include local `.ctxhelm` state, release proof
  bundles, target directories, secrets, demo output generated outside
  `docs/demo-artifacts/`, or machine-local paths.
- Keep cloud embeddings, cloud reranking, hosted sync, and global agent config
  mutation out of the package contract.

This note is intentionally preparatory. It does not publish to a registry.
