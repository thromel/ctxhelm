use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::privacy::PrivacyStatus;
use crate::repo::FileRole;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    BugFix,
    Feature,
    Refactor,
    Review,
    Test,
    Explain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LineRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TargetFile {
    pub path: String,
    pub reason: String,
    pub line_range: Option<LineRange>,
    pub confidence: f32,
    #[serde(default)]
    pub attribution: Vec<RetrievalEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelatedTest {
    pub path: String,
    pub reason: String,
    pub command: Option<String>,
    pub confidence: f32,
    #[serde(default)]
    pub attribution: Vec<RetrievalEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RetrievalCandidateKind {
    File,
    Test,
    Symbol,
    Doc,
    Commit,
    Config,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalSignalKind {
    Lexical,
    Symbol,
    Dependency,
    RelatedTest,
    Semantic,
    CoChange,
    CurrentDiff,
    History,
    Docs,
    Config,
    Anchor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalSignalScore {
    pub signal: RetrievalSignalKind,
    pub score: f32,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalEvidence {
    pub signal: RetrievalSignalKind,
    pub score: f32,
    pub reason_code: String,
    pub path: Option<String>,
    pub role: Option<FileRole>,
    pub edge_label: Option<String>,
    #[serde(default)]
    pub commit_ids: Vec<String>,
    #[serde(default)]
    pub commit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalCandidate {
    pub kind: RetrievalCandidateKind,
    pub path: Option<String>,
    pub role: Option<FileRole>,
    pub reason_code: String,
    pub confidence: f32,
    #[serde(default)]
    pub signal_scores: Vec<RetrievalSignalScore>,
    #[serde(default)]
    pub evidence: Vec<RetrievalEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub command: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PackBudget {
    Brief,
    Standard,
    Deep,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackOption {
    pub budget: PackBudget,
    pub resource_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RiskFlag {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CacheStatusKind {
    Hit,
    Miss,
    Rebuilt,
    WriteFailed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CacheStatus {
    pub status: CacheStatusKind,
    pub path: Option<String>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TraceStatusKind {
    Written,
    Skipped,
    WriteFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TraceStatus {
    pub status: TraceStatusKind,
    pub path: Option<String>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContextPlan {
    pub task_id: Uuid,
    pub task_type: TaskType,
    pub confidence: f32,
    pub target_files: Vec<TargetFile>,
    pub related_tests: Vec<RelatedTest>,
    pub recommended_commands: Vec<Command>,
    pub pack_options: Vec<PackOption>,
    pub missing_info_questions: Vec<String>,
    pub risk_flags: Vec<RiskFlag>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    #[serde(default)]
    pub retrieval_candidates: Vec<RetrievalCandidate>,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackSection {
    pub title: String,
    pub kind: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContextPack {
    pub id: Uuid,
    pub task_id: Uuid,
    pub repo_id: String,
    pub task_hash: String,
    pub task_type: TaskType,
    pub target_agent: String,
    pub budget: PackBudget,
    pub sections: Vec<PackSection>,
    pub token_estimate: usize,
    pub confidence: f32,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EvalTrace {
    pub id: Uuid,
    pub repo_id: String,
    pub task_hash: String,
    pub task_type: TaskType,
    pub pack_id: Option<Uuid>,
    pub target_agent: String,
    pub budget: Option<PackBudget>,
    pub recommended_files: Vec<String>,
    pub recommended_tests: Vec<String>,
    pub recommended_commands: Vec<String>,
    pub created_at_unix_seconds: u64,
    pub source_text_logged: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileRole;

    #[test]
    fn task_type_serializes_as_snake_case() {
        let json = serde_json::to_string(&TaskType::BugFix).unwrap();
        assert_eq!(json, "\"bug_fix\"");
    }

    #[test]
    fn context_plan_public_json_shape_is_stable() {
        let plan = ContextPlan {
            task_id: Uuid::nil(),
            task_type: TaskType::BugFix,
            confidence: 1.0,
            target_files: vec![TargetFile {
                path: "src/lib.rs".to_string(),
                reason: "public API surface".to_string(),
                line_range: Some(LineRange { start: 1, end: 7 }),
                confidence: 0.5,
                attribution: Vec::new(),
            }],
            related_tests: vec![],
            recommended_commands: vec![],
            pack_options: vec![PackOption {
                budget: PackBudget::Brief,
                resource_uri: "ctxpack://packs/brief".to_string(),
            }],
            missing_info_questions: vec![],
            risk_flags: vec![],
            diagnostics: vec![Diagnostic {
                code: "source_policy_excluded".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: "Skipped policy-excluded source file".to_string(),
                paths: vec![".env".to_string()],
                count: 1,
            }],
            retrieval_candidates: Vec::new(),
            privacy_status: PrivacyStatus::local_only(),
        };

        let value = serde_json::to_value(&plan).unwrap();
        let expected = serde_json::json!({
            "taskId": "00000000-0000-0000-0000-000000000000",
            "taskType": "bug_fix",
            "confidence": 1.0,
            "targetFiles": [{
                "path": "src/lib.rs",
                "reason": "public API surface",
                "lineRange": {
                    "start": 1,
                    "end": 7
                },
                "confidence": 0.5,
                "attribution": []
            }],
            "relatedTests": [],
            "recommendedCommands": [],
            "packOptions": [{
                "budget": "brief",
                "resourceUri": "ctxpack://packs/brief"
            }],
            "missingInfoQuestions": [],
            "riskFlags": [],
            "diagnostics": [{
                "code": "source_policy_excluded",
                "severity": "warning",
                "message": "Skipped policy-excluded source file",
                "paths": [".env"],
                "count": 1
            }],
            "retrievalCandidates": [],
            "privacyStatus": {
                "localOnly": true,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false,
                "redactionsApplied": 0
            }
        });

        assert_eq!(value, expected);

        let object = value.as_object().unwrap();
        for key in [
            "taskId",
            "taskType",
            "confidence",
            "targetFiles",
            "relatedTests",
            "recommendedCommands",
            "packOptions",
            "missingInfoQuestions",
            "riskFlags",
            "diagnostics",
            "retrievalCandidates",
            "privacyStatus",
        ] {
            assert!(object.contains_key(key), "missing public field {key}");
        }
        assert_eq!(value["taskId"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(value["taskType"], "bug_fix");
        assert!(value["targetFiles"].is_array());
        assert_eq!(value["targetFiles"][0]["lineRange"]["start"], 1);
        assert_eq!(value["packOptions"][0]["budget"], "brief");
        assert_eq!(
            value["packOptions"][0]["resourceUri"],
            "ctxpack://packs/brief"
        );
        assert_eq!(value["diagnostics"][0]["severity"], "warning");
        assert_eq!(value["diagnostics"][0]["paths"][0], ".env");
        assert_eq!(value["privacyStatus"]["localOnly"], true);

        assert!(value.get("task_id").is_none());
        assert!(value.get("task_type").is_none());
        assert!(value.get("target_files").is_none());
        assert!(value.get("related_tests").is_none());
        assert!(value.get("risk_flags").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(value.get("prompt").is_none());
        assert!(value.get("privacy_status").is_none());
    }

    #[test]
    fn retrieval_contracts_serialize_additive_camel_case_fields() {
        let attribution = vec![RetrievalEvidence {
            signal: RetrievalSignalKind::Lexical,
            score: 0.8,
            reason_code: "lexical_match".to_string(),
            path: Some("src/lib.rs".to_string()),
            role: Some(FileRole::Source),
            edge_label: Some("imports".to_string()),
            commit_ids: vec!["abc1234".to_string()],
            commit_count: 1,
        }];
        let plan = ContextPlan {
            task_id: Uuid::nil(),
            task_type: TaskType::BugFix,
            confidence: 1.0,
            target_files: vec![TargetFile {
                path: "src/lib.rs".to_string(),
                reason: "public API surface".to_string(),
                line_range: None,
                confidence: 0.8,
                attribution: attribution.clone(),
            }],
            related_tests: vec![RelatedTest {
                path: "tests/lib_test.rs".to_string(),
                reason: "related test".to_string(),
                command: Some("cargo test".to_string()),
                confidence: 0.7,
                attribution: attribution.clone(),
            }],
            recommended_commands: vec![],
            pack_options: vec![],
            missing_info_questions: vec![],
            risk_flags: vec![],
            diagnostics: vec![],
            retrieval_candidates: vec![RetrievalCandidate {
                kind: RetrievalCandidateKind::File,
                path: Some("src/lib.rs".to_string()),
                role: Some(FileRole::Source),
                reason_code: "lexical_match".to_string(),
                confidence: 0.8,
                signal_scores: vec![RetrievalSignalScore {
                    signal: RetrievalSignalKind::Lexical,
                    score: 0.8,
                    weight: 1.0,
                }],
                evidence: attribution,
            }],
            privacy_status: PrivacyStatus::local_only(),
        };

        let value = serde_json::to_value(&plan).unwrap();

        assert!(value.get("retrievalCandidates").is_some());
        assert!(value["targetFiles"][0].get("attribution").is_some());
        assert!(value["relatedTests"][0].get("attribution").is_some());
        assert_eq!(value["retrievalCandidates"][0]["kind"], "file");
        assert_eq!(
            value["retrievalCandidates"][0]["signalScores"][0]["signal"],
            "lexical"
        );
        assert_eq!(
            value["targetFiles"][0]["attribution"][0]["reasonCode"],
            "lexical_match"
        );
    }

    #[test]
    fn retrieval_additive_fields_default_when_missing_from_old_json() {
        let old_json = serde_json::json!({
            "taskId": "00000000-0000-0000-0000-000000000000",
            "taskType": "bug_fix",
            "confidence": 1.0,
            "targetFiles": [{
                "path": "src/lib.rs",
                "reason": "public API surface",
                "lineRange": null,
                "confidence": 0.5
            }],
            "relatedTests": [{
                "path": "tests/lib_test.rs",
                "reason": "related test",
                "command": "cargo test",
                "confidence": 0.5
            }],
            "recommendedCommands": [],
            "packOptions": [],
            "missingInfoQuestions": [],
            "riskFlags": [],
            "diagnostics": [],
            "privacyStatus": {
                "localOnly": true,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false,
                "redactionsApplied": 0
            }
        });

        let plan: ContextPlan = serde_json::from_value(old_json).unwrap();

        assert!(plan.retrieval_candidates.is_empty());
        assert!(plan.target_files[0].attribution.is_empty());
        assert!(plan.related_tests[0].attribution.is_empty());
    }

    #[test]
    fn retrieval_attribution_serializes_without_source_or_prompt_text_fields() {
        let evidence = RetrievalEvidence {
            signal: RetrievalSignalKind::CoChange,
            score: 0.6,
            reason_code: "changed_together".to_string(),
            path: Some("src/lib.rs".to_string()),
            role: Some(FileRole::Source),
            edge_label: None,
            commit_ids: vec!["abc1234".to_string()],
            commit_count: 3,
        };

        let serialized = serde_json::to_string(&evidence).unwrap();

        for forbidden in [
            "taskText",
            "task_text",
            "sourceSnippet",
            "source_snippet",
            "symbolSignature",
            "symbol_signature",
            "commitSubject",
            "commit_subject",
            "prompt",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "attribution leaked forbidden field {forbidden}: {serialized}"
            );
        }
    }

    #[test]
    fn retrieval_candidate_kind_serializes_all_phase_three_required_kinds() {
        let cases = [
            (RetrievalCandidateKind::File, "file"),
            (RetrievalCandidateKind::Test, "test"),
            (RetrievalCandidateKind::Symbol, "symbol"),
            (RetrievalCandidateKind::Doc, "doc"),
            (RetrievalCandidateKind::Commit, "commit"),
            (RetrievalCandidateKind::Config, "config"),
        ];

        for (kind, expected) in cases {
            assert_eq!(serde_json::to_value(kind).unwrap(), expected);
        }
    }

    #[test]
    fn context_pack_public_json_shape_is_stable() {
        let pack = ContextPack {
            id: Uuid::nil(),
            task_id: Uuid::nil(),
            repo_id: "repo-1".to_string(),
            task_hash: "hash-1".to_string(),
            task_type: TaskType::BugFix,
            target_agent: "codex".to_string(),
            budget: PackBudget::Brief,
            sections: vec![PackSection {
                title: "Task".to_string(),
                kind: "task".to_string(),
                content: "Fix auth redirect".to_string(),
            }],
            token_estimate: 12,
            confidence: 0.7,
            warnings: vec!["one warning".to_string()],
            diagnostics: vec![Diagnostic {
                code: "source_unreadable".to_string(),
                severity: DiagnosticSeverity::Error,
                message: "Could not read requested source file".to_string(),
                paths: vec!["src/lib.rs".to_string()],
                count: 1,
            }],
            privacy_status: PrivacyStatus::local_only(),
        };

        let value = serde_json::to_value(&pack).unwrap();

        let object = value.as_object().unwrap();
        for key in [
            "id",
            "taskId",
            "repoId",
            "taskHash",
            "taskType",
            "targetAgent",
            "budget",
            "sections",
            "tokenEstimate",
            "confidence",
            "warnings",
            "diagnostics",
            "privacyStatus",
        ] {
            assert!(object.contains_key(key), "missing public field {key}");
        }
        assert_eq!(value["id"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(value["taskId"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(value["repoId"], "repo-1");
        assert_eq!(value["taskHash"], "hash-1");
        assert_eq!(value["taskType"], "bug_fix");
        assert_eq!(value["targetAgent"], "codex");
        assert_eq!(value["budget"], "brief");
        assert_eq!(value["sections"][0]["title"], "Task");
        assert_eq!(value["tokenEstimate"], 12);
        assert_eq!(value["diagnostics"][0]["severity"], "error");
        assert_eq!(value["diagnostics"][0]["paths"][0], "src/lib.rs");
        assert_eq!(value["privacyStatus"]["localOnly"], true);

        assert!(value.get("task_id").is_none());
        assert!(value.get("repo_id").is_none());
        assert!(value.get("task_hash").is_none());
        assert!(value.get("target_agent").is_none());
        assert!(value.get("token_estimate").is_none());
        assert!(value.get("riskFlags").is_none());
        assert!(value.get("source").is_none());
        assert!(value.get("task").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(value.get("prompt").is_none());
    }

    #[test]
    fn diagnostics_public_json_shape_is_source_free_and_backward_compatible() {
        let diagnostic = Diagnostic {
            code: "cache_write_failed".to_string(),
            severity: DiagnosticSeverity::Info,
            message: "Inventory cache was not persisted".to_string(),
            paths: vec![".ctxpack/repos/repo-1/inventory.json".to_string()],
            count: 1,
        };

        let value = serde_json::to_value(&diagnostic).unwrap();
        let object = value.as_object().unwrap();
        for key in ["code", "severity", "message", "paths", "count"] {
            assert!(object.contains_key(key), "missing public field {key}");
        }
        assert_eq!(value["severity"], "info");
        assert!(value.get("source").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(value.get("snippet").is_none());
        assert!(value.get("prompt").is_none());

        let old_plan_json = serde_json::json!({
            "taskId": "00000000-0000-0000-0000-000000000000",
            "taskType": "bug_fix",
            "confidence": 1.0,
            "targetFiles": [],
            "relatedTests": [],
            "recommendedCommands": [],
            "packOptions": [],
            "missingInfoQuestions": [],
            "riskFlags": [],
            "privacyStatus": {
                "localOnly": true,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false,
                "redactionsApplied": 0
            }
        });
        let plan: ContextPlan = serde_json::from_value(old_plan_json).unwrap();
        assert!(plan.diagnostics.is_empty());

        let old_pack_json = serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "taskId": "00000000-0000-0000-0000-000000000000",
            "repoId": "repo-1",
            "taskHash": "hash-1",
            "taskType": "bug_fix",
            "targetAgent": "codex",
            "budget": "brief",
            "sections": [],
            "tokenEstimate": 0,
            "confidence": 1.0,
            "warnings": [],
            "privacyStatus": {
                "localOnly": true,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false,
                "redactionsApplied": 0
            }
        });
        let pack: ContextPack = serde_json::from_value(old_pack_json).unwrap();
        assert!(pack.diagnostics.is_empty());
    }

    #[test]
    fn eval_trace_public_json_shape_is_source_free() {
        let trace = EvalTrace {
            id: Uuid::nil(),
            repo_id: "repo-1".to_string(),
            task_hash: "hash-1".to_string(),
            task_type: TaskType::BugFix,
            pack_id: Some(Uuid::nil()),
            target_agent: "codex".to_string(),
            budget: Some(PackBudget::Brief),
            recommended_files: vec!["src/auth.ts".to_string()],
            recommended_tests: vec!["tests/auth.test.ts".to_string()],
            recommended_commands: vec!["pnpm test tests/auth.test.ts".to_string()],
            created_at_unix_seconds: 1,
            source_text_logged: false,
        };

        let value = serde_json::to_value(&trace).unwrap();

        let object = value.as_object().unwrap();
        for key in [
            "id",
            "repoId",
            "taskHash",
            "taskType",
            "packId",
            "targetAgent",
            "budget",
            "recommendedFiles",
            "recommendedTests",
            "recommendedCommands",
            "createdAtUnixSeconds",
            "sourceTextLogged",
        ] {
            assert!(object.contains_key(key), "missing public field {key}");
        }
        assert_eq!(value["repoId"], "repo-1");
        assert_eq!(value["taskHash"], "hash-1");
        assert_eq!(value["taskType"], "bug_fix");
        assert_eq!(value["packId"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(value["sourceTextLogged"], false);
        assert!(value.get("repo_id").is_none());
        assert!(value.get("task_hash").is_none());
        assert!(value.get("target_agent").is_none());
        assert!(value.get("task").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(value.get("source_text").is_none());
        assert!(value.get("source_text_logged").is_none());
    }
}
