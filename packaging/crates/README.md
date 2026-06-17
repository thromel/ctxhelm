# crates.io preparation

This directory records future crates.io preparation notes. The current release
path is a prebuilt local archive; crates.io publication is not part of v2.4.4.

Before any future registry publication:

- Confirm every workspace crate manifest includes metadata, license, repository,
  README, and explicit versions for internal dependencies.
- Publish order, if this channel is activated later:
  1. `ctxhelm-core`
  2. `ctxhelm-index`
  3. `ctxhelm-compiler`
  4. `ctxhelm-mcp`
  5. `ctxhelm`
- Run `cargo package --manifest-path crates/<crate>/Cargo.toml --locked --list`
  for every crate from a clean checkout.
- Run `cargo package --manifest-path crates/ctxhelm-core/Cargo.toml --locked
  --no-verify` as the current leaf-crate dry-run. Dependent crates cannot fully
  dry-run against crates.io until their internal dependencies are published in
  the order above.
- Run `bash scripts/smoke-distribution-metadata.sh` from a checkout with the
  current release archive available through `CTXHELM_DIST_DIR`; it checks all
  package-list boundaries plus the leaf dry-run without publishing.
- Verify package contents do not include local `.ctxhelm` state, release proof
  bundles, target directories, secrets, demo output generated outside
  `docs/demo-artifacts/`, or machine-local paths.
- Keep cloud embeddings, cloud reranking, hosted sync, and global agent config
  mutation out of the package contract.

This note is intentionally preparatory. It does not publish to a registry.
