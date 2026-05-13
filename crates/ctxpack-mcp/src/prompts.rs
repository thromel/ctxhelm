use crate::protocol::RpcError;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetPromptParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

pub(crate) fn get_prompt(params: Value) -> Result<Value, RpcError> {
    let params: GetPromptParams = serde_json::from_value(params).map_err(|error| {
        RpcError::invalid_params(format!("invalid prompts/get params: {error}"))
    })?;
    let task = params
        .arguments
        .get("task")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();

    let text = match params.name.as_str() {
        "bugfix" => workflow_prompt(
            "bug_fix",
            task,
            "Call ctxpack.prepare_task first, read the returned target files, request ctxpack.get_pack only if needed, make the smallest source change, then run the related test command.",
        ),
        "feature" => workflow_prompt(
            "feature",
            task,
            "Call ctxpack.prepare_task, inspect analogous target files and tests, request a standard pack when examples are needed, then implement within existing repo patterns.",
        ),
        "refactor" => workflow_prompt(
            "refactor",
            task,
            "Call ctxpack.prepare_task, expand with ctxpack.related around the affected files, preserve behavior, and validate with related tests.",
        ),
        "review_diff" => workflow_prompt(
            "review",
            task,
            "Call ctxpack.current_diff, inspect changed paths, use ctxpack.related for risky files, then report findings ordered by severity.",
        ),
        "write_tests" => workflow_prompt(
            "test",
            task,
            "Call ctxpack.prepare_task and ctxpack.related_tests, inspect the source under test and existing test style, then add focused tests.",
        ),
        "explain_area" => workflow_prompt(
            "explain",
            task,
            "Call ctxpack.prepare_task and use ctxpack.search for named concepts, then explain only from files actually read or returned by ctxpack.",
        ),
        name => {
            return Err(RpcError::invalid_params(format!(
                "prompt is not implemented: {name}"
            )))
        }
    };

    Ok(json!({
        "description": format!("ctxpack {} workflow", params.name),
        "messages": [{
            "role": "user",
            "content": {
                "type": "text",
                "text": text
            }
        }]
    }))
}

fn workflow_prompt(mode: &str, task: &str, instruction: &str) -> String {
    let task_line = if task.is_empty() {
        "Task: use the user's current request.".to_string()
    } else {
        format!("Task: {task}")
    };
    format!("{task_line}\nMode: {mode}\n\n{instruction}\n\nWhen the active workspace path is known, pass it as the ctxpack `repo` argument so the MCP server does not infer the wrong working directory.\n\nKeep ctxpack read-only: use it for context and use the host agent's native tools for file reads, edits, and validation commands.")
}
