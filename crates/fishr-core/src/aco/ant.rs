use rand::Rng;
use crate::aco::graph::PrepGraph;
use crate::aco::pheromone::PheromoneMatrix;

#[derive(Debug, Clone)]
pub struct AntPath {
    pub order: Vec<usize>,
    pub total_cost: f64,
}

pub fn build_ant_path(
    graph: &PrepGraph,
    pheromone: &PheromoneMatrix,
    alpha: f64,
    beta: f64,
) -> AntPath {
    let n = graph.node_count();
    if n <= 1 {
        let order: Vec<usize> = (0..n).collect();
        let total_cost = 0.0;
        return AntPath { order, total_cost };
    }

    let mut rng = rand::thread_rng();
    let start = rng.gen_range(0..n);
    let mut visited = vec![false; n];
    visited[start] = true;
    let mut order = vec![start];

    while order.len() < n {
        let current = *order.last().unwrap();
        let mut probabilities = Vec::new();
        let mut sum_prob = 0.0;

        for next in 0..n {
            if visited[next] {
                continue;
            }
            let tau = pheromone.get(current, next).max(1e-10);
            let eta = 1.0 / graph.costs[current][next].max(1e-10);
            let prob = tau.powf(alpha) * eta.powf(beta);
            probabilities.push((next, prob));
            sum_prob += prob;
        }

        if probabilities.is_empty() || sum_prob <= 0.0 {
            break;
        }

        let r: f64 = rng.gen::<f64>() * sum_prob;
        let mut cumulative = 0.0;
        let mut chosen = probabilities[0].0;
        for (next, prob) in &probabilities {
            cumulative += prob;
            if r <= cumulative {
                chosen = *next;
                break;
            }
        }

        visited[chosen] = true;
        order.push(chosen);
    }

    let total_cost = path_cost(&order, graph);
    AntPath { order, total_cost }
}

pub fn path_cost(order: &[usize], graph: &PrepGraph) -> f64 {
    let mut cost = 0.0;
    for w in order.windows(2) {
        cost += graph.costs[w[0]][w[1]];
    }
    cost
}
