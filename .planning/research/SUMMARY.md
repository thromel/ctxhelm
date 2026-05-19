# Research Summary: v2.3 Evaluation Lab & Learned Retrieval Policy

**Status:** External research refresh plus synthesis from the prior ctxpack retrieval, GraphRAG, memory, feedback, and RefactoringMiner E2E evidence.

## Research Thesis

ctxpack has enough retrieval modes to be useful, but it is not yet world-class because the product proof is still too small, too slow, and too close to lexical baseline on the hardest real repo slice. The next milestone should make quality claims repeatable and teach the system when each signal should dominate.

The research points to four priorities:

1. **Evaluate process, not only final answers.** ContextBench argues that coding-agent benchmarks need intermediate gold-context metrics because final task success hides whether agents retrieved and used the right code context.
2. **Use realistic, contamination-aware corpora.** OpenAI now recommends SWE-bench Pro over SWE-bench Verified for frontier coding claims because Verified is contaminated and has flawed tests. ctxpack's own proof should therefore use fixed local corpora, held-out ranges where possible, and source-free metadata.
3. **Optimize useful context under latency.** RepoBench separates retrieval, completion, and pipeline tasks; HEF and RepoFuse-style work emphasize that repository context must be cached, compressed, and fast enough to use repeatedly.
4. **Learn selective policy before adding heavier backends.** Graph-based systems such as RepoGraph and GraphCoder support repository-wide navigation and subgraph retrieval, while SWE-ContextBench shows that accurately selected experience helps and unfiltered context can hurt.

## Prior ctxpack Evidence To Carry Forward

- RefactoringMiner full E2E confirmed real-client Claude Code can call ctxpack through MCP against a large Java repository.
- The same E2E exposed a small retrieval-quality margin: fixed ctxpack Recall@10 `0.5186` versus lexical baseline `0.5008`.
- Historical eval over the 20-commit RefactoringMiner slice was slow enough to block tight iteration, with the documented run taking several minutes.
- Several failures were policy failures, not missing backend failures: strong lexical hits were present but selected too low after graph/history/symbol saturation.
- Related-test output improved after fixing Java/Gradle command inference, which confirms eval-driven iteration is productive.

## Milestone Direction

v2.3 should not start by adding cloud embeddings, production vector stores, or a new GraphRAG framework. It should first build the evaluation lab that can prove those later changes help.

Recommended milestone shape:

1. Fixed benchmark corpus and RefactoringMiner regression suite.
2. Warm historical eval cache and parallel runner.
3. Source-free candidate feature export.
4. Paired baseline and ablation reports with honest lift/neutral/regression verdicts.
5. Offline learned retrieval-policy experiment that proposes weights but is not default until gated.
6. Product proof and release-gate integration for bounded v2.3 eval smoke.

## Sources

- OpenAI, "Why SWE-bench Verified no longer measures frontier coding capabilities": https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/
- RepoBench ICLR 2024 abstract: https://proceedings.iclr.cc/paper_files/paper/2024/hash/d191ba4c8923ed8fd8935b7c98658b5f-Abstract-Conference.html
- ContextBench arXiv: https://arxiv.org/abs/2602.05892
- SWE-ContextBench arXiv: https://arxiv.org/abs/2602.08316
- SWE-Bench Pro arXiv: https://arxiv.org/abs/2509.16941
- RepoGraph arXiv: https://arxiv.org/abs/2410.14684
- Microsoft GraphRAG query overview: https://microsoft.github.io/graphrag/query/overview/
- Hierarchical Embedding Fusion arXiv: https://arxiv.org/abs/2603.06593
