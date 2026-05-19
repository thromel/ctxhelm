# Feature Research: v2.3 Evaluation Lab & Learned Retrieval Policy

## Table Stakes

- Fixed benchmark corpus manifests with stable repo paths, revisions, commit ranges, budgets, and privacy labels.
- RefactoringMiner 20-commit regression suite locked as the first large-history stress target.
- Warm historical eval cache for parent snapshots, inventories, indexes, and stable candidate pools.
- Parallel commit evaluation with deterministic aggregation.
- Stored run reuse and comparison so unchanged eval ranges do not recompute from scratch.
- Source-free candidate feature export for every ranked candidate.
- Paired baseline reports comparing lexical, no-context, current default, and retrieval-signal ablations.
- Offline learned-policy experiment that proposes weights from source-free features and feedback.
- Regression thresholds for recall, precision proxy, token ROI, validation coverage, runtime, and privacy.

## Differentiators

- Process-level context metrics, not just pass/fail: recommended files, read files, edited files, gold changed files, related tests, and missed families.
- Honest product-proof language that distinguishes real lift from lexical parity.
- Source-free learning features that can improve policy without storing source snippets or prompts.
- Large-history speed work that makes eval iteration cheap enough to run during development.
- Policy learning as a gated experiment, not a silent default.

## Anti-Features

- Do not add cloud embeddings, cloud reranking, or hosted evals by default.
- Do not train on source text, snippets, or prompt content.
- Do not overfit to RefactoringMiner alone; use it as the first locked stress target and add more corpora through the same manifest format.
- Do not claim SoTA from a single eval slice.
- Do not make learned weights default until they beat the current default and lexical baselines under thresholds.
