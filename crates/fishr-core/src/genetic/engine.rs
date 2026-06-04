use crate::genetic::population::Population;

#[derive(Debug, Clone)]
pub struct FishTypeStats {
    pub fish_type_id: String,
    pub fish_type_name: String,
    pub avg_daily_sales: f64,
    pub avg_weight_kg: f64,
    pub price_per_kg: f64,
    pub cost_per_kg: f64,
    pub current_stock: i32,
    pub lead_time_days: f64,
}

#[derive(Debug, Clone)]
pub struct PlannerConfig {
    pub population_size: usize,
    pub generations: usize,
    pub crossover_rate: f64,
    pub mutation_rate: f64,
    pub tournament_size: usize,
    pub elitism: usize,
    pub max_qty_per_type: f64,
    pub waste_penalty_factor: f64,
    pub stockout_penalty_factor: f64,
    pub holding_cost_pct: f64,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            population_size: 50,
            generations: 100,
            crossover_rate: 0.8,
            mutation_rate: 0.15,
            tournament_size: 3,
            elitism: 2,
            max_qty_per_type: 200.0,
            waste_penalty_factor: 0.3,
            stockout_penalty_factor: 0.5,
            holding_cost_pct: 0.02,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Suggestion {
    pub fish_type_id: String,
    pub fish_type_name: String,
    pub suggested_qty: f64,
    pub expected_revenue: f64,
    pub expected_cost: f64,
    pub expected_margin: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlannerResult {
    pub suggestions: Vec<Suggestion>,
    pub total_revenue: f64,
    pub total_cost: f64,
    pub total_margin: f64,
    pub generations_run: usize,
    pub final_avg_fitness: f64,
}

#[derive(Clone)]
pub struct InventoryPlanner {
    pub stats: Vec<FishTypeStats>,
    pub config: PlannerConfig,
}

impl InventoryPlanner {
    pub fn new(stats: Vec<FishTypeStats>, config: PlannerConfig) -> Self {
        Self { stats, config }
    }

    pub fn run(&self) -> PlannerResult {
        let n = self.stats.len();
        if n == 0 {
            return PlannerResult {
                suggestions: vec![],
                total_revenue: 0.0,
                total_cost: 0.0,
                total_margin: 0.0,
                generations_run: 0,
                final_avg_fitness: 0.0,
            };
        }

        let mut pop = Population::new(self.config.population_size, n, self.config.max_qty_per_type);
        self.evaluate_population(&mut pop);

        for _gen in 0..self.config.generations {
            pop.evolve(
                self.config.crossover_rate,
                self.config.mutation_rate,
                self.config.tournament_size,
                self.config.elitism,
            );
            self.evaluate_population(&mut pop);

            let _avg = pop.average_fitness();
            let _best = pop.best().fitness;
        }

        let best = pop.best().clone();
        let mut suggestions = Vec::new();
        let mut total_revenue = 0.0;
        let mut total_cost = 0.0;

        for (i, stat) in self.stats.iter().enumerate() {
            let qty = best.genes[i];
            let expected_revenue = qty * stat.price_per_kg * stat.avg_weight_kg;
            let expected_cost = qty * stat.cost_per_kg * stat.avg_weight_kg;
            let confidence = if stat.avg_daily_sales > 0.0 {
                (qty / (stat.avg_daily_sales * stat.lead_time_days.max(1.0))).min(1.0)
            } else {
                1.0
            };

            suggestions.push(Suggestion {
                fish_type_id: stat.fish_type_id.clone(),
                fish_type_name: stat.fish_type_name.clone(),
                suggested_qty: qty,
                expected_revenue,
                expected_cost,
                expected_margin: expected_revenue - expected_cost,
                confidence,
            });
            total_revenue += expected_revenue;
            total_cost += expected_cost;
        }

        PlannerResult {
            suggestions,
            total_revenue,
            total_cost,
            total_margin: total_revenue - total_cost,
            generations_run: self.config.generations,
            final_avg_fitness: pop.average_fitness(),
        }
    }

    fn evaluate_population(&self, pop: &mut Population) {
        for ind in &mut pop.individuals {
            ind.fitness = self.compute_fitness(&ind.genes);
        }
    }

    fn compute_fitness(&self, genes: &[f64]) -> f64 {
        let mut total = 0.0;

        for (i, stat) in self.stats.iter().enumerate() {
            let qty = genes[i];

            // Expected demand over lead time
            let expected_demand = stat.avg_daily_sales * stat.lead_time_days.max(1.0);

            // Revenue for sold units
            let sold = qty.min(expected_demand);
            let revenue = sold * stat.price_per_kg * stat.avg_weight_kg;

            // Cost of all purchased units
            let cost = qty * stat.cost_per_kg * stat.avg_weight_kg;

            // Waste penalty for excess stock
            let excess = (qty - expected_demand).max(0.0);
            let waste_penalty = excess * stat.cost_per_kg * stat.avg_weight_kg * self.config.waste_penalty_factor;

            // Stockout penalty for unmet demand
            let shortfall = (expected_demand - qty).max(0.0);
            let stockout_penalty = shortfall * stat.price_per_kg * stat.avg_weight_kg * self.config.stockout_penalty_factor;

            // Holding cost
            let holding_cost = qty * stat.cost_per_kg * stat.avg_weight_kg * self.config.holding_cost_pct;

            let margin = revenue - cost - waste_penalty - stockout_penalty - holding_cost;
            total += margin;
        }

        total
    }
}
