# Phase 63 Patterns

## Promotion Is More Than Recall

Do not promote a variant because Recall@10 improves in aggregate. Promotion also requires runtime, token ROI, privacy, and named-regression gates.

## Protect Explicit Evidence

Anchors, current diff files, exact lexical matches, and high-confidence symbols are stronger than speculative semantic or graph evidence. Fusion and reranking may reorder weak candidates, but they must not bury protected evidence.

## Use Existing Eval Surfaces

Keep Phase 63 on `ctxpack eval benchmark` and related policy reports. A new evaluator would make Phase 61 and Phase 62 comparisons harder to trust.

## Source-Free Reports

Reports may include paths, symbols, scores, reasons, and metric deltas. They must not expose source snippets unless the user explicitly asks for a context pack.

## Named Regressions Beat Averages

A higher average score is not enough if a critical task, anchor-heavy query, exact identifier query, or test-mapping query regresses beyond the configured threshold.
