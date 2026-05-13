---
phase: 07-documentation-troubleshooting
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - README.md
  - docs/quickstart.md
autonomous: true
requirements: [DOCS-01]
must_haves:
  truths:
    - "User can follow README from release archive install to first `prepare-task` and `get-pack` without a source checkout."
    - "README quickstart commands use the installed `ctxpack` binary, current CLI flags, and explicit `--repo` arguments."
    - "A deeper quickstart doc exists for users who want the same flow with setup validation, MCP proof context, and first-pack explanation."
  artifacts:
    - path: "README.md"
      provides: "Short normal-user install-to-first-pack path"
    - path: "docs/quickstart.md"
      provides: "Task-oriented first-pack quickstart reference"
  key_links:
    - from: "README.md"
      to: "docs/quickstart.md"
      via: "README link from concise quickstart to detailed quickstart"
      pattern: "docs/quickstart\\.md"
    - from: "docs/quickstart.md"
      to: "ctxpack prepare-task/get-pack"
      via: "installed-binary commands with explicit repo"
      pattern: "ctxpack prepare-task.*--repo|ctxpack get-pack.*--repo"
---

<objective>
Create the short install-to-first-pack documentation path.

Purpose: Phase 7 starts by making the normal user journey understandable from README alone, without asking users to build from source or discover development scripts.
Output: README quickstart updates and a dedicated first-pack quickstart reference.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/REQUIREMENTS.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/research/SUMMARY.md
@.planning/research/FEATURES.md
@.planning/research/PITFALLS.md
@.planning/phases/07-documentation-troubleshooting/07-CONTEXT.md
@.planning/phases/05-release-identity-binary-packaging/VERIFICATION.md
@.planning/phases/06-agent-setup-first-pack-adoption/VERIFICATION.md
@.planning/phases/05-release-identity-binary-packaging/05-release-identity-binary-packaging-04-SUMMARY.md
@.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-01-SUMMARY.md
@.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-04-SUMMARY.md
@README.md
@docs/release.md
@scripts/smoke-first-pack.sh

<decision_trace>
- DOCS-01: README must provide a short install-to-first-pack path that matches current CLI flags and does not rely on a source checkout for normal users.
- Phase 7 decision: Keep README short; put deeper operational reference in dedicated docs.
- Phase 5 decision: Prebuilt archives and checksums are the normal v1.1.0 install path; source builds are fallbacks.
- Phase 6 decision: First-pack user path uses `ctxpack`/`CTXPACK_BIN`, explicit `--repo`, `setup-check`, deterministic MCP proof, `prepare-task`, and `get-pack`.
- Scope guard: Do not edit Rust source. Do not add package-manager, cloud, UI, retrieval, or setup behavior.
</decision_trace>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Rewrite README quickstart around installed binary flow</name>
  <files>README.md</files>
  <action>Update README so the first user-facing path after install is a concise "Install to first pack" flow using the installed `ctxpack` binary, not `cargo run`. Keep the existing development commands in a clearly separate Development section. The quickstart must show: `ctxpack --version`, `ctxpack --help`, choose a repo path, `ctxpack init --repo "$REPO" --cursor --claude --opencode`, `ctxpack setup-check --repo "$REPO" --cursor --claude --opencode`, `ctxpack prepare-task "..." --repo "$REPO" --mode bug-fix --path ...`, and `ctxpack get-pack "..." --repo "$REPO" --mode bug-fix --budget brief`. Link to `docs/release.md` for install/release details and to `docs/quickstart.md` for the longer first-pack walkthrough. Remove README examples that present `cargo run -p ctxpack -- init`, `cargo run -p ctxpack -- serve-mcp`, or other source-checkout commands as the normal user setup path; leave source-build commands only in development/fallback contexts.</action>
  <verify>
    <automated>python3 -c "from pathlib import Path; r=Path('README.md').read_text(); assert 'docs/quickstart.md' in r; assert 'ctxpack init --repo' in r; assert 'ctxpack setup-check --repo' in r; assert 'ctxpack prepare-task' in r; assert 'ctxpack get-pack' in r; assert 'cargo run -p ctxpack -- init' not in r; assert 'cargo run -p ctxpack -- serve-mcp' not in r"</automated>
  </verify>
  <done>README gives a normal-user first-pack path with installed-binary commands and current flags.</done>
</task>

<task type="auto">
  <name>Task 2: Add detailed first-pack quickstart</name>
  <files>docs/quickstart.md</files>
  <action>Create `docs/quickstart.md` as the task-oriented first-pack reference for DOCS-01. Include sections for prerequisites, install verification, repo initialization, read-only setup validation, deterministic MCP proof context, first `prepare-task`, first `get-pack`, and how to interpret pack options/session-scoped resource URIs. Normal-user commands must use `ctxpack`, explicit `--repo`, and current flags from `ctxpack --help`; source-checkout scripts such as `scripts/smoke-first-pack.sh` may appear only in a clearly labeled maintainer/source-checkout validation note. Do not claim Cursor/OpenCode real-client tool-call proof and do not add new product behavior.</action>
  <verify>
    <automated>python3 -c "from pathlib import Path; q=Path('docs/quickstart.md').read_text(); required=['ctxpack --version','ctxpack --help','ctxpack init --repo','ctxpack setup-check --repo','ctxpack prepare-task','ctxpack get-pack','--budget brief','explicit `--repo`','session-scoped']; missing=[s for s in required if s not in q]; assert not missing, missing; assert 'cargo run -p ctxpack -- init' not in q; assert 'Cursor real-client' not in q; assert 'OpenCode real-client' not in q"</automated>
  </verify>
  <done>`docs/quickstart.md` lets users complete the first-pack flow without reading source or old development-only README examples.</done>
</task>

</tasks>

<verification>
- `python3 -c "from pathlib import Path; r=Path('README.md').read_text(); q=Path('docs/quickstart.md').read_text(); assert 'ctxpack init --repo' in r and 'ctxpack get-pack' in q"`
- `cargo run -p ctxpack -- --help`
</verification>

<success_criteria>
Plan 01 is complete when DOCS-01 is satisfied by README and `docs/quickstart.md`: users can install or verify `ctxpack`, initialize a repo, validate setup, and request a first plan/pack with installed-binary commands and explicit repo arguments.
</success_criteria>

<output>
After completion, create `.planning/phases/07-documentation-troubleshooting/07-documentation-troubleshooting-01-SUMMARY.md`
</output>
