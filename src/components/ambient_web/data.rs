use serde::Deserialize;

#[derive(Deserialize)]
struct JsonData {
    origin: (f64, f64),
    center: Option<String>,
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
    pub origin: (f64, f64),
    pub center_index: Option<usize>,
    pub nodes: Vec<NodeData>,
    pub edges: Vec<(usize, usize)>,
}

pub fn load() -> GraphData {
    let json_str = include_str!("../../../assets/nodes.json");
    let data: JsonData = serde_json::from_str(json_str).expect("invalid nodes.json");

    let mut nodes: Vec<NodeData> = data
        .nodes
        .into_iter()
        .map(|n| NodeData { label: n.label })
        .collect();

    let mut edges = data.edges;
    let outer_count = nodes.len();

    let center_index = data.center.map(|label| {
        let idx = outer_count;
        for i in 0..outer_count {
            edges.push((idx, i));
        }
        nodes.push(NodeData { label });
        idx
    });

    GraphData {
        origin: (data.origin.0.clamp(0.0, 1.0), data.origin.1.clamp(0.0, 1.0)),
        center_index,
        nodes,
        edges,
    }
}
