use rand::Rng;

#[derive(Debug, Clone)]
pub struct Individual {
    pub genes: Vec<f64>,
    pub fitness: f64,
}

impl Individual {
    pub fn new(genes: Vec<f64>) -> Self {
        Self { genes, fitness: 0.0 }
    }

    pub fn random(len: usize, max_qty: f64) -> Self {
        let mut rng = rand::thread_rng();
        let genes: Vec<f64> = (0..len).map(|_| rng.gen_range(0.0..=max_qty)).collect();
        Self::new(genes)
    }

    pub fn crossover(&self, other: &Individual, rate: f64) -> (Individual, Individual) {
        let mut rng = rand::thread_rng();
        let mut child1 = self.genes.clone();
        let mut child2 = other.genes.clone();

        for i in 0..self.genes.len() {
            if rng.gen::<f64>() < rate {
                child1[i] = other.genes[i];
                child2[i] = self.genes[i];
            }
        }

        (Individual::new(child1), Individual::new(child2))
    }

    pub fn mutate(&mut self, rate: f64, max_qty: f64) {
        let mut rng = rand::thread_rng();
        for g in &mut self.genes {
            if rng.gen::<f64>() < rate {
                let delta = rng.gen_range(-max_qty * 0.2..=max_qty * 0.2);
                *g = (*g + delta).clamp(0.0, max_qty);
                *g = (*g * 10.0).round() / 10.0;
            }
        }
    }

    pub fn clamp(&mut self, max_qty: f64) {
        for g in &mut self.genes {
            *g = g.clamp(0.0, max_qty);
            *g = (*g * 10.0).round() / 10.0;
        }
    }
}
