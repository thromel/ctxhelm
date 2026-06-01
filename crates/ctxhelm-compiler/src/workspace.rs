use crate::packs::compile_context_pack_from_plan_for_agent;
use crate::planning::{normalized_target_agent, prepare_context_plan_with_paths_and_semantic};
use ctxhelm_core::{
    DiagnosticSeverity, PackBudget, TaskType, WorkspaceContextPack, WorkspaceContextPlan,
    WorkspaceRepoDiagnostic, WorkspaceRepoPack, WorkspaceRepoPlan, WorkspaceRepoPrivacyStatus,
};
use ctxhelm_index::{
    default_workspace_manifest_path, load_workspace_manifest, repo_id_for_path,
    workspace_inventory_status_for_manifest, InventoryError,
};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

const WORKSPACE_REPO_LIMIT: usize = 3;

pub fn prepare_workspace_context_plan(
    workspace_root: impl AsRef<Path>,
    manifest_path: Option<&Path>,
    task: &str,
    task_type: TaskType,
    anchor_paths: &[String],
    semantic_enabled: bool,
) -> Result<WorkspaceContextPlan, InventoryError> {
    let (plan, _) = prepare_workspace_context_plan_with_roots(
        workspace_root,
        manifest_path,
        task,
        task_type,
        anchor_paths,
        semantic_enabled,
    )?;
    Ok(plan)
}

#[allow(clippy::too_many_arguments)]
pub fn compile_workspace_context_pack(
    workspace_root: impl AsRef<Path>,
    manifest_path: Option<&Path>,
    task: &str,
    task_type: TaskType,
    budget: PackBudget,
    anchor_paths: &[String],
    target_agent: &str,
    semantic_enabled: bool,
) -> Result<WorkspaceContextPack, InventoryError> {
    let (plan, roots) = prepare_workspace_context_plan_with_roots(
        workspace_root,
        manifest_path,
        task,
        task_type,
        anchor_paths,
        semantic_enabled,
    )?;
    let mut warnings = Vec::new();
    if plan.repo_plans.len() >= WORKSPACE_REPO_LIMIT {
        warnings.push(format!(
            "Workspace pack limited to the top {WORKSPACE_REPO_LIMIT} repository plan(s)."
        ));
    }
    let mut repo_packs = Vec::new();
    for repo_plan in &plan.repo_plans {
        let Some(repo_root) = roots
            .iter()
            .find(|root| root.repo_id == repo_plan.repo_id)
            .map(|root| root.repo_root.as_path())
        else {
            warnings.push(format!(
                "Workspace pack skipped `{}` because its repository root was unavailable.",
                repo_plan.label
            ));
            continue;
        };
        let pack = compile_context_pack_from_plan_for_agent(
            repo_root,
            task,
            &repo_plan.context_plan,
            budget.clone(),
            target_agent,
        );
        repo_packs.push(WorkspaceRepoPack {
            repo_id: repo_plan.repo_id.clone(),
            label: repo_plan.label.clone(),
            path_label: repo_plan.path_label.clone(),
            tags: repo_plan.tags.clone(),
            confidence: repo_plan.confidence,
            reason: repo_plan.reason.clone(),
            context_pack: pack,
        });
    }
    let token_estimate = repo_packs
        .iter()
        .map(|repo| repo.context_pack.token_estimate)
        .sum();
    Ok(WorkspaceContextPack {
        id: Uuid::new_v4(),
        task_id: plan.task_id,
        task_type: plan.task_type,
        target_agent: normalized_target_agent(target_agent),
        budget,
        confidence: plan.confidence,
        token_estimate,
        workspace_root: plan.workspace_root,
        manifest_path: plan.manifest_path,
        selected_repo_count: repo_packs.len(),
        source_text_logged: false,
        privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
        warnings,
        repo_packs,
        diagnostics: plan.diagnostics,
    })
}

fn prepare_workspace_context_plan_with_roots(
    workspace_root: impl AsRef<Path>,
    manifest_path: Option<&Path>,
    task: &str,
    task_type: TaskType,
    anchor_paths: &[String],
    semantic_enabled: bool,
) -> Result<(WorkspaceContextPlan, Vec<WorkspaceRepoRoot>), InventoryError> {
    let workspace_root = canonicalize_existing(workspace_root.as_ref())?;
    let manifest_path = match manifest_path {
        Some(path) => path.to_path_buf(),
        None => default_workspace_manifest_path(&workspace_root)?,
    };
    let (loaded_manifest_path, manifest) =
        load_workspace_manifest(&workspace_root, Some(&manifest_path))?;
    let status =
        workspace_inventory_status_for_manifest(&workspace_root, &loaded_manifest_path, &manifest);
    let manifest_base = manifest_repo_base(&loaded_manifest_path, &workspace_root);
    let mut diagnostics = status.diagnostics.clone();
    let mut repo_plans = Vec::new();
    let mut roots = Vec::new();

    for repo in &manifest.repos {
        let resolved = resolve_repo_path(manifest_base, &repo.path);
        if !resolved.exists() {
            continue;
        }
        let canonical = match canonicalize_existing(&resolved) {
            Ok(path) => path,
            Err(error) => {
                diagnostics.push(WorkspaceRepoDiagnostic {
                    code: "workspace_plan_repo_inaccessible".to_string(),
                    severity: DiagnosticSeverity::Warning,
                    message: format!("Workspace repository could not be planned: {error}"),
                    repo_id: repo.id.clone(),
                    label: repo.label.clone(),
                    paths: vec![repo.path.clone()],
                });
                continue;
            }
        };
        let repo_id = repo
            .id
            .clone()
            .unwrap_or_else(|| repo_id_for_path(&canonical));
        let path_label = path_label(&workspace_root, &canonical);
        let label = repo.label.clone().unwrap_or_else(|| path_label.clone());
        let repo_anchors =
            anchor_paths_for_repo(&workspace_root, &canonical, &path_label, anchor_paths);
        let plan = prepare_context_plan_with_paths_and_semantic(
            &canonical,
            task,
            task_type.clone(),
            &repo_anchors,
            semantic_enabled,
        )?;
        let confidence = workspace_repo_confidence(&plan);
        let reason = workspace_repo_reason(&plan);
        repo_plans.push(WorkspaceRepoPlan {
            repo_id,
            label,
            path_label,
            tags: repo.tags.clone(),
            confidence,
            reason,
            context_plan: plan,
        });
        roots.push(WorkspaceRepoRoot {
            repo_id: repo_plans.last().unwrap().repo_id.clone(),
            repo_root: canonical,
        });
    }

    repo_plans.sort_by(|left, right| {
        right
            .confidence
            .partial_cmp(&left.confidence)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.label.cmp(&right.label))
    });
    repo_plans.truncate(WORKSPACE_REPO_LIMIT);

    if repo_plans.is_empty() {
        diagnostics.push(WorkspaceRepoDiagnostic {
            code: "workspace_plan_no_candidate_repos".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: "No available workspace repositories produced a context plan.".to_string(),
            repo_id: None,
            label: None,
            paths: Vec::new(),
        });
    }

    let confidence = repo_plans
        .first()
        .map(|repo| repo.confidence)
        .unwrap_or(0.0);
    Ok((
        WorkspaceContextPlan {
            task_id: Uuid::new_v4(),
            task_type,
            confidence,
            workspace_root: workspace_root.display().to_string(),
            manifest_path: loaded_manifest_path.display().to_string(),
            selected_repo_count: repo_plans.len(),
            source_text_logged: false,
            privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
            repo_plans,
            diagnostics,
        },
        roots,
    ))
}

#[derive(Debug, Clone)]
struct WorkspaceRepoRoot {
    repo_id: String,
    repo_root: PathBuf,
}

fn workspace_repo_confidence(plan: &ctxhelm_core::ContextPlan) -> f32 {
    let target_bonus = (plan.target_files.len().min(4) as f32) * 0.05;
    let test_bonus = (plan.related_tests.len().min(2) as f32) * 0.05;
    (plan.confidence + target_bonus + test_bonus).min(0.99)
}

fn workspace_repo_reason(plan: &ctxhelm_core::ContextPlan) -> String {
    if plan.target_files.is_empty() {
        "No strong safe file matches; kept as a low-confidence workspace candidate.".to_string()
    } else {
        format!(
            "Task matched {} target file(s), {} related test(s), and {} retrieval candidate(s).",
            plan.target_files.len(),
            plan.related_tests.len(),
            plan.retrieval_candidates.len()
        )
    }
}

fn anchor_paths_for_repo(
    workspace_root: &Path,
    repo_root: &Path,
    path_label: &str,
    anchors: &[String],
) -> Vec<String> {
    let mut selected = BTreeSet::new();
    for anchor in anchors {
        let raw = PathBuf::from(anchor);
        if raw.is_absolute() {
            if let Ok(relative) = raw.strip_prefix(repo_root) {
                selected.insert(normalize_components(relative));
            }
            continue;
        }
        if let Ok(relative) = raw.strip_prefix(path_label) {
            selected.insert(normalize_components(relative));
            continue;
        }
        let workspace_relative = workspace_root.join(&raw);
        if let Ok(relative) = workspace_relative.strip_prefix(repo_root) {
            selected.insert(normalize_components(relative));
            continue;
        }
        let repo_relative = repo_root.join(&raw);
        if repo_relative.exists() {
            selected.insert(normalize_components(&raw));
        }
    }
    selected
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect()
}

fn resolve_repo_path(manifest_base: &Path, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        manifest_base.join(path)
    }
}

fn manifest_repo_base<'a>(manifest_path: &'a Path, workspace_root: &'a Path) -> &'a Path {
    let Some(parent) = manifest_path.parent() else {
        return workspace_root;
    };
    if parent.file_name().is_some_and(|name| name == ".ctxhelm") {
        parent.parent().unwrap_or(workspace_root)
    } else {
        parent
    }
}

fn path_label(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn normalize_components(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf, InventoryError> {
    fs::canonicalize(path).map_err(|source| InventoryError::Canonicalize {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctxhelm_core::{TaskType, WorkspaceManifest, WorkspaceRepo};
    use ctxhelm_index::{write_workspace_manifest, WORKSPACE_MANIFEST_SCHEMA_VERSION};
    use std::process::Command;

    fn git_repo(path: &Path, file: &str, source: &str) {
        fs::create_dir_all(path.join("src")).unwrap();
        fs::create_dir_all(path.join("tests")).unwrap();
        fs::write(path.join(file), source).unwrap();
        fs::write(path.join("tests/flow.test.ts"), source).unwrap();
        Command::new("git")
            .arg("init")
            .current_dir(path)
            .output()
            .unwrap();
    }

    #[test]
    fn workspace_plan_ranks_matching_repo_without_leaking_source() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().join("workspace");
        let api = workspace.join("api");
        let web = workspace.join("web");
        git_repo(
            &api,
            "src/billing.ts",
            "export function invoiceReconciliation() { return true; }\n",
        );
        git_repo(
            &web,
            "src/login.ts",
            "export function loginRedirect() { return true; }\n",
        );
        let manifest = WorkspaceManifest {
            schema_version: WORKSPACE_MANIFEST_SCHEMA_VERSION,
            workspace_id: None,
            repos: vec![
                WorkspaceRepo {
                    id: Some("api".to_string()),
                    path: "api".to_string(),
                    label: Some("api".to_string()),
                    tags: vec!["backend".to_string()],
                },
                WorkspaceRepo {
                    id: Some("web".to_string()),
                    path: "web".to_string(),
                    label: Some("web".to_string()),
                    tags: vec!["frontend".to_string()],
                },
            ],
        };
        write_workspace_manifest(&workspace, &manifest).unwrap();

        let plan = prepare_workspace_context_plan(
            &workspace,
            None,
            "fix login redirect",
            TaskType::BugFix,
            &[],
            false,
        )
        .unwrap();

        assert_eq!(plan.selected_repo_count, 2);
        assert_eq!(plan.repo_plans[0].label, "web");
        assert!(!plan.source_text_logged);
        let json = serde_json::to_string(&plan).unwrap();
        assert!(!json.contains("export function loginRedirect"));
        assert!(!json.contains("export function invoiceReconciliation"));
    }

    #[test]
    fn workspace_pack_preserves_repo_boundaries() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().join("workspace");
        let api = workspace.join("api");
        let web = workspace.join("web");
        git_repo(
            &api,
            "src/billing.ts",
            "export function invoiceReconciliation() { return true; }\n",
        );
        git_repo(
            &web,
            "src/login.ts",
            "export function loginRedirect() { return true; }\n",
        );
        let manifest = WorkspaceManifest {
            schema_version: WORKSPACE_MANIFEST_SCHEMA_VERSION,
            workspace_id: None,
            repos: vec![
                WorkspaceRepo {
                    id: Some("api".to_string()),
                    path: "api".to_string(),
                    label: Some("api".to_string()),
                    tags: Vec::new(),
                },
                WorkspaceRepo {
                    id: Some("web".to_string()),
                    path: "web".to_string(),
                    label: Some("web".to_string()),
                    tags: Vec::new(),
                },
            ],
        };
        write_workspace_manifest(&workspace, &manifest).unwrap();

        let pack = compile_workspace_context_pack(
            &workspace,
            None,
            "fix login redirect",
            TaskType::BugFix,
            PackBudget::Brief,
            &[],
            "codex",
            false,
        )
        .unwrap();

        assert_eq!(pack.selected_repo_count, 2);
        assert_eq!(pack.repo_packs[0].label, "web");
        assert_eq!(pack.repo_packs[0].context_pack.target_agent, "codex");
        assert!(!pack.source_text_logged);
        let json = serde_json::to_value(&pack).unwrap();
        assert_eq!(json["repoPacks"][0]["contextPack"]["targetAgent"], "codex");
        assert!(json["tokenEstimate"].as_u64().unwrap() > 0);
    }
}
