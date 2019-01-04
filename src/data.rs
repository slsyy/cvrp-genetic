use description;

use std::collections::HashMap;

pub type NodeIndex = usize;
type Capacity = i32;

pub type Cost = i32;
type DistanceMatrix = na::base::DMatrix<Cost>;

pub struct Data {
    pub distance_matrix: DistanceMatrix,
    pub depot: NodeIndex,
    pub nodes_demand: Vec<Capacity>,
    pub capacity: Capacity,
    pub index_to_id: Vec<description::NodeID>,
}

impl Data {
    pub fn new(desc: &description::Description) -> Data {
        let index_to_id = order_of_nodes(&desc);

        assert_eq!("EUC_2D", desc.edge_weight_type);

        Data {
            distance_matrix: calculate_euclidean_distance_matrix(&desc.nodes, &index_to_id),
            depot: get_depot(&desc.nodes, &index_to_id),
            nodes_demand: get_node_demand(&desc.nodes, &index_to_id),
            capacity: desc.capacity,
            index_to_id: index_to_id,
        }
    }

    pub fn calculate_cost(&self, path: &[NodeIndex]) -> Cost {
        let mut cost: Cost = 0;

        let mut previous_n: NodeIndex = self.depot;

        self.traverse_path(path, &mut |n| {
            cost += self.distance_matrix[(n, previous_n)];
            previous_n = n;
        });

        cost
    }

    pub fn indices_path_to_index(&self, path: &[NodeIndex]) -> Vec<Vec<description::NodeID>> {
        let mut res: Vec<Vec<description::NodeID>> = Vec::new();

        self.traverse_path(path, &mut |n| {
            if n == self.depot {
                if let Some(current_path) = res.last_mut() {
                    current_path.push(self.index_to_id.get(n).unwrap().clone());
                }
                res.push(Vec::new());
            }

            let current_path = res.last_mut().unwrap();
            current_path.push(self.index_to_id.get(n).unwrap().clone());
        });

        res.pop();
        res
    }

    fn traverse_path(&self, path: &[NodeIndex], f: &mut impl FnMut(NodeIndex) -> ()) {
        f(self.depot);

        let mut current_cargo: Capacity = self.capacity;

        for &n in path.iter() {
            let demand: Capacity = self.nodes_demand[n];

            if current_cargo < demand {
                current_cargo = self.capacity;
                f(self.depot);
            }

            current_cargo -= demand;
            f(n);
        }

        f(self.depot);
    }
}

fn order_of_nodes(desc: &description::Description) -> Vec<description::NodeID> {
    desc.nodes.keys().cloned().collect()
}

fn euclidean_distance(l: &description::Node, r: &description::Node) -> f64 {
    ((l.x - r.x).powi(2) + (l.y - r.y).powi(2)).sqrt()
}

fn calculate_euclidean_distance_matrix(
    nodes: &HashMap<description::NodeID, description::Node>,
    nodes_order: &[description::NodeID],
) -> DistanceMatrix {
    let mut matrix = DistanceMatrix::zeros(nodes.len(), nodes.len());

    for (i, i_node_id) in nodes_order.iter().enumerate() {
        for (j, j_node_id) in nodes_order.iter().enumerate() {
            matrix[(i, j)] =
                euclidean_distance(nodes.get(i_node_id).unwrap(), nodes.get(j_node_id).unwrap())
                    .round() as Cost;
        }
    }

    matrix
}

fn get_depot(
    nodes: &HashMap<description::NodeID, description::Node>,
    nodes_order: &[description::NodeID],
) -> NodeIndex {
    let depot_nodes_ids: Vec<_> = nodes
        .iter()
        .filter(|(_, ref v)| v.is_depot)
        .map(|(ref k, _)| k.clone())
        .collect();

    assert_eq!(depot_nodes_ids.len(), 1);

    nodes_order
        .iter()
        .position(|n| n == depot_nodes_ids[0])
        .unwrap()
}

fn get_node_demand(
    nodes: &HashMap<description::NodeID, description::Node>,
    nodes_order: &[description::NodeID],
) -> Vec<Capacity> {
    nodes_order
        .iter()
        .map(|node_id| nodes.get(node_id).unwrap().demand)
        .collect()
}
