use ctxhelm_core::{
    Diagnostic, FileRole, MemoryCard, MemoryCardKind, MemoryFreshness, MemoryReviewStatus,
    PrivacyStatus,
};
use ctxhelm_index::{
    dependency_edges_report, extract_symbols_report, list_eval_traces, load_or_refresh_inventory,
    persist_memory_card_records, test_map_report, DependencyEdge, DependencyOptions,
    InventoryError, InventoryOptions, RelatedTestResult, RepoInventory, StorageMemoryCardRecord,
    StoreConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
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
    pub memory_card_id: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FallbackCardsOptions {
    pub limit: usize,
    pub target_agent: String,
}

impl Default for FallbackCardsOptions {
    fn default() -> Self {
        Self {
            limit: 40,
            target_agent: "generic".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FallbackCardsReport {
    pub repo_id: String,
    pub target_agent: String,
    pub guide_path: PathBuf,
    pub cards_dir: PathBuf,
    pub card_count: usize,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub privacy_status: PrivacyStatus,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExperienceCardsOptions {
    pub limit: usize,
}

impl Default for ExperienceCardsOptions {
    fn default() -> Self {
        Self { limit: 20 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExperienceCardsReport {
    pub repo_id: String,
    pub cards: Vec<MemoryCard>,
    pub stored_records: usize,
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
    let cards_dir = repo_root.join(".ctxhelm").join("cards");
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
            repo_overview_memory_card(&repo_id, &inventory, limit),
        ),
        (
            "testing",
            "Testing",
            render_testing_card(&repo_id, &tests, limit),
            testing_memory_card(&repo_id, &tests, limit),
        ),
        (
            "dependency-graph",
            "Dependency Graph",
            render_dependency_card(&repo_id, &edges, limit),
            dependency_memory_card(&repo_id, &edges, limit),
        ),
    ];

    let mut generated = Vec::new();
    let mut memory_records = Vec::new();
    for (name, title, content, memory_card) in cards {
        let path = cards_dir.join(format!("{name}.md"));
        fs::write(&path, &content).map_err(|source| InventoryError::Write {
            path: path.clone(),
            source,
        })?;
        let memory_card_id = memory_card.id.clone();
        memory_records.push(StorageMemoryCardRecord { card: memory_card });
        generated.push(GeneratedContextCard {
            name: name.to_string(),
            path,
            title: title.to_string(),
            bytes: content.len(),
            memory_card_id,
        });
    }
    for (name, card) in domain_memory_cards(&repo_id, &inventory, limit) {
        let content = render_domain_card(&repo_id, &card);
        let path = cards_dir.join(format!("{name}.md"));
        fs::write(&path, &content).map_err(|source| InventoryError::Write {
            path: path.clone(),
            source,
        })?;
        let memory_card_id = card.id.clone();
        memory_records.push(StorageMemoryCardRecord { card });
        generated.push(GeneratedContextCard {
            name,
            path,
            title: "Domain Memory".to_string(),
            bytes: content.len(),
            memory_card_id,
        });
    }
    if let Ok(status) =
        persist_memory_card_records(&repo_root, &StoreConfig::default(), &memory_records)
    {
        diagnostics.push(Diagnostic {
            code: "memory_cards_persisted".to_string(),
            severity: ctxhelm_core::DiagnosticSeverity::Info,
            message: format!(
                "Stored {} source-free memory card record(s) in {}",
                status.memory_card_records,
                status.database_path.display()
            ),
            paths: vec![status.database_path.display().to_string()],
            count: status.memory_card_records,
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

pub fn generate_fallback_cards(
    repo_root: impl AsRef<Path>,
    options: &FallbackCardsOptions,
) -> Result<FallbackCardsReport, InventoryError> {
    let repo_root = repo_root.as_ref();
    let target_agent = normalize_target_agent(&options.target_agent);
    let cards_report = generate_context_cards(
        repo_root,
        &ContextCardsOptions {
            limit: options.limit,
        },
    )?;
    let fallback_dir = repo_root.join(".ctxhelm").join("fallback");
    fs::create_dir_all(&fallback_dir).map_err(|source| InventoryError::CreateDir {
        path: fallback_dir.clone(),
        source,
    })?;
    let guide_path = fallback_dir.join(format!("{target_agent}-context.md"));
    let guide = render_fallback_guide(&cards_report, &target_agent);
    fs::write(&guide_path, guide).map_err(|source| InventoryError::Write {
        path: guide_path.clone(),
        source,
    })?;

    Ok(FallbackCardsReport {
        repo_id: cards_report.repo_id,
        target_agent,
        guide_path,
        cards_dir: cards_report.cards_dir,
        card_count: cards_report.cards.len(),
        diagnostics: cards_report.diagnostics,
        privacy_status: PrivacyStatus::local_only(),
        source_text_logged: false,
    })
}

fn render_fallback_guide(report: &ContextCardsReport, target_agent: &str) -> String {
    let mut output = String::new();
    output.push_str("# ctxhelm Disconnected Fallback\n\n");
    output.push_str(&format!("- Target agent: `{target_agent}`\n"));
    output.push_str(&format!("- Repo ID: `{}`\n", report.repo_id));
    output.push_str("- Generated by: `ctxhelm cards fallback`\n");
    output.push_str("- Privacy: local-only `true`\n");
    output.push_str("- Source snippets included: `false`\n");
    output.push_str("- Freshness: regenerate before using for long-running or cloud tasks\n\n");
    output.push_str("## How To Use\n\n");
    output.push_str("Use this guide only when the local ctxhelm MCP server is unavailable. Keep normal daily work inside the coding agent and prefer MCP `prepare_task`/`get_pack` when available.\n\n");
    output.push_str("1. Read `AGENTS.md` first for stable repo rules.\n");
    output.push_str("2. Read the source-free cards listed below for broad orientation.\n");
    output.push_str(
        "3. Use the agent's native file tools to inspect actual source files before editing.\n",
    );
    output.push_str("4. Do not treat these cards as current state after major branch changes; regenerate them.\n\n");
    output.push_str("## Source-Free Cards\n\n");
    for card in &report.cards {
        output.push_str(&format!(
            "- `{}`: {} (`{}`), memory card `{}`\n",
            card.name,
            card.title,
            card.path.display(),
            card.memory_card_id
        ));
    }
    output.push_str("\n## Agent Notes\n\n");
    output.push_str(agent_fallback_note(target_agent));
    output.push('\n');
    output
}

fn agent_fallback_note(target_agent: &str) -> &'static str {
    match target_agent {
        "codex" => "Codex cloud or isolated runs should rely on committed cards and AGENTS.md, then inspect files natively inside the task environment.",
        "claude-code" => "Claude Code should prefer the local MCP server when available; this fallback is for disconnected or copied project contexts.",
        "cursor" => "Cursor should use its own index for broad repo search and these cards only as stable source-free orientation.",
        "opencode" => "OpenCode should keep AGENTS.md as the rule source and use these cards as optional context when MCP is not configured.",
        _ => "Generic MCP or chat clients should treat these cards as source-free orientation, not as a replacement for reading current files.",
    }
}

fn normalize_target_agent(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "codex" | "codex-cli" => "codex".to_string(),
        "claude" | "claude-code" => "claude-code".to_string(),
        "cursor" => "cursor".to_string(),
        "opencode" | "open-code" => "opencode".to_string(),
        _ => "generic".to_string(),
    }
}

pub fn generate_experience_cards(
    repo_root: impl AsRef<Path>,
    options: &ExperienceCardsOptions,
) -> Result<ExperienceCardsReport, InventoryError> {
    let repo_root = repo_root.as_ref();
    let inventory_report = load_or_refresh_inventory(repo_root, &InventoryOptions::default())?;
    let repo_root = inventory_report.inventory.repo_root;
    let repo_id = inventory_report.inventory.repo_id;
    let mut diagnostics = inventory_report.diagnostics;
    let traces = list_eval_traces(&repo_root, options.limit.max(1))?;
    let mut cards = Vec::new();
    for trace in traces {
        let mut links = Vec::new();
        let mut seen_links = BTreeSet::new();
        for link in trace
            .recommended_files
            .iter()
            .chain(trace.recommended_tests.iter())
        {
            if seen_links.insert(link.clone()) {
                links.push(link.clone());
            }
        }
        let title = format!(
            "Experience: {:?} {}",
            trace.task_type,
            short_hash(&trace.task_hash)
        );
        let summary = format!(
            "A prior {:?} run for agent `{}` selected {} file(s), {} test(s), and {} validation command(s). Reuse this only when the current task overlaps the same files or tests.",
            trace.task_type,
            trace.target_agent,
            trace.recommended_files.len(),
            trace.recommended_tests.len(),
            trace.recommended_commands.len()
        );
        cards.push(MemoryCard {
            id: format!("experience:{}", trace.task_hash),
            kind: MemoryCardKind::Experience,
            title,
            summary,
            source_links: links,
            input_hashes: vec![trace.task_hash],
            freshness: MemoryFreshness::Fresh,
            review_status: MemoryReviewStatus::Pending,
            disabled: false,
            confidence: 0.55,
            reason: "Derived from source-free local eval trace metadata; pending review before pack inclusion.".to_string(),
            privacy_status: PrivacyStatus::local_only(),
        });
    }
    let stored_records = cards.len();
    if stored_records > 0 {
        let records = cards
            .iter()
            .cloned()
            .map(|card| StorageMemoryCardRecord { card })
            .collect::<Vec<_>>();
        match persist_memory_card_records(&repo_root, &StoreConfig::default(), &records) {
            Ok(status) => diagnostics.push(Diagnostic {
                code: "experience_cards_persisted".to_string(),
                severity: ctxhelm_core::DiagnosticSeverity::Info,
                message: format!(
                    "Stored {} source-free experience card record(s) in {}",
                    stored_records,
                    status.database_path.display()
                ),
                paths: vec![status.database_path.display().to_string()],
                count: stored_records,
            }),
            Err(error) => diagnostics.push(Diagnostic {
                code: "experience_cards_store_failed".to_string(),
                severity: ctxhelm_core::DiagnosticSeverity::Warning,
                message: format!("Experience cards were generated but not stored: {error}"),
                paths: Vec::new(),
                count: stored_records,
            }),
        }
    }

    Ok(ExperienceCardsReport {
        repo_id,
        cards,
        stored_records,
        diagnostics,
        privacy_status: PrivacyStatus::local_only(),
    })
}

fn repo_overview_memory_card(repo_id: &str, inventory: &RepoInventory, limit: usize) -> MemoryCard {
    let links = inventory
        .files
        .iter()
        .filter(|file| {
            matches!(
                file.role,
                FileRole::Source | FileRole::Config | FileRole::Docs
            )
        })
        .take(limit)
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let hashes = inventory
        .files
        .iter()
        .take(limit)
        .map(|file| file.hash.clone())
        .collect::<Vec<_>>();
    memory_card(
        repo_id,
        "repo-overview",
        MemoryCardKind::Domain,
        "Repo Overview",
        format!(
            "Safe inventory has {} files across source, test, config, schema, docs, and generated categories.",
            inventory.files.len()
        ),
        links,
        hashes,
        "Generated from safe inventory role and language metadata.",
    )
}

fn testing_memory_card(repo_id: &str, tests: &[RelatedTestResult], limit: usize) -> MemoryCard {
    let links = tests
        .iter()
        .take(limit)
        .map(|test| test.path.clone())
        .collect::<Vec<_>>();
    memory_card(
        repo_id,
        "testing",
        MemoryCardKind::Domain,
        "Testing",
        format!(
            "Detected {} safe test file(s) with package-aware validation commands where available.",
            tests.len()
        ),
        links,
        Vec::new(),
        "Generated from source-free test map metadata.",
    )
}

fn dependency_memory_card(repo_id: &str, edges: &[DependencyEdge], limit: usize) -> MemoryCard {
    let mut links = Vec::new();
    for edge in edges.iter().take(limit) {
        links.push(edge.source_path.clone());
        links.push(edge.target_path.clone());
    }
    links.sort();
    links.dedup();
    memory_card(
        repo_id,
        "dependency-graph",
        MemoryCardKind::Domain,
        "Dependency Graph",
        format!(
            "Detected {} safe local dependency edge(s) for graph-aware context expansion.",
            edges.len()
        ),
        links,
        Vec::new(),
        "Generated from source-free local dependency metadata.",
    )
}

fn domain_memory_cards(
    repo_id: &str,
    inventory: &RepoInventory,
    limit: usize,
) -> Vec<(String, MemoryCard)> {
    let mut groups: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for file in &inventory.files {
        if !matches!(
            file.role,
            FileRole::Source
                | FileRole::Test
                | FileRole::Config
                | FileRole::Schema
                | FileRole::Docs
        ) {
            continue;
        }
        let domain = file
            .path
            .split('/')
            .next()
            .filter(|part| !part.is_empty())
            .unwrap_or("repo")
            .to_string();
        groups.entry(domain).or_default().push(file);
    }
    groups
        .into_iter()
        .take(6)
        .map(|(domain, files)| {
            let links = files
                .iter()
                .take(limit)
                .map(|file| file.path.clone())
                .collect::<Vec<_>>();
            let hashes = files
                .iter()
                .take(limit)
                .map(|file| file.hash.clone())
                .collect::<Vec<_>>();
            let slug = slugify(&domain);
            let card = memory_card(
                repo_id,
                &format!("domain-{slug}"),
                MemoryCardKind::Domain,
                &format!("Domain: {domain}"),
                format!(
                    "`{domain}` contains {} safe file(s) that often form a retrieval neighborhood.",
                    files.len()
                ),
                links,
                hashes,
                "Generated from safe inventory path grouping.",
            );
            (format!("domain-{slug}"), card)
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn memory_card(
    repo_id: &str,
    name: &str,
    kind: MemoryCardKind,
    title: &str,
    summary: String,
    source_links: Vec<String>,
    input_hashes: Vec<String>,
    reason: &str,
) -> MemoryCard {
    MemoryCard {
        id: format!(
            "{}:{name}",
            match kind {
                MemoryCardKind::Domain => "domain",
                MemoryCardKind::Experience => "experience",
            }
        ),
        kind,
        title: title.to_string(),
        summary,
        source_links,
        input_hashes,
        freshness: MemoryFreshness::Fresh,
        review_status: MemoryReviewStatus::Deterministic,
        disabled: false,
        confidence: 0.8,
        reason: format!("{reason} Repo ID: {repo_id}."),
        privacy_status: PrivacyStatus::local_only(),
    }
}

fn render_repo_overview_card(
    repo_id: &str,
    inventory: &RepoInventory,
    symbols: &[ctxhelm_index::CodeSymbol],
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
                    ctxhelm_index::SymbolKind::Class
                        | ctxhelm_index::SymbolKind::Interface
                        | ctxhelm_index::SymbolKind::Function
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

fn render_domain_card(repo_id: &str, card: &MemoryCard) -> String {
    let mut output = card_header(&card.title, repo_id);
    output.push_str(&format!(
        "- Memory card ID: `{}`\n- Freshness: `{:?}`\n- Review: `{:?}`\n- Disabled: `{}`\n\n",
        card.id, card.freshness, card.review_status, card.disabled
    ));
    output.push_str("## Summary\n\n");
    output.push_str(&format!("{}\n\n", card.summary));
    output.push_str("## Source Links\n\n");
    push_bullet_items(
        &mut output,
        &card
            .source_links
            .iter()
            .map(|link| format!("`{link}`"))
            .collect::<Vec<_>>(),
        "No source links were selected.",
    );
    output
}

fn card_header(title: &str, repo_id: &str) -> String {
    format!(
        "# {title}\n\n- Generated by: `ctxhelm cards generate`\n- Repo ID: `{repo_id}`\n- Privacy: local-only `true`\n- Source snippets included: `false`\n\n"
    )
}

fn slugify(value: &str) -> String {
    let slug = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "repo".to_string()
    } else {
        slug
    }
}

fn short_hash(value: &str) -> String {
    value.chars().take(8).collect()
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
