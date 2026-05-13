use ctxpack_core::{Diagnostic, FileRole, PrivacyStatus};
use ctxpack_index::{
    dependency_edges_report, extract_symbols_report, load_or_refresh_inventory, test_map_report,
    DependencyEdge, DependencyOptions, InventoryError, InventoryOptions, RelatedTestResult,
    RepoInventory,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextCardsOptions {
    pub limit: usize,
}

impl Default for ContextCardsOptions {
    fn default() -> Self {
        Self { limit: 40 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedContextCard {
    pub name: String,
    pub path: PathBuf,
    pub title: String,
    pub bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextCardsReport {
    pub repo_id: String,
    pub cards_dir: PathBuf,
    pub cards: Vec<GeneratedContextCard>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub privacy_status: PrivacyStatus,
}

pub fn generate_context_cards(
    repo_root: impl AsRef<Path>,
    options: &ContextCardsOptions,
) -> Result<ContextCardsReport, InventoryError> {
    let repo_root = repo_root.as_ref();
    let inventory_report = load_or_refresh_inventory(repo_root, &InventoryOptions::default())?;
    let mut diagnostics = inventory_report.diagnostics.clone();
    let inventory = inventory_report.inventory;
    let repo_root = inventory.repo_root.clone();
    let repo_id = inventory.repo_id.clone();
    let limit = options.limit.max(1);
    let cards_dir = repo_root.join(".ctxpack").join("cards");
    fs::create_dir_all(&cards_dir).map_err(|source| InventoryError::CreateDir {
        path: cards_dir.clone(),
        source,
    })?;

    let symbol_report = extract_symbols_report(&repo_root)?;
    diagnostics.extend(symbol_report.diagnostics.clone());
    let symbols = symbol_report.symbols;
    let tests_report = test_map_report(&repo_root)?;
    diagnostics.extend(tests_report.diagnostics.clone());
    let tests = tests_report.results;
    let dependency_report = dependency_edges_report(&repo_root, &DependencyOptions { limit })?;
    diagnostics.extend(dependency_report.diagnostics.clone());
    let edges = dependency_report.edges;

    let cards = [
        (
            "repo-overview",
            "Repo Overview",
            render_repo_overview_card(&repo_id, &inventory, &symbols, limit),
        ),
        (
            "testing",
            "Testing",
            render_testing_card(&repo_id, &tests, limit),
        ),
        (
            "dependency-graph",
            "Dependency Graph",
            render_dependency_card(&repo_id, &edges, limit),
        ),
    ];

    let mut generated = Vec::new();
    for (name, title, content) in cards {
        let path = cards_dir.join(format!("{name}.md"));
        fs::write(&path, &content).map_err(|source| InventoryError::Write {
            path: path.clone(),
            source,
        })?;
        generated.push(GeneratedContextCard {
            name: name.to_string(),
            path,
            title: title.to_string(),
            bytes: content.len(),
        });
    }

    Ok(ContextCardsReport {
        repo_id,
        cards_dir,
        cards: generated,
        diagnostics,
        privacy_status: PrivacyStatus::local_only(),
    })
}

fn render_repo_overview_card(
    repo_id: &str,
    inventory: &RepoInventory,
    symbols: &[ctxpack_index::CodeSymbol],
    limit: usize,
) -> String {
    let mut role_counts = BTreeMap::new();
    let mut language_counts = BTreeMap::new();
    for file in &inventory.files {
        *role_counts
            .entry(format!("{:?}", file.role))
            .or_insert(0usize) += 1;
        if let Some(language) = &file.language {
            *language_counts.entry(language.clone()).or_insert(0usize) += 1;
        }
    }

    let mut output = card_header("Repo Overview", repo_id);
    output.push_str("## File Roles\n\n");
    push_count_list(&mut output, &role_counts);
    output.push_str("\n## Languages\n\n");
    push_count_list(&mut output, &language_counts);
    output.push_str("\n## Key Files\n\n");
    let key_files = inventory
        .files
        .iter()
        .filter(|file| {
            matches!(
                file.role,
                FileRole::Source
                    | FileRole::Test
                    | FileRole::Config
                    | FileRole::Schema
                    | FileRole::Docs
            )
        })
        .take(limit)
        .map(|file| format!("`{}` ({:?})", file.path, file.role))
        .collect::<Vec<_>>();
    push_bullet_items(&mut output, &key_files, "No safe files were inventoried.");

    output.push_str("\n## Symbols\n\n");
    let symbol_items = symbols
        .iter()
        .filter(|symbol| {
            symbol.exported
                || matches!(
                    symbol.kind,
                    ctxpack_index::SymbolKind::Class
                        | ctxpack_index::SymbolKind::Interface
                        | ctxpack_index::SymbolKind::Function
                )
        })
        .take(limit)
        .map(|symbol| {
            format!(
                "`{}` {:?} at `{}`:{}",
                symbol.name, symbol.kind, symbol.path, symbol.start_line
            )
        })
        .collect::<Vec<_>>();
    push_bullet_items(&mut output, &symbol_items, "No symbols were extracted.");
    output
}

fn render_testing_card(repo_id: &str, tests: &[RelatedTestResult], limit: usize) -> String {
    let mut output = card_header("Testing", repo_id);
    output.push_str("## Test Files\n\n");
    if tests.is_empty() {
        output.push_str("- No safe test files were detected.\n");
        return output;
    }

    for test in tests.iter().take(limit) {
        output.push_str(&format!("- `{}`\n", test.path));
        if let Some(command) = &test.command {
            output.push_str(&format!("  - Command: `{command}`\n"));
        }
    }
    output
}

fn render_dependency_card(repo_id: &str, edges: &[DependencyEdge], limit: usize) -> String {
    let mut output = card_header("Dependency Graph", repo_id);
    output.push_str("## Safe Local Import Edges\n\n");
    if edges.is_empty() {
        output.push_str("- No safe local dependency edges were detected.\n");
        return output;
    }

    for edge in edges.iter().take(limit) {
        output.push_str(&format!(
            "- `{}` -> `{}` ({}, confidence {:.2})\n",
            edge.source_path, edge.target_path, edge.kind, edge.confidence
        ));
    }
    output
}

fn card_header(title: &str, repo_id: &str) -> String {
    format!(
        "# {title}\n\n- Generated by: `ctxpack cards generate`\n- Repo ID: `{repo_id}`\n- Privacy: local-only `true`\n- Source snippets included: `false`\n\n"
    )
}

fn push_count_list(output: &mut String, counts: &BTreeMap<String, usize>) {
    if counts.is_empty() {
        output.push_str("- None detected.\n");
        return;
    }
    for (name, count) in counts {
        output.push_str(&format!("- `{name}`: `{count}`\n"));
    }
}

fn push_bullet_items(output: &mut String, items: &[String], empty_message: &str) {
    if items.is_empty() {
        output.push_str(&format!("- {empty_message}\n"));
        return;
    }
    for item in items {
        output.push_str(&format!("- {item}\n"));
    }
}
