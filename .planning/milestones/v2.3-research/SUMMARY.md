# Research Summary: v2.3 Evaluation Lab & Learned Retrieval Policy

See `.planning/research/SUMMARY.md` for the active research synthesis. This milestone is grounded in the same findings:

- Process-level context retrieval metrics matter more than final pass/fail alone.
- Contamination-aware, fixed corpora are required before making world-class claims.
- ctxpack's current RefactoringMiner result is useful but only slightly above lexical baseline, so v2.3 must improve proof quality before larger retrieval architecture.
- Evaluation speed is now a product bottleneck.
- Learned policy must be source-free, opt-in, and threshold-gated.

Primary sources:

- https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/
- https://proceedings.iclr.cc/paper_files/paper/2024/hash/d191ba4c8923ed8fd8935b7c98658b5f-Abstract-Conference.html
- https://arxiv.org/abs/2602.05892
- https://arxiv.org/abs/2602.08316
- https://arxiv.org/abs/2509.16941
- https://arxiv.org/abs/2410.14684
- https://microsoft.github.io/graphrag/query/overview/
- https://arxiv.org/abs/2603.06593
