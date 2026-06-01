# Phase 87: Validation Gap Accounting

## Goal

Keep production proof diagnostics honest by separating context-file misses from
validation-channel coverage. Several broad proof reports still listed
`testMapping` gap families even when the same tests were already present in
`recommendedTests` or covered by targeted validation commands.

## Change

- Specific Java validation commands such as
  `./gradlew test --tests org.example.TestClass` and
  `mvn -Dtest=TestClass test` now count as covering their matching test files.
- Retrieval gap summaries now skip test files that are missing from the
  context-file top 10 but covered by the validation channel.
- All-file recall remains unchanged; this only prevents validation-covered
  tests from being reported as unresolved retrieval gaps.

## Evidence

Focused tests:

```bash
cargo test -p ctxhelm-compiler validation_command_coverage_recognizes_java_class_selectors -- --nocapture
cargo test -p ctxhelm-compiler retrieval_gap_summaries_skip_validation_covered_tests -- --nocapture
```

Pinned broader proof:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxhelm-phase87-validation-gap-accounting-proof.json
```

Committed artifact:

```text
.ctxhelm/e2e/phase87-validation-gap-accounting-proof.json
```

## Result

Quality metrics are non-regressing versus Phase 84:

| Corpus | File Recall@10 | Source Recall@10 | Test Recall@10 | Validation Command Recall | Effective Validation Recall@10 | Protected Target Miss-Rate@10 |
| --- | --- | --- | --- | --- | --- | --- |
| RefactoringMiner | `0.6 -> 0.6` | `1.0 -> 1.0` | `1.0 -> 1.0` | `0.0 -> 1.0` | `1.0 -> 1.0` | `0.0 -> 0.0` |
| ctxhelm | `0.44603175 -> 0.44603175` | `0.6333333 -> 0.6333333` | `0.0 -> 0.0` | `0.0 -> 0.0` | `0.0 -> 0.0` | `0.0 -> 0.0` |
| ReAgent | `0.5 -> 0.5` | `1.0 -> 1.0` | `1.0 -> 1.0` | `1.0 -> 1.0` | `1.0 -> 1.0` | `0.0 -> 0.0` |
| VeriSchema | `0.17936651 -> 0.17936651` | `0.30409357 -> 0.30409357` | `0.7089947 -> 0.7089947` | `1.0 -> 1.0` | `1.0 -> 1.0` | `0.2857143 -> 0.2857143` |

The broader proof still blocks on the known cold runtime threshold:

```text
Blocked because proof runtime exceeded 5000ms per commit for: RefactoringMiner.
```

That is a runtime gate, not a quality regression. The diagnostic improvement is
visible in the gap summaries: RefactoringMiner and ReAgent no longer report
validation-covered tests as unresolved `testMapping` gaps, while VeriSchema's
remaining top gaps are source retrieval pressure.

## Rejected Experiment

Before this fix, a broad source-area diversity floor was tested and rejected.
It reduced VeriSchema File Recall@10 from `0.17936651` to `0.17269985` and
increased protected target miss-rate from `0.2857143` to `0.42857143`.

## Next Work

- Continue with source candidate generation for VeriSchema `no_candidate_signal`
  areas.
- Avoid broad source diversity selectors unless they improve source recall
  without increasing protected target misses.
- Keep validation accounting separate from context-file ranking in future proof
  diagnostics.
