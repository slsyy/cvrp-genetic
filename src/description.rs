extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

type NodeID = String;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Node {
    x: f64,
    y: f64,
    demand: isize,
    is_depot: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Description {
    capacity: isize,
    edge_weight_type: String,
    nodes: HashMap<NodeID, Node>,
}