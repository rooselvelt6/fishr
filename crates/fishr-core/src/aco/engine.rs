use crate::aco::ant::{build_ant_path, AntPath};
use crate::aco::graph::PrepGraph;
use crate::aco::pheromone::PheromoneMatrix;

#[derive(Debug, Clone)]
pub struct AcoConfig {
    pub ant_count: usize,
    pub iterations: usize,
    pub alpha: f64,
    pub beta: f64,
    pub evaporation_rate: f64,
    pub initial_pheromone: f64,
    pub q: f64,
}

impl Default for AcoConfig {
    fn default() -> Self {
        Self {
            ant_count: 10,
            iterations: 30,
            alpha: 1.0,
            beta: 2.0,
            evaporation_rate: 0.3,
            initial_pheromone: 0.1,
            q: 5.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AcoResult {
    pub best_path: AntPath,
    pub best_cost: f64,
    pub iterations_run: usize,
    pub had_multiple: bool,
}

pub struct AcoSolver {
    pub config: AcoConfig,
}

impl AcoSolver {
    pub fn new(config: AcoConfig) -> Self {
        Self { config }
    }

    pub fn solve(&self, graph: &PrepGraph) -> AcoResult {
        let n = graph.node_count();
        if n <= 1 {
            let order: Vec<usize> = (0..n).collect();
            return AcoResult {
                best_path: AntPath {
                    order: order.clone(),
                    total_cost: 0.0,
                },
                best_cost: 0.0,
                iterations_run: 0,
                had_multiple: false,
            };
        }

        let mut pheromone = PheromoneMatrix::new(
            n,
            self.config.initial_pheromone,
            self.config.evaporation_rate,
        );

        let mut best_path = build_ant_path(graph, &pheromone, self.config.alpha, self.config.beta);

        for _iter in 0..self.config.iterations {
            pheromone.evaporate();

            for _ant in 0..self.config.ant_count {
                let path = build_ant_path(graph, &pheromone, self.config.alpha, self.config.beta);
                let deposit = self.config.q / path.total_cost.max(1e-10);
                for w in path.order.windows(2) {
                    pheromone.deposit(w[0], w[1], deposit);
                }

                if path.total_cost < best_path.total_cost {
                    best_path = path;
                }
            }

            pheromone.clamp(0.001, 10.0);
        }

        let best_cost = best_path.total_cost;
        AcoResult {
            best_path,
            best_cost,
            iterations_run: self.config.iterations,
            had_multiple: true,
        }
    }
}
