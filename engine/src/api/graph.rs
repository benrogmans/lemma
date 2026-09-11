//! Graph query JSON shapes.

use crate::graph_query::{
    GraphEdge as DomainGraphEdge, GraphEdgeKind as DomainGraphEdgeKind,
    GraphNode as DomainGraphNode, GraphNodeKind as DomainGraphNodeKind,
    GraphQueryResponse as DomainGraphQueryResponse,
};
use crate::parsing::ast::DateTimeValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphNodeKind {
    Rule,
    Data,
}

impl From<DomainGraphNodeKind> for GraphNodeKind {
    fn from(value: DomainGraphNodeKind) -> Self {
        match value {
            DomainGraphNodeKind::Rule => Self::Rule,
            DomainGraphNodeKind::Data => Self::Data,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphEdgeKind {
    RuleDependsOnRule,
    RuleUsesData,
}

impl From<DomainGraphEdgeKind> for GraphEdgeKind {
    fn from(value: DomainGraphEdgeKind) -> Self {
        match value {
            DomainGraphEdgeKind::RuleDependsOnRule => Self::RuleDependsOnRule,
            DomainGraphEdgeKind::RuleUsesData => Self::RuleUsesData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub kind: GraphNodeKind,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub metadata: Option<indexmap::IndexMap<String, String>>,
}

impl From<&DomainGraphNode> for GraphNode {
    fn from(node: &DomainGraphNode) -> Self {
        Self {
            id: node.id.clone(),
            kind: GraphNodeKind::from(node.kind),
            name: node.name.clone(),
            metadata: node.metadata.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub kind: GraphEdgeKind,
    pub from: String,
    pub to: String,
}

impl From<&DomainGraphEdge> for GraphEdge {
    fn from(edge: &DomainGraphEdge) -> Self {
        Self {
            kind: GraphEdgeKind::from(edge.kind),
            from: edge.from.clone(),
            to: edge.to.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphQueryResponse {
    pub spec: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub effective_from: Option<DateTimeValue>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub effective_to: Option<DateTimeValue>,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub truncated: bool,
}

impl From<&DomainGraphQueryResponse> for GraphQueryResponse {
    fn from(response: &DomainGraphQueryResponse) -> Self {
        Self {
            spec: response.spec.clone(),
            effective_from: response.effective_from.clone(),
            effective_to: response.effective_to.clone(),
            nodes: response.nodes.iter().map(GraphNode::from).collect(),
            edges: response.edges.iter().map(GraphEdge::from).collect(),
            truncated: response.truncated,
        }
    }
}
