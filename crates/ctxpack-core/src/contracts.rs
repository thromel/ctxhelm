use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::privacy::PrivacyStatus;

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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelatedTest {
    pub path: String,
    pub reason: String,
    pub command: Option<String>,
    pub confidence: f32,
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
    pub task_type: TaskType,
    pub budget: PackBudget,
    pub sections: Vec<PackSection>,
    pub token_estimate: usize,
    pub confidence: f32,
    pub warnings: Vec<String>,
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

    #[test]
    fn task_type_serializes_as_snake_case() {
        let json = serde_json::to_string(&TaskType::BugFix).unwrap();
        assert_eq!(json, "\"bug_fix\"");
    }

    #[test]
    fn context_plan_serializes_with_camel_case_contract_fields() {
        let plan = ContextPlan {
            task_id: Uuid::nil(),
            task_type: TaskType::BugFix,
            confidence: 1.0,
            target_files: vec![TargetFile {
                path: "src/lib.rs".to_string(),
                reason: "public API surface".to_string(),
                line_range: Some(LineRange { start: 1, end: 7 }),
                confidence: 0.5,
            }],
            related_tests: vec![],
            recommended_commands: vec![],
            pack_options: vec![PackOption {
                budget: PackBudget::Brief,
                resource_uri: "ctxpack://packs/brief".to_string(),
            }],
            missing_info_questions: vec![],
            risk_flags: vec![],
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
                "confidence": 0.5
            }],
            "relatedTests": [],
            "recommendedCommands": [],
            "packOptions": [{
                "budget": "brief",
                "resourceUri": "ctxpack://packs/brief"
            }],
            "missingInfoQuestions": [],
            "riskFlags": [],
            "privacyStatus": {
                "localOnly": true,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false,
                "redactionsApplied": 0
            }
        });

        assert_eq!(value, expected);

        assert_eq!(value["taskId"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(value["taskType"], "bug_fix");
        assert!(value["targetFiles"].is_array());
        assert_eq!(value["targetFiles"][0]["lineRange"]["start"], 1);
        assert_eq!(value["packOptions"][0]["budget"], "brief");
        assert_eq!(
            value["packOptions"][0]["resourceUri"],
            "ctxpack://packs/brief"
        );
        assert_eq!(value["privacyStatus"]["localOnly"], true);

        assert!(value.get("task_id").is_none());
        assert!(value.get("target_files").is_none());
        assert!(value.get("privacy_status").is_none());
    }

    #[test]
    fn context_pack_serializes_with_sections_and_privacy_status() {
        let pack = ContextPack {
            id: Uuid::nil(),
            task_id: Uuid::nil(),
            task_type: TaskType::BugFix,
            budget: PackBudget::Brief,
            sections: vec![PackSection {
                title: "Task".to_string(),
                kind: "task".to_string(),
                content: "Fix auth redirect".to_string(),
            }],
            token_estimate: 12,
            confidence: 0.7,
            warnings: vec!["one warning".to_string()],
            privacy_status: PrivacyStatus::local_only(),
        };

        let value = serde_json::to_value(&pack).unwrap();

        assert_eq!(value["id"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(value["taskId"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(value["taskType"], "bug_fix");
        assert_eq!(value["budget"], "brief");
        assert_eq!(value["sections"][0]["title"], "Task");
        assert_eq!(value["tokenEstimate"], 12);
        assert_eq!(value["privacyStatus"]["localOnly"], true);

        assert!(value.get("task_id").is_none());
        assert!(value.get("token_estimate").is_none());
    }

    #[test]
    fn eval_trace_serializes_without_source_text() {
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

        assert_eq!(value["taskHash"], "hash-1");
        assert_eq!(value["taskType"], "bug_fix");
        assert_eq!(value["packId"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(value["sourceTextLogged"], false);
        assert!(value.get("task").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(value.get("source_text").is_none());
    }
}
