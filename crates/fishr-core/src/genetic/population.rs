use rand::Rng;
use crate::genetic::individual::Individual;

pub struct Population {
    pub individuals: Vec<Individual>,
    pub gene_count: usize,
    pub max_qty: f64,
}

impl Population {
    pub fn new(size: usize, gene_count: usize, max_qty: f64) -> Self {
        let individuals: Vec<Individual> = (0..size)
            .map(|_| Individual::random(gene_count, max_qty))
            .collect();
        Self { individuals, gene_count, max_qty }
    }

    pub fn from_individuals(individuals: Vec<Individual>, max_qty: f64) -> Self {
        let gene_count = individuals.first().map(|i| i.genes.len()).unwrap_or(0);
        Self { individuals, gene_count, max_qty }
    }

    pub fn tournament_selection(&self, tournament_size: usize) -> &Individual {
        let mut rng = rand::thread_rng();
        let mut best_idx = rng.gen_range(0..self.individuals.len());
        for _ in 1..tournament_size {
            let idx = rng.gen_range(0..self.individuals.len());
            if self.individuals[idx].fitness > self.individuals[best_idx].fitness {
                best_idx = idx;
            }
        }
        &self.individuals[best_idx]
    }

    pub fn evolve(&mut self, crossover_rate: f64, mutation_rate: f64, tournament_size: usize, elitism: usize) {
        let mut new_pop = Vec::with_capacity(self.individuals.len());

        // Elitism: keep top N
        self.individuals.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));
        for i in 0..elitism.min(self.individuals.len()) {
            new_pop.push(self.individuals[i].clone());
        }

        // Fill rest with offspring
        while new_pop.len() < self.individuals.len() {
            let parent1 = self.tournament_selection(tournament_size);
            let parent2 = self.tournament_selection(tournament_size);
            let (mut child1, mut child2) = parent1.crossover(parent2, crossover_rate);
            child1.mutate(mutation_rate, self.max_qty);
            child2.mutate(mutation_rate, self.max_qty);
            child1.clamp(self.max_qty);
            child2.clamp(self.max_qty);
            new_pop.push(child1);
            if new_pop.len() < self.individuals.len() {
                new_pop.push(child2);
            }
        }

        self.individuals = new_pop;
    }

    pub fn best(&self) -> &Individual {
        self.individuals.iter()
            .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal))
            .expect("empty population")
    }

    pub fn average_fitness(&self) -> f64 {
        let sum: f64 = self.individuals.iter().map(|i| i.fitness).sum();
        sum / self.individuals.len() as f64
    }
}
