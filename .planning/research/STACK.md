# Stack Research: v2.3 Evaluation Lab & Learned Retrieval Policy

## Current Stack

- Rust workspace with CLI, compiler, index, core contracts, MCP, and local storage crates.
- Source-free SQLite storage for inventories, packs, eval traces, benchmark metadata, memory cards, and feedback.
- Existing historical eval and benchmark commands.
- Existing policy-profile controls from v1.7.
- Existing retrieval-health and policy experiment reports from v2.1.
- Existing release-gate and product-proof scripts from v2.2.

## Additions

- Benchmark corpus manifest schema for fixed local/public corpora.
- Historical eval cache records keyed by repo snapshot, parent commit, options, and compiler/index versions.
- Parallel eval runner with deterministic merged output.
- Source-free candidate feature table/export format.
- Paired baseline/ablation comparison report.
- Offline policy learner that produces non-default profile proposals.
- Regression-threshold file for recall, precision proxy, token ROI, runtime, validation coverage, and source-free privacy.
- Bounded v2.3 eval smoke for release gates.

## Not Added Yet

- Hosted benchmark service.
- Cloud embeddings/reranking as default.
- Full SWE-bench Pro execution harness.
- Production vector backend migration.
- SCIP/LSP automatic indexer installation.
- Real-client agent outcome execution as a required release blocker.
