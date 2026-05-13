use crate::prompts::get_prompt;
use crate::resources::read_resource;
use crate::schemas::{
    initialize_result, prompts_list_result, resources_list_result, tools_list_result,
};
use crate::tools::call_tool;
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

pub(crate) const MCP_PROTOCOL_VERSION: &str = "2025-11-25";
const JSONRPC_VERSION: &str = "2.0";

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

pub(crate) struct RpcError {
    code: i32,
    message: String,
}

impl RpcError {
    pub(crate) fn parse_error(message: impl Into<String>) -> Self {
        Self {
            code: -32700,
            message: message.into(),
        }
    }

    pub(crate) fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: message.into(),
        }
    }

    pub(crate) fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("method not found: {method}"),
        }
    }
}

pub fn run_stdio_server() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_server(stdin.lock(), stdout.lock())
}

pub fn run_server<R, W>(reader: R, mut writer: W) -> io::Result<()>
where
    R: BufRead,
    W: Write,
{
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        if let Some(response) = handle_line(&line) {
            serde_json::to_writer(&mut writer, &response)?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
    }

    Ok(())
}

pub(crate) fn handle_line(line: &str) -> Option<Value> {
    let parsed = match serde_json::from_str::<JsonRpcRequest>(line) {
        Ok(request) => request,
        Err(error) => {
            return Some(error_response(
                Value::Null,
                RpcError::parse_error(format!("invalid JSON-RPC request: {error}")),
            ));
        }
    };

    let id = parsed.id.clone()?;

    match handle_request(&parsed) {
        Ok(result) => Some(success_response(id, result)),
        Err(error) => Some(error_response(id, error)),
    }
}

fn handle_request(request: &JsonRpcRequest) -> Result<Value, RpcError> {
    match request.method.as_str() {
        "initialize" => Ok(initialize_result()),
        "tools/list" => Ok(tools_list_result()),
        "tools/call" => call_tool(request.params.clone()),
        "resources/list" => Ok(resources_list_result()),
        "resources/read" => read_resource(request.params.clone()),
        "prompts/list" => Ok(prompts_list_result()),
        "prompts/get" => get_prompt(request.params.clone()),
        method => Err(RpcError::method_not_found(method)),
    }
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id,
        "result": result
    })
}

fn error_response(id: Value, error: RpcError) -> Value {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id,
        "error": {
            "code": error.code,
            "message": error.message
        }
    })
}
