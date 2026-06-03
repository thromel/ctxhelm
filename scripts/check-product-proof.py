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


def require_number(verdict: dict, field: str) -> float:
    if field not in verdict:
        fail(
            "product proof corpus verdict was missing source-recall field "
            + field
            + ": "
            + str(verdict.get("repository"))
        )
    try:
        return float(verdict.get(field))
    except (TypeError, ValueError):
        fail(
            "product proof corpus verdict had non-numeric source-recall field "
            + field
            + ": "
            + str(verdict.get("repository"))
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
        if not isinstance(uri, str) or not uri.startswith("ctxhelm://repo/context-area/"):
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


def iter_context_area_lists(value):
    if isinstance(value, dict):
        areas = value.get("contextAreas")
        if isinstance(areas, list):
            yield areas
        for child in value.values():
            yield from iter_context_area_lists(child)
    elif isinstance(value, list):
        for child in value:
            yield from iter_context_area_lists(child)


def validate_context_area_pressure_contract(report: dict) -> None:
    for areas in iter_context_area_lists(report):
        for area in areas:
            if not isinstance(area, dict):
                fail("contextAreas contained a non-object entry")
            area_name = area.get("area", "<unknown>")
            for field in ("coveragePercent", "inspectionPressure"):
                if not isinstance(area.get(field), int):
                    fail(
                        "context area was missing integer "
                        + field
                        + ": "
                        + str(area_name)
                    )
            breakdown = area.get("inspectionPressureBreakdown")
            if not isinstance(breakdown, dict):
                fail(
                    "context area was missing inspectionPressureBreakdown: "
                    + str(area_name)
                )
            for field in (
                "sourceLikeUnselected",
                "validationUnselected",
                "docsUnselected",
                "sourceLikeWeight",
                "validationWeight",
                "docsWeight",
                "total",
            ):
                if not isinstance(breakdown.get(field), int):
                    fail(
                        "context area pressure breakdown was missing integer "
                        + field
                        + ": "
                        + str(area_name)
                    )
            expected_total = (
                breakdown["sourceLikeUnselected"] * breakdown["sourceLikeWeight"]
                + breakdown["validationUnselected"] * breakdown["validationWeight"]
                + breakdown["docsUnselected"] * breakdown["docsWeight"]
            )
            if breakdown["total"] != expected_total:
                fail(
                    "context area pressure breakdown total was inconsistent: "
                    + str(area_name)
                )
            if area["inspectionPressure"] != breakdown["total"]:
                fail(
                    "context area inspectionPressure did not match breakdown total: "
                    + str(area_name)
                )


def iter_context_area_pressure_summaries(value):
    if isinstance(value, dict):
        summary = value.get("contextAreaPressureSummary")
        if isinstance(summary, dict):
            yield summary
        for child in value.values():
            yield from iter_context_area_pressure_summaries(child)
    elif isinstance(value, list):
        for child in value:
            yield from iter_context_area_pressure_summaries(child)


def validate_context_area_pressure_summaries(report: dict) -> None:
    for summary in iter_context_area_pressure_summaries(report):
        for field in (
            "contextAreaCount",
            "zeroSelectedAreaCount",
            "totalInspectionPressure",
            "sourceLikePressure",
            "validationPressure",
            "docsPressure",
        ):
            if not isinstance(summary.get(field), int):
                fail(
                    "context area pressure summary was missing integer "
                    + field
                )
        expected_total = (
            summary["sourceLikePressure"]
            + summary["validationPressure"]
            + summary["docsPressure"]
        )
        if summary["totalInspectionPressure"] != expected_total:
            fail("context area pressure summary total was inconsistent")
        if summary.get("sourceTextLogged") is not False:
            fail("context area pressure summary was not source-free")


BROAD_FIXED_CORPUS_ID = "phase92-area-aware-gap-taxonomy-2026-05-31"
BROAD_FIXED_CORPUS_FLOORS = {
    "RefactoringMiner": {
        "fileRecallAt10": 0.6,
        "sourceRecallAt10": 1.0,
        "testRecallAt10": 1.0,
        "effectiveValidationRecallAt10": 1.0,
    },
    "ctxhelm": {
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
    validate_context_area_pressure_contract(report)
    validate_context_area_pressure_summaries(report)
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
        require_number(verdict, "sourceRecallAt10")
        require_number(verdict, "lexicalSourceRecallAt10")
        source_delta = require_number(verdict, "sourceDeltaAt10")
        if source_delta < -0.03:
            fail(
                "product proof corpus had source recall regression: "
                + str(verdict.get("repository"))
                + " sourceDeltaAt10 "
                + str(source_delta)
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
