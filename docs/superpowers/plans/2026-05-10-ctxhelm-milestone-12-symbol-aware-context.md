# ctxhelm Milestone 12: Symbol-Aware Context Plans

## Goal

Use the local symbol index during task preparation so context plans prefer precise symbol evidence over broad file-level lexical matches.

## Scope

- Merge symbol search results into `prepare_context_plan`.
- Prefer symbol hits before lexical file hits.
- Carry symbol line ranges into `TargetFile`.
- Render target file line hints in packs.
- Materialize target snippets around symbol line ranges instead of always starting at line 1.
- Preserve source-only anchors for related-test and co-change expansion.

## Verification

- focused compiler test for symbol line ranges in plans
- focused compiler test for snippets centered around a matched symbol below file header content
- existing related-test and co-change fusion tests
- full workspace test, clippy, CLI help, and command smoke before closing the milestone
