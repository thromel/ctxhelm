# Phase 327: Agent-Run Identity Proof

## Goal

Prevent a saved real-agent outcome report from passing release validation after
the claimed ctxhelm or Codex client identity changes.

Phase 325 tied saved proof to the current Codex runner script. Phase 326 tied it
to the current four-task R&D breadth suite. Phase 327 ties it to the report's
declared ctxhelm version and Codex client identity.

## Change

`scripts/check-agent-run-proof.py` now accepts:

```text
--expected-ctxhelm-version "ctxhelm 2.4.0"
--expected-client-name codex
--expected-client-version "codex-cli 0.137.0"
```

When provided, the checker rejects reports whose top-level `ctxhelmVersion`,
`client.name`, or `client.version` no longer match the expected proof claim.

The JSON audit artifact now records:

```text
identity.ctxhelmVersion
identity.clientName
identity.clientVersion
identity.expectedCtxhelmVersion
identity.expectedClientName
identity.expectedClientVersion
identity.matchesExpectedCtxhelmVersion
identity.matchesExpectedClientName
identity.matchesExpectedClientVersion
thresholds.expectedCtxhelmVersion
thresholds.expectedClientName
thresholds.expectedClientVersion
```

The release gate passes the selected release-gate binary version as
`--expected-ctxhelm-version`, plus Codex `codex-cli 0.137.0` by default.
Maintainers can override the expected Codex identity with:

```text
CTXHELM_AGENT_RUN_EXPECTED_CLIENT_NAME
CTXHELM_AGENT_RUN_EXPECTED_CLIENT_VERSION
```

This keeps a saved proof report explicit about the exact ctxhelm and agent
client it supports, without forcing every release gate to rerun a live Codex
suite.

## Validation

The committed Phase 322 report still passes because it declares:

```text
ctxhelmVersion = ctxhelm 2.4.0
client.name = codex
client.version = codex-cli 0.137.0
```

The focused Rust contract now creates a stale-client fixture by replacing
`client.version` and proves the checker rejects it.

```bash
cargo test -p ctxhelm --test release_packaging \
  agent_run_proof_checker_accepts_phase322_and_rejects_regression --locked
```
