# Pitfalls Research: v2.3 Evaluation Lab & Learned Retrieval Policy

## Common Mistakes

- Treating a tiny positive lift over lexical as product proof.
- Running large-history evals slowly enough that developers stop using them.
- Mixing source-bearing traces into learning datasets.
- Optimizing Recall@10 while precision, token ROI, and agent utilization get worse.
- Training or tuning on a single repository and then claiming general quality.
- Letting graph, symbol, or history candidates drown out exact lexical matches.
- Making learned weights default before they have bounded regression gates.
- Treating SWE-bench Verified-style results as sufficient despite contamination and test-quality concerns.

## Prevention

- Keep RefactoringMiner as a locked regression suite but require multi-repo manifests for claims beyond that repo.
- Report lexical and no-context baselines next to every ctxpack score.
- Add confidence or threshold verdicts: lift, neutral, regression, insufficient evidence.
- Cache historical eval inputs and reuse unchanged run artifacts.
- Export only source-free candidate features.
- Add signal saturation diagnostics to show when one retriever family overwhelms the final pack.
- Keep learned policies opt-in until they pass thresholded comparisons.
- Document benchmark boundaries clearly in product proof.
