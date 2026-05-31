# Release Governance

ctxpack release governance separates readiness metadata from publication. The
scripts in this section are source-free and local-only; they do not publish,
tag, upload, install, or mutate global agent configuration.

## Proof Levels

The deterministic protocol proof is required. It exercises ctxpack through
direct JSON-RPC/MCP calls and verifies the stable tool/resource behavior without
depending on a particular interactive agent client.

Optional real-client proof can be added for Codex CLI and Claude Code by setting
the release-gate real-client environment variables documented in
`docs/release.md`. Cursor and OpenCode real-client proof is not claimed for
v1.1.0; their public support remains config/rules plus deterministic MCP
compatibility.

When `CTXPACK_REAL_CLIENT_EVIDENCE_DIR` is set, the real-client wrappers write
source-free evidence only: client/version metadata, a request-log SHA-256,
request line count, explicit repo tool-call count, sanitized observed tool-call
metadata, and a sanitized request-summary JSON sidecar. The wrappers do not
persist raw MCP request logs, prompt text, task text, or source snippets.

## Candidate Status

Create source-free readiness metadata:

```bash
bash scripts/release-candidate-status.sh create \
  --output dist/release-candidate-status.json \
  --status ready \
  --proof-level deterministic \
  --proof-summary dist/release-proof-summary.json
```

For `ready` candidates, `--proof-summary` should point at the source-free
summary written by `scripts/release-gate.sh`. The status metadata records the
archive checksum, binary checksum, archive-binary proof source, required clean
cold fixture proof status, and the local archive distribution decision. A
`ready` candidate fails validation unless the attached summary passed, used the
archive binary, and required the clean fixture proof.

Allowed candidate statuses:

- `ready`: required release gate, docs, packaging, archive verification, demo,
  distribution metadata, and governance smokes are acceptable for publication.
- `deferred`: release is intentionally delayed because optional proof,
  package-manager metadata, or communication work is incomplete.
- `blocked`: release must not be announced because a required gate failed or a
  privacy/source-free boundary regressed.

For v1.1.0, `ready` means the local archive channel is ready. Homebrew,
crates.io, signed installers, and self-update remain explicitly deferred in the
candidate status metadata.

Validate metadata:

```bash
bash scripts/release-candidate-status.sh validate \
  --input dist/release-candidate-status.json
```

## Public Release Verification

After publishing archive-first GitHub release assets, verify the release metadata
against the local artifacts without uploading or mutating anything:

```bash
bash scripts/verify-github-release.sh \
  --tag v1.1.0 \
  --target 68383cbfc2fff00c4f53fbd2b7bf90527ac4bd7e \
  --assets-dir dist
```

The verifier checks that the GitHub release is not a draft, is not a prerelease,
targets the expected commit, and exposes uploaded assets whose SHA-256 digests
match the local archive, manifest, audit report, and checksum files.

## Rollback

Rollback removes local candidate artifacts only after the candidate directory
contains `.ctxpack-release-candidate`:

```bash
bash scripts/release-candidate-rollback.sh \
  --candidate-dir dist/candidate \
  --metadata packaging/release/update-metadata.example.json \
  --previous-metadata packaging/release/update-metadata.previous.json
```

The rollback command restores previous metadata when both metadata paths are
provided, removes the marked candidate directory, refuses dangerous paths, and
does not touch repository source files.

## Verification

Run:

```bash
bash scripts/smoke-release-governance.sh
```

The smoke covers ready, deferred, and blocked candidate status metadata,
deterministic protocol proof language, optional real-client proof language,
Cursor/OpenCode non-claims, and rollback safety.
