# Phase 206 - Consumption Guidance Hardening

## Goal

Reduce the real-agent risk exposed by Phase 205: ctxhelm-assisted lanes can
discover a target path without actually reading the current file. The fix should
strengthen the source-free agent-consumption contract without changing retrieval
ranking, adding source text to reports, or weakening the local-only boundary.

## Change

- `prepare_task` MCP text now includes source-free consumption guidance before
  the pretty `ContextPlan` JSON.
- Generated AGENTS, Cursor, Claude Code, OpenCode, and Codex guidance now state
  that discovering a path is not the same as consuming it.
- Generated packs now include a source-free `Consumption guidance` section
  before target files, validation evidence, and source-bearing snippets.

The structured `ContextPlan` JSON remains unchanged. This avoids a broad
contract migration while making the human/agent-facing text more explicit.

## Validation

Focused commands:

```bash
cargo fmt --all -- --check
cargo test -p ctxhelm-core init::tests::adapter_guidance --locked -- --nocapture
cargo test -p ctxhelm-mcp prepare_task_call_returns_structured_context_plan --locked -- --nocapture
cargo test -p ctxhelm-compiler \
  compile_context_pack_materializes_plan_snippets_and_validation --locked -- --nocapture
```

Result: pass.

Real Claude Code command used for both probes:

```bash
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=120 \
bash scripts/e2e-agent-run.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --task "Identify files relevant to improving paired agent-run consumption diagnostics in ctxhelm" \
  --target-file scripts/e2e-agent-run.sh \
  --target-file crates/ctxhelm/src/main.rs \
  --target-file docs/feedback.md \
  --output /tmp/ctxhelm-rd/phase206-pack-consumption-guidance-real.json
```

Probe A, after MCP and generated-guidance wording:

- Report status: `passed`
- Best lane: `ctxhelm-plan`
- Native baseline target-read coverage: `0.33`
- `ctxhelm-plan` target-read coverage: `0.67`
- `ctxhelm-brief` target-read coverage: `0.00`
- Forbidden calls observed: `false`
- ctxhelm under-read targets observed: `true`

Probe B, after adding pack-level consumption guidance:

- Report status: `passed`
- Best lane: `ctxhelm-plan`
- Native baseline target-read coverage: `0.67`
- `ctxhelm-plan` target-read coverage: `0.67`
- `ctxhelm-brief` failed before making ctxhelm calls
- Forbidden calls observed: `false`
- ctxhelm under-read targets observed: `true`

## Interpretation

This phase improves the product-facing consumption contract, and the first
post-change real run showed the plan lane reading more targets than native
baseline. The second run was noisy and did not prove brief-pack improvement
because the brief lane failed before calling ctxhelm. The next R&D loop should
make the paired real-agent harness more robust to lane-level failures and should
separately test brief-pack consumption after ensuring the lane actually calls
`get_pack`.
