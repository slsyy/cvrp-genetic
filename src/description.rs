extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

pub type NodeID = String;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub x: f64,
    pub y: f64,
    pub demand: i32,
    pub is_depot: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Description {
    pub capacity: i32,
    pub edge_weight_type: String,
    pub nodes: HashMap<NodeID, Node>,
}
