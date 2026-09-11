use crate::parsing::ast::DateTimeValue;
use crate::planning::execution_plan::{reachable_data_paths, ExecutionPlan};
use crate::planning::semantics::{DataDefinition, DataPath, RulePath};
use crate::Error;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const DEFAULT_MAX_DEPTH: usize = 4;
const DEFAULT_MAX_NODES: usize = 200;
const MAX_MAX_DEPTH: usize = 64;
const MAX_MAX_NODES: usize = 2000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphDirection {
    Outbound,
    Inbound,
    Both,
}

impl GraphDirection {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "outbound" => Ok(Self::Outbound),
            "inbound" => Ok(Self::Inbound),
            "both" => Ok(Self::Both),
            _ => Err(format!(
                "invalid graph direction '{raw}'; expected one of: outbound, inbound, both"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphEdgeKind {
    RuleDependsOnRule,
    RuleUsesData,
}

impl GraphEdgeKind {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "rule_depends_on_rule" => Ok(Self::RuleDependsOnRule),
            "rule_uses_data" => Ok(Self::RuleUsesData),
            _ => Err(format!(
                "invalid graph edge kind '{raw}'; expected one of: \
                 rule_depends_on_rule, rule_uses_data"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphNodeKind {
    Rule,
    Data,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GraphQueryRequest {
    pub roots: Option<Vec<String>>,
    pub edge_kinds: Option<Vec<GraphEdgeKind>>,
    pub direction: Option<GraphDirection>,
    pub max_depth: Option<usize>,
    pub max_nodes: Option<usize>,
    pub include_metadata: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub kind: GraphNodeKind,
    pub name: String,
    pub metadata: Option<IndexMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub kind: GraphEdgeKind,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphQueryResponse {
    pub spec: String,
    pub effective_from: Option<DateTimeValue>,
    pub effective_to: Option<DateTimeValue>,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Copy)]
struct EffectiveQuery {
    direction: GraphDirection,
    max_depth: usize,
    max_nodes: usize,
    include_metadata: bool,
}

impl GraphQueryRequest {
    fn normalize(&self) -> Result<(Vec<GraphEdgeKind>, EffectiveQuery), Error> {
        let edge_kinds = self.edge_kinds.clone().unwrap_or_else(|| {
            vec![
                GraphEdgeKind::RuleDependsOnRule,
                GraphEdgeKind::RuleUsesData,
            ]
        });
        if edge_kinds.is_empty() {
            return Err(Error::request(
                "graph_query edge_kinds cannot be empty".to_string(),
                None::<String>,
            ));
        }
        if let Some(roots) = &self.roots {
            if roots.is_empty() {
                return Err(Error::request(
                    "graph_query roots cannot be an empty array".to_string(),
                    None::<String>,
                ));
            }
        }
        let max_depth = self.max_depth.unwrap_or(DEFAULT_MAX_DEPTH);
        if max_depth > MAX_MAX_DEPTH {
            return Err(Error::request(
                format!("graph_query max_depth must be <= {MAX_MAX_DEPTH}, got {max_depth}"),
                None::<String>,
            ));
        }
        let max_nodes = self.max_nodes.unwrap_or(DEFAULT_MAX_NODES);
        if max_nodes == 0 || max_nodes > MAX_MAX_NODES {
            return Err(Error::request(
                format!("graph_query max_nodes must be in 1..={MAX_MAX_NODES}, got {max_nodes}"),
                None::<String>,
            ));
        }

        Ok((
            edge_kinds,
            EffectiveQuery {
                direction: self.direction.unwrap_or(GraphDirection::Both),
                max_depth,
                max_nodes,
                include_metadata: self.include_metadata.unwrap_or(false),
            },
        ))
    }
}

#[derive(Debug, Clone)]
struct FullGraph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    node_index: IndexMap<String, usize>,
    out_edges: IndexMap<String, Vec<usize>>,
    in_edges: IndexMap<String, Vec<usize>>,
}

pub(crate) fn query_execution_plan(
    plan: &ExecutionPlan,
    request: &GraphQueryRequest,
) -> Result<GraphQueryResponse, Error> {
    let (edge_kinds, effective_query) = request.normalize()?;
    let graph = build_full_graph(plan, effective_query.include_metadata, &edge_kinds);

    let (node_ids, traversed_edges, truncated) = match &request.roots {
        None => select_default(&graph, effective_query.max_nodes),
        Some(roots) => select_from_roots(&graph, roots, effective_query)?,
    };

    let nodes: Vec<GraphNode> = graph
        .nodes
        .iter()
        .filter(|node| node_ids.contains(&node.id))
        .cloned()
        .collect();
    let edges: Vec<GraphEdge> = graph
        .edges
        .iter()
        .enumerate()
        .filter(|(index, edge)| {
            traversed_edges.contains(index)
                && node_ids.contains(&edge.from)
                && node_ids.contains(&edge.to)
        })
        .map(|(_, edge)| edge.clone())
        .collect();

    Ok(GraphQueryResponse {
        spec: plan.spec_name.clone(),
        effective_from: plan.effective_from.clone(),
        effective_to: plan.effective_to.clone(),
        nodes,
        edges,
        truncated,
    })
}

fn select_default(
    graph: &FullGraph,
    max_nodes: usize,
) -> (IndexSet<String>, IndexSet<usize>, bool) {
    let mut node_ids = IndexSet::new();
    let mut truncated = false;
    for node in &graph.nodes {
        if node_ids.len() == max_nodes {
            truncated = true;
            break;
        }
        node_ids.insert(node.id.clone());
    }
    let edge_ids = graph
        .edges
        .iter()
        .enumerate()
        .filter(|(_, edge)| node_ids.contains(&edge.from) && node_ids.contains(&edge.to))
        .map(|(index, _)| index)
        .collect();
    (node_ids, edge_ids, truncated)
}

fn select_from_roots(
    graph: &FullGraph,
    roots: &[String],
    query: EffectiveQuery,
) -> Result<(IndexSet<String>, IndexSet<usize>, bool), Error> {
    for root in roots {
        if !graph.node_index.contains_key(root) {
            return Err(Error::request(
                format!("graph_query root '{root}' is not a known node id"),
                None::<String>,
            ));
        }
    }

    let mut seen = IndexSet::new();
    let mut traversed_edges = IndexSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    let mut truncated = false;
    for root in roots {
        if seen.len() == query.max_nodes {
            truncated = true;
            continue;
        }
        if seen.insert(root.clone()) {
            queue.push_back((root.clone(), 0));
        }
    }

    while let Some((node_id, depth)) = queue.pop_front() {
        if depth >= query.max_depth {
            continue;
        }

        let neighbor_lists = match query.direction {
            GraphDirection::Outbound => vec![graph.out_edges.get(&node_id)],
            GraphDirection::Inbound => vec![graph.in_edges.get(&node_id)],
            GraphDirection::Both => {
                vec![graph.out_edges.get(&node_id), graph.in_edges.get(&node_id)]
            }
        };

        for list in neighbor_lists.into_iter().flatten() {
            for edge_index in list {
                traversed_edges.insert(*edge_index);
                let edge = &graph.edges[*edge_index];
                let next_id = if edge.from == node_id {
                    edge.to.clone()
                } else {
                    edge.from.clone()
                };
                if seen.contains(&next_id) {
                    continue;
                }
                if seen.len() == query.max_nodes {
                    truncated = true;
                    continue;
                }
                seen.insert(next_id.clone());
                queue.push_back((next_id, depth + 1));
            }
        }
    }

    Ok((seen, traversed_edges, truncated))
}

fn build_full_graph(
    plan: &ExecutionPlan,
    include_metadata: bool,
    edge_kinds: &[GraphEdgeKind],
) -> FullGraph {
    let enabled: IndexSet<GraphEdgeKind> = edge_kinds.iter().copied().collect();
    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut node_index: IndexMap<String, usize> = IndexMap::new();
    let mut edges: Vec<GraphEdge> = Vec::new();

    let mut push_node = |node: GraphNode| {
        if node_index.contains_key(&node.id) {
            return;
        }
        node_index.insert(node.id.clone(), nodes.len());
        nodes.push(node);
    };

    for (path, definition) in &plan.data {
        let mut metadata = include_metadata.then(IndexMap::new);
        if let Some(meta) = metadata.as_mut() {
            meta.insert("input_key".to_string(), path.input_key());
            meta.insert(
                "definition_kind".to_string(),
                match definition {
                    DataDefinition::Value { .. } => "value",
                    DataDefinition::TypeDeclaration { .. } => "type_declaration",
                    DataDefinition::Import { .. } => "import",
                    DataDefinition::Reference { .. } => "reference",
                }
                .to_string(),
            );
        }
        push_node(GraphNode {
            id: data_id(path),
            kind: GraphNodeKind::Data,
            name: path.data.clone(),
            metadata,
        });
    }
    for path in plan.rules.keys() {
        let mut metadata = include_metadata.then(IndexMap::new);
        if let Some(meta) = metadata.as_mut() {
            meta.insert(
                "scope".to_string(),
                if path.segments.is_empty() {
                    "local".to_string()
                } else {
                    "external".to_string()
                },
            );
        }
        push_node(GraphNode {
            id: rule_id(path),
            kind: GraphNodeKind::Rule,
            name: path.rule.clone(),
            metadata,
        });
    }

    if enabled.contains(&GraphEdgeKind::RuleDependsOnRule) {
        for rule in plan.rules.values() {
            let from = rule_id(&rule.path);
            for dependency in &rule.depends_on_rules {
                if !plan.rules.contains_key(dependency) {
                    continue;
                }
                edges.push(GraphEdge {
                    kind: GraphEdgeKind::RuleDependsOnRule,
                    from: from.clone(),
                    to: rule_id(dependency),
                });
            }
        }
    }

    if enabled.contains(&GraphEdgeKind::RuleUsesData) {
        let value_slots = vec![None; plan.normal_forms.len()];
        for rule in plan.rules.values() {
            let mut seen_targets: IndexSet<String> = IndexSet::new();
            let data_paths = reachable_data_paths(plan, rule.normal_form, &value_slots);
            for data_path in &data_paths {
                let Some(promptable) = plan.promptable_data_path(data_path) else {
                    continue;
                };
                let to = data_id(promptable);
                if !seen_targets.insert(to.clone()) {
                    continue;
                }
                edges.push(GraphEdge {
                    kind: GraphEdgeKind::RuleUsesData,
                    from: rule_id(&rule.path),
                    to,
                });
            }
        }
    }

    let mut out_edges: IndexMap<String, Vec<usize>> = IndexMap::new();
    let mut in_edges: IndexMap<String, Vec<usize>> = IndexMap::new();
    for (index, edge) in edges.iter().enumerate() {
        out_edges.entry(edge.from.clone()).or_default().push(index);
        in_edges.entry(edge.to.clone()).or_default().push(index);
    }

    FullGraph {
        nodes,
        edges,
        node_index,
        out_edges,
        in_edges,
    }
}

fn data_id(path: &DataPath) -> String {
    let mut value = String::from("data:");
    for segment in &path.segments {
        value.push_str(&segment.data);
        value.push('@');
        value.push_str(&segment.spec);
        value.push('.');
    }
    value.push_str(&path.data);
    value
}

fn rule_id(path: &RulePath) -> String {
    let mut value = String::from("rule:");
    for segment in &path.segments {
        value.push_str(&segment.data);
        value.push('@');
        value.push_str(&segment.spec);
        value.push('.');
    }
    value.push_str(&path.rule);
    value
}
