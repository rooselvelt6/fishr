#[derive(Debug, Clone)]
pub struct PrepNode {
    pub index: usize,
    pub fish_item_id: String,
    pub fish_type_name: String,
    pub preparation_id: String,
    pub preparation_name: String,
    pub category: String,
}

pub struct PrepGraph {
    pub nodes: Vec<PrepNode>,
    pub costs: Vec<Vec<f64>>,
}

impl PrepGraph {
    pub fn new(nodes: Vec<PrepNode>) -> Self {
        let n = nodes.len();
        let mut costs = vec![vec![1.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    costs[i][j] = 0.0;
                } else {
                    costs[i][j] = Self::transition_cost(&nodes[i], &nodes[j]);
                }
            }
        }
        Self { nodes, costs }
    }

    fn transition_cost(from: &PrepNode, to: &PrepNode) -> f64 {
        let mut cost: f64 = 1.0;
        if from.preparation_id == to.preparation_id {
            cost -= 0.3;
        }
        if from.category == to.category {
            cost -= 0.2;
        }
        if from.category == "White" && to.category == "White" {
            cost -= 0.1;
        }
        if (from.category == "Shellfish" || from.category == "Crustacean")
            && (to.category == "White" || to.category == "Blue")
        {
            cost += 0.3;
        }
        cost.clamp(0.1, 2.0)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}
