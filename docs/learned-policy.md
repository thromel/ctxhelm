# Offline Learned Retrieval Policy

`ctxpack eval policy learn` proposes a local retrieval-policy profile from
source-free evidence. It does not change default ranking by itself.

## Run

```bash
ctxpack eval features export "fix login redirect" --repo /path/to/repo --format json
ctxpack eval feedback record --repo /path/to/repo ...
ctxpack eval policy learn --repo /path/to/repo --format markdown
ctxpack eval policy learn --repo /path/to/repo --format json
```

Thresholds are configurable:

```bash
ctxpack eval policy learn \
  --repo /path/to/repo \
  --min-context-precision 0.45 \
  --min-validation-coverage 0.30 \
  --min-pass-rate 0.50 \
  --min-gold-or-selected-rows 10
```

## Evidence

The learner reads local source-free state:

- candidate feature exports under ctxpack local state;
- historical labels carried by feature rows, such as `selected` or `gold`;
- feedback quality reports;
- outcome comparison reports.

It writes a `RetrievalPolicyProfile` with:

- `profileSchemaVersion`;
- `trainingCorpusId`;
- `trainingSources`;
- `metricSummary`;
- signal weights;
- safety floors;
- `baselineThresholds`;
- `defaultEligible`;
- regression warnings.

## Guardrails

Learned profiles are candidates. They cannot become active through
`ctxpack eval policy apply` unless `defaultEligible` is true. This prevents a
profile trained from sparse or low-quality evidence from silently replacing the
current retrieval policy.

Safety floors keep anchor, lexical, and related-test signals above conservative
minimums. Semantic evidence remains opt-in and bounded by source-free selected
or gold rows.

## Privacy

The learner does not read or store raw source snippets, prompt text, terminal
logs, stack traces, commit subjects, model transcripts, or cloud payloads.
Reports and stored profiles remain local metadata.

