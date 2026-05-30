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
    )


def main() -> None:
    if len(sys.argv) != 2:
        fail("usage: check-product-proof.py <product-proof.json>")

    report = json.loads(pathlib.Path(sys.argv[1]).read_text())
    if not report.get("privacyStatus", {}).get("localOnly"):
        fail("product proof privacyStatus.localOnly was not true")
    if not report.get("benchmarkReport", {}).get("privacyStatus", {}).get("localOnly"):
        fail("embedded benchmark privacyStatus.localOnly was not true")
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
        if verdict.get("status") != "beat" and not is_perfect_ceiling_match(verdict):
            fail(
                "product proof corpus did not beat lexical or reach a perfect lexical ceiling: "
                + str(verdict.get("repository"))
                + " status "
                + str(verdict.get("status"))
            )


if __name__ == "__main__":
    main()
