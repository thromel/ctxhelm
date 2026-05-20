use crate::protocol::MCP_PROTOCOL_VERSION;
use serde_json::{json, Value};

pub(crate) fn initialize_result() -> Value {
    json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "capabilities": {
            "tools": {
                "listChanged": false
            },
            "resources": {
                "listChanged": false
            },
            "prompts": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": "ctxpack",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

pub(crate) fn resources_list_result() -> Value {
    json!({
        "resources": [
            {
                "uri": "ctxpack://repo/summary",
                "name": "Repository Summary",
                "description": "Safe inventory summary for the current repository.",
                "mimeType": "application/json"
            },
            {
                "uri": "ctxpack://repo/test-map",
                "name": "Repository Test Map",
                "description": "Test files and inferred targeted commands.",
                "mimeType": "application/json"
            },
            {
                "uri": "ctxpack://repo/dependency-graph",
                "name": "Repository Dependency Graph",
                "description": "Safe local import edges inferred from source and test files.",
                "mimeType": "application/json"
            },
            {
                "uri": "ctxpack://repo/memory",
                "name": "Repository Memory Cards",
                "description": "Fresh approved or deterministic source-free memory cards for the current repository.",
                "mimeType": "application/json"
            },
            {
                "uri": "ctxpack://workspace/status",
                "name": "Workspace Status",
                "description": "Source-free local workspace status from .ctxpack/workspace.json.",
                "mimeType": "application/json"
            },
            {
                "uri": "ctxpack://workspace/shared-artifacts",
                "name": "Workspace Shared Artifacts",
                "description": "Source-free shared artifact manifest inspection for the current workspace root.",
                "mimeType": "application/json"
            },
            {
                "uri": "ctxpack://pack/guide",
                "name": "Context Pack Guide",
                "description": "How to request task-conditioned context packs with ctxpack.get_pack.",
                "mimeType": "text/markdown"
            }
        ]
    })
}

pub(crate) fn prompts_list_result() -> Value {
    json!({
        "prompts": [
            prompt_descriptor("bugfix", "Prepare and solve a bug fix with targeted repo context."),
            prompt_descriptor("feature", "Prepare and implement a feature using analogous repo context."),
            prompt_descriptor("refactor", "Plan a refactor using callers, tests, and constraints."),
            prompt_descriptor("review_diff", "Review the current diff with repo-aware context."),
            prompt_descriptor("write_tests", "Find source context and write focused tests."),
            prompt_descriptor("explain_area", "Explain an area of the codebase with grounded files.")
        ]
    })
}

fn prompt_descriptor(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "description": description,
        "arguments": [{
            "name": "task",
            "description": "The developer task or area to work on.",
            "required": name != "review_diff"
        }]
    })
}

pub(crate) fn tools_list_result() -> Value {
    json!({
        "tools": [
            {
                "name": "prepare_task",
                "title": "Prepare Task Context",
                "description": "Return a compact, local-only ContextPlan for a coding task.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task": {
                            "type": "string",
                            "description": "The developer task to prepare context for."
                        },
                        "repo": {
                            "type": "string",
                            "description": "Optional repository path. Pass the active workspace path when known; defaults to the MCP server working directory."
                        },
                        "mode": {
                            "type": "string",
                            "description": "Optional task type override.",
                            "enum": ["bug_fix", "feature", "refactor", "review", "test", "explain"]
                        },
                        "paths": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Optional active/open repo-relative or absolute paths to pin as context anchors."
                        },
                        "includeCurrentDiff": {
                            "type": "boolean",
                            "description": "When true, add safe changed paths from the current local diff as context anchors without returning source text."
                        },
                        "targetAgent": {
                            "type": "string",
                            "description": "Optional host agent label for local eval traces."
                        },
                        "semantic": {
                            "type": "boolean",
                            "description": "Explicitly enable local-only semantic retrieval for this task."
                        },
                        "semanticProvider": {
                            "type": "string",
                            "description": "Optional local semantic provider id, such as local_hash or local_fastembed."
                        },
                        "semanticModel": {
                            "type": "string",
                            "description": "Optional local semantic model name for the selected provider."
                        },
                        "semanticDimensions": {
                            "type": "integer",
                            "minimum": 8,
                            "maximum": 4096,
                            "description": "Optional embedding dimension count for the selected local provider."
                        },
                        "recordTrace": {
                            "type": "boolean",
                            "description": "When false, skip local eval trace recording for this read call."
                        }
                    },
                    "required": ["task"],
                    "additionalProperties": false
                }
            },
            {
                "name": "search",
                "title": "Search Repository Context",
                "description": "Run compact local search over safe inventoried repository files and symbols.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Identifier, path fragment, error text, or concept to search for."
                        },
                        "repo": {
                            "type": "string",
                            "description": "Optional repository path. Pass the active workspace path when known; defaults to the MCP server working directory."
                        },
                        "limit": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 50,
                            "description": "Maximum result count."
                        },
                        "semantic": {
                            "type": "boolean",
                            "description": "When true, use the explicit local semantic provider for file results."
                        },
                        "semanticProvider": {
                            "type": "string",
                            "description": "Optional local semantic provider id, such as local_hash or local_fastembed."
                        },
                        "semanticModel": {
                            "type": "string",
                            "description": "Optional local semantic model name for the selected provider."
                        },
                        "semanticDimensions": {
                            "type": "integer",
                            "minimum": 8,
                            "maximum": 4096,
                            "description": "Optional embedding dimension count for the selected local provider."
                        },
                        "kinds": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["file", "symbol"]
                            },
                            "description": "Optional result kinds. Defaults to file and symbol matches."
                        }
                    },
                    "required": ["query"],
                    "additionalProperties": false
                }
            },
            {
                "name": "related",
                "title": "Related Repository Context",
                "description": "Expand around a path or symbol with related tests, dependency edges, and local git co-change hints.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Repository-relative or absolute file path to expand from."
                        },
                        "symbol": {
                            "type": "string",
                            "description": "Symbol name or query to resolve first, then expand from matching symbol paths."
                        },
                        "repo": {
                            "type": "string",
                            "description": "Optional repository path. Pass the active workspace path when known; defaults to the MCP server working directory."
                        },
                        "includeCurrentDiff": {
                            "type": "boolean",
                            "description": "When true, add safe changed paths from the current local diff as expansion anchors without returning source text."
                        },
                        "include": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["tests", "commits", "dependencies"]
                            },
                            "description": "Optional expansion categories. Defaults to tests, commits, and dependencies."
                        },
                        "limit": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 50,
                            "description": "Maximum count for each expansion category."
                        }
                    },
                    "additionalProperties": false
                }
            },
            {
                "name": "get_pack",
                "title": "Get Context Pack",
                "description": "Return a budgeted, local-only ContextPack for a coding task.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task": {
                            "type": "string",
                            "description": "The developer task to compile context for."
                        },
                        "repo": {
                            "type": "string",
                            "description": "Optional repository path. Pass the active workspace path when known; defaults to the MCP server working directory."
                        },
                        "mode": {
                            "type": "string",
                            "description": "Optional task type override.",
                            "enum": ["bug_fix", "feature", "refactor", "review", "test", "explain"]
                        },
                        "budget": {
                            "type": "string",
                            "description": "Context budget.",
                            "enum": ["brief", "standard", "deep"]
                        },
                        "format": {
                            "type": "string",
                            "description": "Text response format.",
                            "enum": ["markdown", "json"]
                        },
                        "paths": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Optional active/open repo-relative or absolute paths to pin as context anchors."
                        },
                        "includeCurrentDiff": {
                            "type": "boolean",
                            "description": "When true, add safe changed paths from the current local diff as context anchors without returning source text."
                        },
                        "targetAgent": {
                            "type": "string",
                            "description": "Optional host agent label for local eval traces."
                        },
                        "semantic": {
                            "type": "boolean",
                            "description": "Explicitly enable local-only semantic retrieval while compiling the pack."
                        },
                        "semanticProvider": {
                            "type": "string",
                            "description": "Optional local semantic provider id, such as local_hash or local_fastembed."
                        },
                        "semanticModel": {
                            "type": "string",
                            "description": "Optional local semantic model name for the selected provider."
                        },
                        "semanticDimensions": {
                            "type": "integer",
                            "minimum": 8,
                            "maximum": 4096,
                            "description": "Optional embedding dimension count for the selected local provider."
                        },
                        "recordTrace": {
                            "type": "boolean",
                            "description": "When false, skip local eval trace recording for this read call."
                        }
                    },
                    "required": ["task"],
                    "additionalProperties": false
                }
            },
            {
                "name": "related_tests",
                "title": "Find Related Tests",
                "description": "Find likely test files and targeted commands for source paths.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "paths": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Repository-relative or absolute source paths."
                        },
                        "repo": {
                            "type": "string",
                            "description": "Optional repository path. Pass the active workspace path when known; defaults to the MCP server working directory."
                        }
                    },
                    "required": ["paths"],
                    "additionalProperties": false
                }
            },
            {
                "name": "current_diff",
                "title": "Current Diff Summary",
                "description": "Return safe changed path lists from the local git working tree without returning source content.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "repo": {
                            "type": "string",
                            "description": "Optional repository path. Pass the active workspace path when known; defaults to the MCP server working directory."
                        },
                        "includeUntracked": {
                            "type": "boolean",
                            "description": "Include untracked non-ignored paths."
                        }
                    },
                    "additionalProperties": false
                }
            }
        ]
    })
}
