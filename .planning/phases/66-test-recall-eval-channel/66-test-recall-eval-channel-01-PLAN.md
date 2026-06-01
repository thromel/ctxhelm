# Phase 66 Plan: Test Recall Evaluation Channel

Date: 2026-05-30

## Goal

Fix the v2.5 product proof's zero Test Recall@10 signal without degrading target-file retrieval quality.

## Problem

Phase 65 reported `testRecallAt10 = 0.0` for both RefactoringMiner and ctxhelm. The follow-up inspection showed that `prepare_task` already returned relevant `related_tests`, but the historical eval measured test recall against `recommended_context_files`, where the 10-slot target-file ranking could be full before related tests appeared.

That made validation quality look absent even when the product had produced a dedicated validation channel.

## Plan

1. Verify whether related tests are being discovered before changing ranking behavior.
2. Avoid bluntly reserving source-ranking slots for tests if it harms file recall or protected evidence.
3. Measure Test Recall@K against `recommended_tests`, the product's dedicated validation output.
4. Improve related-test ordering where capped confidence ties hide stronger exact test relationships.
5. Re-run the two-repo product proof and keep the release gate honest.

## Success Criteria

- Test Recall@10 is non-zero on both configured proof corpora.
- Source/file recall is not degraded by forcing tests into the source ranking.
- The release gate still blocks default promotion unless every required corpus beats lexical.
- Focused tests cover the changed behavior.
