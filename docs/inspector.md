# Pack Inspector

The pack inspector is a source-free diagnostic view of a generated context pack.
It is for maintainers and product evaluation, not a separate daily coding UI.

Use it when you need to see why ctxhelm selected files, tests, commands,
warnings, memory cards, candidates, and pack sections without copying raw source
snippets into a report artifact.

## Static Export

Generate an inspector artifact from the same planner/compiler path used by
`get-pack`:

```bash
ctxhelm inspector export "fix requireSession bug" \
  --repo /path/to/repo \
  --mode bug-fix \
  --budget brief \
  --target-agent codex \
  --format json
```

Supported formats:

```bash
ctxhelm inspector export "fix requireSession bug" --repo /path/to/repo --format json
ctxhelm inspector export "fix requireSession bug" --repo /path/to/repo --format markdown
ctxhelm inspector export "fix requireSession bug" --repo /path/to/repo --format html --output pack.html
```

The HTML export is a static, read-only local inspector. It includes filters for
text, category, and source-bearing sections, plus tables for target files,
related tests, validation commands, warnings, diagnostics, retrieval
candidates, selected memory, and section token estimates.

## Proof Summary

Summarize a saved source-free proof report:

```bash
ctxhelm inspector proof \
  --report .ctxhelm/e2e/phase322-agent-run-codex-target-first-breadth-suite.json
```

Machine-readable output is available for release notes, dashboards, or adoption
checks:

```bash
ctxhelm inspector proof \
  --report .ctxhelm/e2e/phase322-agent-run-codex-target-first-breadth-suite.json \
  --format json
```

Summarize a product-proof report together with an agent outcome report when you
need an adoption-facing evidence bundle:

```bash
ctxhelm inspector proof \
  --report .ctxhelm/proof/product-proof.json \
  --report .ctxhelm/e2e/phase322-agent-run-codex-target-first-breadth-suite.json
```

Use `--require-ready` when a CI job or release checklist should fail unless
the report is ready for its evidence claim:

```bash
ctxhelm inspector proof \
  --report .ctxhelm/proof/product-proof.json \
  --report .ctxhelm/e2e/phase322-agent-run-codex-target-first-breadth-suite.json \
  --require-ready
```

The proof inspector summarizes agent-run proof reports and product-proof
reports. For agent-run reports, it renders outcome claim, comparable task/lane
counts, target-read coverage, client availability, evidence-only target state,
retry cost, memory guard status when reported, client failures/rate limits,
forbidden boundary events, source-free privacy flags, and a recommended next
action. The client availability section reports `tinyPromptAvailable`,
`pairedSuiteAvailable`, `availabilityBlocker`, rate-limit status, client-failure
status, and comparable lane count, so a tiny-prompt pass is not confused with a
comparable paired-suite proof. For
product-proof reports, it renders release-gate decision, promotion allowance,
corpus verdict count, lexical/context claims, protected target miss-rate, and
the source-free privacy boundary. When more than one `--report` is provided, it
renders a bundle verdict with clean product-proof count, clean agent-outcome
count, availability-blocked count, source-free/privacy status, read-only
boundary status, and the next evidence action. With `--require-ready`, single
reports pass only when they are clean product-proof or clean agent-outcome
evidence, and bundles pass only when they contain both clean product proof and
clean agent outcome evidence. It does not print raw task text, raw prompts, raw
transcripts, MCP traffic, or target file lists.

## Local Shell

Run the optional localhost-only diagnostic shell:

```bash
ctxhelm inspector serve "fix requireSession bug" \
  --repo /path/to/repo \
  --mode bug-fix \
  --budget brief \
  --target-agent codex \
  --port 8765
```

The shell prints a loopback URL such as:

```text
ctxhelm inspector shell listening on http://127.0.0.1:8765
```

Available routes:

| Route | Purpose |
| --- | --- |
| `/` | Source-free diagnostic shell with links and summary counts. |
| `/pack-inspector.html` | Filterable pack inspector HTML. |
| `/pack-inspector.json` | Machine-readable `PackInspectorView`. |
| `/graph.html` | Filterable source-free graph neighborhood view. |
| `/graph.json` | Machine-readable graph neighborhood report. |
| `/setup-status.json` | Read-only generated-agent-artifact setup status. |
| `/health.json` | Local shell health, route inventory, and privacy flags. |

The shell is diagnostic only. It does not edit files, does not mutate global
agent configuration, does not run user project tests, and does not replace the
daily coding workflow inside Codex, Claude Code, Cursor, OpenCode, or another
agent.

## Source Boundary

`ctxhelm get-pack` may include source-bearing sections such as target snippets
and test snippets. `ctxhelm inspector export` does not copy those section
contents.

Instead, `PackInspectorView` records:

- section title and kind
- whether the section is source-bearing
- estimated section tokens
- target file paths, reasons, line ranges, confidence, and attribution
- related tests and validation commands
- retrieval candidates and signal evidence
- selected memory metadata
- warnings, diagnostics, and privacy status
- `sourceTextLogged: false`

This makes inspector artifacts suitable for local evaluation, release smokes,
and future UI rendering without creating another source cache.

## Contract

The public contract is `PackInspectorView` in
`crates/ctxhelm-core/src/contracts.rs`. It links back to:

- `packId`
- `taskId`
- `repoId`
- `taskHash`
- `taskType`
- `targetAgent`
- `budget`
- source-free retrieval candidate IDs

Later phases build on this contract for the local web inspector, retrieval
health reports, graph diagnostics, embedding controls, and agent previews.

## Privacy Guarantees

The inspector is metadata-only by design:

- It does not store raw source text.
- It does not store raw prompts, terminal logs, or model transcripts.
- It labels source-bearing pack sections instead of copying their content.
- It summarizes proof reports from source-free metrics instead of copying task
  text, target files, prompts, transcripts, or MCP traffic.
- It preserves `PrivacyStatus` from the compiled pack.
- It keeps cloud embeddings and cloud reranking disabled unless a future
  explicit policy gate enables them.

## Smoke Test

Maintainers can run:

```bash
bash scripts/smoke-inspector.sh
```

The smoke creates a temporary repository, exports JSON and HTML inspector
artifacts, summarizes synthetic source-free agent-run, product-proof, and
multi-report proof bundle reports, starts `ctxhelm inspector serve`, verifies
the shell, graph, setup, and health routes, and rejects a source sentinel in all
inspected outputs.
