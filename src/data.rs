use description;

use std::collections::HashMap;

type NodeIndex = usize;
type Capacity = isize;

type Cost = isize;
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
    pub fn calculate_cost(&self, path: &[NodeIndex]) -> isize {
        let mut cost: Cost = 0;
        let mut previous_n: NodeIndex = self.depot;
        let mut current_cargo: Capacity = self.capacity;

        for &n in path.iter() {
            let demand: Capacity = self.nodes_demand[n];

            // back to depot
            if current_cargo < demand {
                cost += self.distance_matrix[(n, self.depot)];
                previous_n = self.depot;
                current_cargo = self.capacity;
            }

            assert!(current_cargo >= demand);
            cost += self.distance_matrix[(previous_n, n)];
            current_cargo -= demand;
            previous_n = n;
        }

        cost
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
        .position(|ref n| *n == depot_nodes_ids[0])
        .unwrap()
}

fn get_node_demand(
    nodes: &HashMap<description::NodeID, description::Node>,
    nodes_order: &[description::NodeID],
) -> Vec<Capacity> {
    nodes_order
        .iter()
        .map(|ref node_id| nodes.get(*node_id).unwrap().demand)
        .collect()
}
