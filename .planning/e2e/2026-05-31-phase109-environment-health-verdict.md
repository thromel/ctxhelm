# Phase 109: Environment Health Verdicts

## Goal

Make cold proof failures distinguish local Git/object-store health from
ctxpack retrieval quality.

## What Changed

- Benchmark repository reports now include source-free `environmentHealth`
  metadata.
- Product-proof corpus verdicts carry the same health metadata for each corpus.
- The release gate now blocks with an environment-health reason when a corpus
  has insufficient evidence because Git history or object reads are degraded.
- The health classifier keeps raw error details out of the verdict reason and
  records only source-free statuses:
  - `healthy`
  - `git_history_timeout`
  - `git_history_unavailable`
  - `git_object_store_unavailable`
  - `degraded`

## Evidence

Focused validation passed:

```text
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase109-target cargo test -p ctxpack-compiler benchmark_suite_embeds_report_when_history_sampling_is_unavailable -- --nocapture
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase109-target cargo test -p ctxpack-compiler benchmark_repo_environment_health_classifies_source_free_failures -- --nocapture
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase109-target cargo test -p ctxpack-compiler product_proof_release_gate_blocks_mixed_or_trailing_corpora -- --nocapture
```

Cold proof artifact:

```text
.ctxpack/e2e/phase109-environment-health-proof.json
```

The refreshed cold proof still blocks, but now blocks for environment health
instead of ambiguous retrieval quality:

```text
decision = block
reason = benchmark environment health is degraded before retrieval quality can be proven
evaluatedRepositoryCount = 1
evaluatedCommitCount = 5

RefactoringMiner = insufficient_evidence, git_history_unavailable
ctxpack = insufficient_evidence, git_history_timeout
ReAgent = insufficient_evidence, git_history_timeout
VeriSchema = beat, healthy
```

## Remaining Blocker

This phase does not make the cold four-repo proof promotable. It makes the
blocking reason honest and machine-readable. The next production-readiness work
should either repair the local Git/object-store environment for the affected
corpora or add a clean cold fixture path that proves retrieval quality without
depending on the degraded interactive checkouts.
