# Phase 153 - BM25 Symbol Lexical Index

## Goal

Start the R&D memo's second bet: replace the heuristic lexical scanner with a
real local lexical/symbol index path while preserving ctxhelm's local-first and
source-free output contract.

## Implementation

- Added Tantivy `0.25.0` to `ctxhelm-index` with default features disabled.
  - Pinned to the Rust 1.87-compatible line.
  - Pinned `time` to `0.3.45` through `Cargo.lock` because newer transitive
    `time` releases require Rust 1.88.
  - Avoided default mmap/compression features because this phase uses an
    in-memory query-time index.
- Replaced the lexical search hot path with a query-time in-memory Tantivy index.
  - Indexed fields: `path`, `filename`, `role`, `language`, `symbols`,
    and safe file `content`.
  - Stored fields: source-free metadata only (`path`, `role`, `language`).
  - Source-derived content is indexed in RAM and is not persisted.
- Added symbol-name facets from the existing Tree-sitter/fallback symbol
  extractor into the BM25 index.
- Combined BM25 score with the existing exact path/content bonuses so current
  safety floors and tests keep their behavior while the primary lexical signal
  becomes fielded BM25.
- Kept common task verbs out of the BM25 query by filtering zero-weight terms
  before parsing the Tantivy query.
- Exposed the new evidence in result reasons with `bm25 fielded score` and
  `exact symbol index available`.

## Privacy Boundary

This phase deliberately does **not** create a persistent source-derived inverted
index. Persistent BM25/Tantivy storage needs a separate explicit policy gate
because tokenized source/index data is still source-derived even when raw source
is not emitted.

## Validation

```bash
cargo test -p ctxhelm-index lexical_search --locked
```

Passed.

## Remaining Work

This is the first production lexical substrate slice, not the full Bet 2
completion. Remaining R&D work:

- measure BM25 versus the previous scanner on fixed corpora and native-agent
  outcome suites
- add an explicit persistent-index privacy policy before storing source-derived
  inverted indexes
- add exact symbol table lookup separate from the Tantivy field, with stable
  symbol IDs and line ranges
- report latency and recall lift/regression in product proof
