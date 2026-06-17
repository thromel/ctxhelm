# Release Governance

ctxhelm release governance separates readiness metadata from publication. The
scripts in this section are source-free and local-only; they do not publish,
tag, upload, install, or mutate global agent configuration.

## Proof Levels

The deterministic protocol proof is required. It exercises ctxhelm through
direct JSON-RPC/MCP calls and verifies the stable tool/resource behavior without
depending on a particular interactive agent client.

Optional real-client proof can be added for Codex CLI, Claude Code, Cursor, and
OpenCode by setting the release-gate real-client environment variables
documented in `docs/release.md`. Cursor and OpenCode proof is still treated as
optional: a pass requires server-side `prepare_task` and `get_pack` request
evidence with the explicit repo, while unavailable auth/provider state is
recorded as a source-free skip.

When `CTXHELM_REAL_CLIENT_EVIDENCE_DIR` is set, the real-client wrappers write
source-free evidence only: client/version metadata, a request-log SHA-256,
request line count, explicit repo tool-call count, sanitized observed tool-call
metadata, and a sanitized request-summary JSON sidecar. The wrappers do not
persist raw MCP request logs, prompt text, task text, or source snippets. In
optional mode, unavailable or failing real clients write `status: skipped` with
a source-free reason so publication notes can distinguish unsupported client
state from product proof.

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

For v2.4.4, `ready` means the local archive channel is ready and the Apple
Silicon Homebrew tap is ready. It also means the multi-platform archive workflow
at `.github/workflows/release-artifacts.yml` is ready to build and verify Linux
x64, macOS Intel, and macOS Apple Silicon artifacts, and to upload those
verified target assets to the matching GitHub release on version-tag pushes.
In the v2.4.4 candidate metadata, published additional platform release assets
are ready after the tag workflow succeeds. Crates.io, signed installers, and
self-update remain explicitly deferred in the candidate status metadata.

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
  --tag v2.4.4 \
  --target "$(git rev-parse HEAD)" \
  --assets-dir dist
```

The verifier checks that the GitHub release is not a draft, is not a prerelease,
targets the expected commit, and exposes uploaded assets whose SHA-256 digests
match the local archive, manifest, audit report, and checksum files.

Then verify the public user install path:

```bash
bash scripts/verify-public-archive-install.sh \
  --repo thromel/ctxhelm \
  --tag v2.4.4 \
  --target-label aarch64-apple-darwin \
  --expected-version "ctxhelm 2.4.4" \
  --output .ctxhelm/e2e/phase130-public-archive-install.json
```

This downloads the public release assets, verifies checksums, installs the
binary into a temporary bin directory, runs `--version`, `--help`, `doctor`, and
the first-pack smoke, then writes source-free proof metadata.

Then, when Codex CLI or Claude Code proof should be refreshed against the public
archive binary, run:

```bash
bash scripts/smoke-public-real-clients.sh \
  --repo thromel/ctxhelm \
  --tag v2.4.4 \
  --target-label aarch64-apple-darwin \
  --expected-version "ctxhelm 2.4.4" \
  --output .ctxhelm/e2e/phase130-public-real-client-smoke.json
```

This records source-free pass evidence for clients that call `prepare_task` and
`get_pack`, and source-free skip evidence for optional clients that are missing,
unauthenticated, disconnected, or otherwise unable to produce machine-checkable
tool-call evidence.

Finally, verify whether the already-published public release still represents
the current branch tip:

```bash
bash scripts/check-public-release-freshness.sh \
  --repo thromel/ctxhelm \
  --tag v2.4.4 \
  --output .ctxhelm/e2e/phase130-public-release-freshness.json
```

The freshness checker writes source-free metadata with `status`,
`releaseTargetCommit`, `currentCommit`, `gitRelation`, `commitsAhead`,
`productStatus`, `productCommitsAhead`, `proofOnlyCommitsAhead`, and
`ignoredFreshnessPaths`. Use `--require-current` when a publication or
announcement requires the public archive to match the current commit exactly.
Use `--require-product-current` when proof/planning-only commits may exist after
the release but no product-impacting commits may be ahead. The checker does not
publish, create tags, upload assets, install globally, mutate agent
configuration, or read source text.

## Rollback

Rollback removes local candidate artifacts only after the candidate directory
contains `.ctxhelm-release-candidate`:

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
Cursor/OpenCode source-free proof boundaries, public release freshness metadata,
multi-platform archive workflow governance, and rollback safety.
