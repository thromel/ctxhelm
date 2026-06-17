# ctxhelm Release Checklist

Use this checklist before marking a candidate `ready`.

## Required Gates

- [ ] `cargo test --workspace`
- [ ] `bash scripts/check-release-docs.sh`
- [ ] `bash scripts/release-package.sh`
- [ ] `bash scripts/verify-release-archive.sh --archive ... --manifest ... --checksums ...`
- [ ] manual or tag-triggered `.github/workflows/release-artifacts.yml` run when additional platform archives are part of the candidate
- [ ] `bash scripts/smoke-demo-artifacts.sh`
- [ ] `bash scripts/smoke-distribution-metadata.sh`
- [ ] `bash scripts/smoke-release-governance.sh`
- [ ] deterministic protocol proof through `scripts/smoke-mcp-protocol.sh`
- [ ] clean cold fixture proof with `scripts/prepare-proof-fixtures.sh` and
      `CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF=1 bash scripts/release-gate.sh`
- [ ] after publication, `bash scripts/verify-github-release.sh --tag ... --target ... --assets-dir ...`
- [ ] after publication, `bash scripts/verify-public-archive-install.sh --repo ... --tag ... --target-label ...`

## Optional Proof

- [ ] optional real-client proof for Codex CLI
- [ ] optional real-client proof for Claude Code
- [ ] optional public-archive real-client refresh with `bash scripts/smoke-public-real-clients.sh --repo ... --tag ... --target-label ...`
- [ ] Cursor real-client proof: not claimed for v2.4.5
- [ ] OpenCode real-client proof: not claimed for v2.4.5
- [ ] additional benchmark product proof with `CTXHELM_BENCHMARK_CONFIG`

## Candidate Status

Allowed statuses are `ready`, `deferred`, and `blocked`.

- Use `ready` only when all required gates pass and known limitations are
  acceptable. Attach the source-free release proof summary with
  `--proof-summary`; ready status must prove the archive binary, required clean
  fixture proof, archive checksum, and binary checksum.
- Use `deferred` when optional proof or distribution communication is still
  intentionally pending.
- Use `blocked` when a required gate, source-free check, or privacy boundary
  fails.

For v2.4.5, the local archive, multi-platform archive workflow, published
additional platform release assets, and Apple Silicon Homebrew tap channels can
be `ready`; crates.io, signed installers, and self-update stay deferred.

## Rollback

If a candidate must be withdrawn, run rollback against a marked candidate
artifact directory and restore previous metadata:

```bash
bash scripts/release-candidate-rollback.sh \
  --candidate-dir dist/candidate \
  --metadata packaging/release/update-metadata.example.json \
  --previous-metadata packaging/release/update-metadata.previous.json
```

Rollback must remove local candidate artifacts only. It must not touch source
files, publish, tag, upload, install globally, or mutate agent configuration.
