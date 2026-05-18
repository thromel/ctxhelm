# ctxpack Release Checklist

Use this checklist before marking a candidate `ready`.

## Required Gates

- [ ] `cargo test --workspace`
- [ ] `bash scripts/check-release-docs.sh`
- [ ] `bash scripts/release-package.sh`
- [ ] `bash scripts/verify-release-archive.sh --archive ... --manifest ... --checksums ...`
- [ ] `bash scripts/smoke-demo-artifacts.sh`
- [ ] `bash scripts/smoke-distribution-metadata.sh`
- [ ] `bash scripts/smoke-release-governance.sh`
- [ ] deterministic protocol proof through `scripts/smoke-mcp-protocol.sh`

## Optional Proof

- [ ] optional real-client proof for Codex CLI
- [ ] optional real-client proof for Claude Code
- [ ] Cursor real-client proof: not claimed for v1.1.0
- [ ] OpenCode real-client proof: not claimed for v1.1.0
- [ ] benchmark product proof with `CTXPACK_BENCHMARK_CONFIG`

## Candidate Status

Allowed statuses are `ready`, `deferred`, and `blocked`.

- Use `ready` only when all required gates pass and known limitations are
  acceptable.
- Use `deferred` when optional proof or distribution communication is still
  intentionally pending.
- Use `blocked` when a required gate, source-free check, or privacy boundary
  fails.

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

