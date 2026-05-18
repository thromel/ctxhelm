# Pitfalls Research: v2.2 Release & Distribution Hardening

## Common Mistakes

- Running release packaging from a dirty checkout and treating the artifact as shippable.
- Bundling `.ctxpack`, target directories, git internals, traces, logs, or machine-local paths.
- Treating optional real-client evidence as a required gate on machines that cannot run those clients.
- Writing installer docs that mutate global Codex/Claude/Cursor/OpenCode config.
- Claiming package-manager support before metadata and verification are ready.

## Prevention

- Keep a strict clean-checkout release gate.
- Expand artifact audit and selected-binary smoke coverage.
- Document proof levels explicitly.
- Keep install docs copy/paste-oriented and reviewable.
- Add release candidate status files before publishing.
