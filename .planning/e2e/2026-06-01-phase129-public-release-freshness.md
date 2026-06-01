# Phase 129: Public Release Freshness Proof

## Purpose

Phase 129 closes a production-readiness honesty gap: the public `v1.1.0`
archive release is verified, but current `main` has continued production
hardening after that archive was published. Release notes and readiness claims
need a source-free way to distinguish "the public archive is verified" from
"the public archive contains the current branch tip."

## Implementation

- Added `scripts/check-public-release-freshness.sh`.
- Wired syntax and fixture checks into `scripts/smoke-release-governance.sh`.
- Added Rust release-packaging contract coverage for the script.
- Added release, distribution, and release-governance documentation.
- Added release-doc consistency checks for the freshness script, `commitsAhead`,
  `releaseTargetCommit`, and `--require-current`.

The checker is read-only. It does not publish, create tags, upload assets,
install globally, mutate agent configuration, or read source text.

## Evidence

Command:

```bash
bash scripts/check-public-release-freshness.sh \
  --tag v1.1.0 \
  --repo thromel/ctxhelm \
  --output .ctxhelm/e2e/phase129-public-release-freshness.json
```

Durable artifact:

```text
.ctxhelm/e2e/phase129-public-release-freshness.json
```

Result:

```text
status: outdated
releaseTargetCommit: 68383cbfc2fff00c4f53fbd2b7bf90527ac4bd7e
currentCommit: a07e6be31a0605af1810e79cb18b34245fa7def0
gitRelation: current_descends_from_release
commitsAhead: 19
sourceFree: true
privacyStatus.sourceTextLogged: false
```

## Interpretation

The public `v1.1.0` archive remains a verified public archive release, but it is
not current with the latest production-hardening work on `main`. Future public
announcements that claim current-main behavior should either publish a refreshed
archive or run the checker with `--require-current` and pass.

## Validation

This phase is validated by the release governance smoke, release docs check, the
focused Rust packaging test, full workspace tests, CLI help, and JSON parsing of
the durable proof artifact.
