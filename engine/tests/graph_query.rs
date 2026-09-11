use lemma::{DateTimeValue, Engine, GraphDirection, GraphEdgeKind, GraphQueryRequest, SourceType};
use std::path::PathBuf;
use std::sync::Arc;

fn load_sample() -> Engine {
    let mut engine = Engine::new();
    engine
        .load([(
            SourceType::Path(Arc::new(PathBuf::from("graph_query.lemma"))),
            r#"
spec graph_query
data base: number
data copy: base
rule upstream: base
rule downstream: upstream + copy
"#
            .to_string(),
        )])
        .expect("load");
    engine
}

#[test]
fn graph_query_returns_semantic_edges() {
    let engine = load_sample();
    let now = DateTimeValue::now();
    let response = engine
        .graph_query(
            None,
            "graph_query",
            Some(&now),
            GraphQueryRequest::default(),
        )
        .expect("graph query");

    assert!(response.nodes.iter().any(|n| n.id == "rule:upstream"));
    assert!(response
        .edges
        .iter()
        .any(|edge| edge.kind == GraphEdgeKind::RuleDependsOnRule));
    assert!(response
        .edges
        .iter()
        .any(|edge| edge.kind == GraphEdgeKind::RuleUsesData));
}

#[test]
fn graph_query_respects_root_depth_and_node_bounds() {
    let engine = load_sample();
    let now = DateTimeValue::now();
    let response = engine
        .graph_query(
            None,
            "graph_query",
            Some(&now),
            GraphQueryRequest {
                roots: Some(vec!["rule:downstream".to_string()]),
                direction: Some(GraphDirection::Outbound),
                max_depth: Some(1),
                max_nodes: Some(2),
                ..GraphQueryRequest::default()
            },
        )
        .expect("graph query");

    assert!(response.nodes.len() <= 2);
    assert!(response.truncated || response.nodes.len() < 2 || response.edges.len() <= 1);
}

#[test]
fn graph_query_rejects_unknown_root() {
    let engine = load_sample();
    let now = DateTimeValue::now();
    let error = engine
        .graph_query(
            None,
            "graph_query",
            Some(&now),
            GraphQueryRequest {
                roots: Some(vec!["rule:missing".to_string()]),
                ..GraphQueryRequest::default()
            },
        )
        .expect_err("unknown root must fail");
    assert_eq!(error.kind(), lemma::ErrorKind::Request);
}
