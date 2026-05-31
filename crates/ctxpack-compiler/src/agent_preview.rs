use crate::packs::compile_context_pack_from_plan_for_agent;
use crate::planning::normalized_target_agent;
use ctxpack_core::{
    AgentPreview, AgentPreviewReport, AgentPreviewStep, AgentPreviewSurface,
    AgentPreviewSurfaceKind, Diagnostic, DiagnosticSeverity, PackBudget, PrivacyStatus, TaskType,
};
use ctxpack_index::{task_hash, InventoryError, SemanticProviderConfig};
use std::path::Path;

pub fn build_agent_preview_report(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    budget: PackBudget,
    target_agent: &str,
    paths: &[String],
    semantic_enabled: bool,
) -> Result<AgentPreviewReport, InventoryError> {
    build_agent_preview_report_with_provider(
        repo_root,
        task,
        task_type,
        budget,
        target_agent,
        paths,
        semantic_enabled,
        SemanticProviderConfig::default(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn build_agent_preview_report_with_provider(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    budget: PackBudget,
    target_agent: &str,
    paths: &[String],
    semantic_enabled: bool,
    semantic_provider: SemanticProviderConfig,
) -> Result<AgentPreviewReport, InventoryError> {
    let repo_root = repo_root.as_ref();
    let target_agents = expand_target_agents(target_agent);
    let plan = crate::planning::prepare_context_plan_with_paths_and_semantic_provider(
        repo_root,
        task,
        task_type.clone(),
        paths,
        semantic_enabled,
        semantic_provider,
    )?;
    let pack_target_agent = target_agents
        .first()
        .map(String::as_str)
        .unwrap_or("generic");
    let pack = compile_context_pack_from_plan_for_agent(
        repo_root,
        task,
        &plan,
        budget.clone(),
        pack_target_agent,
    );
    let repo_id = pack.repo_id.clone();
    let pack_resource_uri = format!(
        "ctxpack://pack/{}/{}",
        pack.id,
        budget_resource_label(&budget)
    );
    let mut previews = Vec::new();
    let mut diagnostics = Vec::new();

    for agent in target_agents {
        let mcp_resources = mcp_resources_for_plan(&pack_resource_uri, &plan);
        previews.push(AgentPreview {
            target_agent: agent.clone(),
            display_name: display_name(&agent).to_string(),
            pack_resource_uri: pack_resource_uri.clone(),
            mcp_tools: vec![
                "prepare_task".to_string(),
                "search".to_string(),
                "related".to_string(),
                "get_pack".to_string(),
                "related_tests".to_string(),
                "current_diff".to_string(),
            ],
            mcp_resources,
            guidance: guidance_surfaces(&agent),
            native_rules: native_rule_surfaces(&agent),
            next_steps: next_steps(&agent),
            boundary: boundary_lines(),
            source_text_included: false,
        });
    }

    if !is_supported_agent_target(target_agent) {
        diagnostics.push(Diagnostic {
            code: "unknown_agent_preview_target".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: format!(
                "Preview target `{}` is not one of codex, claude-code, cursor, opencode, generic, or all.",
                target_agent
            ),
            paths: Vec::new(),
            count: 1,
        });
    }

    Ok(AgentPreviewReport {
        repo_id,
        task_hash: task_hash(task),
        task_type,
        budget,
        source_text_logged: false,
        privacy_status: PrivacyStatus::local_only(),
        previews,
        diagnostics,
    })
}

fn expand_target_agents(target_agent: &str) -> Vec<String> {
    match normalized_target_agent(target_agent).as_str() {
        "" | "all" => vec![
            "codex".to_string(),
            "claude-code".to_string(),
            "cursor".to_string(),
            "opencode".to_string(),
            "generic".to_string(),
        ],
        "claude" => vec!["claude-code".to_string()],
        "open-code" => vec!["opencode".to_string()],
        "mcp" => vec!["generic".to_string()],
        other => vec![other.to_string()],
    }
}

fn is_supported_agent_target(target_agent: &str) -> bool {
    matches!(
        normalized_target_agent(target_agent).as_str(),
        "" | "all"
            | "codex"
            | "claude"
            | "claude-code"
            | "cursor"
            | "opencode"
            | "open-code"
            | "generic"
            | "mcp"
    )
}

fn budget_resource_label(budget: &PackBudget) -> &'static str {
    match budget {
        PackBudget::Brief => "brief",
        PackBudget::Standard => "standard",
        PackBudget::Deep => "deep",
    }
}

fn display_name(target_agent: &str) -> &'static str {
    match target_agent {
        "codex" => "Codex CLI",
        "claude-code" => "Claude Code",
        "cursor" => "Cursor",
        "opencode" => "OpenCode",
        "generic" => "Generic MCP Client",
        _ => "Custom Agent",
    }
}

fn mcp_resources_for_plan(
    pack_resource_uri: &str,
    plan: &ctxpack_core::ContextPlan,
) -> Vec<String> {
    let mut resources = vec![
        "ctxpack://repo/summary".to_string(),
        "ctxpack://repo/test-map".to_string(),
        "ctxpack://repo/dependency-graph".to_string(),
        "ctxpack://repo/memory".to_string(),
        "ctxpack://repo/context-areas".to_string(),
        pack_resource_uri.to_string(),
    ];
    for area in plan.context_areas.iter().take(4) {
        if !area.resource_uri.is_empty() {
            resources.push(area.resource_uri.clone());
        }
    }
    for target in plan.target_files.iter().take(3) {
        resources.push(format!("ctxpack://file/{}", target.path));
    }
    resources
}

fn guidance_surfaces(target_agent: &str) -> Vec<AgentPreviewSurface> {
    let mut surfaces = vec![AgentPreviewSurface {
        kind: AgentPreviewSurfaceKind::AgentsMd,
        label: "AGENTS.md ctxpack policy".to_string(),
        path: Some("AGENTS.md".to_string()),
        summary:
            "Stable repo guidance tells agents to call prepare_task for non-trivial edits and read files natively."
                .to_string(),
    }];

    match target_agent {
        "codex" => surfaces.push(AgentPreviewSurface {
            kind: AgentPreviewSurfaceKind::AdapterSnippet,
            label: "Codex MCP setup guidance".to_string(),
            path: None,
            summary:
                "User-owned Codex MCP config can launch `ctxpack serve-mcp`; ctxpack does not mutate global config."
                    .to_string(),
        }),
        "claude-code" => surfaces.push(AgentPreviewSurface {
            kind: AgentPreviewSurfaceKind::NativeCommand,
            label: "Claude Code bugfix command".to_string(),
            path: Some(".claude/commands/ctxpack-bugfix.md".to_string()),
            summary:
                "Project-local command instructs Claude to call prepare_task, inspect targets natively, and use get_pack progressively."
                    .to_string(),
        }),
        "cursor" => surfaces.push(AgentPreviewSurface {
            kind: AgentPreviewSurfaceKind::NativeRule,
            label: "Cursor project rule".to_string(),
            path: Some(".cursor/rules/ctxpack.mdc".to_string()),
            summary:
                "Project rule steers Cursor toward ctxpack MCP planning while leaving file reads and edits to Cursor."
                    .to_string(),
        }),
        "opencode" => surfaces.push(AgentPreviewSurface {
            kind: AgentPreviewSurfaceKind::AdapterSnippet,
            label: "OpenCode MCP snippet".to_string(),
            path: Some(".ctxpack/adapters/opencode.jsonc.snippet".to_string()),
            summary:
                "Repo-local snippet can be reviewed and merged by the user; ctxpack does not mutate OpenCode config."
                    .to_string(),
        }),
        "generic" => surfaces.push(AgentPreviewSurface {
            kind: AgentPreviewSurfaceKind::McpResource,
            label: "Generic MCP prompt guide".to_string(),
            path: Some("ctxpack://guides/context-pack".to_string()),
            summary:
                "Generic clients can list tools/resources/prompts and request pack resources on demand."
                    .to_string(),
        }),
        _ => surfaces.push(AgentPreviewSurface {
            kind: AgentPreviewSurfaceKind::CliFallback,
            label: "CLI fallback".to_string(),
            path: None,
            summary:
                "Custom agents can call `ctxpack prepare-task` and `ctxpack get-pack` when MCP is unavailable."
                    .to_string(),
        }),
    }

    surfaces
}

fn native_rule_surfaces(target_agent: &str) -> Vec<AgentPreviewSurface> {
    match target_agent {
        "claude-code" => vec![AgentPreviewSurface {
            kind: AgentPreviewSurfaceKind::AdapterSnippet,
            label: "Claude project MCP snippet".to_string(),
            path: Some(".ctxpack/adapters/claude-mcp.json".to_string()),
            summary: "Mergeable project MCP snippet for local stdio ctxpack.".to_string(),
        }],
        "cursor" => vec![AgentPreviewSurface {
            kind: AgentPreviewSurfaceKind::NativeRule,
            label: "Cursor rule file".to_string(),
            path: Some(".cursor/rules/ctxpack.mdc".to_string()),
            summary: "Versioned rule file, not an editor extension or global setting.".to_string(),
        }],
        "opencode" => vec![AgentPreviewSurface {
            kind: AgentPreviewSurfaceKind::AdapterSnippet,
            label: "OpenCode config snippet".to_string(),
            path: Some(".ctxpack/adapters/opencode.jsonc.snippet".to_string()),
            summary: "Manual merge snippet for OpenCode local MCP config.".to_string(),
        }],
        _ => Vec::new(),
    }
}

fn next_steps(target_agent: &str) -> Vec<AgentPreviewStep> {
    vec![
        AgentPreviewStep {
            order: 1,
            action: "Call ctxpack.prepare_task with the active repo path.".to_string(),
            owner: target_agent.to_string(),
            source_bearing: false,
        },
        AgentPreviewStep {
            order: 2,
            action: "Read returned target files and related tests with native file tools."
                .to_string(),
            owner: target_agent.to_string(),
            source_bearing: true,
        },
        AgentPreviewStep {
            order: 3,
            action: "Call ctxpack.get_pack with brief or standard budget only if more context is needed."
                .to_string(),
            owner: target_agent.to_string(),
            source_bearing: true,
        },
        AgentPreviewStep {
            order: 4,
            action: "Edit files and run validation commands through the agent permission model."
                .to_string(),
            owner: target_agent.to_string(),
            source_bearing: false,
        },
    ]
}

fn boundary_lines() -> Vec<String> {
    vec![
        "ctxpack is read-only and does not edit source files.".to_string(),
        "ctxpack suggests target files, related tests, context packs, and validation commands."
            .to_string(),
        "The target coding agent owns file reads, edits, shell commands, approvals, and final changes."
            .to_string(),
        "Preview output is source-free; source-bearing snippets only appear in explicit pack exports."
            .to_string(),
        "Cloud embeddings and cloud reranking remain disabled by default.".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn agent_preview_report_is_source_free_and_agent_specific() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("src/auth")).unwrap();
        fs::write(
            temp.path().join("src/auth/session.ts"),
            "export function requireSession() { return 'AGENT_PREVIEW_SENTINEL'; }\n",
        )
        .unwrap();

        let report = build_agent_preview_report(
            temp.path(),
            "fix requireSession",
            TaskType::BugFix,
            PackBudget::Brief,
            "all",
            &[],
            false,
        )
        .unwrap();

        let serialized = serde_json::to_string(&report).unwrap();
        assert_eq!(report.previews.len(), 5);
        assert!(serialized.contains("\"targetAgent\":\"codex\""));
        assert!(serialized.contains("\"targetAgent\":\"claude-code\""));
        assert!(serialized.contains("\"targetAgent\":\"cursor\""));
        assert!(serialized.contains("\"targetAgent\":\"opencode\""));
        assert!(serialized.contains("\"targetAgent\":\"generic\""));
        assert!(serialized.contains("prepare_task"));
        assert!(serialized.contains("get_pack"));
        assert!(serialized.contains("AGENTS.md"));
        assert!(serialized.contains("native file tools"));
        assert!(!serialized.contains("AGENT_PREVIEW_SENTINEL"));
        assert!(!report.source_text_logged);
        assert!(report
            .previews
            .iter()
            .all(|preview| !preview.source_text_included));
    }
}
