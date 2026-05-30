# ctxpack v1.1.0 Release Guide

This document describes the local binary release path for ctxpack v1.1.0. The primary user path is a prebuilt archive plus SHA-256 checksums; source builds are fallback paths.

## User Install

Download the archive for your platform from the release artifacts. Archive names follow this shape:

```text
ctxpack-v1.1.0-{target}.tar.gz
sha256sums.txt
```

Verify checksums before installing:

```bash
shasum -a 256 -c sha256sums.txt
sha256sum -c sha256sums.txt
```

Extract and install the binary on `PATH`:

```bash
tar -xzf ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz
install -m 0755 ctxpack-v1.1.0-aarch64-apple-darwin/ctxpack ~/.local/bin/ctxpack
ctxpack --version
ctxpack --help
ctxpack doctor --binary "$(command -v ctxpack)" --release-manifest ctxpack-v1.1.0-aarch64-apple-darwin.manifest.json
```

The expected diagnostic is:

```text
ctxpack 1.1.0
```

## Source Build Fallbacks

Build from the tagged repository with locked dependencies:

```bash
cargo install --git https://github.com/thromel/ctxpack --tag v1.1.0 ctxpack --locked
ctxpack --version
ctxpack --help
```

Build from a local checkout:

```bash
cargo install --path crates/ctxpack --locked
cargo build -p ctxpack --release --locked
target/release/ctxpack --version
target/release/ctxpack --help
```

## Maintainer Packaging

From a clean checkout at the v1.1.0 tag, run:

```bash
bash scripts/release-package.sh
```

The script builds with:

```bash
cargo build -p ctxpack --release --locked
```

It writes release artifacts under `dist/` by default, or under `CTXPACK_DIST_DIR` when that environment variable is set:

```text
dist/ctxpack-v1.1.0-{target}.tar.gz
dist/ctxpack-v1.1.0-{target}.manifest.json
dist/ctxpack-v1.1.0-{target}.audit.json
dist/ctxpack-v1.1.0-{target}.tar.gz.sha256
dist/sha256sums.txt
```

The package script stages only the `ctxpack` binary, `README.md`, `LICENSE`, and `VERSION`, then extracts the archive in a temporary directory and verifies:

```bash
ctxpack --version
ctxpack --help
```

The release manifest records the version, target label, archive checksum, binary checksum, included files, local-only privacy status, unsupported publish actions, and the matching artifact audit report. `sha256sums.txt` covers the archive, manifest, and audit report.

Maintainers can set `CARGO_TARGET_DIR=/absolute/path/to/target` when they need a clean build cache for packaging or release-gate verification.

Maintainers can verify a built archive from a clean extraction directory:

```bash
bash scripts/verify-release-archive.sh \
  --archive dist/ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz \
  --manifest dist/ctxpack-v1.1.0-aarch64-apple-darwin.manifest.json \
  --checksums dist/sha256sums.txt
```

During multi-plan local work, maintainers can set `CTXPACK_ALLOW_DIRTY=1` for verification, but release artifacts should be produced from a clean checkout.

## Release Gate

Before publishing or announcing a release, run the local release gate:

```bash
bash scripts/release-gate.sh
```

This is the pre-publication blocker for v1.1.0. When `CTXPACK_BIN` is not set, the gate runs `scripts/release-package.sh`, audits the archive, extracts the generated artifact, and uses the extracted `ctxpack` binary for installed-binary proof.

To prove a selected installed or previously extracted binary, pass an absolute path:

```bash
CTXPACK_BIN=/absolute/path/to/ctxpack bash scripts/release-gate.sh
```

The release gate preserves the packaging script's clean-checkout guard by default. During multi-plan local verification, set `CTXPACK_ALLOW_DIRTY=1` explicitly; release artifacts intended for publication should be built from a clean checkout.

The release gate runs these required checks:

- `cargo test --workspace`
- `scripts/check-release-docs.sh`
- `scripts/release-package.sh`, including `scripts/audit-release-artifact.sh`
- `scripts/verify-release-archive.sh` clean extraction verification
- selected or extracted binary `ctxpack --version`
- selected or extracted binary `ctxpack --help`
- `scripts/smoke-first-pack.sh`
- `scripts/smoke-storage.sh`
- `scripts/smoke-memory.sh`
- `scripts/smoke-feedback.sh`
- `scripts/smoke-workspace.sh`
- `scripts/smoke-shared-artifacts.sh`
- `scripts/smoke-inspector.sh`
- `scripts/smoke-retrieval-health.sh`
- `scripts/smoke-graph.sh`
- `scripts/smoke-policy-embedding.sh`
- `scripts/smoke-agent-preview.sh`
- `scripts/smoke-demo-artifacts.sh`
- `scripts/smoke-distribution-metadata.sh`
- `scripts/smoke-release-governance.sh`
- `scripts/smoke-semantic.sh`
- `scripts/smoke-precision.sh`
- `scripts/smoke-v23-eval.sh`
- `scripts/smoke-mcp-protocol.sh` from a wrong cwd with an explicit `--repo`/MCP `repo` argument
- optional `ctxpack eval proof` benchmark product proof when `CTXPACK_BENCHMARK_CONFIG` is set

After all required checks pass, the gate writes a source-free proof bundle summary. By default it lives in the gate's temporary workspace; pass `CTXPACK_PROOF_DIR=/absolute/path/to/proof` to persist it:

```bash
CTXPACK_PROOF_DIR=/absolute/path/to/proof bash scripts/release-gate.sh
```

The proof summary records the checked `ctxpack` version, binary SHA-256, archive SHA-256, manifest name, audit report name, required check outcomes, optional benchmark/client proof status, and privacy status. It records file names and checksums instead of machine-local binary or repository paths.

The optional real-client evidence wrappers are:

- `scripts/smoke-codex-mcp.sh`
- `scripts/smoke-claude-mcp.sh`

Cursor and OpenCode have deterministic setup/protocol proof wrappers:

- `scripts/smoke-cursor-mcp.sh`
- `scripts/smoke-opencode-mcp.sh`

These wrappers validate repo-local setup artifacts and MCP protocol behavior,
and their evidence explicitly marks `realClientToolCalls: false`.

The semantic smoke proves explicit local semantic retrieval, source-free vector metadata, semantic provenance in plans, semantic-enabled eval metadata, scaffold/provider status for `local_hash`, and cloud-disabled privacy status. It does not call cloud embedding or reranking services. The optional `local_fastembed` backend remains behind the `local-embeddings` Cargo feature and is not a default release requirement.

The precision smoke proves Java/Kotlin symbol extraction, Java/Kotlin package import edges, source-free precision edge import, rejection of sensitive paths, and additive precision dependency output.

The v2.3 eval smoke proves the fixed-corpus proof path without external
repositories. It creates a small local git corpus and exercises source-free
candidate feature export, feedback recording, offline learned-policy proposal,
paired baseline verdicts, runtime diagnostics, and product proof fields for
fixed corpus identity, feature-export privacy, learned-policy status, and proof
boundary language.

The v2.4 gate smoke proves semantic/precision release-gate reporting without
external repositories. It creates a small local git corpus and exercises
promote/hold/block decisions, lexical/default/semantic/precision/full-hybrid
variant rows, policy-blocked reranker skips, provider policy, precision status,
named-case arrays, and local-only privacy metadata.

```bash
bash scripts/smoke-v24-gate.sh
```

The feedback smoke proves source-free feedback ingestion, policy report generation, candidate policy tuning, apply/rollback metadata, and budget outcome comparison.

The workspace smoke proves local multi-repo manifest initialization, source-free workspace status JSON, workspace prepare-task routing, repo-boundary-aware workspace packs, missing source sentinel leakage, and single-repo command compatibility without an explicit workspace manifest.

The shared-artifacts smoke proves source-free team policy templates, shared artifact export, schema/privacy inspection, compatible manifest import, MCP workspace resources for status and shared artifacts, and absence of sensitive sentinel leakage in outputs and local ctxpack state.

The inspector smoke proves source-free JSON and static HTML inspector exports, local filter UI hooks, source-bearing section labels, and absence of source sentinel leakage in inspector artifacts.

The retrieval-health smoke proves source-free health JSON and Markdown reports from real git history, including metrics, signal contributions, gap families, and absence of source sentinel leakage.

The graph smoke proves source-free graph neighborhood JSON and Markdown reports from real dependency/test edges, including nodes, edges, communities, cap-safe metadata, and absence of source sentinel leakage.

The policy/embedding smoke proves semantic provider status reporting, `deterministic_scaffold` labeling for `local_hash`, explicit provider-policy decisions, disabled default reranking, explicit cloud-disabled/source-transfer-denied flags, source-free policy experiment rows, graph comparison metadata, and absence of source sentinel leakage.

The agent-preview smoke proves Codex, Claude Code, Cursor, OpenCode, and generic MCP preview metadata, including MCP tools/resources, guidance paths, read/edit boundary notes, source-free flags, and absence of source sentinel leakage.

The demo and distribution metadata smokes prove public source-free examples,
package-manager preparation templates, update metadata, clean extraction
verification syntax, and explicit signing and notarization gaps. They do not
publish package-manager artifacts and are not a self-update implementation.

The release governance smoke proves source-free candidate status metadata,
ready/deferred/blocked lifecycle states, deterministic protocol proof language,
optional real-client proof boundaries, Cursor/OpenCode non-claims, and rollback
safety for marked local candidate directories.

The gate passes the same selected or extracted `CTXPACK_BIN` into the first-pack, storage, memory, feedback, workspace, shared-artifact, inspector, retrieval-health, graph, policy/embedding, agent-preview, semantic, precision, v2.3 eval, v2.4 semantic/precision gate, MCP protocol, Cursor/OpenCode setup-proof wrappers, and optional real-client smokes. Demo, distribution metadata, and release governance smokes are source-free metadata checks and do not need the binary. Real-client proof is not required by default. Use these environment variables when needed:

- `CTXPACK_SKIP_REAL_CLIENT=1` keeps Codex and Claude checks deterministic-only after the protocol proof.
- `CTXPACK_REQUIRE_REAL_CLIENT=1` makes missing Codex or Claude tool-call evidence fail the gate.
- `CTXPACK_REAL_CLIENT_EVIDENCE_DIR=/absolute/path/to/evidence` writes stable JSON evidence files with client version, ctxpack version, repo path, `prepare_task`, and `get_pack` proof when real-client checks run.
- `CTXPACK_BENCHMARK_CONFIG=/absolute/path/to/suite.json` runs `ctxpack eval proof --config ... --format json` and fails on report-generation, local-only privacy regressions, missing v2.3 product proof summary, missing paired baseline verdict contract, feature-export privacy regressions, learned-policy status regressions, missing proof-boundary language, or a non-promote `releaseGate.decision`. Neutral, mixed, unsafe, or too-expensive default retrieval proof blocks publication.

Current v2.5 proof status: the fixed two-repo production-retrieval proof
promotes default local retrieval under the channel-aware release gate. The gate
compares non-test context recall against lexical retrieval and validates tests
through the dedicated `recommended_tests` channel. The current source-free proof
is `.ctxpack/e2e/phase77-validation-command-coverage-proof.json`, where
`releaseGate.decision` is `promote`. RefactoringMiner context Recall@10 is
`0.7778` vs lexical `0.7407`; ctxpack context Recall@10 is `0.4225` vs lexical
`0.3521`. RefactoringMiner Test Recall@10 and Effective Validation Recall@10
are both `1.0`; the ctxpack required slice has no validation-test targets in
this refreshed proof. Phase 74 also separates
overall protected-evidence miss-rate from protected retrieval-target miss-rate:
RefactoringMiner target miss-rate@10 is `0.0588`, and ctxpack target
miss-rate@10 is `0.0833`. Phase 77 adds broad validation fallback commands and
effective validation-command coverage for multi-area smoke/eval tasks. The
latest optional four-repo probe in
`.ctxpack/e2e/phase78-ceiling-aware-broader-proof.json` now promotes broader
proof because RefactoringMiner is treated as a safe lexical-ceiling match:
ctxpack and lexical both have context Recall@10 `1.0`, validation is covered,
and protected retrieval-target miss-rate is `0.0`. VeriSchema also beats
through Effective Validation Recall@10 `1.0` while raw Test Recall@10 remains
`0.7090`.
For repeatable local investigation, use the pinned optional fixture at
`.planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json`; it is
expected to report `releaseGate.decision = promote` under the ceiling-aware
gate while still reporting protected target miss diagnostics for ctxpack and
VeriSchema. Phase 79 adds bounded protected target floors and archive deferral:
the latest required proof is
`.ctxpack/e2e/phase79-protected-target-floors-proof.json`, and the latest
broader proof is
`.ctxpack/e2e/phase79-broader-protected-target-floors-proof.json`. Both
promote. Required RefactoringMiner and broader VeriSchema protected
retrieval-target miss-rates are `0.0`; ctxpack still reports one residual
protected source-symbol miss in each proof.

Latest optional real-client proof: Codex CLI `0.130.0` and Claude Code
`2.1.158` both passed the smoke wrappers on 2026-05-30 with server-side
`prepare_task` and `get_pack` evidence against an explicit repo path. See
`.planning/e2e/2026-05-30-phase70-real-client-mcp-proof.md`. Cursor and
OpenCode real-client proof is still not claimed for v1.1.0.

RefactoringMiner and multi-repo proof are optional external gates. They are
skipped by default because they require a separate local checkout and longer
runtime. To reproduce the large-history gate, keep the repository local and run:

```bash
CTXPACK_BENCHMARK_CONFIG="$(pwd)/.ctxpack/benchmarks/refactoringminer-v23.json" bash scripts/release-gate.sh
```

To run a broader corpus, add more local repositories to a suite JSON and pass
that suite through `CTXPACK_BENCHMARK_CONFIG`. If the external repo is missing,
record the skip reason as "external corpus unavailable" rather than treating it
as a product regression. The mandatory gate remains `scripts/smoke-v23-eval.sh`,
which proves the v2.3 contract without external repos.

The release gate does not publish, upload, or create GitHub releases, and does not create tags. It does not mutate global agent config and does not run user project tests. Cursor and OpenCode real-client proof is not claimed for v1.1.0.

## Artifact Audit

`scripts/release-package.sh` runs `scripts/audit-release-artifact.sh` immediately after archive creation and before checksum success output. It writes a machine-readable `ctxpack-v1.1.0-{target}.audit.json` report next to the archive.

The audit lists archive members and extracts the artifact to a temporary directory. It fails on local state, traces, request logs, cache or target debris, git internals, secret-looking filenames, absolute local paths, and text payloads with machine-specific or secret-looking values. It does not upload artifacts or call cloud scanning services.

You can audit an existing archive directly:

```bash
bash scripts/audit-release-artifact.sh dist/ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz
```

## Out of Scope for v1.1

The v1.1.0 release does not require crates.io publishing, Homebrew taps, self-update support, signed installers, cloud telemetry, cloud indexing, cloud embeddings, hosted release services, or global agent config mutation.

ctxpack remains local-first and read-only. Release scripts build and audit ctxpack artifacts only; they do not mutate user repositories, global Codex or Claude configuration, MCP client config, or package-manager registries.
