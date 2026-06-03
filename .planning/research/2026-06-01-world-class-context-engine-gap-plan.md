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
