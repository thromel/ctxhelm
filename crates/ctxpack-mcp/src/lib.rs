mod prompts;
mod protocol;
mod resources;
mod schemas;
mod tools;

pub use protocol::{run_server, run_stdio_server};

pub const PLANNED_MCP_TOOL_NAMES: &[&str] = &[
    "prepare_task",
    "search",
    "related",
    "get_pack",
    "related_tests",
    "current_diff",
];

pub const IMPLEMENTED_MCP_TOOL_NAMES: &[&str] = &[
    "prepare_task",
    "search",
    "related",
    "get_pack",
    "related_tests",
    "current_diff",
];

#[cfg(test)]
use protocol::handle_line;
#[cfg(test)]
use resources::{clear_pack_resource_cache, pack_resource_cache_len, pack_resource_cache_limit};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::sync::{Mutex, OnceLock};

    struct FixtureRepo {
        _temp: tempfile::TempDir,
        repo: std::path::PathBuf,
        home: std::path::PathBuf,
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn planned_mcp_tool_surface_stays_small() {
        assert_eq!(PLANNED_MCP_TOOL_NAMES.len(), 6);
    }

    #[test]
    fn implemented_tool_surface_stays_small() {
        assert_eq!(IMPLEMENTED_MCP_TOOL_NAMES, PLANNED_MCP_TOOL_NAMES);
    }

    #[test]
    fn initialize_public_capabilities_are_stable() {
        let response =
            handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#).unwrap();

        assert_eq!(response["result"]["serverInfo"]["name"], "ctxpack");
        assert_eq!(
            response["result"]["capabilities"]["tools"]["listChanged"],
            false
        );
        assert_eq!(
            response["result"]["capabilities"]["resources"]["listChanged"],
            false
        );
        assert_eq!(
            response["result"]["capabilities"]["prompts"]["listChanged"],
            false
        );
    }

    #[test]
    fn initialize_returns_tool_capability() {
        let response =
            handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#).unwrap();

        assert_eq!(response["result"]["serverInfo"]["name"], "ctxpack");
        assert_eq!(
            response["result"]["capabilities"]["tools"]["listChanged"],
            false
        );
        assert_eq!(
            response["result"]["capabilities"]["resources"]["listChanged"],
            false
        );
        assert_eq!(
            response["result"]["capabilities"]["prompts"]["listChanged"],
            false
        );
    }

    #[test]
    fn initialized_notification_returns_no_response() {
        let response =
            handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#);

        assert!(response.is_none());
    }

    #[test]
    fn tools_list_only_exposes_implemented_tools() {
        let response =
            handle_line(r#"{"jsonrpc":"2.0","id":"tools","method":"tools/list","params":{}}"#)
                .unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();

        assert_eq!(tools.len(), 6);
        assert_eq!(tools[0]["name"], "prepare_task");
        assert_eq!(tools[0]["inputSchema"]["required"][0], "task");
        assert_eq!(
            tools[0]["inputSchema"]["properties"]["paths"]["items"]["type"],
            "string"
        );
        assert_eq!(
            tools[0]["inputSchema"]["properties"]["includeCurrentDiff"]["type"],
            "boolean"
        );
        assert_eq!(
            tools[0]["inputSchema"]["properties"]["recordTrace"]["type"],
            "boolean"
        );
        assert_eq!(
            tools[0]["inputSchema"]["properties"]["semanticProvider"]["type"],
            "string"
        );
        assert_eq!(tools[1]["name"], "search");
        assert_eq!(
            tools[1]["inputSchema"]["properties"]["semanticDimensions"]["maximum"],
            4096
        );
        assert_eq!(tools[2]["name"], "related");
        assert_eq!(
            tools[2]["inputSchema"]["properties"]["includeCurrentDiff"]["type"],
            "boolean"
        );
        assert_eq!(tools[3]["name"], "get_pack");
        assert_eq!(tools[3]["inputSchema"]["required"][0], "task");
        assert_eq!(
            tools[3]["inputSchema"]["properties"]["paths"]["items"]["type"],
            "string"
        );
        assert_eq!(
            tools[3]["inputSchema"]["properties"]["includeCurrentDiff"]["type"],
            "boolean"
        );
        assert_eq!(
            tools[3]["inputSchema"]["properties"]["recordTrace"]["type"],
            "boolean"
        );
        assert_eq!(
            tools[3]["inputSchema"]["properties"]["semanticModel"]["type"],
            "string"
        );
        assert_eq!(tools[4]["name"], "related_tests");
        assert_eq!(tools[5]["name"], "current_diff");
    }

    #[test]
    fn tools_list_public_surface_is_exact() {
        let expected_tools = [
            "prepare_task",
            "search",
            "related",
            "get_pack",
            "related_tests",
            "current_diff",
        ];
        assert_eq!(IMPLEMENTED_MCP_TOOL_NAMES, expected_tools);

        let response =
            handle_line(r#"{"jsonrpc":"2.0","id":"tools","method":"tools/list","params":{}}"#)
                .unwrap();
        let tool_names = response["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap())
            .collect::<Vec<_>>();

        assert_eq!(tool_names, expected_tools);
        for tool in response["result"]["tools"].as_array().unwrap() {
            assert_eq!(tool["inputSchema"]["type"], "object");
            assert_eq!(tool["inputSchema"]["additionalProperties"], false);
        }
        assert_eq!(
            response["result"]["tools"][0]["inputSchema"]["required"][0],
            "task"
        );
        assert_eq!(
            response["result"]["tools"][4]["inputSchema"]["required"][0],
            "paths"
        );
    }

    #[test]
    fn resources_list_exposes_repo_resources() {
        let response = handle_line(
            r#"{"jsonrpc":"2.0","id":"resources","method":"resources/list","params":{}}"#,
        )
        .unwrap();
        let resources = response["result"]["resources"].as_array().unwrap();

        assert_eq!(resources.len(), 7);
        assert_eq!(resources[0]["uri"], "ctxpack://repo/summary");
        assert_eq!(resources[1]["uri"], "ctxpack://repo/test-map");
        assert_eq!(resources[2]["uri"], "ctxpack://repo/dependency-graph");
        assert_eq!(resources[3]["uri"], "ctxpack://repo/memory");
        assert_eq!(resources[4]["uri"], "ctxpack://workspace/status");
        assert_eq!(resources[5]["uri"], "ctxpack://workspace/shared-artifacts");
        assert_eq!(resources[6]["uri"], "ctxpack://pack/guide");
    }

    #[test]
    fn resources_public_uri_shapes_are_stable() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo.repo).unwrap();
        ctxpack_index::write_workspace_manifest(
            &repo.repo,
            &ctxpack_core::WorkspaceManifest {
                schema_version: ctxpack_index::WORKSPACE_MANIFEST_SCHEMA_VERSION,
                workspace_id: None,
                repos: vec![ctxpack_core::WorkspaceRepo {
                    id: None,
                    path: ".".to_string(),
                    label: Some("fixture".to_string()),
                    tags: vec!["primary".to_string()],
                }],
            },
        )
        .unwrap();
        ctxpack_index::export_shared_artifact_manifest(&repo.repo).unwrap();

        let list = handle_line(
            r#"{"jsonrpc":"2.0","id":"resources","method":"resources/list","params":{}}"#,
        )
        .unwrap();
        let uris = list["result"]["resources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|resource| resource["uri"].as_str().unwrap())
            .collect::<Vec<_>>();

        assert_eq!(
            uris,
            vec![
                "ctxpack://repo/summary",
                "ctxpack://repo/test-map",
                "ctxpack://repo/dependency-graph",
                "ctxpack://repo/memory",
                "ctxpack://workspace/status",
                "ctxpack://workspace/shared-artifacts",
                "ctxpack://pack/guide",
            ]
        );

        let summary = handle_line(
            r#"{"jsonrpc":"2.0","id":30,"method":"resources/read","params":{"uri":"ctxpack://repo/summary"}}"#,
        )
        .unwrap();
        let test_map = handle_line(
            r#"{"jsonrpc":"2.0","id":31,"method":"resources/read","params":{"uri":"ctxpack://repo/test-map"}}"#,
        )
        .unwrap();
        let dependency_graph = handle_line(
            r#"{"jsonrpc":"2.0","id":32,"method":"resources/read","params":{"uri":"ctxpack://repo/dependency-graph"}}"#,
        )
        .unwrap();
        let guide = handle_line(
            r#"{"jsonrpc":"2.0","id":33,"method":"resources/read","params":{"uri":"ctxpack://pack/guide"}}"#,
        )
        .unwrap();
        let memory = handle_line(
            r#"{"jsonrpc":"2.0","id":36,"method":"resources/read","params":{"uri":"ctxpack://repo/memory"}}"#,
        )
        .unwrap();
        let workspace = handle_line(
            r#"{"jsonrpc":"2.0","id":37,"method":"resources/read","params":{"uri":"ctxpack://workspace/status"}}"#,
        )
        .unwrap();
        let shared_artifacts = handle_line(
            r#"{"jsonrpc":"2.0","id":38,"method":"resources/read","params":{"uri":"ctxpack://workspace/shared-artifacts"}}"#,
        )
        .unwrap();
        let file = handle_line(
            r#"{"jsonrpc":"2.0","id":34,"method":"resources/read","params":{"uri":"ctxpack://file/src/auth/session.ts?lines=1-2"}}"#,
        )
        .unwrap();
        let symbol = handle_line(
            r#"{"jsonrpc":"2.0","id":35,"method":"resources/read","params":{"uri":"ctxpack://symbol/requireSession"}}"#,
        )
        .unwrap();

        assert_eq!(
            summary["result"]["contents"][0]["uri"],
            "ctxpack://repo/summary"
        );
        assert_eq!(
            test_map["result"]["contents"][0]["uri"],
            "ctxpack://repo/test-map"
        );
        assert_eq!(
            dependency_graph["result"]["contents"][0]["uri"],
            "ctxpack://repo/dependency-graph"
        );
        assert_eq!(guide["result"]["contents"][0]["mimeType"], "text/markdown");
        assert_eq!(
            memory["result"]["contents"][0]["uri"],
            "ctxpack://repo/memory"
        );
        assert_eq!(
            workspace["result"]["contents"][0]["uri"],
            "ctxpack://workspace/status"
        );
        assert_eq!(
            shared_artifacts["result"]["contents"][0]["uri"],
            "ctxpack://workspace/shared-artifacts"
        );
        assert!(workspace["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("\"sourceTextLogged\": false"));
        assert!(shared_artifacts["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("\"compatible\": true"));
        assert!(file["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("requireSession"));
        assert!(symbol["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("src/auth/session.ts"));

        std::env::set_current_dir(cwd).unwrap();
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prompts_list_exposes_workflow_prompts() {
        let response =
            handle_line(r#"{"jsonrpc":"2.0","id":"prompts","method":"prompts/list","params":{}}"#)
                .unwrap();
        let prompts = response["result"]["prompts"].as_array().unwrap();

        assert_eq!(prompts.len(), 6);
        assert_eq!(prompts[0]["name"], "bugfix");
        assert_eq!(prompts[5]["name"], "explain_area");
    }

    #[test]
    fn prompts_public_surface_is_stable() {
        let list =
            handle_line(r#"{"jsonrpc":"2.0","id":"prompts","method":"prompts/list","params":{}}"#)
                .unwrap();
        let prompt_names = list["result"]["prompts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|prompt| prompt["name"].as_str().unwrap())
            .collect::<Vec<_>>();

        assert_eq!(
            prompt_names,
            vec![
                "bugfix",
                "feature",
                "refactor",
                "review_diff",
                "write_tests",
                "explain_area",
            ]
        );
        for prompt in list["result"]["prompts"].as_array().unwrap() {
            assert_eq!(prompt["arguments"][0]["name"], "task");
            assert_eq!(prompt["arguments"][0]["description"].is_string(), true);
        }

        let get = handle_line(
            r#"{"jsonrpc":"2.0","id":36,"method":"prompts/get","params":{"name":"review_diff","arguments":{"task":"review current branch"}}}"#,
        )
        .unwrap();
        let message = &get["result"]["messages"][0];
        assert_eq!(message["role"], "user");
        assert_eq!(message["content"]["type"], "text");
        assert!(message["content"]["text"]
            .as_str()
            .unwrap()
            .contains("ctxpack.current_diff"));
    }

    #[test]
    fn public_surface_compatibility_guards_cover_all_protocol_descriptors() {
        initialize_public_capabilities_are_stable();
        tools_list_public_surface_is_exact();
        tool_calls_include_text_and_structured_content();
        resources_public_uri_shapes_are_stable();
        prompts_public_surface_is_stable();
    }

    #[test]
    fn tool_calls_include_text_and_structured_content() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        fs::write(
            repo.repo.join("src/auth/session.ts"),
            "import { parseCookie } from './cookies';\nexport function requireSession() {\n  return !parseCookie();\n}\n",
        )
        .unwrap();

        let requests = [
            format!(
                r#"{{"jsonrpc":"2.0","id":40,"method":"tools/call","params":{{"name":"prepare_task","arguments":{{"task":"fix requireSession bug","repo":"{}","mode":"bug_fix"}}}}}}"#,
                repo.repo.display()
            ),
            format!(
                r#"{{"jsonrpc":"2.0","id":41,"method":"tools/call","params":{{"name":"search","arguments":{{"query":"requireSession","repo":"{}","limit":2}}}}}}"#,
                repo.repo.display()
            ),
            format!(
                r#"{{"jsonrpc":"2.0","id":42,"method":"tools/call","params":{{"name":"related","arguments":{{"path":"src/auth/session.ts","repo":"{}","limit":2}}}}}}"#,
                repo.repo.display()
            ),
            format!(
                r#"{{"jsonrpc":"2.0","id":43,"method":"tools/call","params":{{"name":"get_pack","arguments":{{"task":"fix requireSession bug","repo":"{}","mode":"bug_fix","budget":"brief","format":"markdown"}}}}}}"#,
                repo.repo.display()
            ),
            format!(
                r#"{{"jsonrpc":"2.0","id":44,"method":"tools/call","params":{{"name":"current_diff","arguments":{{"repo":"{}","includeUntracked":true}}}}}}"#,
                repo.repo.display()
            ),
        ];

        for request in requests {
            let response = handle_line(&request).unwrap();
            assert!(response["result"]["content"][0]["text"].is_string());
            assert!(response["result"]["structuredContent"].is_object());
            assert_eq!(response["result"]["isError"], false);
        }

        let related_tests = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":45,"method":"tools/call","params":{{"name":"related_tests","arguments":{{"paths":["src/auth/session.ts"],"repo":"{}"}}}}}}"#,
            repo.repo.display()
        ))
        .unwrap();
        assert!(related_tests["result"]["content"][0]["text"].is_string());
        assert!(related_tests["result"]["structuredContent"].is_array());
        assert_eq!(
            related_tests["result"]["structuredContent"][0]["path"],
            "tests/auth/session.test.ts"
        );

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prepare_task_call_returns_structured_context_plan() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"prepare_task","arguments":{{"task":"fix requireSession bug","repo":"{}","mode":"bug_fix"}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["taskType"],
            "bug_fix"
        );
        assert_eq!(
            response["result"]["structuredContent"]["privacyStatus"]["localOnly"],
            true
        );
        assert_eq!(
            response["result"]["structuredContent"]["targetFiles"][0]["path"],
            "src/auth/session.ts"
        );
        assert!(
            response["result"]["structuredContent"]["targetFiles"][0]["attribution"]
                .as_array()
                .unwrap()
                .iter()
                .any(
                    |evidence| evidence["reasonCode"].is_string() && evidence["signal"].is_string()
                )
        );
        assert_eq!(
            response["result"]["structuredContent"]["relatedTests"][0]["path"],
            "tests/auth/session.test.ts"
        );
        assert!(
            response["result"]["structuredContent"]["relatedTests"][0]["attribution"]
                .as_array()
                .unwrap()
                .iter()
                .any(
                    |evidence| evidence["reasonCode"].is_string() && evidence["signal"].is_string()
                )
        );
        assert!(
            response["result"]["structuredContent"]["retrievalCandidates"]
                .as_array()
                .unwrap()
                .iter()
                .any(|candidate| candidate["path"] == "src/auth/session.ts"
                    && candidate["kind"] == "file"
                    && candidate["signalScores"].is_array()
                    && candidate["evidence"].is_array())
        );
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("\"taskType\": \"bug_fix\""));
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn diagnostics_prepare_task_and_get_pack_include_trace_write_failures() {
        let _guard = env_lock();
        let repo = fixture_repo();
        let blocked_home = repo._temp.path().join("ctxpack-home-file");
        fs::write(&blocked_home, "not a directory\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &blocked_home);

        let prepare = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":70,"method":"tools/call","params":{{"name":"prepare_task","arguments":{{"task":"fix requireSession bug","repo":"{}","mode":"bug_fix"}}}}}}"#,
            repo.repo.display()
        ))
        .unwrap();
        let pack = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":71,"method":"tools/call","params":{{"name":"get_pack","arguments":{{"task":"fix requireSession bug","repo":"{}","mode":"bug_fix","budget":"brief","format":"json"}}}}}}"#,
            repo.repo.display()
        ))
        .unwrap();

        assert!(prepare.get("error").is_none(), "{prepare:?}");
        assert!(pack.get("error").is_none(), "{pack:?}");
        assert_eq!(
            prepare["result"]["structuredContent"]["targetFiles"][0]["path"],
            "src/auth/session.ts"
        );
        assert_eq!(pack["result"]["structuredContent"]["budget"], "brief");
        assert!(diagnostic_codes(&prepare["result"]["structuredContent"])
            .contains(&"trace_write_failed"));
        assert!(
            diagnostic_codes(&pack["result"]["structuredContent"]).contains(&"trace_write_failed")
        );

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn diagnostics_mcp_tools_expose_machine_readable_fields() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);

        let prepare = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":72,"method":"tools/call","params":{{"name":"prepare_task","arguments":{{"task":"Fixes #1061","repo":"{}","mode":"bug_fix","recordTrace":false}}}}}}"#,
            repo.repo.display()
        ))
        .unwrap();
        let search = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":73,"method":"tools/call","params":{{"name":"search","arguments":{{"query":"requireSession","repo":"{}","limit":2}}}}}}"#,
            repo.repo.display()
        ))
        .unwrap();
        let related = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":74,"method":"tools/call","params":{{"name":"related","arguments":{{"path":"src/auth/session.ts","repo":"{}","limit":2}}}}}}"#,
            repo.repo.display()
        ))
        .unwrap();
        let related_tests = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":75,"method":"tools/call","params":{{"name":"related_tests","arguments":{{"paths":["src/auth/session.ts"],"repo":"{}"}}}}}}"#,
            repo.repo.display()
        ))
        .unwrap();
        let current_diff = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":76,"method":"tools/call","params":{{"name":"current_diff","arguments":{{"repo":"{}","includeUntracked":true}}}}}}"#,
            repo.repo.display()
        ))
        .unwrap();

        assert!(diagnostic_codes(&prepare["result"]["structuredContent"])
            .contains(&"low_information_task"));
        assert!(search["result"]["structuredContent"]["diagnostics"].is_array());
        assert!(related["result"]["structuredContent"]["diagnostics"].is_array());
        assert!(related_tests["result"]["structuredContent"].is_array());
        assert!(related_tests["result"]["diagnostics"].is_array());
        assert!(current_diff["result"]["structuredContent"]["diagnostics"].is_array());

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn diagnostics_search_reports_stale_cache_rebuild() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        ctxpack_index::write_inventory(&repo.repo, &ctxpack_index::InventoryOptions::default())
            .unwrap();
        fs::write(
            repo.repo.join("src/auth/refreshed.ts"),
            "export const refreshed = true;\n",
        )
        .unwrap();

        let response = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":77,"method":"tools/call","params":{{"name":"search","arguments":{{"query":"refreshed","repo":"{}","limit":2}}}}}}"#,
            repo.repo.display()
        ))
        .unwrap();

        let codes = diagnostic_codes(&response["result"]["structuredContent"]);
        assert!(codes.contains(&"inventory_stale"));
        assert!(codes.contains(&"inventory_rebuilt"));
        assert_eq!(
            response["result"]["structuredContent"]["files"][0]["path"],
            "src/auth/refreshed.ts"
        );

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn file_resource_revalidates_against_current_safe_inventory() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo.repo).unwrap();
        ctxpack_index::write_inventory(&repo.repo, &ctxpack_index::InventoryOptions::default())
            .unwrap();
        fs::write(repo.repo.join(".ctxpackignore"), "src/auth/session.ts\n").unwrap();

        let response = handle_line(
            r#"{"jsonrpc":"2.0","id":78,"method":"resources/read","params":{"uri":"ctxpack://file/src/auth/session.ts?lines=1-2"}}"#,
        )
        .unwrap();

        assert_eq!(response["error"]["code"], -32602);
        assert!(response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("safe inventory"));

        std::env::set_current_dir(cwd).unwrap();
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prepare_task_call_caches_pack_resources_for_session_reads() {
        let _guard = env_lock();
        let repo = fixture_repo();
        clear_pack_resource_cache();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":18,"method":"tools/call","params":{{"name":"prepare_task","arguments":{{"task":"fix requireSession bug","repo":"{}","mode":"bug_fix"}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();
        let task_id = response["result"]["structuredContent"]["taskId"]
            .as_str()
            .unwrap();
        assert_eq!(
            response["result"]["structuredContent"]["packOptions"][2]["budget"],
            "deep"
        );
        let pack_uri = response["result"]["structuredContent"]["packOptions"][0]["resourceUri"]
            .as_str()
            .unwrap();
        let deep_pack_uri = response["result"]["structuredContent"]["packOptions"][2]
            ["resourceUri"]
            .as_str()
            .unwrap();
        let resource_request = format!(
            r#"{{"jsonrpc":"2.0","id":19,"method":"resources/read","params":{{"uri":"{pack_uri}"}}}}"#
        );
        let resource_response = handle_line(&resource_request).unwrap();
        let deep_resource_request = format!(
            r#"{{"jsonrpc":"2.0","id":28,"method":"resources/read","params":{{"uri":"{deep_pack_uri}.json"}}}}"#
        );
        let deep_resource_response = handle_line(&deep_resource_request).unwrap();
        let text = resource_response["result"]["contents"][0]["text"]
            .as_str()
            .unwrap();

        assert_eq!(
            resource_response["result"]["contents"][0]["mimeType"],
            "text/markdown"
        );
        assert!(text.contains("# Context Pack"));
        assert!(text.contains("src/auth/session.ts"));
        assert!(text.contains("tests/auth/session.test.ts"));
        assert!(text.contains(&format!("Task ID: `{task_id}`")));
        assert_eq!(
            deep_resource_response["result"]["contents"][0]["mimeType"],
            "application/json"
        );
        assert!(deep_resource_response["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("\"budget\": \"deep\""));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn pack_resources_are_session_scoped_characterization() {
        let _guard = env_lock();
        let repo = fixture_repo();
        clear_pack_resource_cache();
        std::env::set_var("CTXPACK_HOME", &repo.home);

        let guide = handle_line(
            r#"{"jsonrpc":"2.0","id":49,"method":"resources/read","params":{"uri":"ctxpack://pack/guide"}}"#,
        )
        .unwrap();
        let guide_text = guide["result"]["contents"][0]["text"].as_str().unwrap();
        assert!(guide_text.contains("MCP-session scoped"));
        assert!(guide_text.contains("get_pack"));
        assert!(guide_text.contains("reconnect-safe"));

        let missing = handle_line(
            r#"{"jsonrpc":"2.0","id":50,"method":"resources/read","params":{"uri":"ctxpack://pack/not-yet-created/brief"}}"#,
        )
        .unwrap();
        assert_eq!(missing["error"]["code"], -32602);
        let missing_message = missing["error"]["message"].as_str().unwrap();
        assert!(missing_message.contains("session-scoped"));
        assert!(missing_message.contains("same MCP server process"));
        assert!(missing_message.contains("call prepare_task first"));

        let request = format!(
            r#"{{"jsonrpc":"2.0","id":51,"method":"tools/call","params":{{"name":"prepare_task","arguments":{{"task":"fix requireSession bug","repo":"{}","mode":"bug_fix","targetAgent":"codex"}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();
        let pack_uri = response["result"]["structuredContent"]["packOptions"][0]["resourceUri"]
            .as_str()
            .unwrap();
        let markdown_request = format!(
            r#"{{"jsonrpc":"2.0","id":52,"method":"resources/read","params":{{"uri":"{pack_uri}"}}}}"#
        );
        let json_request = format!(
            r#"{{"jsonrpc":"2.0","id":53,"method":"resources/read","params":{{"uri":"{pack_uri}.json"}}}}"#
        );
        let markdown = handle_line(&markdown_request).unwrap();
        let json = handle_line(&json_request).unwrap();

        assert_eq!(markdown["result"]["contents"][0]["uri"], pack_uri);
        assert_eq!(
            markdown["result"]["contents"][0]["mimeType"],
            "text/markdown"
        );
        assert!(markdown["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("# Context Pack"));
        assert_eq!(
            json["result"]["contents"][0]["mimeType"],
            "application/json"
        );
        assert!(json["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("\"targetAgent\": \"codex\""));

        clear_pack_resource_cache();
        let unavailable_after_clear = handle_line(&markdown_request).unwrap();
        assert_eq!(unavailable_after_clear["error"]["code"], -32602);
        let unavailable_message = unavailable_after_clear["error"]["message"]
            .as_str()
            .unwrap();
        assert!(unavailable_message.contains("session-scoped"));
        assert!(unavailable_message.contains("same MCP server process"));
        assert!(unavailable_message.contains("call prepare_task first"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn pack_resource_cache_bounds_growth_and_evicts_oldest_entries() {
        let _guard = env_lock();
        let repo = fixture_repo();
        clear_pack_resource_cache();
        std::env::set_var("CTXPACK_HOME", &repo.home);

        let mut first_uri = String::new();
        let mut newest_uri = String::new();
        for index in 0..(pack_resource_cache_limit() + 3) {
            let request = format!(
                r#"{{"jsonrpc":"2.0","id":{},"method":"tools/call","params":{{"name":"prepare_task","arguments":{{"task":"fix requireSession bug {index}","repo":"{}","mode":"bug_fix","targetAgent":"codex","recordTrace":false}}}}}}"#,
                100 + index,
                repo.repo.display()
            );
            let response = handle_line(&request).unwrap();
            let uri = response["result"]["structuredContent"]["packOptions"][0]["resourceUri"]
                .as_str()
                .unwrap()
                .to_string();
            if first_uri.is_empty() {
                first_uri = uri.clone();
            }
            newest_uri = uri;
        }

        assert!(
            pack_resource_cache_len() <= pack_resource_cache_limit(),
            "pack cache grew to {} entries with limit {}",
            pack_resource_cache_len(),
            pack_resource_cache_limit()
        );

        let newest_read = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":201,"method":"resources/read","params":{{"uri":"{newest_uri}"}}}}"#
        ))
        .unwrap();
        assert_eq!(
            newest_read["result"]["contents"][0]["mimeType"],
            "text/markdown"
        );

        let old_read = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":202,"method":"resources/read","params":{{"uri":"{first_uri}"}}}}"#
        ))
        .unwrap();
        assert_eq!(old_read["error"]["code"], -32602);
        let message = old_read["error"]["message"].as_str().unwrap();
        assert!(message.contains("session-scoped"));
        assert!(message.contains("call prepare_task first"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prepare_task_call_prefers_path_anchor() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":15,"method":"tools/call","params":{{"name":"prepare_task","arguments":{{"task":"explain unrelated area","repo":"{}","mode":"explain","paths":["src/auth/session.ts"]}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["targetFiles"][0]["path"],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["targetFiles"][0]["reason"],
            "explicit path anchor from active context"
        );
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prepare_task_call_can_anchor_current_diff() {
        let _guard = env_lock();
        let repo = fixture_repo();
        fs::write(
            repo.repo.join("src/auth/session.ts"),
            "import { parseCookie } from './cookies';\nexport function requireSession() {\n  return !parseCookie();\n}\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":26,"method":"tools/call","params":{{"name":"prepare_task","arguments":{{"task":"explain unrelated area","repo":"{}","mode":"explain","includeCurrentDiff":true}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["targetFiles"][0]["path"],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["targetFiles"][0]["reason"],
            "explicit path anchor from active context"
        );
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prepare_task_requires_task_argument() {
        let response = handle_line(
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"prepare_task","arguments":{"mode":"explain"}}}"#,
        )
        .unwrap();

        assert_eq!(response["error"]["code"], -32602);
        assert!(response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("missing field `task`"));
    }

    #[test]
    fn resource_read_returns_repo_summary_and_test_map() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo.repo).unwrap();

        let summary = handle_line(
            r#"{"jsonrpc":"2.0","id":10,"method":"resources/read","params":{"uri":"ctxpack://repo/summary"}}"#,
        )
        .unwrap();
        let test_map = handle_line(
            r#"{"jsonrpc":"2.0","id":11,"method":"resources/read","params":{"uri":"ctxpack://repo/test-map"}}"#,
        )
        .unwrap();
        let dependency_graph = handle_line(
            r#"{"jsonrpc":"2.0","id":17,"method":"resources/read","params":{"uri":"ctxpack://repo/dependency-graph"}}"#,
        )
        .unwrap();

        assert!(summary["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("\"fileCount\""));
        assert!(test_map["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("tests/auth/session.test.ts"));
        assert!(dependency_graph["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("src/auth/cookies.ts"));

        std::env::set_current_dir(cwd).unwrap();
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn resource_test_map_uses_package_aware_commands() {
        let _guard = env_lock();
        let repo = fixture_repo();
        fs::write(
            repo.repo.join("package.json"),
            r#"{"scripts":{"test":"vitest run"}}"#,
        )
        .unwrap();
        fs::write(repo.repo.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo.repo).unwrap();

        let test_map = handle_line(
            r#"{"jsonrpc":"2.0","id":20,"method":"resources/read","params":{"uri":"ctxpack://repo/test-map"}}"#,
        )
        .unwrap();
        let text = test_map["result"]["contents"][0]["text"].as_str().unwrap();

        assert!(text.contains("tests/auth/session.test.ts"));
        assert!(text.contains("pnpm vitest run tests/auth/session.test.ts"));
        assert!(text.contains("safe test file from inventory"));

        std::env::set_current_dir(cwd).unwrap();
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn resource_read_returns_safe_file_slice_and_symbol_results() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo.repo).unwrap();

        let file = handle_line(
            r#"{"jsonrpc":"2.0","id":12,"method":"resources/read","params":{"uri":"ctxpack://file/src/auth/session.ts?lines=1-2"}}"#,
        )
        .unwrap();
        let symbol = handle_line(
            r#"{"jsonrpc":"2.0","id":13,"method":"resources/read","params":{"uri":"ctxpack://symbol/requireSession"}}"#,
        )
        .unwrap();

        let file_text = file["result"]["contents"][0]["text"].as_str().unwrap();
        assert!(file_text.contains("1: import { parseCookie } from './cookies';"));
        assert!(file_text.contains("2: export function requireSession"));
        assert!(!file_text.contains("4: }"));
        assert!(symbol["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("src/auth/session.ts"));

        std::env::set_current_dir(cwd).unwrap();
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prompts_get_returns_agent_workflow_instruction() {
        let response = handle_line(
            r#"{"jsonrpc":"2.0","id":14,"method":"prompts/get","params":{"name":"bugfix","arguments":{"task":"fix auth redirect"}}}"#,
        )
        .unwrap();
        let text = response["result"]["messages"][0]["content"]["text"]
            .as_str()
            .unwrap();

        assert!(text.contains("Task: fix auth redirect"));
        assert!(text.contains("ctxpack.prepare_task"));
        assert!(text.contains("host agent's native tools"));
    }

    #[test]
    fn search_call_returns_file_and_symbol_matches() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{{"name":"search","arguments":{{"query":"requireSession","repo":"{}","limit":2}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["files"][0]["path"],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["symbols"][0]["symbol"]["name"],
            "requireSession"
        );
        assert_eq!(
            response["result"]["structuredContent"]["privacyStatus"]["sourceTextReturned"],
            false
        );
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("src/auth/session.ts"));
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn semantic_mcp_calls_are_additive_and_source_free() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);

        let search = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":86,"method":"tools/call","params":{{"name":"search","arguments":{{"query":"auth required","repo":"{}","limit":10,"semantic":true,"semanticProvider":"local_hash","semanticModel":"ctxpack-mcp-test","semanticDimensions":128}}}}}}"#,
            repo.repo.display()
        ))
        .unwrap();
        assert!(!search["result"]["structuredContent"]["semanticFiles"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(
            search["result"]["structuredContent"]["semanticProvider"]["provider"],
            "local_hash"
        );
        assert_eq!(
            search["result"]["structuredContent"]["semanticProvider"]["model"],
            "ctxpack-mcp-test"
        );
        assert_eq!(
            search["result"]["structuredContent"]["semanticProvider"]["dimensions"],
            128
        );
        assert_eq!(
            search["result"]["structuredContent"]["privacyStatus"]["sourceTextReturned"],
            false
        );

        let prepare = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":87,"method":"tools/call","params":{{"name":"prepare_task","arguments":{{"task":"fix auth required flow","repo":"{}","mode":"bug_fix","semantic":true,"semanticProvider":"local_hash","semanticModel":"ctxpack-mcp-test","semanticDimensions":128,"recordTrace":false}}}}}}"#,
            repo.repo.display()
        ))
        .unwrap();
        let has_semantic = prepare["result"]["structuredContent"]["retrievalCandidates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|candidate| {
                candidate["signalScores"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|signal| signal["signal"] == "semantic")
            });
        assert!(has_semantic);

        let pack = handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":88,"method":"tools/call","params":{{"name":"get_pack","arguments":{{"task":"fix auth required flow","repo":"{}","mode":"bug_fix","semantic":true,"semanticProvider":"ctxpack_mcp_unknown_provider","recordTrace":false,"format":"json"}}}}}}"#,
            repo.repo.display()
        ))
        .unwrap();
        let decisions = pack["result"]["structuredContent"]["providerPolicy"]["decisions"]
            .as_array()
            .unwrap();
        assert!(decisions.iter().any(|decision| {
            decision["provider"] == "ctxpack_mcp_unknown_provider"
                && decision["status"] == "unavailable"
        }));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn search_call_filters_to_symbol_kind() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":25,"method":"tools/call","params":{{"name":"search","arguments":{{"query":"requireSession","repo":"{}","limit":2,"kinds":["symbol"]}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["files"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            response["result"]["structuredContent"]["symbols"][0]["symbol"]["path"],
            "src/auth/session.ts"
        );
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_call_returns_tests_co_change_hints_and_dependency_edges() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{{"name":"related","arguments":{{"path":"src/auth/session.ts","repo":"{}","limit":3}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["relatedTests"][0]["path"],
            "tests/auth/session.test.ts"
        );
        let co_change_paths = response["result"]["structuredContent"]["coChangeHints"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|hint| hint["path"].as_str())
            .collect::<Vec<_>>();
        assert!(co_change_paths.contains(&"tests/auth/session.test.ts"));
        assert_eq!(
            response["result"]["structuredContent"]["dependencyEdges"][0]["sourcePath"],
            "tests/auth/session.test.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["dependencyEdges"][0]["targetPath"],
            "src/auth/session.ts"
        );
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_call_accepts_symbol_anchor() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":22,"method":"tools/call","params":{{"name":"related","arguments":{{"symbol":"requireSession","repo":"{}","limit":3}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["resolvedPaths"][0],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["symbolMatches"][0]["symbol"]["name"],
            "requireSession"
        );
        assert_eq!(
            response["result"]["structuredContent"]["relatedTests"][0]["path"],
            "tests/auth/session.test.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["dependencyEdges"][0]["sourcePath"],
            "src/auth/session.ts"
        );
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_call_can_expand_current_diff_anchor() {
        let _guard = env_lock();
        let repo = fixture_repo();
        fs::write(
            repo.repo.join("src/auth/session.ts"),
            "import { parseCookie } from './cookies';\nexport function requireSession() {\n  return !parseCookie();\n}\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":27,"method":"tools/call","params":{{"name":"related","arguments":{{"includeCurrentDiff":true,"repo":"{}","limit":3}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["resolvedPaths"][0],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["relatedTests"][0]["path"],
            "tests/auth/session.test.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["dependencyEdges"][0]["sourcePath"],
            "tests/auth/session.test.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["dependencyEdges"][0]["targetPath"],
            "src/auth/session.ts"
        );
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_call_requires_path_or_symbol() {
        let response = handle_line(
            r#"{"jsonrpc":"2.0","id":23,"method":"tools/call","params":{"name":"related","arguments":{"limit":3}}}"#,
        )
        .unwrap();

        assert_eq!(response["error"]["code"], -32602);
        assert!(response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("path, symbol, or current diff"));
    }

    #[test]
    fn related_call_degrades_when_git_history_is_unavailable() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":24,"method":"tools/call","params":{{"name":"related","arguments":{{"symbol":"requireSession","repo":"{}","limit":3}}}}}}"#,
            repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["resolvedPaths"][0],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["relatedTests"][0]["path"],
            "tests/auth/session.test.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["coChangeHints"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert!(response["result"]["structuredContent"]["warnings"][0]
            .as_str()
            .unwrap()
            .contains("co-change hints were unavailable"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_tests_call_returns_targeted_command() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{{"name":"related_tests","arguments":{{"paths":["src/auth/session.ts"],"repo":"{}"}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"][0]["path"],
            "tests/auth/session.test.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"][0]["command"],
            "pnpm test tests/auth/session.test.ts"
        );
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn current_diff_call_returns_changed_paths_without_source_content() {
        let repo = fixture_repo();
        fs::write(
            repo.repo.join("src/auth/session.ts"),
            "export function requireSession() {\n  return false;\n}\n",
        )
        .unwrap();
        fs::write(repo.repo.join("notes.md"), "scratch\n").unwrap();
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{{"name":"current_diff","arguments":{{"repo":"{}","includeUntracked":true}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["unstaged"][0],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["untracked"][0],
            "notes.md"
        );
        assert!(!response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("return false"));
        assert_eq!(
            response["result"]["structuredContent"]["privacyStatus"]["sourceTextReturned"],
            false
        );
    }

    #[test]
    fn current_diff_filters_sensitive_and_generated_paths() {
        let repo = fixture_repo();
        fs::create_dir_all(repo.repo.join("dist")).unwrap();
        fs::write(repo.repo.join("src/auth/session.ts"), "safe change\n").unwrap();
        fs::write(repo.repo.join("dist/generated.js"), "generated change\n").unwrap();
        fs::write(repo.repo.join(".env"), "TOKEN=secret\n").unwrap();
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":21,"method":"tools/call","params":{{"name":"current_diff","arguments":{{"repo":"{}","includeUntracked":true}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();
        let text = response["result"]["content"][0]["text"].as_str().unwrap();

        assert_eq!(
            response["result"]["structuredContent"]["unstaged"][0],
            "src/auth/session.ts"
        );
        assert_eq!(
            response["result"]["structuredContent"]["excluded"]["untracked"],
            2
        );
        assert!(!text.contains("dist/generated.js"));
        assert!(!text.contains(".env"));
        assert!(!text.contains("TOKEN=secret"));
    }

    #[test]
    fn unknown_method_returns_method_not_found() {
        let response = handle_line(
            r#"{"jsonrpc":"2.0","id":4,"method":"sampling/createMessage","params":{}}"#,
        )
        .unwrap();

        assert_eq!(response["error"]["code"], -32601);
    }

    #[test]
    fn json_rpc_error_codes_are_stable() {
        let parse = handle_line(r#"{"jsonrpc":"2.0","id":null,"method":"initialize""#).unwrap();
        let missing_method = handle_line(
            r#"{"jsonrpc":"2.0","id":60,"method":"sampling/createMessage","params":{}}"#,
        )
        .unwrap();
        let invalid_params = handle_line(
            r#"{"jsonrpc":"2.0","id":61,"method":"tools/call","params":{"name":"prepare_task","arguments":{"task":""}}}"#,
        )
        .unwrap();
        let unsupported_resource = handle_line(
            r#"{"jsonrpc":"2.0","id":62,"method":"resources/read","params":{"uri":"ctxpack://unknown/resource"}}"#,
        )
        .unwrap();

        assert_eq!(parse["error"]["code"], -32700);
        assert_eq!(missing_method["error"]["code"], -32601);
        assert_eq!(invalid_params["error"]["code"], -32602);
        assert_eq!(unsupported_resource["error"]["code"], -32602);
        assert!(missing_method["error"]["message"]
            .as_str()
            .unwrap()
            .contains("method not found"));
        assert!(invalid_params["error"]["message"]
            .as_str()
            .unwrap()
            .contains("task must not be empty"));
    }

    #[test]
    fn get_pack_call_returns_markdown_and_structured_pack() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{{"name":"get_pack","arguments":{{"task":"fix requireSession bug","repo":"{}","mode":"bug_fix","budget":"brief","format":"markdown","targetAgent":"codex"}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();
        let text = response["result"]["content"][0]["text"].as_str().unwrap();

        assert_eq!(response["result"]["structuredContent"]["budget"], "brief");
        assert_eq!(
            response["result"]["structuredContent"]["targetAgent"],
            "codex"
        );
        assert!(response["result"]["structuredContent"]["taskHash"]
            .as_str()
            .is_some_and(|hash| hash.len() == 64));
        assert!(response["result"]["structuredContent"]["repoId"]
            .as_str()
            .is_some_and(|repo_id| !repo_id.is_empty()));
        assert_eq!(
            response["result"]["structuredContent"]["privacyStatus"]["localOnly"],
            true
        );
        assert!(text.contains("# Context Pack"));
        assert!(text.contains("- Target agent: `codex`"));
        assert!(text.contains("- Task hash: `"));
        assert!(text.contains("src/auth/session.ts"));
        assert!(text.contains("tests/auth/session.test.ts"));
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn get_pack_call_uses_path_anchor_in_pack() {
        let _guard = env_lock();
        let repo = fixture_repo();
        std::env::set_var("CTXPACK_HOME", &repo.home);
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":16,"method":"tools/call","params":{{"name":"get_pack","arguments":{{"task":"explain unrelated area","repo":"{}","mode":"explain","budget":"brief","format":"markdown","paths":["src/auth/session.ts"]}}}}}}"#,
            repo.repo.display()
        );
        let response = handle_line(&request).unwrap();
        let text = response["result"]["content"][0]["text"].as_str().unwrap();

        assert!(text.contains("explicit path anchor from active context"));
        assert!(text.contains("src/auth/session.ts"));
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn run_server_writes_one_response_per_request_line() {
        let input = br#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
"#;
        let mut output = Vec::new();

        run_server(&input[..], &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert_eq!(output.lines().count(), 1);
        assert!(output.contains("\"prepare_task\""));
    }

    fn fixture_repo() -> FixtureRepo {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(
            repo.join("src/auth/cookies.ts"),
            "export function parseCookie() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "import { parseCookie } from './cookies';\nexport function requireSession() {\n  return parseCookie();\n}\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add session"]);

        FixtureRepo {
            _temp: temp,
            repo,
            home,
        }
    }

    fn run_git(repo: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn diagnostic_codes(value: &serde_json::Value) -> Vec<&str> {
        value["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|diagnostic| diagnostic["code"].as_str())
            .collect()
    }
}
