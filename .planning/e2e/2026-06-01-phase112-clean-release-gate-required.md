# Phase 112 - Clean Release Gate With Required Fixture Proof

Date: 2026-06-01

## Goal

Prove the packaged release gate from a clean checkout with the clean cold
four-repo fixture proof required, not merely available.

## Clean Checkout

Checkout:

```text
/Users/romel/Documents/GitHub/ctxhelm-release-gate-clean-20260601
```

Revision:

```text
20c367dc7eafc1231559c9110901961c55645089
```

The checkout had no uncommitted files and no macOS `dataless` placeholders.

## Command

```bash
CARGO_NET_OFFLINE=true \
CARGO_TARGET_DIR=/tmp/ctxhelm-phase112-release-target \
CTXHELM_PROOF_DIR=/tmp/ctxhelm-phase112-release-proof \
CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF=1 \
bash scripts/release-gate.sh
```

## Result

The release gate passed.

Durable summary:

```text
.ctxhelm/e2e/phase112-clean-release-gate-summary.json
```

Proof bundle facts:

- `status = passed`
- `binaryIdentity.source = archive`
- `optionalProofs.cleanColdFixtureProductProof = passed`
- `optionalProofs.cleanColdFixtureRequired = true`
- `optionalProofs.resourceBackedGapSummaryContract = checked`
- required check count: `28`

Archive identity:

```text
ctxhelm-v1.1.0-aarch64-apple-darwin.tar.gz
sha256: 5c3a4842e6afd3c1601846f21e6367c1dec664af04083e8f65f389f880ac4005
```

Binary identity:

```text
ctxhelm
sha256: 92700827037f34b72e24fde627dd8b9f6506037cd0bf2a6e11dc66b3ac9887ee
```

## Clean Fixture Product Proof

The required clean cold fixture proof promoted:

- RefactoringMiner: `match`, context Recall@10 `1.0`, lexical context Recall@10 `1.0`, Test Recall@10 `1.0`, Effective Validation Recall@10 `1.0`
- ctxhelm: `beat`, context Recall@10 `0.3888889`, lexical context Recall@10 `0.30555555`
- ReAgent: `beat`, context Recall@10 `1.0`, lexical context Recall@10 `0.2857143`, Test Recall@10 `1.0`, Effective Validation Recall@10 `1.0`
- VeriSchema: `beat`, context Recall@10 `0.20547946`, lexical context Recall@10 `0.08219178`, Test Recall@10 `0.71957666`, Effective Validation Recall@10 `1.0`

## Gate Coverage

The release gate covered:

- `cargo test --workspace`
- `scripts/check-release-docs.sh`
- release package build
- release archive audit
- clean extraction archive verification
- extracted archive binary `ctxhelm --version`
- extracted archive binary `ctxhelm --help`
- first-pack, storage, memory, feedback, workspace, shared artifact, inspector, retrieval-health, graph, policy/embedding, agent-preview, demo, distribution, governance, semantic, precision, v2.3 eval, v2.4 gate, wrong-cwd MCP, Cursor setup, OpenCode setup smokes
- required clean cold fixture product proof
- optional Codex/Claude deterministic protocol gates, with real-client tool-call proof intentionally skipped by default

## Remaining Work

This proves the release gate can pass in a clean release-candidate checkout with
the real four-repo proof required. Remaining production-readiness work should
focus on real-client proof refreshes when client versions change, distribution
candidate status/signing gaps, and further source-free area/ranking quality
improvements.
