use serde::Deserialize;

#[derive(Deserialize)]
struct JsonData {
    nodes: Vec<JsonNode>,
    edges: Vec<(usize, usize)>,
}

#[derive(Deserialize)]
struct JsonNode {
    label: String,
}

pub struct NodeData {
    pub label: String,
}

pub struct GraphData {
    pub nodes: Vec<NodeData>,
    pub edges: Vec<(usize, usize)>,
}

pub fn load() -> GraphData {
    let json_str = include_str!("../../../assets/nodes.json");
    let data: JsonData = serde_json::from_str(json_str).expect("invalid nodes.json");

    let nodes = data
        .nodes
        .into_iter()
        .map(|n| NodeData { label: n.label })
        .collect();

    GraphData {
        nodes,
        edges: data.edges,
    }
}
