# Research Summary: v2.2 Release & Distribution Hardening

**Status:** Lightweight planning synthesis from the existing product vision and shipped release gates. No new external ecosystem research was required for this milestone.

## Stack Additions

- Keep the current Rust CLI/MCP workspace.
- Extend existing shell release scripts, docs checks, and release packaging tests.
- Add source-free release manifest/proof artifacts before adding external package channels.

## Feature Table Stakes

- Clean-checkout release gate.
- Versioned archives, checksums, release manifest, and artifact audit.
- Install and upgrade verification.
- Troubleshooting for PATH, MCP startup, stale binaries, incompatible state, and wrong cwd.
- Public first-pack examples and demo artifacts.
- Release readiness checklist and candidate lifecycle.

## Watch Outs

- Do not let package-manager prep imply Homebrew/crates.io are required release blockers.
- Do not mutate global agent config during install or setup.
- Keep deterministic protocol proof separate from optional real-client proof.
- Do not publish, tag, or upload as a side effect of local release validation.
