use crate::packs::pack_repo_id;
use crate::planning::prepare_context_plan_with_paths;
use ctxhelm_core::{
    Diagnostic, DiagnosticSeverity, FileRole, GraphCommunityView, GraphEdgeView,
    GraphNeighborhoodReport, GraphNodeKind, GraphNodeView, PrivacyStatus, TaskType,
};
use ctxhelm_index::{
    list_feedback_events, list_memory_cards, load_or_refresh_inventory, related_dependency_edges,
    related_tests, task_hash, DependencyOptions, InventoryError, InventoryOptions, StoreConfig,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub fn build_graph_neighborhood_report(
    repo_root: impl AsRef<Path>,
    task: Option<&str>,
    task_type: TaskType,
    anchor_paths: &[String],
    max_nodes: usize,
    max_edges: usize,
) -> Result<GraphNeighborhoodReport, InventoryError> {
    let repo_root = repo_root.as_ref();
    let max_nodes = max_nodes.max(1);
    let max_edges = max_edges.max(1);
    let mut diagnostics = Vec::new();
    let inventory_report = load_or_refresh_inventory(repo_root, &InventoryOptions::default())?;
    diagnostics.extend(inventory_report.diagnostics.clone());
    let roles = inventory_report
        .inventory
        .files
        .iter()
        .map(|file| (file.path.clone(), file.role.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut anchors = anchor_paths
        .iter()
        .filter(|path| !path.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    if anchors.is_empty() {
        if let Some(task) = task {
            let plan = prepare_context_plan_with_paths(repo_root, task, task_type, &[])?;
            anchors = plan
                .target_files
                .iter()
                .take(5)
                .map(|target| target.path.clone())
                .collect();
        }
    }
    anchors.sort();
    anchors.dedup();

    let mut nodes = BTreeMap::<String, GraphNodeView>::new();
    let mut edges = Vec::<GraphEdgeView>::new();
    for anchor in &anchors {
        upsert_file_node(&mut nodes, anchor, roles.get(anchor), 1.0, "anchor");
    }

    let dependency_edges = related_dependency_edges(
        repo_root,
        &anchors,
        &DependencyOptions {
            limit: max_edges.saturating_mul(2),
        },
    )?;
    for edge in dependency_edges {
        upsert_file_node(
            &mut nodes,
            &edge.source_path,
            roles.get(&edge.source_path),
            edge.confidence,
            "dependency",
        );
        upsert_file_node(
            &mut nodes,
            &edge.target_path,
            roles.get(&edge.target_path),
            edge.confidence,
            "dependency",
        );
        edges.push(GraphEdgeView {
            source: file_node_id(&edge.source_path),
            target: file_node_id(&edge.target_path),
            kind: edge.kind,
            weight: edge.confidence,
            reason: edge.reason,
        });
    }

    let related_tests = related_tests(repo_root, &anchors)?;
    for test in related_tests {
        upsert_file_node(
            &mut nodes,
            &test.path,
            Some(&FileRole::Test),
            test.confidence,
            "related_test",
        );
        for anchor in &anchors {
            edges.push(GraphEdgeView {
                source: file_node_id(anchor),
                target: file_node_id(&test.path),
                kind: "tests".to_string(),
                weight: test.confidence,
                reason: test.reason.clone(),
            });
        }
    }

    match list_memory_cards(repo_root, &StoreConfig::default(), false) {
        Ok(cards) => {
            for card in cards.into_iter().take(max_nodes) {
                let memory_id = format!("memory:{}", card.id);
                nodes.entry(memory_id.clone()).or_insert(GraphNodeView {
                    id: memory_id.clone(),
                    kind: GraphNodeKind::Memory,
                    label: card.title.clone(),
                    path: None,
                    role: None,
                    weight: card.confidence,
                    source: "memory".to_string(),
                });
                for link in card.source_links {
                    if anchors.iter().any(|anchor| anchor == &link) {
                        edges.push(GraphEdgeView {
                            source: file_node_id(&link),
                            target: memory_id.clone(),
                            kind: "documents".to_string(),
                            weight: card.confidence,
                            reason: card.reason.clone(),
                        });
                    }
                }
            }
        }
        Err(error) => diagnostics.push(Diagnostic {
            code: "memory_graph_unavailable".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: format!("Memory graph inputs were unavailable: {error}"),
            paths: Vec::new(),
            count: 0,
        }),
    }

    match list_feedback_events(repo_root, 50) {
        Ok(events) => {
            for event in events.into_iter().take(20) {
                let feedback_id = format!("feedback:{}", event.id);
                nodes.entry(feedback_id.clone()).or_insert(GraphNodeView {
                    id: feedback_id.clone(),
                    kind: GraphNodeKind::Feedback,
                    label: format!("{:?} {}", event.outcome, event.task_hash),
                    path: None,
                    role: None,
                    weight: 0.5,
                    source: "feedback".to_string(),
                });
                for path in event
                    .read_files
                    .iter()
                    .chain(event.edited_files.iter())
                    .chain(event.user_corrected_files.iter())
                {
                    if anchors.iter().any(|anchor| anchor == path) {
                        edges.push(GraphEdgeView {
                            source: file_node_id(path),
                            target: feedback_id.clone(),
                            kind: "observed_in".to_string(),
                            weight: 0.5,
                            reason: "source-free feedback event referenced this path".to_string(),
                        });
                    }
                }
            }
        }
        Err(error) => diagnostics.push(Diagnostic {
            code: "feedback_graph_unavailable".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: format!("Feedback graph inputs were unavailable: {error}"),
            paths: Vec::new(),
            count: 0,
        }),
    }

    let mut node_values = nodes.into_values().collect::<Vec<_>>();
    node_values.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.label.cmp(&right.label))
    });
    let capped = node_values.len() > max_nodes || edges.len() > max_edges;
    if node_values.len() > max_nodes {
        diagnostics.push(cap_diagnostic(
            "graph_nodes_capped",
            node_values.len(),
            max_nodes,
        ));
        node_values.truncate(max_nodes);
    }
    let node_ids = node_values
        .iter()
        .map(|node| node.id.clone())
        .collect::<BTreeSet<_>>();
    edges.retain(|edge| node_ids.contains(&edge.source) && node_ids.contains(&edge.target));
    if edges.len() > max_edges {
        diagnostics.push(cap_diagnostic("graph_edges_capped", edges.len(), max_edges));
        edges.truncate(max_edges);
    }
    let communities = graph_communities(&node_values, &edges);

    Ok(GraphNeighborhoodReport {
        repo_id: pack_repo_id(repo_root),
        task_hash: task.map(task_hash),
        anchors,
        max_nodes,
        max_edges,
        capped,
        nodes: node_values,
        edges,
        communities,
        diagnostics,
        source_text_logged: false,
        privacy_status: PrivacyStatus::local_only(),
    })
}

fn upsert_file_node(
    nodes: &mut BTreeMap<String, GraphNodeView>,
    path: &str,
    role: Option<&FileRole>,
    weight: f32,
    source: &str,
) {
    let id = file_node_id(path);
    nodes
        .entry(id.clone())
        .and_modify(|node| node.weight = node.weight.max(weight))
        .or_insert(GraphNodeView {
            id,
            kind: if matches!(role, Some(FileRole::Test)) {
                GraphNodeKind::Test
            } else {
                GraphNodeKind::File
            },
            label: path.to_string(),
            path: Some(path.to_string()),
            role: role.cloned(),
            weight,
            source: source.to_string(),
        });
}

fn file_node_id(path: &str) -> String {
    format!("file:{path}")
}

fn cap_diagnostic(code: &str, actual: usize, limit: usize) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: format!("Graph output was capped from {actual} item(s) to {limit}."),
        paths: Vec::new(),
        count: actual.saturating_sub(limit),
    }
}

fn graph_communities(nodes: &[GraphNodeView], edges: &[GraphEdgeView]) -> Vec<GraphCommunityView> {
    let mut counts = BTreeMap::<String, (usize, usize)>::new();
    for node in nodes {
        let label = node
            .path
            .as_deref()
            .and_then(|path| path.split('/').next())
            .unwrap_or(match node.kind {
                GraphNodeKind::Memory => "memory",
                GraphNodeKind::Feedback => "feedback",
                GraphNodeKind::Community => "community",
                GraphNodeKind::File | GraphNodeKind::Test => "repo",
            })
            .to_string();
        counts.entry(label).or_insert((0, 0)).0 += 1;
    }
    for edge in edges {
        let family = edge
            .source
            .strip_prefix("file:")
            .and_then(|path| path.split('/').next())
            .unwrap_or("metadata")
            .to_string();
        counts.entry(family).or_insert((0, 0)).1 += 1;
    }
    counts
        .into_iter()
        .map(|(label, (node_count, edge_count))| GraphCommunityView {
            id: format!("community:{label}"),
            label: label.clone(),
            node_count,
            edge_count,
            summary: format!(
                "`{label}` contains {node_count} graph node(s) and {edge_count} edge(s)."
            ),
        })
        .collect()
}
