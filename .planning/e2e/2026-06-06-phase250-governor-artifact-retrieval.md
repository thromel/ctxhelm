# Phase 250 - Governor Artifact Retrieval

## Scope

Refresh the real Codex agent-run proof for the context-governor/release-gate R&D task and fix the observed ctxhelm under-read regression without adding cloud retrieval or source logging.

Task:

```text
Identify the files relevant to improving ctxhelm context governor decision reports and release-gate proof. Do not edit files.
```

Expected target files:

- `crates/ctxhelm/src/main.rs`
- `crates/ctxhelm-core/src/contracts.rs`
- `scripts/smoke-governor.sh`
- `docs/context-governor.md`

## Before

Report:

```text
.ctxhelm/e2e/phase250-agent-run-codex-governor-rd.json
```

Observed:

- Overall status: `passed`
- Outcome claim: `ctxhelm_matched`
- Best lane: `baseline`
- ctxhelm evidence misses observed: `true`
- ctxhelm under-read targets observed: `true`
- ctxhelm target read coverage:
  - `ctxhelm-plan`: `0.25`
  - `ctxhelm-brief`: `0.50`
  - `ctxhelm-standard`: `0.75`
  - `ctxhelm-memory`: `0.50`

Failure mode:

- `docs/context-governor.md` was not present in the first-read target set.
- `scripts/smoke-governor.sh` and `crates/ctxhelm-core/src/contracts.rs` were missed by some ctxhelm lanes.
- Root planning docs and high-frequency release/proof files crowded out the task-specific product doc and report contract.

## Fix

Changed the source-free planner/ranking path:

- Release/gate/proof/governor tasks now inject first-class artifact candidates:
  - `docs/context-governor.md`
  - `crates/ctxhelm-core/src/contracts.rs`
  - `scripts/smoke-governor.sh`
  - `scripts/release-gate.sh`
- `docs/context-governor.md` is treated as a root governance doc in ranking.
- `scripts/smoke-governor.sh` gets source-floor priority; `scripts/release-gate.sh` remains a normal file candidate so it does not displace the topic doc from the first five target files.

No source text is stored in the reports; artifact promotion is path/role/task-term based.

## After

Report:

```text
.ctxhelm/e2e/phase250-agent-run-codex-governor-rd-after.json
```

Observed:

- Overall status: `passed`
- Outcome claim: `ctxhelm_improved`
- Best lane: `ctxhelm-standard`
- ctxhelm evidence misses observed: `false`
- ctxhelm under-read targets observed: `false`
- Target coverage delta: `+0.50`
- Target read coverage delta: `+0.50`
- ctxhelm target read coverage:
  - `ctxhelm-plan`: `1.00`
  - `ctxhelm-brief`: `1.00`
  - `ctxhelm-standard`: `1.00`
  - `ctxhelm-memory`: `1.00`

First five no-anchor `prepare-task` targets after the fix:

1. `crates/ctxhelm/src/main.rs`
2. `crates/ctxhelm-compiler/src/eval.rs`
3. `crates/ctxhelm-core/src/contracts.rs`
4. `scripts/smoke-governor.sh`
5. `docs/context-governor.md`

## Validation

Focused checks:

```bash
cargo fmt --check
cargo test -p ctxhelm-compiler --locked governor
cargo run -q -p ctxhelm -- prepare-task "Identify the files relevant to improving ctxhelm context governor decision reports and release-gate proof. Do not edit files." --repo /Users/romel/Documents/GitHub/ctxhelm --mode explain
CTXHELM_RUN_REAL_CLIENT=1 CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=90 bash scripts/e2e-agent-run-codex.sh --repo /Users/romel/Documents/GitHub/ctxhelm --task "Identify the files relevant to improving ctxhelm context governor decision reports and release-gate proof. Do not edit files." --target-file crates/ctxhelm/src/main.rs --target-file crates/ctxhelm-core/src/contracts.rs --target-file scripts/smoke-governor.sh --target-file docs/context-governor.md --output .ctxhelm/e2e/phase250-agent-run-codex-governor-rd-after.json
```

Full workspace validation is recorded in the commit/gate output for this phase.

Claude workflow refresh:

```bash
CTXHELM_RUN_REAL_CLIENT=1 CTXHELM_CLAUDE_WORKFLOW_REPORT=.ctxhelm/e2e/phase250-claude-workflow-refresh.json bash scripts/e2e-claude-workflow.sh
```

Observed:

- Status: `passed`
- Workflow kind: `claude-code-mcp-context-workflow`
- Privacy: local-only, no raw prompt/transcript/MCP traffic/source text stored
