# Phase 204 - Agent-Run Forbidden Tool Accounting

## Goal

Make real-agent outcome proof stricter. The Phase 203 follow-up real Claude
Code run exposed that paired agent-run reports could show source-free read and
tool counts without explicitly flagging forbidden tool families such as shell,
edit, or write tools. A read-only outcome harness should make those boundary
violations visible instead of relying on the prompt text.

## Change

`scripts/e2e-agent-run.sh` now records source-free forbidden-tool evidence:

- per-lane `metrics.forbiddenToolCallCount`
- per-lane `forbiddenToolCalls` with tool name and input-key names only
- comparison-level `forbiddenToolCallsObserved`
- suite aggregate `forbiddenToolCallsObserved`
- suite lane `forbiddenToolCallCount`

Single-task reports degrade from `passed` to `degraded` when any lane observes a
forbidden tool call. Suite reports do the same when any task observes forbidden
tool calls. `ctxhelm eval agent-run` now renders forbidden call counts for both
single-task lanes and suite lane summaries.

## Validation

Focused commands:

```bash
bash -n scripts/e2e-agent-run.sh
cargo fmt --all -- --check
cargo test -p ctxhelm --test release_packaging \
  agent_run_e2e_script_contract --locked -- --nocapture
cargo test -p ctxhelm --test cli_compat \
  eval_agent_run_renders_source_free --locked -- --nocapture
```

Result: pass.

Real Claude Code command:

```bash
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=120 \
bash scripts/e2e-agent-run.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --task "Identify files relevant to improving validation evidence consumption in generated ctxhelm packs" \
  --target-file crates/ctxhelm-compiler/src/packs.rs \
  --target-file crates/ctxhelm-compiler/src/lib.rs \
  --target-file docs/benchmarking.md \
  --output /tmp/ctxhelm-rd/phase204-forbidden-tool-agent-run.json
```

Result:

- Report status: `passed`
- Client: Claude Code `2.1.159`
- Forbidden calls observed: `false`
- Outcome claim: `ctxhelm_matched`
- Native baseline: target coverage `0.6667`, read files `6`, irrelevant reads
  `4`, forbidden calls `0`, target hits `packs.rs`, `lib.rs`
- `ctxhelm-plan`: target coverage `0.3333`, read files `3`, irrelevant reads
  `2`, ctxhelm calls `1`, forbidden calls `0`, target hit `packs.rs`
- `ctxhelm-brief`: target coverage `0.3333`, read files `2`, irrelevant reads
  `1`, ctxhelm calls `2`, forbidden calls `0`, target hit `packs.rs`

Privacy:

- local-only: `true`
- raw prompt stored: `false`
- raw transcript stored: `false`
- raw MCP traffic stored: `false`
- source text logged: `false`

## Interpretation

This is a negative agent-behavior result, not a retrieval win. The brief pack
reduced read count and irrelevant reads, but it also caused Claude Code to
under-read target support files for this task. The next real-agent R&D loop
should investigate why pack-assisted lanes under-consume docs and implementation
support files even when retrieval/product proof is healthy.
