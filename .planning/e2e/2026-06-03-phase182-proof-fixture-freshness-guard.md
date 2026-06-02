# Phase 182: Proof Fixture Freshness Guard

## Goal

Close the proof-infrastructure gap exposed by Phase 181: the old pinned
VeriSchema fixture SHA is no longer reachable from the remote, so clean
four-repo proof can look like a ranking problem when it is actually missing
fixture evidence.

## Change

- `scripts/prepare-proof-fixtures.sh` now checks each requested revision with
  `git cat-file -e <revision>^{commit}` before checkout.
- The script writes one source-free readiness report per fixture through
  `CTXHELM_PROOF_FIXTURE_REPORT_DIR`:
  - requested revision
  - checked-out head
  - revision availability
  - dirty count
  - privacy status
- Reports omit source snippets, commit subjects, diffs, terminal logs, prompts,
  and remote URLs.
- `scripts/release-gate.sh` now verifies clean proof fixture directories contain
  the configured `head` and have `HEAD` checked out at that commit before it
  treats the clean cold fixture proof as available.

## Focused Proof

```bash
cargo test -p ctxhelm --test release_packaging \
  prepare_proof_fixtures_reports_unavailable_revisions_source_free -- --nocapture

cargo test -p ctxhelm --test release_packaging \
  release_gate_script_contract -- --nocapture
```

Results:

- `prepare_proof_fixtures_reports_unavailable_revisions_source_free` passed.
- `release_gate_script_contract` passed.

## Live Fixture Probe

```bash
rm -rf /tmp/ctxhelm-rd/phase182-fixture-reports
CTXHELM_PROOF_FIXTURE_REPORT_DIR=/tmp/ctxhelm-rd/phase182-fixture-reports \
  bash scripts/prepare-proof-fixtures.sh || true
```

Result:

- RefactoringMiner: `ready`, requested revision available.
- ctxhelm: `ready`, requested revision available.
- ReAgent: `ready`, requested revision available.
- VeriSchema: `blocked`, `revisionAvailable: false`.
- VeriSchema requested revision:
  `b5cfb2a551d026514f505c45863db31277bcd1ad`.
- Current VeriSchema fixture head:
  `33578667304472d3d58be2301dcc31d07e5c9bc4`.

## Interpretation

This phase does not claim a retrieval-quality lift. It makes proof evidence
harder to misuse. A clean fixture proof now cannot silently run against a stale
or wrong detached checkout, and an unreachable pinned object is reported as a
fixture freshness blocker instead of surfacing as a late checkout failure or a
missing benchmark report.
