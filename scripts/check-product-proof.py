#!/usr/bin/env python3
"""Validate source-free product proof JSON for release-gate use."""

import json
import pathlib
import sys


def fail(message: str) -> None:
    raise SystemExit(message)


def is_perfect_ceiling_match(verdict: dict) -> bool:
    return (
        verdict.get("status") == "match"
        and float(verdict.get("contextRecallAt10", 0.0)) >= 0.999
        and float(verdict.get("lexicalContextRecallAt10", 0.0)) >= 0.999
        and float(verdict.get("protectedEvidenceTargetMissRateAt10", 1.0)) == 0.0
        and verdict.get("allFileDivergenceExplained") is True
    )


def current_reachable_gap_summaries(report: dict):
    repositories = report.get("benchmarkReport", {}).get("repositories", [])
    if not isinstance(repositories, list):
        fail("embedded benchmark repositories were not a list")
    for repository in repositories:
        repo_name = repository.get("name") or repository.get("repoId") or "<unknown>"
        repo_report = repository.get("report", {})
        if not isinstance(repo_report, dict):
            fail("embedded benchmark repository report was missing: " + str(repo_name))
        gaps = repo_report.get("retrievalGapSummaries", [])
        if not isinstance(gaps, list):
            fail("retrievalGapSummaries was not a list for " + str(repo_name))
        for index, gap in enumerate(gaps):
            if gap.get("targetStatus") == "currentReachable":
                yield repo_name, index, gap


def validate_resource_backed_gap_summaries(report: dict) -> None:
    for repo_name, index, gap in current_reachable_gap_summaries(report):
        uri = gap.get("contextAreaResourceUri")
        if not isinstance(uri, str) or not uri.startswith("ctxpack://repo/context-area/"):
            fail(
                "current reachable retrieval gap lacked context-area resource URI: "
                + str(repo_name)
                + " gap "
                + str(index)
            )
        next_reads = gap.get("nextReadPaths")
        if not isinstance(next_reads, list) or not next_reads:
            fail(
                "current reachable retrieval gap lacked next-read paths: "
                + str(repo_name)
                + " gap "
                + str(index)
            )
        if any(not isinstance(path, str) or not path.strip() for path in next_reads):
            fail(
                "current reachable retrieval gap had invalid next-read path: "
                + str(repo_name)
                + " gap "
                + str(index)
            )


BROAD_FIXED_CORPUS_ID = "phase92-area-aware-gap-taxonomy-2026-05-31"
BROAD_FIXED_CORPUS_FLOORS = {
    "RefactoringMiner": {
        "fileRecallAt10": 0.6,
        "sourceRecallAt10": 1.0,
        "testRecallAt10": 1.0,
        "effectiveValidationRecallAt10": 1.0,
    },
    "ctxpack": {
        "fileRecallAt10": 0.47460318,
        "sourceRecallAt10": 0.7166667,
        "broadContextAreaRecall": 1.0,
    },
    "ReAgent": {
        "fileRecallAt10": 0.5,
        "sourceRecallAt10": 1.0,
        "testRecallAt10": 1.0,
        "effectiveValidationRecallAt10": 1.0,
    },
    "VeriSchema": {
        "fileRecallAt10": 0.18449473,
        "sourceRecallAt10": 0.31067252,
        "testRecallAt10": 0.7089947,
        "effectiveValidationRecallAt10": 1.0,
        "broadContextAreaRecall": 0.71851856,
    },
}


def validate_broad_fixed_corpus_floors(report: dict) -> None:
    benchmark = report.get("benchmarkReport", {})
    if benchmark.get("corpusId") != BROAD_FIXED_CORPUS_ID:
        return
    repositories = {
        repository.get("name"): repository.get("report", {})
        for repository in benchmark.get("repositories", [])
        if isinstance(repository, dict)
    }
    for repo_name, floors in BROAD_FIXED_CORPUS_FLOORS.items():
        repo_report = repositories.get(repo_name)
        if not isinstance(repo_report, dict):
            fail("broad fixed corpus proof was missing repository: " + repo_name)
        for field, floor in floors.items():
            try:
                value = float(repo_report.get(field))
            except (TypeError, ValueError):
                fail(
                    "broad fixed corpus proof was missing metric "
                    + repo_name
                    + "."
                    + field
                )
            if value + 0.000001 < floor:
                fail(
                    "broad fixed corpus metric regressed below floor: "
                    + repo_name
                    + "."
                    + field
                    + " "
                    + str(value)
                    + " < "
                    + str(floor)
                )


def main() -> None:
    if len(sys.argv) != 2:
        fail("usage: check-product-proof.py <product-proof.json>")

    report = json.loads(pathlib.Path(sys.argv[1]).read_text())
    if not report.get("privacyStatus", {}).get("localOnly"):
        fail("product proof privacyStatus.localOnly was not true")
    if not report.get("benchmarkReport", {}).get("privacyStatus", {}).get("localOnly"):
        fail("embedded benchmark privacyStatus.localOnly was not true")
    validate_resource_backed_gap_summaries(report)
    validate_broad_fixed_corpus_floors(report)
    if not report.get("headlineMetrics"):
        fail("product proof headlineMetrics were empty")

    summary = report.get("v23EvalSummary", {})
    if not summary.get("fixedCorpusId"):
        fail("product proof v23EvalSummary.fixedCorpusId was empty")
    if not isinstance(summary.get("pairedBaselineVerdicts"), list):
        fail("product proof paired baseline verdicts were missing")

    feature_privacy = summary.get("featureExportPrivacy", {})
    if not feature_privacy.get("localOnly") or feature_privacy.get("sourceTextLogged"):
        fail("product proof feature-export privacy contract failed")

    learned_status = summary.get("learnedPolicyStatus", {})
    if not learned_status.get("defaultRequiresThresholds") or learned_status.get(
        "silentDefaultAllowed"
    ):
        fail("product proof learned-policy status contract failed")

    if "world-class claims require repeated lift" not in summary.get("proofBoundary", ""):
        fail("product proof boundary language was missing")

    release_gate = report.get("releaseGate", {})
    if release_gate.get("decision") != "promote":
        fail(
            "product proof releaseGate.decision was not promote: "
            + str(release_gate.get("decision"))
        )
    if not release_gate.get("defaultPromotionAllowed"):
        fail("product proof releaseGate.defaultPromotionAllowed was not true")

    verdicts = release_gate.get("corpusVerdicts")
    if not isinstance(verdicts, list) or not verdicts:
        fail("product proof releaseGate.corpusVerdicts were missing")
    for verdict in verdicts:
        for field in (
            "contextVsAllFileDeltaAt10",
            "lexicalContextVsAllFileDeltaAt10",
            "allFileDivergenceExplained",
        ):
            if field not in verdict:
                fail(
                    "product proof corpus verdict was missing divergence field "
                    + field
                    + ": "
                    + str(verdict.get("repository"))
                )
        if (
            float(verdict.get("lexicalDeltaAt10", 0.0)) < -0.03
            and verdict.get("allFileDivergenceExplained") is not True
        ):
            fail(
                "product proof corpus had unexplained all-file lexical divergence: "
                + str(verdict.get("repository"))
            )
        if verdict.get("status") != "beat" and not is_perfect_ceiling_match(verdict):
            fail(
                "product proof corpus did not beat lexical or reach a perfect lexical ceiling: "
                + str(verdict.get("repository"))
                + " status "
                + str(verdict.get("status"))
            )


if __name__ == "__main__":
    main()
