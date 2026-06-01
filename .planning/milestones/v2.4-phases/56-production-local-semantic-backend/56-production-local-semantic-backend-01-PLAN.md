---
phase: 56
plan: 56-production-local-semantic-backend-01
title: Production Local Semantic Backend
type: implementation
wave: 1
depends_on: []
files_modified:
  - crates/ctxhelm-index/Cargo.toml
  - crates/ctxhelm-index/src/semantic.rs
  - crates/ctxhelm-index/src/storage.rs
  - crates/ctxhelm-core/src/contracts.rs
  - crates/ctxhelm-compiler/src/policy.rs
  - crates/ctxhelm/src/main.rs
  - docs/semantic.md
  - docs/policy-embedding.md
  - scripts/smoke-semantic.sh
  - scripts/check-release-docs.sh
autonomous: true
requirements:
  - SEM-01
  - SEM-02
  - SEM-03
  - SEM-04
requirements_addressed:
  - SEM-01
  - SEM-02
  - SEM-03
  - SEM-04
---

# Plan 56.01: Production Local Semantic Backend

<objective>
Add the production local semantic backend foundation: a real local embedding-provider path behind explicit opt-in, source-free vector/provider metadata, provider status diagnostics, and clear `local_hash` scaffold labeling while preserving existing CLI/MCP compatibility and local-only defaults.
</objective>

<threat_model>
**Assets:** proprietary source code, generated semantic vectors, provider/model metadata, local cache paths, benchmark reports, and ctxhelm storage.

**Threats:**

- **High:** source text or semantic document text is persisted into SQLite, eval reports, feature exports, product proof, or logs.
- **High:** a cloud embedding/reranking provider is enabled implicitly through the new backend abstraction.
- **Medium:** optional model downloads make default tests non-deterministic or dependent on network access.
- **Medium:** provider status implies quality lift even when the backend is only scaffold/test behavior.
- **Medium:** semantic candidates crowd out explicit anchors or exact lexical matches.

**Mitigations required by this plan:**

- All embedding input must flow through existing safe inventory and `read_safe_source` checks.
- Default provider remains local-only and disabled unless explicit semantic flags are passed.
- `local_hash` must be labeled deterministic scaffold/test behavior in contracts, CLI status, and docs.
- Real local embedding backend must be optional and feature-gated so normal workspace tests do not require model downloads.
- No cloud provider execution is added in Phase 56.
</threat_model>

<must_haves>
<truths>
- `local_hash` remains deterministic scaffold/test behavior and is not described as a quality embedding backend.
- Cloud embeddings and cloud reranking remain disabled by default.
- Semantic storage and reports remain source-free: provider/model/vector metadata only, no raw source text or semantic document text.
- Existing `--semantic`, `ctxhelm semantic status`, MCP additive semantic arguments, and existing JSON field names stay backward-compatible.
- Phase 56 does not enable semantic defaults; quality promotion belongs to Phase 60 gates.
</truths>
</must_haves>

<tasks>
<task id="T1" type="tdd">
  <title>Extend semantic provider metadata and scaffold labeling</title>
  <read_first>
    - crates/ctxhelm-index/src/semantic.rs
    - crates/ctxhelm-core/src/contracts.rs
    - crates/ctxhelm-compiler/src/policy.rs
    - docs/semantic.md
    - docs/policy-embedding.md
  </read_first>
  <files>
    - crates/ctxhelm-index/src/semantic.rs
    - crates/ctxhelm-core/src/contracts.rs
    - crates/ctxhelm-compiler/src/policy.rs
    - crates/ctxhelm/src/main.rs
    - docs/semantic.md
    - docs/policy-embedding.md
  </files>
  <action>
    Add explicit provider status fields that distinguish provider id, model id, dimensions, distance metric, local-only status, default-enabled status, and quality/scaffold status. Preserve existing `SemanticProviderConfig` camelCase JSON fields. Add a stable label for `local_hash` such as `qualityBackend: false` or `providerRole: "deterministic_scaffold"` and propagate it into `semantic_provider_status_report` plus `render_semantic_provider_status`. Update docs so `local_hash` is described as deterministic scaffold/test behavior, not a retrieval-quality backend.
  </action>
  <verify>
    - cargo test -p ctxhelm-index semantic_search_is_disabled_by_default semantic_search_finds_conceptual_safe_files -- --nocapture
    - cargo test -p ctxhelm-compiler semantic_provider_status -- --nocapture
    - cargo run -p ctxhelm -- semantic status --repo . --format json
  </verify>
  <acceptance_criteria>
    - `cargo run -p ctxhelm -- semantic status --repo . --format json` includes provider id `local_hash`, model id `ctxhelm-local-hash-v1`, `enabledByDefault: false`, `cloudEmbeddingsAllowed: false`, and a machine-readable field showing `local_hash` is scaffold/test behavior rather than a quality backend.
    - `docs/semantic.md` contains the string `local_hash` and states it is deterministic scaffold/test behavior.
    - Existing semantic disabled-by-default tests still pass.
  </acceptance_criteria>
</task>

<task id="T2" type="tdd">
  <title>Add optional real local embedding provider behind a feature gate</title>
  <read_first>
    - crates/ctxhelm-index/Cargo.toml
    - crates/ctxhelm-index/src/semantic.rs
    - Cargo.toml
    - Cargo.lock
  </read_first>
  <files>
    - Cargo.toml
    - Cargo.lock
    - crates/ctxhelm-index/Cargo.toml
    - crates/ctxhelm-index/src/semantic.rs
  </files>
  <action>
    Add an optional `local-embeddings` Cargo feature for `ctxhelm-index` and wire a real local embedding provider id `local_fastembed` behind that feature. Use the `fastembed` crate as the first real local backend when the feature is enabled. Keep `local_hash` as the default provider for normal builds. Add provider construction so unsupported or feature-disabled `local_fastembed` requests return a warning diagnostic such as `semantic_provider_unavailable` instead of panicking or falling back silently.
  </action>
  <verify>
    - cargo check -p ctxhelm-index
    - cargo check -p ctxhelm-index --features local-embeddings
    - cargo test -p ctxhelm-index semantic_provider_unavailable_without_feature -- --nocapture
  </verify>
  <acceptance_criteria>
    - `crates/ctxhelm-index/Cargo.toml` defines feature `local-embeddings`.
    - Default `cargo check -p ctxhelm-index` does not require model downloads or cloud credentials.
    - `cargo check -p ctxhelm-index --features local-embeddings` compiles the real local provider path.
    - Requesting provider `local_fastembed` without the feature yields diagnostic code `semantic_provider_unavailable` and does not use cloud services.
  </acceptance_criteria>
</task>

<task id="T3" type="execute">
  <title>Persist source-free provider/vector metadata for both providers</title>
  <read_first>
    - crates/ctxhelm-index/src/storage.rs
    - crates/ctxhelm-index/src/semantic.rs
    - docs/storage.md
  </read_first>
  <files>
    - crates/ctxhelm-index/src/storage.rs
    - crates/ctxhelm-index/src/semantic.rs
    - docs/storage.md
  </files>
  <action>
    Ensure semantic vector persistence records provider id, model id, dimensions, distance metric, safe file hash, privacy status, and vector metadata for `local_hash` and `local_fastembed` without storing raw source text or semantic document text. If schema changes are required, add a backward-compatible migration and keep existing semantic vector records readable. Extend the existing `persists_semantic_vectors_without_source_text` style test to cover the new provider metadata fields.
  </action>
  <verify>
    - cargo test -p ctxhelm-index persists_semantic_vectors_without_source_text -- --nocapture
    - cargo run -p ctxhelm -- index --repo . --store --semantic
    - cargo run -p ctxhelm -- storage status --repo . --format json
  </verify>
  <acceptance_criteria>
    - The `semantic_vectors` persistence path stores provider/model/dimensions/distance metadata for every vector record.
    - Storage tests assert that a sentinel source string is absent from stored semantic metadata.
    - `ctxhelm storage status --repo . --format json` reports semantic vector counts without exposing source text.
  </acceptance_criteria>
</task>

<task id="T4" type="execute">
  <title>Expose backend status through CLI and existing semantic workflows</title>
  <read_first>
    - crates/ctxhelm/src/main.rs
    - crates/ctxhelm-compiler/src/policy.rs
    - crates/ctxhelm-compiler/src/planning.rs
    - crates/ctxhelm-compiler/src/ranking.rs
  </read_first>
  <files>
    - crates/ctxhelm/src/main.rs
    - crates/ctxhelm-compiler/src/policy.rs
    - crates/ctxhelm-compiler/src/planning.rs
    - crates/ctxhelm-compiler/src/ranking.rs
  </files>
  <action>
    Update `ctxhelm semantic status`, `ctxhelm index --semantic`, `ctxhelm search --semantic`, `ctxhelm prepare-task --semantic`, and `ctxhelm get-pack --semantic` output paths so semantic candidates and status reports include the selected provider id, model id, scaffold/quality status, local-only privacy status, and provider warnings. Preserve existing command names and additive MCP semantics. Do not add a new MCP tool.
  </action>
  <verify>
    - cargo run -p ctxhelm -- semantic status --repo . --query "semantic backend provider" --format json
    - cargo run -p ctxhelm -- search "semantic backend provider" --repo . --semantic --format json
    - cargo run -p ctxhelm -- prepare-task "improve semantic backend provider status" --repo . --semantic --format json
  </verify>
  <acceptance_criteria>
    - Semantic candidate reasons include provider id and model id.
    - JSON output remains camelCase and backward-compatible for existing fields.
    - `prepare-task --semantic` still treats semantic as an additive signal and does not remove explicit-anchor or lexical provenance.
  </acceptance_criteria>
</task>

<task id="T5" type="execute">
  <title>Update docs, release checks, and smoke coverage</title>
  <read_first>
    - docs/semantic.md
    - docs/policy-embedding.md
    - docs/release.md
    - scripts/smoke-semantic.sh
    - scripts/check-release-docs.sh
    - .planning/e2e/2026-05-19-semantic-ablation/summary.md
  </read_first>
  <files>
    - docs/semantic.md
    - docs/policy-embedding.md
    - docs/release.md
    - scripts/smoke-semantic.sh
    - scripts/check-release-docs.sh
  </files>
  <action>
    Document the Phase 56 backend boundary: `local_hash` is deterministic scaffold/test behavior, `local_fastembed` is the optional real local backend behind `local-embeddings`, cloud providers remain disabled, and semantic defaults are not promoted until Phase 60 gates. Update semantic smoke and release-doc checks to assert provider status strings, cloud-disabled status, and source-free storage behavior.
  </action>
  <verify>
    - bash scripts/smoke-semantic.sh
    - bash scripts/check-release-docs.sh
    - cargo run -p ctxhelm -- --help
  </verify>
  <acceptance_criteria>
    - `docs/semantic.md` documents `local_hash`, `local_fastembed`, `local-embeddings`, and cloud-disabled behavior.
    - `scripts/check-release-docs.sh` checks for the new provider/status documentation strings.
    - `scripts/smoke-semantic.sh` verifies source-free provider metadata and does not require cloud credentials.
  </acceptance_criteria>
</task>
</tasks>

<verification>
Run focused checks while implementing each task, then run:

```bash
cargo fmt --all
cargo test --workspace
cargo check -p ctxhelm-index --features local-embeddings
cargo run -p ctxhelm -- --help
bash scripts/smoke-semantic.sh
bash scripts/check-release-docs.sh
```

If `cargo check -p ctxhelm-index --features local-embeddings` requires a network download or unavailable local model cache, record the skip reason and prove the default workspace remains green without that feature.
</verification>

<success_criteria>
- SEM-01: A real local embedding provider path exists behind `local-embeddings` and source text stays local.
- SEM-02: CLI/status reports expose provider id, model id, dimensions, cache/freshness/degraded state, and privacy status.
- SEM-03: `local_hash` remains available only as deterministic scaffold/test behavior and is labeled that way in CLI JSON/Markdown and docs.
- SEM-04: Semantic candidates retain typed provider provenance and source-free eval features without incompatible CLI/MCP contract changes.
</success_criteria>

## PLANNING COMPLETE
