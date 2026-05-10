# Repo Context Packer Milestone 11: Local Symbol Index

## Goal

Move beyond flat lexical matching by extracting local code symbols from safe inventoried files and making symbol lookup available to both CLI and MCP consumers.

## Scope

- Add typed symbol contracts in `ctxpack-index`.
- Extract definitions from TypeScript/JavaScript, Python, Rust, and Go.
- Include symbol name, kind, path, language, line range, signature, and export status.
- Add `ctxpack symbols` for listing and querying symbols.
- Back `ctxpack://symbol/<query>` with symbol search instead of plain lexical search.
- Keep extraction local-only and filtered through the safe inventory.

## Implemented Symbol Kinds

- function
- method
- class
- interface
- type
- constant
- import
- module

## Verification

- unit coverage for language-aware symbol extraction
- unit coverage for symbol-name-prioritized search
- existing MCP symbol resource tests
- CLI smoke for `ctxpack symbols --query`
- MCP stdio smoke for `ctxpack://symbol/<query>`
- full workspace test, clippy, and CLI help before closing the milestone
