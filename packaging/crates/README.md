# crates.io preparation

This directory records future crates.io preparation notes. The current release
path is a prebuilt local archive; crates.io publication is not part of v1.1.2.

Before any future registry publication:

- Confirm `crates/ctxpack/Cargo.toml` metadata, license, repository, README, and
  binary package boundaries.
- Run `cargo package --manifest-path crates/ctxpack/Cargo.toml --locked` from a
  clean checkout.
- Verify package contents do not include local `.ctxpack` state, release proof
  bundles, target directories, secrets, demo output generated outside
  `docs/demo-artifacts/`, or machine-local paths.
- Keep cloud embeddings, cloud reranking, hosted sync, and global agent config
  mutation out of the package contract.

This note is intentionally preparatory. It does not publish to a registry.

