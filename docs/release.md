# ctxhelm `ctxhelm` v2.4.3 Release Guide

This document describes the local binary release path for ctxhelm's
`ctxhelm` CLI v2.4.3. The primary user path is a prebuilt archive plus SHA-256
checksums; source builds are fallback paths.

Current public archive release:

```text
https://github.com/thromel/ctxhelm/releases/tag/v2.4.3
```

## User Install

On Apple Silicon macOS, install from the public Homebrew tap:

```bash
brew tap thromel/tap
brew install ctxhelm
ctxhelm --version
ctxhelm --help
```

Download the archive for your platform from the release artifacts. Archive names follow this shape:

```text
ctxhelm-v2.4.3-{target}.tar.gz
sha256sums.txt
```

Verify checksums before installing:

```bash
shasum -a 256 -c sha256sums.txt
sha256sum -c sha256sums.txt
```

Extract and install the binary on `PATH`:

```bash
tar -xzf ctxhelm-v2.4.3-aarch64-apple-darwin.tar.gz
install -m 0755 ctxhelm-v2.4.3-aarch64-apple-darwin/ctxhelm ~/.local/bin/ctxhelm
ctxhelm --version
ctxhelm --help
ctxhelm doctor --binary "$(command -v ctxhelm)" --release-manifest ctxhelm-v2.4.3-aarch64-apple-darwin.manifest.json
```

The expected diagnostic is:

```text
ctxhelm 2.4.3
```

## Source Build Fallbacks

Build from the tagged repository with locked dependencies:

```bash
cargo install --git https://github.com/thromel/ctxhelm --tag v2.4.3 ctxhelm --locked
ctxhelm --version
ctxhelm --help
```

Build from a local checkout:

```bash
cargo install --path crates/ctxhelm --locked
cargo build -p ctxhelm --release --locked
target/release/ctxhelm --version
target/release/ctxhelm --help
```

## Maintainer Packaging

From a clean checkout at the v2.4.3 tag, run:

```bash
bash scripts/release-package.sh
```

The script builds the host target by default with:

```bash
cargo build -p ctxhelm --release --locked --target "$(rustc -vV | awk '/^host:/ { print $2 }')"
```

It writes release artifacts under `dist/` by default, or under `CTXHELM_DIST_DIR` when that environment variable is set:

```text
dist/ctxhelm-v2.4.3-{target}.tar.gz
dist/ctxhelm-v2.4.3-{target}.manifest.json
dist/ctxhelm-v2.4.3-{target}.audit.json
dist/ctxhelm-v2.4.3-{target}.tar.gz.sha256
dist/sha256sums.txt
```

The package script stages only the `ctxhelm` binary, `README.md`, `LICENSE`, and `VERSION`, then extracts the archive in a temporary directory and verifies:

```bash
ctxhelm --version
ctxhelm --help
```

The release manifest records the version, target label, archive checksum, binary checksum, included files, local-only privacy status, unsupported publish actions, and the matching artifact audit report. `sha256sums.txt` covers the archive, manifest, and audit report.

Maintainers can set `CARGO_TARGET_DIR=/absolute/path/to/target` when they need a clean build cache for packaging or release-gate verification. They can also set `CTXHELM_BUILD_TARGET=<rust-target-triple>` and, when needed, `CTXHELM_TARGET_LABEL=<archive-label>` to build and package an explicit target. The packaged binary is copied from `target/<target>/release/ctxhelm`, so a target label is no longer a cosmetic archive-name override.

The public repository includes a release-artifact workflow:

```text
.github/workflows/release-artifacts.yml
```

It runs on manual dispatch and version-tag pushes, builds `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, and `aarch64-apple-darwin`, verifies every archive with `scripts/verify-release-archive.sh`, and uses `actions/upload-artifact@v6` for workflow artifacts. On version-tag pushes it also creates or updates the matching GitHub release and uploads the verified archive, manifest, audit report, and per-archive checksum for each target. It does not create tags, Homebrew commits, crates.io packages, signed installers, or self-update metadata.

Maintainers can verify a built archive from a clean extraction directory:

```bash
bash scripts/verify-release-archive.sh \
  --archive dist/ctxhelm-v2.4.3-aarch64-apple-darwin.tar.gz \
  --manifest dist/ctxhelm-v2.4.3-aarch64-apple-darwin.manifest.json \
  --checksums dist/sha256sums.txt
```

Before using a large-history repository in product-proof or benchmark evidence,
prepare it as a clean detached fixture instead of relying on an ambient sibling
checkout:

```bash
bash scripts/prepare-benchmark-corpus.sh \
  --source https://github.com/tsantalis/RefactoringMiner.git \
  --revision e319af8d6b51d821b61d2f735ad211631775adfb \
  --worktree ../ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --min-commits 20 \
  --output .ctxhelm/e2e/refactoringminer-corpus-health.json \
  --refresh
```

The resulting report is source-free and records readiness, revision identity,
commit count, dirty-file count, object-store connectivity, history usability,
and privacy metadata. A `blocked` report must be treated as an environment or
fixture problem, not as product-quality proof.

Maintainers can verify the public archive install path end to end without
installing globally:

```bash
bash scripts/verify-public-archive-install.sh \
  --repo thromel/ctxhelm \
  --tag v2.4.3 \
  --target-label aarch64-apple-darwin \
  --expected-version "ctxhelm 2.4.3"
```

The public install verifier downloads the release archive, manifest, audit
report, and checksum files from GitHub, verifies checksums, extracts the archive,
installs `ctxhelm` into a temporary bin directory, and runs `ctxhelm --version`,
`ctxhelm --help`, `ctxhelm doctor`, and the first-pack smoke against that
temporary binary.

Maintainers can also verify optional real-client behavior against the public
archive binary:

```bash
bash scripts/smoke-public-real-clients.sh \
  --repo thromel/ctxhelm \
  --tag v2.4.3 \
  --target-label aarch64-apple-darwin \
  --expected-version "ctxhelm 2.4.3" \
  --output .ctxhelm/e2e/phase130-public-real-client-smoke.json
```

This check downloads and verifies the same public release assets, then runs the
Codex and Claude Code real-client wrappers with the extracted binary. Passing
clients write source-free `prepare_task`/`get_pack` request evidence. Skipped or
unavailable clients write source-free skip evidence instead unless
`CTXHELM_REQUIRE_REAL_CLIENT=1` makes them required. Cursor and OpenCode have
separate local wrappers, `scripts/smoke-cursor-real-client.sh` and
`scripts/smoke-opencode-real-client.sh`, for client-specific proof.

Maintainers can verify the published Homebrew tap path on Apple Silicon macOS:

```bash
bash scripts/verify-homebrew-tap.sh \
  --tap thromel/tap \
  --formula ctxhelm \
  --expected-version "ctxhelm 2.4.3" \
  --expected-url https://github.com/thromel/ctxhelm/releases/download/v2.4.3/ctxhelm-v2.4.3-aarch64-apple-darwin.tar.gz \
  --expected-sha256 <sha256-from-release-asset>
```

This check taps `thromel/tap`, audits the formula, installs `ctxhelm` through
Homebrew, runs `brew test`, and verifies `ctxhelm --version`.

For public archives whose released MCP protocol predates current resource-scope
assertions, the script can run the protocol smoke in
release-compatible mode by setting `CTXHELM_REQUIRE_RESOURCE_SCOPE=0`. Current
source-tree and release-candidate gates keep `CTXHELM_REQUIRE_RESOURCE_SCOPE=1`
by default, so newer `resourceScope` MCP assertions remain enforced for current
builds while older public archives stay verifiable.

During multi-plan local work, maintainers can set `CTXHELM_ALLOW_DIRTY=1` for verification, but release artifacts should be produced from a clean checkout.

## Release Gate

Before publishing or announcing a release, run the local release gate:

```bash
bash scripts/release-gate.sh
```

The public repository also runs `.github/workflows/ci.yml` on pushes, pull
requests, and manual dispatch. That workflow uses the Node 24 major versions
`actions/checkout@v5` and `actions/cache@v5`, opts into
`FORCE_JAVASCRIPT_ACTIONS_TO_NODE24`, then enforces formatting,
`cargo clippy --workspace --all-targets --locked -- -D warnings`, locked
workspace tests, CLI help, release-doc consistency, and the local release gate
with external fixture and real-client checks explicitly skipped unless a
maintainer runs the full local gate with those optional proofs enabled.
The Actions cache is intentionally limited to Cargo registry/git metadata and
does not restore `target/`; prior target-output caches grew large enough to
exhaust hosted-runner disk before validation could start.

This is the pre-publication blocker for v2.4.3. When `CTXHELM_BIN` is not set, the gate runs `scripts/release-package.sh`, audits the archive, extracts the generated artifact, and uses the extracted `ctxhelm` binary for installed-binary proof.

To prove a selected installed or previously extracted binary, pass an absolute path:

```bash
CTXHELM_BIN=/absolute/path/to/ctxhelm bash scripts/release-gate.sh
```

The release gate preserves the packaging script's clean-checkout guard by default. During multi-plan local verification, set `CTXHELM_ALLOW_DIRTY=1` explicitly; release artifacts intended for publication should be built from a clean checkout.

The release gate runs these required checks:

- `cargo test --workspace`
- `scripts/check-release-docs.sh`
- `scripts/release-package.sh`, including `scripts/audit-release-artifact.sh`
- `scripts/verify-release-archive.sh` clean extraction verification
- selected or extracted binary `ctxhelm --version`
- selected or extracted binary `ctxhelm --help`
- `scripts/smoke-first-pack.sh`
- `scripts/smoke-storage.sh`
- `scripts/smoke-memory.sh`
- `scripts/smoke-memory-reuse.sh`
- `scripts/smoke-memory-history-lift.sh`
- `scripts/smoke-memory-parent-snapshot-lift.sh`
- `scripts/smoke-memory-benchmark-lift.sh`
- `scripts/smoke-feedback.sh`
- `scripts/smoke-governor.sh`
- `scripts/smoke-workspace.sh`
- `scripts/smoke-shared-artifacts.sh`
- `scripts/smoke-inspector.sh`
- `scripts/smoke-retrieval-health.sh`
- `scripts/smoke-graph.sh`
- `scripts/smoke-policy-embedding.sh`
- `scripts/smoke-agent-preview.sh`
- `scripts/smoke-agent-native-fallback.sh`
- `scripts/smoke-demo-artifacts.sh`
- `scripts/smoke-distribution-metadata.sh`
- `scripts/render-homebrew-formula.sh`
- `scripts/smoke-release-governance.sh`
- `scripts/smoke-semantic.sh`
- `scripts/smoke-precision.sh`
- `scripts/smoke-v23-eval.sh`
- `scripts/smoke-mcp-protocol.sh` from a wrong cwd with an explicit `--repo`/MCP `repo` argument
- optional `ctxhelm eval proof` benchmark product proof when `CTXHELM_BENCHMARK_CONFIG` is set
- clean cold fixture product proof when the Phase 110 detached fixtures are present
- optional `scripts/smoke-cursor-real-client.sh` when Cursor Agent CLI is logged in
- optional `scripts/smoke-opencode-real-client.sh` when OpenCode has usable provider credentials

For broad workflow/eval/lint tasks, `prepare-task` and generated packs include
`contextAreas`. This field is source-free and additive: it gives agents
area-level inspection hints, representative paths, concrete `nextReadPaths`,
and `unselectedCount` while preserving the top target-file and validation
channels used by release proof metrics. Docs candidates are included in this
progressive guidance channel.

Archive/docs retrieval tasks can also be classified as broad for source-free
context-area guidance. Those tasks do not automatically spend target-file budget
on broad source floors unless they also match the stricter implementation/eval
floor gate.

After all required checks pass, the gate writes a source-free proof bundle summary. By default it lives in the gate's temporary workspace; pass `CTXHELM_PROOF_DIR=/absolute/path/to/proof` to persist it:

```bash
CTXHELM_PROOF_DIR=/absolute/path/to/proof bash scripts/release-gate.sh
```

The proof summary records the checked `ctxhelm` version, binary SHA-256, archive SHA-256, manifest name, audit report name, required check outcomes, optional benchmark/client proof status, optional agent-run outcome proof status, resource-backed gap-summary contract status, and privacy status. It records file names and checksums instead of machine-local binary or repository paths.

The clean cold fixture proof is the production retrieval-quality release check
for the four-repo corpus. Prepare the detached fixtures once:

```bash
bash scripts/prepare-proof-fixtures.sh
```

The fixture prep script writes one source-free readiness report per fixture,
for example `VeriSchema.fixture-status.json`. Those reports record requested
revision, checked-out head, revision availability, dirty count, and privacy
status; they omit source snippets, commit subjects, diffs, terminal logs,
prompts, and remote URLs. If a pinned revision is no longer reachable, fixture
prep fails before checkout with `revisionAvailable: false`.

Then run the release gate normally. The gate uses
`.planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json` by default,
writes `phase183-clean-fixture-product-proof.json` into `CTXHELM_PROOF_DIR`,
and validates it with `scripts/check-product-proof.py`. Before running the proof,
the gate verifies every configured fixture exists, contains the requested commit,
and has `HEAD` checked out at the configured `head`. Missing, stale, or
unreachable fixtures are treated as unavailable proof evidence. If the fixtures
are not available, the gate records `cleanColdFixtureProductProof:
skipped_missing_fixtures`; set `CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF=1` to make
missing or stale fixtures fail the gate. Maintainers may override the config
with `CTXHELM_CLEAN_FIXTURE_CONFIG=/absolute/path/to/config.json` or skip the
check explicitly with `CTXHELM_SKIP_CLEAN_FIXTURE_PROOF=1` for non-release local
diagnostics.

Agent-run outcome proof is optional because it validates a saved real-client
report instead of running Codex, Claude, Cursor, or OpenCode during the release
gate. To enforce the current Codex breadth-suite claim, pass a persisted
source-free suite report:

```bash
CTXHELM_AGENT_RUN_PROOF_REPORT="$(pwd)/.ctxhelm/e2e/phase322-agent-run-codex-target-first-breadth-suite.json" \
  CTXHELM_REQUIRE_AGENT_RUN_PROOF=1 \
  bash scripts/release-gate.sh
```

The gate validates the report with `scripts/check-agent-run-proof.py`, writes a
source-free `agent-run-outcome-proof.json` audit artifact into
`CTXHELM_PROOF_DIR`, and records `agentRunOutcomeProof`,
`agentRunOutcomeProofRequired`, and `agentRunOutcomeProofReport` in
`release-proof-summary.json`. The audit artifact uses
`ctxhelm-agent-run-proof-check-v1` and records the saved report filename,
report SHA-256, thresholds, factual `privacyStatus`, source-free
`privacyChecks`, identity checks, runner
metadata, current runner script freshness, current suite freshness, aggregate
metrics, aggregate consistency checks, suite-task checks, suite status checks,
task-lane checks, factual `boundaryStatus`, boundary checks, and lane quality
summaries. `privacyStatus` mirrors the original report booleans, such as
`sourceTextLogged: false`, while `privacyChecks.strictFalseFields` records
which source-free checks passed. `boundaryStatus` mirrors observed boundary
booleans, such as `clientFailuresObserved: false`, while `boundaryChecks`
records strict-false pass/fail results for the same fields. The checker also
derives `suite.taskCount` from `tasks[*]` and derives `report.status` from
nested task statuses, comparison eligibility, and strict boundary flags, so a
stale suite envelope cannot support a release claim. When `--current-suite` is provided,
the checker re-parses the current suite and validates every saved
`tasks[*].taskSha256` plus `tasks[*].targetFiles` against the current task text
and target list. It also validates every task comparison and task lane, so a
saved report fails if a nested ctxhelm lane contains a client failure, rate
limit, forbidden command, missing required ctxhelm call, under-read target,
evidence miss, evidence-only target, or target-read coverage below the release
floor even when the aggregate summary still looks clean. It derives aggregate
task counts, comparison-eligible counts, comparable ctxhelm lane counts,
strict boundary booleans, and lane summary names from `tasks[*]`, so a report
also fails when top-level aggregate fields drift from derived task comparisons.
It also derives lane-summary metrics from `tasks[*].lanes[*].metrics`,
including read counts, target-read coverage, read precision, irrelevant-read
rate, read-role counts, missed-target role counts, required-call counts, and
failure/rate-limit counters. That prevents stale `aggregate.laneSummaries`
metrics from supporting release claims after nested task records change.
Finally, it derives aggregate retry-cost metrics from per-task retry costs and
read-efficiency metrics from lane summaries, so stale `aggregate.retryCost` or
`aggregate.readEfficiency` values cannot support claims about retry overhead,
recovered target reads, or read precision.
It also derives top-level comparison aggregates such as
`targetReadCoverageDeltaAverage`, `targetCoverageDeltaAverage`,
`readFileDeltaSum`, `irrelevantReadDeltaSum`, `commandExecutionDeltaSum`, and
`ctxhelmToolCallsObserved` from `tasks[*].comparison`.
The JSON audit summary uses the same validated aggregate field names in
`metrics.targetReadCoverageDeltaAverage`,
`metrics.targetCoverageDeltaAverage`, `metrics.commandExecutionDeltaSum`, and
`metrics.ctxhelmToolCallsObserved`; it does not publish stale shortened aliases
or omit validated comparison aggregate fields.
For a single-run workflow, the JSON audit summary also publishes
`metrics.commandExecutionDelta` and `metrics.ctxhelmToolCallsObserved`, so a
standalone accepted report exposes command-cost and ctxhelm-call observation
without requiring suite JSON archaeology.
It also enforces suite status checks by deriving `suite.taskCount` and
`report.status` from the nested task records and boundary state.
The checker derives `aggregate.outcomeClaim` and
`aggregate.recommendedResearchActions` from those source-free comparison
aggregates and boundary flags, so stale outcome labels or R&D routing actions
cannot support release claims.
The release gate passes
`--expected-ctxhelm-version` from the selected release-gate binary,
`--expected-client-name codex`, and `--expected-client-version "codex-cli
0.137.0"` by default, so a saved report fails if it no longer matches the
versioned Codex proof claim. Maintainers can override the expected client
identity with `CTXHELM_AGENT_RUN_EXPECTED_CLIENT_NAME` and
`CTXHELM_AGENT_RUN_EXPECTED_CLIENT_VERSION` when refreshing proof for a newer
client. The release gate also passes `--current-runner-script
scripts/e2e-agent-run-codex.sh`, so a saved report fails if its
`runner.scriptSha256` no longer matches the current Codex runner script. It also
passes `--current-suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json`,
so a saved report fails if `suite.suiteSha256` no longer matches the current
four-task R&D suite. The default thresholds require a suite report with
`ctxhelm_improved`, the selected ctxhelm version, Codex `codex-cli 0.137.0`, at
least four tasks, four comparison-eligible tasks, sixteen comparable ctxhelm
lanes, `1.0` minimum average target-read coverage in each ctxhelm lane,
retry-cost fields, no runner-fingerprint gap, no more than two extra reads
across best lanes, and no client failures, rate limits, forbidden commands,
evidence misses, evidence-only targets, or under-read targets.
Maintainers can adjust the thresholds with
`CTXHELM_AGENT_RUN_MIN_TASK_COUNT`,
`CTXHELM_AGENT_RUN_MIN_COMPARISON_ELIGIBLE`,
`CTXHELM_AGENT_RUN_MIN_COMPARABLE_CTXHELM_LANES`,
`CTXHELM_AGENT_RUN_MIN_TARGET_READ_COVERAGE`,
`CTXHELM_AGENT_RUN_MAX_EXTRA_READ_DELTA`, and
`CTXHELM_AGENT_RUN_MIN_IRRELEVANT_READ_DELTA`.

Phase 183 refreshes the default clean fixture config after the old pinned
VeriSchema object became unreachable. The refreshed four-repo proof keeps the
standard `5000ms` per-commit runtime ceiling for ctxhelm, ReAgent, and
VeriSchema, and uses an explicit source-free `proofRuntimeCeilingMillis: 15000`
only for the detached RefactoringMiner fixture. Product proof reports the
effective ceiling in each repository's `effectiveConfig` and records a verdict
note when a repo-scoped ceiling is used, so a release cannot silently hide a
global runtime-threshold change. The cold Phase 183 proof promotes with
RefactoringMiner as a context-channel lexical-ceiling `match` and ctxhelm,
ReAgent, and VeriSchema as `beat`.

Phase 184 adds source-free `signalCounts` to task-conditioned context areas and
renders them in context packs as `Signals:`. This is release-relevant because it
keeps the protected top-10 ranking stable while making broad progressive reads
auditable: an agent can see whether an area was surfaced by lexical,
dependency, co-change, semantic, test, docs, config, anchor, history,
current-diff, or memory signals before deciding which native file reads to do
next. The accepted change followed a rejected dependency-reserve widening
experiment that produced no recall lift in the clean four-repo proof.

Phase 185 carries those source-free area profiles into retrieval-gap summaries.
When a missed target belongs to a surfaced task-conditioned area, the gap now
reports the area's signal counts, role counts, selected-role counts, and
unselected count. This keeps product proof diagnostics actionable while leaving
ranking, budgets, and release thresholds unchanged.

Phase 186 deduplicates those area-profile counts for grouped gap summaries:
multiple missed files can still raise `missedCount`, `examplePaths`, and
`nextReadPaths`, but the same task-conditioned context-area profile is merged
only once. This keeps release proof diagnostics from overstating source-free
area pressure when a grouped gap contains several files from the same area.

Phase 187 adds a bounded source-history reserve for source files whose
co-change evidence is corroborated by dependency, lexical, lexical-expansion,
or symbol signals. The fresh four-repo release proof still promotes and improves
source recall for ctxhelm (`0.42857143 -> 0.5`) and VeriSchema
(`0.32258064 -> 0.38709676`). The proof also records a ctxhelm all-file
Recall@10 tradeoff (`0.6666667 -> 0.5587302`) because broad docs lose some
top-10 budget; this is accepted as a source-channel R&D improvement, not as a
blanket all-file recall win.

Phase 188 adds source-free selected-signal profiles to historical commit eval
reports. Each commit records selected top-10 counts by signal and role, plus how
many selected paths were retrieval targets. The fresh four-repo proof still
promotes, all 16 evaluated commits include non-empty profiles, and recall
metrics stay unchanged from Phase 187. Release reviewers can use this profile to
explain ranking-budget allocation before accepting future source/doc tradeoffs.

Phase 189 uses those selected-signal profiles to rebalance broad governance docs
against source-history reserve. Broad root governance docs now run before broad
source-history, while source-history remains protected for standard scope and for
broad tasks without governance pressure. Source-history candidates also prefer
module entrypoints such as `src/lib.rs`. The fresh four-repo proof promotes;
ctxhelm File Recall@10 improves (`0.5587302 -> 0.67777777`) while ctxhelm Source
Recall@10 stays `0.55`, and the other measured repositories stay unchanged.

Phase 190 adds source-free context-area coverage and inspection-pressure fields
to prepared plans and packs. `coveragePercent` reports selected paths divided by
candidate paths, while `inspectionPressure` weights under-read
source/config/schema paths ahead of tests and docs. Pack guidance now orders
zero-selected areas by pressure so agents can prioritize progressive native
reads without changing target-file budget. The fresh four-repo proof promotes
with Phase 189 file/source/test/context-area metrics unchanged.

Phase 191 makes context-area pressure explainable. Prepared plans now include
`inspectionPressureBreakdown`, packs render the source-like/validation/docs
pressure split, retrieval-gap summaries carry the same breakdown when a
task-conditioned area profile is available, and `scripts/check-product-proof.py`
validates emitted pressure totals. The fresh four-repo proof promotes with Phase
190 metrics unchanged and validates 144 context-area entries with zero invalid
breakdowns.

Phase 192 aggregates context-area pressure into eval and product-proof reports.
`HistoricalEvalReport.contextAreaPressureSummary` reports total areas,
zero-selected areas, source-like/validation/docs pressure, and the highest
pressure area. Historical eval and benchmark markdown render the summary, and
`scripts/check-product-proof.py` validates emitted summary totals. The fresh
four-repo proof promotes with Phase 191 metrics unchanged while showing
repository-level broad-area bottlenecks.

Phase 193 measures whether progressive context-area guidance recovers files
missed by top-10 context. `HistoricalEvalReport.contextAreaNextReadSummary`
reports missed@10 count, next-read recoverable count, top-pressure recoverable
count, and zero-selected-area recoverable count. Historical eval and benchmark
markdown render the summary, and `scripts/check-product-proof.py` validates the
recovery arithmetic. The fresh four-repo proof promotes with Phase 192 metrics
unchanged while showing `9 / 12` ctxhelm missed@10 paths and `10 / 39`
VeriSchema missed@10 paths recoverable via source-free next-read guidance.

Phase 194 makes context-area next-read ordering source-free and signal-aware.
Unselected area candidates now sort by role priority, signal priority, weighted
signal score, confidence, and stable tie-breakers before `nextReadPaths` are
truncated. The fresh four-repo proof promotes with Phase 193 metrics unchanged
while next-read recovery improves to `10 / 12` on ctxhelm and `14 / 39` on
VeriSchema.

Phase 195 makes context-area next-read budgets adaptive. Low-pressure areas
still expose four `nextReadPaths`, while high-pressure source-like or
validation-heavy areas can expose six or eight bounded progressive reads. The
fresh four-repo proof promotes with selected-file/source/test/validation and
broad-area metrics unchanged while next-read recovery improves to `11 / 12` on
ctxhelm and `16 / 39` on VeriSchema.

Phase 196 reserves selected validation areas in broad context-area guidance.
Package-mirrored related-test affinity now maps source directories such as
`schema_agent/agents` to `tests/agents` and JVM main roots to their test roots.
The accepted four-repo proof promotes with selected-file/source/test/validation
metrics unchanged while VeriSchema broad context-area recall improves to
`0.84444445` and next-read recovery improves to `19 / 39`.

Phase 197 adds agent-evidence recovery accounting to context-area next-read
summaries. `nextReadRecoverableCount` continues to mean progressive
context-area reads only; `agentEvidenceRecoverableCount` also counts selected
related tests and selected context files. The fresh four-repo proof promotes
with retrieval metrics unchanged while VeriSchema reports `29 / 39` missed@10
files recoverable through the full agent evidence bundle.

Phase 198 adds source-free candidate coverage accounting for selected-file
misses. `candidateCoverageSummary` reports missed@10 paths, how many of those
paths were present in the generated candidate set, and how many had no candidate
signal. The fresh four-repo proof promotes with Phase 197 retrieval and lexical
comparison metrics unchanged while showing ctxhelm `11 / 12`, RefactoringMiner
`1 / 1`, ReAgent `0 / 0`, and VeriSchema `36 / 39` missed@10 files are already
candidate-recoverable. That makes the next retrieval-quality bottleneck
selection/ranking pressure, especially in VeriSchema, rather than broad missing
candidate generation.

Phase 199 adds source-free candidate miss pressure profiles. The same summary
now includes recoverable role counts, recoverable signal counts, no-candidate
role counts, and top recoverable context areas. The fresh four-repo proof
promotes with Phase 198 metrics unchanged while showing VeriSchema pressure
concentrated in `schema_agent/agents=7`, `tests/agents=6`, and
`tests/evaluation=6`, led by `co_change=17`, `related_test=17`, and
`dependency=12` recoverable signals.

Phase 200 adds a bounded contextual README doc reserve for broad tasks. The
fresh four-repo proof promotes and improves average File Recall@10 from
`0.658268` to `0.7082679`, average file delta versus lexical from
`+0.17873949` to `+0.22873944`, Agent Evidence Recall@10 from `0.76052284` to
`0.81052285`, Context Recall@10 from `0.7638889` to `0.7708334`, and VeriSchema
File Recall@10 from `0.35529414` to `0.55529416` without source/test/effective-
validation/broad-area regression. A source-dependency-before-workflow ordering
experiment was rejected because it displaced a true workflow-script hit and
regressed VeriSchema File Recall@10.

Phase 201 adds source-free agent-evidence-only gap profiles to
`contextAreaNextReadSummary`. The fresh four-repo proof promotes with Phase 200
retrieval metrics unchanged while showing VeriSchema has `10` missed@10 files
recoverable through agent evidence but not progressive next reads. All `10` are
tests, concentrated in `tests/agents=5`, `tests/evaluation=4`, and
`tests/core=1`; RefactoringMiner has `1` agent-evidence-only test gap in
`src/test/java/org/refactoringminer/mcp`. A broad agent-source reserve,
role-aware related-test next-read priority, and larger high-pressure next-read
cap were measured and rejected because they produced no recovery movement.

Phase 202 renders those source-free profiles in historical eval markdown
reports. The report now includes agent-evidence-only recovery count, role
counts, and top areas, so validation-only residual gaps are visible during
normal proof review without opening raw JSON. Retrieval metrics remain governed
by the Phase 201 proof.

The optional real-client evidence wrappers are:

- `scripts/smoke-codex-mcp.sh`
- `scripts/smoke-claude-mcp.sh`
- `scripts/smoke-public-real-clients.sh`
- `scripts/e2e-claude-workflow.sh`

`scripts/e2e-claude-workflow.sh` is the deeper Claude Code integration eval.
It wraps the Claude smoke, then writes a source-free workflow report with
request-log hashes, sanitized observed tool calls, explicit-repo call counts,
and privacy flags. The report stores no raw prompt, raw MCP traffic, source
text, or user-project command output. In the release gate this is optional by
default; set `CTXHELM_RUN_CLAUDE_WORKFLOW_EVAL=1` to record it, or
`CTXHELM_REQUIRE_CLAUDE_WORKFLOW_EVAL=1` to make a missing/passing real Claude
workflow eval block the release gate.

Cursor and OpenCode have deterministic setup/protocol proof wrappers:

- `scripts/smoke-cursor-mcp.sh`
- `scripts/smoke-opencode-mcp.sh`

These wrappers validate repo-local setup artifacts and MCP protocol behavior,
and their evidence explicitly marks `realClientToolCalls: false`.

The semantic smoke proves explicit local semantic retrieval, source-free vector metadata, semantic provenance in plans, semantic-enabled eval metadata, scaffold/provider status for `local_hash`, and cloud-disabled privacy status. It does not call cloud embedding or reranking services. The optional `local_fastembed` backend remains behind the `local-embeddings` Cargo feature and is not a default release requirement.

The memory benchmark-lift smoke proves experience-memory lift is visible through
the same benchmark/product-proof aggregation path used by release evidence. It
creates two local repos, approves one source-free experience card per repo, runs
`eval proof`, and requires each embedded historical report to contain a
memory-only target hit beyond lexical plus product-level
`evaluate_memory_reuse_lift` routing. It does not claim broad memory
generalization across arbitrary repositories.

The memory parent-snapshot lift smoke proves approved source-repo memory remains
visible when historical eval builds a parent-revision snapshot with a different
local storage identity. It runs a two-commit local repo through `eval history`
before and after approval, requiring a memory-only target hit beyond lexical
without persisting source sentinels or raw task text.

The precision smoke proves Java/Kotlin symbol extraction, Java/Kotlin package import edges, source-free precision edge import, rejection of sensitive paths, and additive precision dependency output.

The v2.3 eval smoke proves the fixed-corpus proof path without external
repositories. It creates a small local git corpus and exercises source-free
candidate feature export, feedback recording, offline learned-policy proposal,
paired baseline verdicts, runtime diagnostics, and product proof fields for
fixed corpus identity, feature-export privacy, learned-policy status, and proof
boundary language.

The v2.4 gate smoke proves semantic/precision release-gate reporting without
external repositories. It creates a small local git corpus and exercises
promote/hold/block decisions, lexical/default/semantic/precision/full-hybrid
variant rows, policy-blocked reranker skips, provider policy, precision status,
named-case arrays, and local-only privacy metadata.

```bash
bash scripts/smoke-v24-gate.sh
```

The feedback smoke proves source-free feedback ingestion, policy report generation, candidate policy tuning, apply/rollback metadata, and budget outcome comparison.

The workspace smoke proves local multi-repo manifest initialization, source-free workspace status JSON, workspace prepare-task routing, repo-boundary-aware workspace packs, missing source sentinel leakage, and single-repo command compatibility without an explicit workspace manifest.

The shared-artifacts smoke proves source-free team policy templates, shared artifact export, schema/privacy inspection, compatible manifest import, MCP workspace resources for status and shared artifacts, and absence of sensitive sentinel leakage in outputs and local ctxhelm state.

The inspector smoke proves source-free JSON and static HTML inspector exports, source-free proof summaries for saved agent-run, product-proof, and multi-report proof bundle reports, a localhost-only read-only inspector shell, local filter UI hooks, source-bearing section labels, graph/setup/health diagnostic routes, and absence of source sentinel leakage in inspector artifacts.

The retrieval-health smoke proves source-free health JSON and Markdown reports from real git history, including metrics, signal contributions, gap families, and absence of source sentinel leakage.

The graph smoke proves source-free graph neighborhood JSON and Markdown reports from real dependency/test edges, including nodes, edges, communities, cap-safe metadata, and absence of source sentinel leakage.

The policy/embedding smoke proves semantic provider status reporting, `deterministic_scaffold` labeling for `local_hash`, explicit provider-policy decisions, disabled default reranking, explicit cloud-disabled/source-transfer-denied flags, source-free policy experiment rows, graph comparison metadata, and absence of source sentinel leakage.

The agent-preview smoke proves Codex, Claude Code, Cursor, OpenCode, and generic MCP preview metadata, including MCP tools/resources, guidance paths, read/edit boundary notes, source-free flags, and absence of source sentinel leakage.

The demo and distribution metadata smokes prove public source-free examples,
package-manager preparation templates, Homebrew formula renderability from the
exact archive digest when `CTXHELM_DIST_DIR` is available, crates package boundary checks,
update metadata, clean extraction verification syntax, explicit signing and notarization gaps,
and package-manager publication boundaries. They do not publish package-manager artifacts
and are not a self-update implementation.

The release governance smoke proves source-free candidate status metadata,
ready/deferred/blocked lifecycle states, deterministic protocol proof language,
optional real-client proof boundaries, Cursor/OpenCode source-free proof boundaries, and rollback
safety for marked local candidate directories.

The gate passes the same selected or extracted `CTXHELM_BIN` into the first-pack, storage, memory, feedback, workspace, shared-artifact, inspector, retrieval-health, graph, policy/embedding, agent-preview, semantic, precision, v2.3 eval, v2.4 semantic/precision gate, MCP protocol, Cursor/OpenCode setup-proof wrappers, and optional real-client smokes. It also passes `CTXHELM_DIST_DIR` and a temporary `CTXHELM_DISTRIBUTION_METADATA_OUT` to the distribution metadata smoke so Homebrew formula renderability is checked against the exact packaged archive digest without rewriting the tracked `.ctxhelm/distribution-metadata-smoke.json` artifact. Demo, distribution metadata, and release governance smokes are source-free metadata checks and do not need the binary. Real-client proof is not required by default. Use these environment variables when needed:

- `CTXHELM_SKIP_REAL_CLIENT=1` keeps Codex and Claude checks deterministic-only after the protocol proof.
- `CTXHELM_REQUIRE_REAL_CLIENT=1` makes missing Codex or Claude tool-call evidence fail the gate.
- `CTXHELM_UPDATE_DISTRIBUTION_METADATA=1` refreshes the tracked `.ctxhelm/distribution-metadata-smoke.json` artifact when running `scripts/smoke-distribution-metadata.sh` directly. Without it, the smoke writes generated metadata to a temporary path or to `CTXHELM_DISTRIBUTION_METADATA_OUT` when that override is set.
- `CTXHELM_SKIP_WORKTREE_CLEAN_CHECK=1` disables the release gate's post-smoke worktree cleanliness check for unusual local diagnostics. Do not use it for release proof.
- `CTXHELM_RUN_CURSOR_REAL_CLIENT=1` or `CTXHELM_REQUIRE_CURSOR_REAL_CLIENT=1` runs or requires Cursor Agent CLI request evidence.
- `CTXHELM_RUN_OPENCODE_REAL_CLIENT=1` or `CTXHELM_REQUIRE_OPENCODE_REAL_CLIENT=1` runs or requires OpenCode request evidence.
- `CTXHELM_REAL_CLIENT_EVIDENCE_DIR=/absolute/path/to/evidence` writes stable JSON evidence files with client version, ctxhelm version, repo path, `prepare_task`, and `get_pack` proof when real-client checks run. These files also include a source-free request-log hash, request line count, MCP request `methodCounts`, explicit repo tool-call count, sanitized observed tool-call metadata, and a separate sanitized request-summary JSON. If Codex is skipped or fails in optional mode, the wrapper writes `status: skipped` plus `clientFailureKind`, `clientExitStatus`, `stderrSha256`, and `stderrLineCount` diagnostics so client stream-disconnects are not confused with ctxhelm protocol failures. Raw request logs, raw stderr, prompts, task text, and source snippets are not persisted by the wrapper.
- `CTXHELM_BENCHMARK_CONFIG=/absolute/path/to/suite.json` runs `ctxhelm eval proof --config ... --format json` and fails on report-generation, local-only privacy regressions, missing embedded repository reports, history-unavailable insufficient-evidence reports, missing v2.3 product proof summary, missing paired baseline verdict contract, feature-export privacy regressions, learned-policy status regressions, missing proof-boundary language, missing resource-backed current-reachable gap summaries, pinned broad fixed-corpus regressions, or a non-promote `releaseGate.decision`. Neutral, mixed, unsafe, or too-expensive default retrieval proof blocks publication.

Current v2.5 proof status: the fixed two-repo production-retrieval proof
promotes default local retrieval under the channel-aware release gate. The gate
compares non-test context recall against lexical retrieval and validates tests
through the dedicated `recommended_tests` channel. The current source-free proof
is `.ctxhelm/e2e/phase77-validation-command-coverage-proof.json`, where
`releaseGate.decision` is `promote`. RefactoringMiner context Recall@10 is
`0.7778` vs lexical `0.7407`; ctxhelm context Recall@10 is `0.4225` vs lexical
`0.3521`. RefactoringMiner Test Recall@10 and Effective Validation Recall@10
are both `1.0`; the ctxhelm required slice has no validation-test targets in
this refreshed proof. Phase 74 also separates
overall protected-evidence miss-rate from protected retrieval-target miss-rate:
RefactoringMiner target miss-rate@10 is `0.0588`, and ctxhelm target
miss-rate@10 is `0.0833`. Phase 77 adds broad validation fallback commands and
effective validation-command coverage for multi-area smoke/eval tasks. The
latest optional four-repo release proof in
`.ctxhelm/e2e/phase89-fast-inventory-freshness-release-proof.json` promotes
broader proof because RefactoringMiner is treated as a safe lexical-ceiling
match: ctxhelm and lexical both have context Recall@10 `1.0`, validation is
covered, and protected retrieval-target miss-rate is `0.0`. VeriSchema also
beats through Effective Validation Recall@10 `1.0` while raw Test Recall@10
remains `0.7090`.
For repeatable local investigation, use the pinned optional fixture at
`.planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json`; it is
expected to report `releaseGate.decision = promote` under the ceiling-aware
gate while still reporting protected target miss diagnostics for ctxhelm and
VeriSchema. Phase 79 adds bounded protected target floors and archive deferral:
the latest required proof is
`.ctxhelm/e2e/phase79-protected-target-floors-proof.json`, and the latest
broader proof is
`.ctxhelm/e2e/phase79-broader-protected-target-floors-proof.json`. Both
promote. Required RefactoringMiner and broader VeriSchema protected
retrieval-target miss-rates are `0.0`; ctxhelm still reports one residual
protected source-symbol miss in each proof.

Phase 80 fixes symbol-floor duplicate accounting. The current latest required
proof is `.ctxhelm/e2e/phase80-unique-symbol-floor-proof.json`, and the current
latest broader proof is
`.ctxhelm/e2e/phase80-broader-unique-symbol-floor-proof.json`. Both promote
with protected retrieval-target miss-rate `0.0` across measured corpora.

Phase 81 fixes warm-cache runtime reporting. The current cold/warm cache proof
uses `.planning/e2e/2026-05-30-phase81-warm-cache-proof-config.json` and
produces `.ctxhelm/e2e/phase81-warm-cache-cold-proof.json` plus
`.ctxhelm/e2e/phase81-warm-cache-warm-proof.json`. Both promote. The warm proof
records cache hits on every corpus, zero cache misses, zero commit-loop time,
and zero slow-commit diagnostics.

Phase 82 makes that warm-cache evidence enforceable. Product proofs now block
cached reports that mix cache misses, retain stale cold-run timings, retain
slow-commit diagnostics, or exceed `1000ms` warm lookup runtime. The latest
proof artifacts are `.ctxhelm/e2e/phase82-warm-cache-gate-cold-proof.json` and
`.ctxhelm/e2e/phase82-warm-cache-gate-warm-proof.json`; both promote.

Phase 83 makes context-vs-all-file divergence enforceable instead of prose-only.
Corpus verdicts now include `contextVsAllFileDeltaAt10`,
`lexicalContextVsAllFileDeltaAt10`, and `allFileDivergenceExplained`. The
release checker fails if those fields are missing or if raw all-file recall
trails lexical without an explanation from non-regressed context recall plus
covered validation targets. The latest broader proof artifact is
`.ctxhelm/e2e/phase83-context-divergence-proof.json`; it promotes with
RefactoringMiner and ReAgent all-file lexical deficits marked as explained by
the context/validation split.

Phase 84 adds broad-scope task accounting and scoped dependency source floors.
Historical eval JSON now includes report-level `broadScopeCommitCount` and
per-commit `broadScopeTask`; prepare-task emits `multi_area_task` diagnostics
for workflow/eval/lint prompts. Dependency source floors are enabled only for
those broad-scope prompts so ordinary lexical/doc targets are not displaced.
The latest broader proof artifact is
`.ctxhelm/e2e/phase84-broad-scope-dependency-proof.json`; it promotes and
improves VeriSchema Source Recall@10 from `0.249` to `0.304` while preserving
RefactoringMiner, ctxhelm, and ReAgent metrics.

Phase 85 adds source-free `contextAreas` to broad multi-area prepare-task plans
and packs. This gives agents area-level inspection hints without changing the
target-file, test, validation-command, or protected-evidence budgets. The
latest evidence artifact is
`.planning/e2e/2026-05-31-phase85-broad-context-areas.md`; the committed
warm-cache proof is `.ctxhelm/e2e/phase85-context-areas-warm-proof.json` and
promotes. Cold broad and required proofs kept quality metrics stable but still
hit the existing local RefactoringMiner runtime threshold.

Phase 86 adds bounded Python package re-export graph coverage. Dependency
extraction now resolves `package/__init__.py`, records imported submodule paths
from `from module import member`, and exposes `python_reexport` related edges
without enabling general depth-2 graph expansion. The evidence artifact is
`.planning/e2e/2026-05-31-phase86-python-package-reexports.md`; the broader
proof remains Recall@10-flat and non-regressing, so the next production gap is
selection/budget pressure rather than only missing Python package edges.

Phase 87 fixes validation gap accounting. Specific Java test selectors now count
as validation-command coverage, and validation-covered tests are no longer
reported as unresolved retrieval-gap summaries simply because they are absent
from context-file top 10. The evidence artifact is
`.planning/e2e/2026-05-31-phase87-validation-gap-accounting.md`; the committed
proof is `.ctxhelm/e2e/phase87-validation-gap-accounting-proof.json`.

Phase 88 adds broad source-area candidates after graph/test seed selection.
Broad multi-area tasks can now surface bounded same-root source-area inventory
candidates without expanding dependency or related-test seeds. The evidence
artifact is `.planning/e2e/2026-05-31-phase88-broad-source-area-candidates.md`;
the committed proof is
`.ctxhelm/e2e/phase88-broad-source-area-candidates-proof.json`.

Phase 89 reduces broad-proof runtime by making inventory cache-hit freshness
metadata-only instead of re-hashing all source files on every hit. The evidence
artifact is `.planning/e2e/2026-05-31-phase89-fast-inventory-freshness.md`;
the committed release proof is
`.ctxhelm/e2e/phase89-fast-inventory-freshness-release-proof.json` and reports
`releaseGate.decision = promote`.

Phase 90 proves the packaged release path from a clean worktree with the broad
benchmark enabled. The evidence artifact is
`.planning/e2e/2026-05-31-phase90-packaged-release-gate.md`. The gate packaged
and audited the archive, verified clean extraction, used the extracted
`ctxhelm 1.1.0` binary, passed required smokes and MCP protocol checks, and
passed the broad product proof. Optional Codex and Claude real-client tool-call
evidence was intentionally skipped with `CTXHELM_SKIP_REAL_CLIENT=1`.

Phase 91 adds broad context-area eval evidence for tasks too wide for a
top-10 file list. The evidence artifact is
`.planning/e2e/2026-05-31-phase91-broad-context-area-eval.md`; the committed
release proof is `.ctxhelm/e2e/phase91-broad-context-area-release-proof.json`
and reports `releaseGate.decision = promote`. VeriSchema broad-scope commits
now report `broadContextAreaRecall = 0.64708996` while existing file/source,
validation, protected-target, and runtime gates stay stable.

Phase 92 makes retrieval-gap taxonomy area-aware. Missed files inside surfaced
context areas now report `area_context_only` and map to the `contextPlanning`
recommendation area instead of false `no_candidate_signal` storage/candidate
failures. The historical eval cache schema was bumped so cached reports cannot
hide new broad-area or area-aware fields. The clean RefactoringMiner proof path
now uses a detached fixture worktree at
`/Users/romel/Documents/GitHub/RefactoringMiner-ctxhelm-proof`; the original
interactive checkout was dirty and is not used for committed evidence. The
evidence artifact is
`.planning/e2e/2026-05-31-phase92-area-aware-gap-taxonomy.md`. The committed
warm-cache proof is
`.ctxhelm/e2e/phase92-area-aware-gap-taxonomy-warm-proof.json` and reports
`releaseGate.decision = promote` with VeriSchema
`broadContextAreaRecall = 0.64708996`. The clean force-refresh proof is also
kept as `.ctxhelm/e2e/phase92-area-aware-gap-taxonomy-clean-force-proof.json`;
it preserves quality but documents that clean RefactoringMiner still exceeds
the hard cold runtime ceiling without cached historical reports.

Phase 93 adds source-free symbol extraction and dependency edge report caches
for repeated planner work during historical eval. The caches are keyed by the
inventory file path, hash, role, language, and cache-version marker, and are
stored under the ctxhelm cache path derived from the repo inventory rather than
the source tree. The evidence artifact is
`.planning/e2e/2026-05-31-phase93-source-free-index-cache.md`; the committed
proof is `.ctxhelm/e2e/phase93-index-cache-cold-proof.json` and reports
`releaseGate.decision = promote`. The clean detached RefactoringMiner
force-refresh row now runs in `4517ms` with context Recall@10 `1.0`, Test
Recall@10 `1.0`, Effective Validation Recall@10 `1.0`, and protected target
miss-rate `0.0`, removing the Phase 92 cold runtime blocker without release
threshold tuning.

Phase 94 increases broad `contextAreas` from 12 to 16 after proof rejected a
top-10 area-diverse selector that regressed VeriSchema file/source recall. The
evidence artifact is
`.planning/e2e/2026-05-31-phase94-broad-context-area-cap.md`; the committed
proof is `.ctxhelm/e2e/phase94-context-area-cap-proof.json` and reports
`releaseGate.decision = promote`. VeriSchema broad context-area recall improves
from `0.64708996` to `0.71851856`, while File Recall@10 remains
`0.18449473`, Source Recall@10 remains `0.31067252`, Test Recall@10 remains
`0.7089947`, Effective Validation Recall@10 remains `1.0`, and protected
target miss-rate remains `0.2857143`.

Phase 95 makes broad context-area packs progressive. The `Context areas`
section now explains that agents should use source-free area hints for native
reads and lists `Zero-selected areas to inspect next` with representative
paths. The evidence artifact is
`.planning/e2e/2026-05-31-phase95-progressive-area-pack-guidance.md`; the
committed proof is `.ctxhelm/e2e/phase95-progressive-area-pack-proof.json` and
reports `releaseGate.decision = promote`. VeriSchema broad context-area recall
remains `0.71851856`, and file/source/test/validation metrics remain stable.

Phase 96 makes broad context-area guidance MCP-readable. Each surfaced
`contextArea` now includes a source-free `resourceUri`, the static
`ctxhelm://repo/context-areas` resource lists safe area summaries, and dynamic
`ctxhelm://repo/context-area/{encoded-area}` resources return representative
paths and role counts without source text. The evidence artifact is
`.planning/e2e/2026-05-31-phase96-context-area-resources.md`; the committed
proof is `.ctxhelm/e2e/phase96-context-area-resources-proof.json` and reports
`releaseGate.decision = promote`. VeriSchema broad context-area recall remains
`0.71851856`, File Recall@10 remains `0.18449473`, Source Recall@10 remains
`0.31067252`, Test Recall@10 remains `0.7089947`, Effective Validation
Recall@10 remains `1.0`, and the six-tool MCP surface is unchanged.

Phase 97 improves broad governance/proof/eval task classification. Historical
commit tasks with wording such as "evaluate retrievable historical targets" and
"promote channel aware product proof" now receive governance planning docs and
broad source-area signals. The evidence artifact is
`.planning/e2e/2026-05-31-phase97-broad-governance-classification.md`; the
committed proof is
`.ctxhelm/e2e/phase97-broad-governance-classification-proof.json` and reports
`releaseGate.decision = promote`. ctxhelm File Recall@10 improves from
`0.44603175` to `0.47460318`, Source Recall@10 improves from `0.6333333` to
`0.7166667`, and broad context-area recall improves from `0.0` to `1.0`.
VeriSchema file/source/test/validation metrics remain stable.

Phase 98 splits source-free broad classification from target-file source-floor
spending. Archive/docs retrieval tasks now get context-area guidance, but they
do not spend target-file slots on broad source floors unless they match the
stricter implementation/eval gate. The evidence artifact is
`.planning/e2e/2026-05-31-phase98-progressive-broad-classification.md`; the
committed proof is
`.ctxhelm/e2e/phase98-broader-broad-task-classification-proof.json` and reports
`releaseGate.decision = promote`. ctxhelm File Recall@10 remains
`0.47460318`, Source Recall@10 remains `0.7166667`, and broad context-area
recall remains `1.0`.

Phase 99 deepens dynamic context-area resources. The
`ctxhelm://repo/context-area/{encoded-area}` resource now returns source-free
`roleBuckets`, `pathFamilies`, and `nextReadBatches` so agents can choose
primary, validation, and docs reads without loading source text through MCP. The
evidence artifact is
`.planning/e2e/2026-05-31-phase99-context-area-read-batches.md`; the committed
proof is `.ctxhelm/e2e/phase99-context-area-read-batches-proof.json` and
reports `releaseGate.decision = promote`. File/source/test/validation and broad
context-area metrics are unchanged from Phase 98 across the four-repo proof.

Phase 100 makes retrieval-gap summaries resource-backed. Historical eval gap
groups now include source-free `contextArea`, `contextAreaResourceUri`, and
`nextReadPaths`, turning remaining `area_context_only` and ranked-below-budget
misses into progressive read instructions. The evidence artifact is
`.planning/e2e/2026-05-31-phase100-resource-backed-gap-summaries.md`; the
committed proof is
`.ctxhelm/e2e/phase100-resource-backed-gap-summaries-proof.json` and reports
`releaseGate.decision = promote`. File/source/test/validation and broad
context-area metrics are unchanged from Phase 99 across the four-repo proof.

Phase 101 makes the resource-backed gap shape part of the product-proof release
contract. `scripts/check-product-proof.py` now rejects current reachable
retrieval-gap summaries that lack a `ctxhelm://repo/context-area/...` URI or
bounded `nextReadPaths`, and the Phase 100 four-repo proof passes the stricter
checker. The evidence artifact is
`.planning/e2e/2026-05-31-phase101-release-gated-gap-summary-contract.md`.

Phase 102 fixes explicit-repo MCP resource consumption. Repo-scoped resources
such as `ctxhelm://repo/context-areas` and
`ctxhelm://repo/context-area/{encoded-area}` now fall back to the last explicit
repo used by `prepare_task`, `get_pack`, `search`, `related`, or
`related_tests` when the MCP server cwd is not inside a repository. The
deterministic MCP protocol smoke now reads both context-area resource shapes and
verifies `nextReadBatches`; Cursor/OpenCode setup evidence and optional
Codex/Claude evidence record deterministic context-area resource-read coverage.
The evidence artifact is
`.planning/e2e/2026-05-31-phase102-explicit-repo-mcp-resource-consumption.md`.

Phase 103 adds broad fixed-corpus floors to the product-proof checker. For the
pinned `phase92-area-aware-gap-taxonomy-2026-05-31` four-repo corpus,
`scripts/check-product-proof.py` now rejects reports that drop below recorded
RefactoringMiner, ctxhelm, ReAgent, or VeriSchema file/source/test,
effective-validation, or broad-context-area floors. This caught a rejected
dependency-priority ranking experiment that still promoted overall but regressed
VeriSchema File Recall@10 from `0.18449473` to `0.17936651`. The evidence
artifact is
`.planning/e2e/2026-05-31-phase103-broad-fixed-corpus-floors.md`.

Phase 104 adds concrete source-free next-read guidance to broad context areas.
`ContextArea` now includes `nextReadPaths` and `unselectedCount`, docs
candidates participate in broad context-area guidance, and generated packs
render explicit `Next reads` so agents can inspect ranked-below-budget
source/docs pressure with native reads before requesting deeper packs.
`scripts/check-product-proof.py` also fails cleanly when a benchmark repository
has no embedded report instead of crashing during gap validation. The evidence
artifact is
`.planning/e2e/2026-05-31-phase104-context-area-next-read-paths.md`; the
available three-repo proof promotes, while the full four-repo proof is not
claimed because the local RefactoringMiner checkout timed out during
`git rev-list`.

Phase 117 adds source-free role signals to broad context-area guidance.
Plan-level `ContextArea` entries now include `roleCounts` and
`selectedRoleCounts`, and generated packs render those counts next to
representative paths and `Next reads`. This helps agents distinguish
source-heavy, validation-heavy, and docs-only areas before loading files
natively, without changing the top-10 target-file selection budget.

Phase 118 clarifies the dynamic MCP resource boundary for context areas.
`ctxhelm://repo/context-areas` and
`ctxhelm://repo/context-area/{encoded-area}` now include
`resourceScope.kind = safeInventoryArea`, `taskConditioned = false`,
`countsSource = safeInventory`, and `pathSource = safeInventory`. These fields
make it machine-checkable that MCP context-area resources are inventory-wide
progressive-read aids, not task-conditioned candidate or selected-file counts.

Phase 123 adds source-free `coverageProfile` metadata to context-area MCP
resources. The profile summarizes whether an area is implementation-heavy,
validation-heavy, docs-only, or mixed, and exposes `recommendedFirstBatch` so
agents can choose between `primary`, `validation`, and `docs` next-read batches
without reading source text or changing target-file ranking.

Phase 124 adds source-free `inspectionStrategy` metadata to the same context
area resources. The strategy turns the coverage profile into an explicit
progressive-read order, a small path budget, and a stop rule so agents can
inspect broad areas through native file reads without loading unnecessary
context or changing target-file ranking.

Phase 125 adds an explicit source-free `releaseGate.lexicalComparison` summary
to product proof output. The summary separates all-file lexical comparison from
non-test context-channel comparison, so release artifacts can say when ctxhelm
beats lexical on the production context claim without implying a universal
all-file win.

Phase 126 adds the source-free `agentEvidenceClaim` and
`averageAgentEvidenceRecallAt10` fields to the same summary. Agent evidence
counts the files and tests ctxhelm actually gives the agent through selected
context files, related tests, and validation commands. The clean four-repo proof
reported `agentEvidenceClaim = mixed` with zero trailing corpora and average
agent-evidence delta `+0.18792826` against lexical.

Phase 127 adds a narrow-plan validation-test reserve to the context ranking.
The Phase 127 clean four-repo proof reported zero trailing corpora for raw
target-file recall: `allFileClaim = mixed`, beat `3`, match `1`, trail `0`,
average File Recall@10 `0.5927659` versus lexical `0.45709258`, and average file
delta `+0.13567334`. Later broad-area hardening keeps raw all-file trails
auditable and separates explained trails from unexplained regressions. Broad
context-area plans stay file-first so validation tests do not displace broad
source evidence.

Phase 128 adds broad operational floors for root governance docs, exact config
matches, and workflow lifecycle scripts before lower-priority expansion. The
Phase 128 clean four-repo proof still promoted, kept zero trailing corpora, and cleared
protected target misses on all four corpora with average File Recall@10
`0.5986343`, average file delta `+0.14154172`, average agent-evidence delta
`+0.19379663`, and average context delta `+0.23717105`.

Phase 178 separates explained raw all-file trails from unexplained lexical
regressions in `releaseGate.lexicalComparison`. The clean four-repo proof still
promotes and reports `allFileClaim = mixed`, beat `3`, raw match `0`, raw trail
`1`, explained trail `1`, unexplained trail `0`, average File Recall@10
`0.61190045` versus lexical `0.45709258`, average file delta `+0.15480787`,
average agent-evidence delta `+0.2570628`, and average context delta
`+0.30652046`.

Phase 156 makes BM25-vs-legacy lexical backend proof available inside benchmark
suites and product-proof output. Suites opt in with
`lexicalBackendComparison = true`; benchmark JSON then records
`lexicalBackendCorpus` or a source-free `lexicalBackendError` per repository,
and product proof aggregates successful reports under
`releaseGate.lexicalBackendComparison`. This evidence answers whether the
active BM25 lexical backend improves over the old scanner. It does not replace
the existing ctxhelm-vs-lexical release verdicts for the full context compiler.

Phase 129 adds a public release freshness check for the already-published
archive channel. `scripts/check-public-release-freshness.sh` compares the
public `v1.1.0` release target with the current commit and writes source-free
`status`, `releaseTargetCommit`, `currentCommit`, `gitRelation`, and
`commitsAhead` metadata. Phase 130 extends that check with `productStatus`,
`productCommitsAhead`, `proofOnlyCommitsAhead`, and `ignoredFreshnessPaths` so
proof/planning-only commits after a release do not look like product drift. Use
`--require-current` before announcing that a public archive matches the exact
current commit. Use `--require-product-current` before announcing that the
public archive has no product-impacting commits ahead.

Phase 119 removes an observed release-validation flake in `ctxhelm-index`.
Tests in `lib.rs`, `freshness.rs`, and `storage.rs` now share one crate-wide
test lock before mutating process-global `CTXHELM_HOME`. This prevents one
parallel test from removing another test's temporary ctxhelm home while
inventory, trace, freshness, or storage fallback artifacts are being written.

Phase 105 keeps history-unavailable benchmark runs machine-checkable. If git
history sampling fails or times out, historical eval emits an embedded
zero-commit report, benchmark output records a source-free history-unavailable
error, and product proof blocks the corpus as `insufficient_evidence` rather
than emitting `report: null`. Degraded zero-commit reports are not cached, so a
transient large-repo timeout cannot poison later hydrated proof runs. The
evidence artifact is
`.planning/e2e/2026-05-31-phase105-history-unavailable-report.md`; the CLI proof
fixture is `.ctxhelm/e2e/phase105-history-unavailable-proof.json`.

Latest local real-client outcome proof: Codex CLI `0.137.0` passed the
source-free five-lane agent-run matrix with server-side `prepare_task` and
`get_pack` evidence against an explicit repo path, no forbidden commands, no
client failures, no ctxhelm evidence misses, and outcome claim
`ctxhelm_improved`. Best lane `ctxhelm-memory` improved target coverage by
`+0.33`, reduced irrelevant reads by `2`, reduced command executions by `14`,
and reduced read-file count by `2`. See
`.planning/e2e/2026-06-05-phase237-codex-agent-run-outcome.md` and
`.ctxhelm/e2e/phase237-agent-run-codex.json`. Claude Code `2.1.163` is still
classified separately as rate-limited with API status `429`, so current Claude
availability is not treated as ctxhelm retrieval or protocol failure. OpenCode
`1.14.25` has source-free real-client MCP evidence for `prepare_task` and
`get_pack` through `scripts/smoke-opencode-real-client.sh`; Cursor `3.6.21`
has the same optional proof path but is currently skipped locally because
`cursor agent status` reports not logged in.

RefactoringMiner and multi-repo proof are optional external gates. They are
skipped by default because they require a separate local checkout and longer
runtime. To reproduce the large-history gate, keep the repository local and run:

```bash
CTXHELM_BENCHMARK_CONFIG="$(pwd)/.ctxhelm/benchmarks/refactoringminer-v23.json" bash scripts/release-gate.sh
```

To run a broader corpus, add more local repositories to a suite JSON and pass
that suite through `CTXHELM_BENCHMARK_CONFIG`. If the external repo is missing,
record the skip reason as "external corpus unavailable" rather than treating it
as a product regression. The mandatory gate remains `scripts/smoke-v23-eval.sh`,
which proves the v2.3 contract without external repos.

The release gate does not publish, upload, or create GitHub releases, and does not create tags. It does not mutate global agent config and does not run user project tests. Cursor and OpenCode real-client proof is optional and source-free: a pass requires server-side `prepare_task` and `get_pack` request evidence with the explicit repo, while unavailable auth/provider state is recorded as a skip.

Phase 203 improves pack consumption of validation evidence without changing the
release artifact contract. Generated packs now include a source-free `Related
test evidence` section so agents see selected related tests, context areas,
reasons, confidence, and targeted commands even when those tests are not
repeated in context-area next-read lists.

Phase 204 strengthens real-agent outcome evidence without changing packaging.
Paired Claude Code agent-run reports now surface forbidden shell/edit/write
tool calls as source-free counts and tool-name/input-key summaries, and degraded
status is used when a read-only run observes such calls. The hardened Claude
Code `2.1.159` Phase 204 run observed no forbidden calls but recorded
`ctxhelm_matched`, not improved, for the validation-evidence pack task.

## Artifact Audit

`scripts/release-package.sh` runs `scripts/audit-release-artifact.sh` immediately after archive creation and before checksum success output. It writes a machine-readable `ctxhelm-v2.4.3-{target}.audit.json` report next to the archive.

The audit lists archive members and extracts the artifact to a temporary directory. It fails on local state, traces, request logs, cache or target debris, git internals, secret-looking filenames, absolute local paths, and text payloads with machine-specific or secret-looking values. It does not upload artifacts or call cloud scanning services.

You can audit an existing archive directly:

```bash
bash scripts/audit-release-artifact.sh dist/ctxhelm-v2.4.3-aarch64-apple-darwin.tar.gz
```

## Out of Scope for v1.1

The v2.4.3 release includes a public Apple Silicon Homebrew tap. It does not require crates.io publishing, self-update support, signed installers, cloud telemetry, cloud indexing, cloud embeddings, hosted release services, or global agent config mutation.

ctxhelm remains local-first and read-only. Release scripts build and audit ctxhelm artifacts only; they do not mutate user repositories, global Codex or Claude configuration, MCP client config, or package-manager registries.
