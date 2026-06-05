# World-Class Context Engine Gap Plan

Date: 2026-06-01

## Research Signals

Recent repository-agent research and current agent integration docs point to the
same product direction: ctxhelm should become an evidence governor, not a
larger search box.

Key signals:

- OpenAI no longer treats SWE-bench Verified as sufficient for frontier coding
  capability and recommends SWE-bench Pro for harder, longer-horizon tasks:
  https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/
- ContextBench evaluates coding agents by context retrieval process, not just
  final task success, and reports gaps between explored and actually useful
  context: https://arxiv.org/abs/2602.05892
- SWE-ContextBench focuses on experience reuse across related programming
  tasks and shows that selected summarized experience can improve accuracy,
  runtime, and token cost, while poorly selected experience can hurt:
  https://papers.cool/arxiv/2602.08316
- RepoBench frames repository-level coding as retrieval, completion, and
  pipeline evaluation, validating ctxhelm's separate retrieval and agent
  outcome gates: https://arxiv.org/abs/2306.03091
- Repoformer argues retrieval should be selective, not automatic:
  https://proceedings.mlr.press/v235/wu24a.html
- GraphCoder and newer repository-graph work point toward structural retrieval
  over flat chunks:
  https://arxiv.org/abs/2406.07003
  https://arxiv.org/abs/2505.14394
- Long-context work shows relevant evidence can be underused when buried in
  the middle of large prompts, so packs need deliberate ordering and
  progressive disclosure: https://arxiv.org/abs/2307.03172
- Claude Code exposes MCP tools, resources, prompts, resource references, and
  hooks, and MCP tool output can be a token-budget risk:
  https://code.claude.com/docs/en/mcp
  https://code.claude.com/docs/en/hooks
- MCP's core server features map directly to ctxhelm's integration surface:
  tools, resources, and prompts over JSON-RPC:
  https://modelcontextprotocol.io/specification/2025-03-26/basic/index

## Current Strengths

- Public archive channel is source-free and verifiable.
- v1.1.2 release is published and install-verified from GitHub assets.
- Claude Code real-client smoke passes against the public archive binary.
- Codex CLI smoke records source-free skip evidence instead of pretending
  success.
- Retrieval proof beats or matches lexical on the current four-corpus product
  proof with zero trailing corpora for target-file, agent-evidence, and context
  claims.
- Context-area resources give broad tasks progressive source-free next reads.

## Current Gaps

### 1. Agent Outcome Proof Is Still Thin

We prove retrieval and protocol integration, but not enough end-to-end agent
task improvement.

Next feature:

- Add a `ctxhelm eval agent-run` harness that runs paired agent tasks:
  - baseline agent native behavior
  - ctxhelm plan only
  - ctxhelm brief pack
  - ctxhelm standard pack
  - ctxhelm plus memory cards
- Output source-free metrics:
  - task status
  - target files read
  - target files edited
  - tests run
  - elapsed time
  - request/tool counts
  - context token estimate
  - pass/fail/skip reason

Success gate:

- At least one Claude Code paired run over a real repo task shows fewer
  irrelevant reads or better target-file coverage than baseline without
  increasing privacy risk.

Status: Phase 143 added this proof path. The first real Claude Code run over
the ctxhelm workflow-eval task preserved target-file coverage at `1.00` and
reduced irrelevant reads from `5` to `2` in the `ctxhelm-brief` lane while
keeping raw prompts, transcripts, MCP traffic, and source text out of the
persisted report.

Status: Phase 225 closes the lane-coverage gap in that proof path. The paired
agent-run harness now evaluates native baseline, `ctxhelm-plan`,
`ctxhelm-brief`, `ctxhelm-standard`, and `ctxhelm-memory`. Standard and memory
lanes require explicit-repo `prepare_task` plus `get_pack` with
`budget = "standard"`, `format = "json"`, and `recordTrace = false`; the memory
lane instructs Claude to consume selected memory evidence only as guidance and
still read current files natively. The first local real-client attempt with
Claude Code `2.1.163` was rate-limited across all lanes, so it correctly
records `insufficient_comparable_lanes` and `retry_real_client_when_available`
rather than claiming outcome lift.

### 2. Experience Memory Is Not Yet Proven By Reuse

The memory layer exists, but the product proof does not yet show that prior
episodes improve future related tasks.

Next feature:

- Add experience-card retrieval eval:
  - store source-free episode summaries
  - retrieve related experience for later historical commits
  - compare with and without experience cards
  - track when memory hurts

Success gate:

- Experience cards must improve Recall@10, validation recall, agent read
  precision, or runtime on related tasks. If neutral or harmful, keep them
  opt-in and diagnostic only.

Status: Phase 214 adds source-free `memoryReuseSummary` to historical eval
reports and memory-aware `recommendedResearchActions` to historical and
product-proof reports. This does not yet prove memory improves end-to-end agent
outcomes, but it creates the required measurement surface: active memory
candidates, selected memory evidence, target hits/misses, unique target hits
beyond lexical, unique non-targets, and selected memory roles.

Status: Phase 215 promotes deterministic experience-memory reuse proof into the
release gate. The required smoke proves pending memory is blocked, approved
memory can promote a source-linked target on a related task, and the storage
boundary remains source-free. This closes deterministic workflow proof, while
multi-repo historical memory-lift proof remains the next higher bar.

Status: Phase 216 adds a controlled historical memory-lift proof. The new
release-gated smoke runs `eval history` before and after approval and proves
memory changes a historical report from no memory candidates and a missed target
to a selected memory source with a unique target hit beyond lexical plus
`evaluate_memory_reuse_lift`. This proves the historical eval plumbing can
measure positive memory lift; broad multi-repo memory generalization remains a
higher bar.

Status: Phase 217 adds benchmark/product-proof memory-lift aggregation proof.
The new release-gated smoke runs `eval proof` over two local repositories with
approved source-free experience cards and requires both embedded reports to show
memory-only target hits beyond lexical plus product-level
`evaluate_memory_reuse_lift`. This proves product-proof plumbing can aggregate
positive memory lift across repositories; broad memory generalization on real
histories remains the next higher bar.

Status: Phase 218 fixes the first real-history memory blocker. A RefactoringMiner
repeated-file probe showed approved source-repo memory disappeared when
historical eval used a parent snapshot with a different local storage identity.
Historical eval now projects approved source-free memory cards into parent
snapshots before planning, the release gate covers the controlled non-root path,
and the RefactoringMiner pair now reports memory candidates plus one unique
memory target hit beyond lexical. This closes parent-snapshot memory visibility;
multi-pair and multi-repo memory generalization remain open.

Status: Phase 219 adds the first repeatable real-corpus memory-generalization
measurement harness. `scripts/measure-memory-generalization.sh` scans repeated
historical file pairs, seeds approved experience memory from older tasks, and
measures newer-task before/after lift without storing source text or raw task
subjects. The two-pair RefactoringMiner run reports one unique memory lift
beyond lexical and one combined-target recovery, but also eight unique
non-target memory selections. This means memory visibility works; memory
precision and broader multi-pair/multi-repo generalization remain open.

Status: Phase 220 reduces the measured memory precision noise without losing
the observed lift. Experience cards now preserve source-free eval-trace
recommendation order, keep recommended files before tests, and cap memory
source-link candidate injection to a small source-like context set per card.
The same two-pair RefactoringMiner measurement keeps one unique memory lift and
one combined-target recovery while reducing unique non-target memory selections
from 8 to 2. `precisionNeedsWork` remains true, so the next higher bar is more
pairs, more repos, and a stricter memory-selection policy.

Status: Phase 221 adds the multi-repo measurement layer. The suite wrapper runs
the real-corpus memory harness across RefactoringMiner, VeriSchema, ReAgent,
and ctxhelm fixtures with one repeated-file pair per repo. The source-free
aggregate reports two unique memory lifts, two unique target hits, one combined
target recovery, and three unique non-target selections across four evaluated
pairs. This is bounded multi-repo lift proof under the current criterion, but
the remaining noise and tiny pair count mean the next bar is larger pair counts
plus graph/semantic ablation comparison.

Status: Phase 222 adds graph/semantic comparison to that memory measurement.
The single-repo and suite harnesses now support `--semantic --semantic-provider
local_hash` and write v2 source-free signal diagnostics: semantic selected
target pairs, graph-edge ablation target-hit loss, graph/semantic
memory-corroboration upper bounds, and uncorroborated memory lower bounds. The
first four-repo semantic-enabled probe shows semantic selected target evidence
on two repositories, but zero semantic ablation-lift pairs, one memory-unique
lift, one unique memory non-target, and one lower-bound memory target hit
without graph-or-semantic support. This narrows the next memory R&D from
"measure ablations" to "test a stricter memory-candidate corroboration policy
and scale the pair count."

Status: Phase 223 tests that stricter memory-corroboration policy and improves
the metric. Ranking no longer attaches memory to lexical-expansion-only paths
and only allows uncorroborated memory-only rescue when no target file was
selected otherwise. Historical reports now separate old
`memoryUniqueNonTargetCount` from
`memoryUniqueNonTargetWithoutCurrentSupportCount`, so memory attached to current
lexical, semantic, graph, symbol, or history evidence is not mislabeled as pure
memory noise. The four-repo rerun keeps one lexical-baseline-relative memory
non-target but reports zero unsupported memory non-targets and zero unsupported
memory-unique target hits. The next higher bar is not another precision patch;
it is larger repeated-history pair counts and real-agent outcome lift.

Status: Phase 224 expands that repeated-history measurement. The harness now
discovers all repeated-file candidates in the scan window, prefers distinct
target files before duplicate-path pairs, and reports pair-diversity counters.
The suite default is three pairs per repo. The four-repo semantic-enabled run
measures 12 pairs across 12 distinct target files from 971 candidate pairs and
256 candidate target files. It restores broad memory-lift evidence with two
unique memory lifts and two unique memory target hits while keeping unsupported
pure-memory noise at zero. Raw lexical-baseline-relative memory non-targets
remain measurable, but the next higher bar is now real-agent outcome lift.

Status: Phase 228 refines that raw-noise interpretation. Reports now expose
`memoryUniqueNonTargetWithCurrentSupportCount` and
`supportedMemoryNoiseNeedsReview`, and route supported memory non-targets to
current-signal pressure review instead of uncorroborated-memory demotion. The
fresh four-repo semantic-enabled suite still proves two unique memory lifts and
zero unsupported pure-memory non-targets; its four raw memory non-targets all
have current support.

Status: Phase 229 makes the pressure review measurable. Historical memory
summaries and both memory-generalization harnesses now expose current-support
signal count maps for memory-unique target hits and memory-unique non-targets.
The fresh four-repo semantic-enabled suite keeps two unique memory lifts and
zero unsupported memory non-targets, while showing supported memory non-target
pressure dominated by dependency, lexical-expansion, and symbol support. The
next memory R&D step is a targeted weighting experiment against supported signal
pressure, not another broad memory demotion.

Status: Phase 230 completes that targeted experiment. Ranking now attaches
memory to existing candidates only when they have strong current support:
anchor, current diff, lexical, semantic, or co-change. Dependency-only and
symbol-only candidates keep their own ranking evidence but no longer receive
extra memory pressure. The fresh four-repo semantic-enabled suite keeps two
unique memory lifts and zero unsupported memory non-targets while reducing raw
memory unique non-targets from four to one. The remaining memory R&D gap is now
larger pair-count validation, inspection of the one remaining strong-signal
overlap, and real-agent outcome lift, not local weak-signal memory-pressure
tuning.

Status: Phase 231 closes the same-corpus larger pair-count validation step. The
four-repo semantic-enabled suite now measures 20 pairs across 20 distinct target
files, keeps two memory-unique target hits, keeps unsupported memory non-targets
at zero, and keeps weak-supported memory noise cleared. The remaining local
memory R&D gap is repository diversity plus inspection of the VeriSchema
strong-signal overlap; real-agent outcome lift remains a separate client
availability/evaluation issue.

Status: Phase 232 closes the first repository-diversity measurement step and
reopens memory precision work under broader evidence. The suite now has an
explicit six-repo diversity target and repository-level lift/noise counters.
The fresh six-repo semantic-enabled run over VeriSchema, ReAgent, ctxhelm,
flask, fd, and express measures 30 pairs across 30 distinct target files, sets
`repositoryDiversityTargetMet = true`, preserves two memory-unique target hits,
and uncovers one unsupported memory non-target in express. The next local memory
R&D step is no longer broad expansion; it is demoting or re-corroborating
uncorroborated memory candidates and rerunning the six-repo suite.

Status: Phase 233 clears that unsupported memory precision issue without losing
the measured lift. Pair-level reproduction showed the express unsupported
non-target was an uncorroborated memory-only source rescued while native
semantic/related-test evidence was already available. Ranking now blocks that
rescue shape while preserving memory-only rescue when memory is the only
available evidence. The focused express rerun drops unsupported memory
non-targets from one to zero, and the six-repo rerun preserves
`memoryUniqueLiftPairs = 2` with `unsupportedMemoryNoiseRepositoryCount = 0`.
The remaining local memory R&D target is strong-signal overlap inspection for
the one co-change/dependency-supported express non-target.

### 3. Semantic/Embedding Signal Is Wired But Not Driving Lift

The current product lift comes from hybrid ranking, graph/test/history/context
area reserves, not semantic embeddings. That is honest but not world-class yet.

Next feature:

- Add a stronger local semantic backend benchmark lane:
  - compare `local_hash`, `local_fastembed`, and any available local model
  - evaluate semantic-only, lexical-only, graph-only, and fused variants
  - report per-query-family where semantic helps or hurts

Success gate:

- Semantic stays secondary unless it provides measured lift on at least one
  pinned corpus without crowding out exact lexical and protected evidence.

### 4. GraphRAG Needs More Precise Graph Semantics

Current graph expansion is useful but still mostly dependency/test/history
driven. World-class behavior needs typed graph evidence and budgets per task.

Next feature:

- Add graph neighborhood profiles:
  - import edges
  - symbol references
  - test edges
  - co-change edges
  - config/schema/doc edges
  - precision edges when available
- Add graph ablation reports by edge family.

Success gate:

- Graph expansion must prove edge-family lift and bounded context growth. Weak
  semantic-only seeds must not recursively expand when exact anchors exist.

### 5. Claude Code Integration Should Move From Smoke To Workflow Eval

The current Claude Code proof confirms tool calls. It does not yet evaluate
whether Claude actually uses ctxhelm better across a real task.

Next feature:

- Add a Claude Code workflow eval mode:
  - install public ctxhelm binary into temp path
  - generate temp repo-local `.mcp.json` or wrapper config
  - run a task that requires target-file retrieval and a test command
  - record sanitized server-side request log
  - record source-free file-read/edit/test summaries when available

Success gate:

- Claude Code must call `prepare_task` and either `get_pack` or a resource read,
  then touch/read expected target evidence or record why not.

### 6. Distribution Is Still Archive-Only

Archive-first is acceptable for current proof, but not world-class developer
distribution.

Next feature candidates:

- Homebrew tap with checksums and release automation.
- Signed/notarized macOS artifact.
- crates.io package only after dependency/readme/license metadata and install
  UX are ready.
- `ctxhelm doctor --public-release vX.Y.Z` to verify release and product
  freshness from a user install.

Success gate:

- Each channel must have source-free install verification, rollback notes, and
  no silent update behavior.

## Recommended Next Milestone

Phase 132 should focus on Claude Code workflow evaluation, not more release
plumbing.

Deliverables:

1. `scripts/e2e-claude-workflow.sh` or equivalent source-free harness.
2. A real-repo task fixture over ctxhelm or RefactoringMiner.
3. Baseline-vs-ctxhelm comparison where possible.
4. Durable proof artifact under `.ctxhelm/e2e/`.
5. Planning summary with observed bugs, relevance quality, token/tool counts,
   and next fixes.

Reason:

The product is now publicly installable and proof-gated. The next world-class
question is whether a real coding agent does better with ctxhelm on a realistic
task, not whether another protocol smoke passes.

## June 5 Memory R&D Update

Phases 219-234 turned experience memory from a smoke-tested feature into a
measured local R&D lane:

- controlled memory smokes prove parent-snapshot and product-proof plumbing
- real-corpus generalization suites measure repeated-file commit pairs
- suite reports compare memory against lexical, graph, and semantic signals
- unsupported memory-only non-target rescue is now blocked when native related
  test evidence already exists
- final-pack impact is now separated from signal-only memory overlap

Current six-repo evidence preserves two memory-unique target lifts, has zero
unsupported memory-noise repositories, and has zero final-pack non-target
additions from memory. The remaining memory overlap is signal-only. The next
world-class question is therefore outcome lift: does an actual Claude/Codex
agent use memory-backed ctxhelm evidence to read fewer irrelevant files or solve
more tasks, not another local ranking demotion.

## June 6 R&D Audit Update

Phase 250 closes the latest observed Codex under-read regression for
context-governor and release-gate R&D tasks. The before artifact
`.ctxhelm/e2e/phase250-agent-run-codex-governor-rd.json` matched baseline but
showed ctxhelm evidence misses and under-read targets. The after artifact
`.ctxhelm/e2e/phase250-agent-run-codex-governor-rd-after.json` reports
`ctxhelm_improved`, best lane `ctxhelm-standard`, no ctxhelm evidence misses,
no under-read targets, and `1.00` target-read coverage in every ctxhelm lane.

The current R&D state is therefore:

- Codex real-agent proof is strong for measured failure classes: Phase 245
  proves a two-task suite, and Phase 250 proves a governor/release task
  improvement.
- Claude Code workflow proof is current again through
  `.ctxhelm/e2e/phase250-claude-workflow-refresh.json`, but a fresh paired
  Claude outcome suite still depends on client availability.
- Experience memory is source-free, bounded, and has local six-repo
  measurement, single-repo Codex consumption evidence, three-repo Codex
  target-consumption evidence, and three-repo Codex read-efficiency evidence.
  Phase 255 closes the current measured memory-efficiency gap for the same
  three-repo slice.
- GraphRAG is implemented and measured through edge profiles, edge ablations,
  and bounded edge-family budget allocation. Future graph work should target
  ranking pressure, not unbounded edge expansion.
- Semantic retrieval is integrated, bounded, source-free, and policy-gated, but
  it is not yet proven as a default-lift channel. Current semantic contribution
  diagnostics correctly hold promotion when semantic-selected files overlap
  lexical evidence without semantic-only target hits.

Tracked audit: `.planning/e2e/2026-06-06-rd-completion-audit.md`.

## June 6 Phase 251 Update

Phase 251 adds the larger single-repo Codex R&D suite requested by the audit:
one selected-memory native-read task, one semantic contribution diagnostics
task, one GraphRAG edge-budget task, and one governor/release proof task. The
suite exposed and fixed two issues before the final proof:

- GraphRAG edge-budget tasks did not initially surface
  `crates/ctxhelm-compiler/src/ranking.rs` or
  `crates/ctxhelm-index/src/dependencies.rs` in the top target window.
- The Codex harness prompt allowed discovery commands to be interpreted as
  target consumption and did not explicitly forbid `awk`/redirection.

The final source-free report
`.ctxhelm/e2e/phase251-agent-run-codex-rd-breadth-suite.json` passes with four
comparison-eligible tasks, 16 comparable ctxhelm lanes, outcome
`ctxhelm_improved`, no evidence misses, no evidence-only targets, no under-read
targets, no forbidden commands, no client failures, and `1.00` average
target-read coverage in every ctxhelm lane. This closes the current larger
single-repo Codex suite gap. Remaining R&D should now focus on semantic
default-lift proof, broader cross-repo memory outcome diversity, and fresh
paired Claude outcome proof when the client is available. Phase 253 later
closes the memory outcome-diversity item for target consumption, but not for
read efficiency.

## June 6 Phase 252 Update

Phase 252 fixes a semantic gate classification bug exposed by the next R&D
probe. The semantic/precision gate previously mixed named regressions from the
eval-only `local_metadata_reranked` variant into the top-level semantic
promotion decision. That made semantic look `block` even when the actual
semantic variant should be `hold`.

The fix keeps eval-only reranker regressions visible in `namedRegressions` and
diagnostics, but only semantic promotion variants can block the semantic
default decision. Fresh `local_fastembed` source-free gates now report:

- RefactoringMiner: `hold`, semantic Recall@K `0.5104166` vs default
  `0.48541665`, one semantic-only target hit.
- ctxhelm: `hold`, semantic Recall@K `0.3212704` vs default `0.3212704`, one
  semantic-only target hit but neutral aggregate recall.

The current semantic truth is therefore more precise: not blocked by an
unrelated reranker experiment, but still not strong enough for default
promotion. Next semantic R&D should focus on query-family lift and
query/document construction, not default enablement.

## June 6 Phase 253 Update

Phase 253 closes the current cross-repo real-agent memory outcome gap with a
source-free Codex CLI suite. `scripts/e2e-codex-memory-outcome-suite.sh` scans
each requested repository for repeated-file historical pairs, seeds one
approved experience-memory card from the older pair task, and runs the existing
read-only Codex paired harness against the newer pair task with isolated
`CTXHELM_HOME`.

The source-free artifact
`.ctxhelm/e2e/phase253-codex-memory-outcome-suite.json` covers VeriSchema,
ReAgent, and RefactoringMiner with one repeated-file memory pair per repository.
It reports:

- `status = passed`
- `evaluatedRepositoryCount = 3`
- `comparisonEligibleCount = 3`
- `improvedPairCount = 3`
- `memoryTargetReadImprovedPairCount = 3`
- `memoryTargetReadMatchedOrImprovedPairCount = 3`
- `ctxhelmEvidenceMissPairCount = 0`
- `ctxhelmUnderReadPairCount = 0`
- `clientFailurePairCount = 0`
- `rateLimitPairCount = 0`
- `forbiddenCommandPairCount = 0`
- `missingRequiredCtxhelmCallPairCount = 0`
- `invalidRequiredCtxhelmCallPairCount = 0`

This is real outcome evidence that approved memory-backed ctxhelm guidance gets
Codex to consume the target file across multiple repositories. It is not an
efficiency win: `memoryIrrelevantReadImprovedPairCount = 0`. The remaining
memory R&D target is therefore narrower and more honest: preserve target
consumption as a regression guard while improving read efficiency.

## June 6 Phase 254 Update

Phase 254 refreshes the Claude Code R&D breadth-suite status with the current
local client instead of relying on older evidence. The source-free artifact
`.ctxhelm/e2e/phase254-agent-run-claude-rd-breadth-suite.json` uses Claude Code
`2.1.163` and the same four-task R&D breadth suite hash as Phase 252.

The result remains `degraded`: all four client preflights report rate limiting,
there are zero comparison-eligible tasks, and zero comparable ctxhelm lanes.
The report has no evidence misses, under-read targets, malformed required calls,
or privacy regressions. This is current client-availability evidence, not
retrieval-quality evidence. The correct action remains
`retry_real_client_when_available`.

## June 6 Phase 255 Update

Phase 255 fixes the memory read-efficiency gap exposed by Phase 253. The
`ctxhelm-memory` lane in `scripts/e2e-agent-run-codex.sh` now uses a tighter
memory-efficiency prompt: at most four post-ctxhelm shell commands, at most two
memory-backed current-file reads first, and an explicit stop rule when those
reads answer the task. Baseline, plan, brief, and standard lane prompts are
unchanged.

The source-free artifact
`.ctxhelm/e2e/phase255-codex-memory-efficiency-suite.json` reruns the same
VeriSchema, ReAgent, and RefactoringMiner memory outcome suite as Phase 253. It
reports:

- `status = passed`
- `comparisonEligibleCount = 3`
- `improvedPairCount = 3`
- `memoryTargetReadImprovedPairCount = 3`
- `memoryTargetReadMatchedOrImprovedPairCount = 3`
- `memoryIrrelevantReadImprovedPairCount = 3`
- `ctxhelmEvidenceMissPairCount = 0`
- `ctxhelmUnderReadPairCount = 0`
- `clientFailurePairCount = 0`
- `rateLimitPairCount = 0`
- `forbiddenCommandPairCount = 0`
- `missingRequiredCtxhelmCallPairCount = 0`
- `invalidRequiredCtxhelmCallPairCount = 0`

Compared with Phase 253, memory-lane target-read coverage stays at `1.0` for
all three repos, while memory reads drop from `6 -> 2`, `6 -> 4`, and `5 -> 2`,
and memory irrelevant reads drop from `5 -> 1`, `5 -> 3`, and `4 -> 1`.

Memory R&D is now in a much stronger state: target consumption and read
efficiency both improve on the measured real-client slice. Future memory work
should preserve Phase 255 as a regression guard and broaden only to hunt for
new counterexamples.
