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

The command does not start a persistent server and does not require the local
web inspector planned for later v2.1 phases.

The HTML export is a static, read-only local inspector. It includes filters for
text, category, and source-bearing sections, plus tables for target files,
related tests, validation commands, warnings, diagnostics, retrieval
candidates, selected memory, and section token estimates.

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
- It preserves `PrivacyStatus` from the compiled pack.
- It keeps cloud embeddings and cloud reranking disabled unless a future
  explicit policy gate enables them.

## Smoke Test

Maintainers can run:

```bash
bash scripts/smoke-inspector.sh
```

The smoke creates a temporary repository, exports JSON and HTML inspector
artifacts, verifies the UI hooks, and rejects a source sentinel in both outputs.
