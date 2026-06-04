#[derive(Debug, Clone)]
pub struct PheromoneMatrix {
    pub tau: Vec<Vec<f64>>,
    evaporation_rate: f64,
}

impl PheromoneMatrix {
    pub fn new(n: usize, initial: f64, evaporation_rate: f64) -> Self {
        let tau = vec![vec![initial; n]; n];
        Self { tau, evaporation_rate }
    }

    pub fn get(&self, i: usize, j: usize) -> f64 {
        self.tau[i][j]
    }

    pub fn deposit(&mut self, i: usize, j: usize, amount: f64) {
        self.tau[i][j] += amount;
    }

    pub fn evaporate(&mut self) {
        let er = 1.0 - self.evaporation_rate;
        for row in &mut self.tau {
            for v in row.iter_mut() {
                *v *= er;
            }
        }
    }

    pub fn clamp(&mut self, min: f64, max: f64) {
        for row in &mut self.tau {
            for v in row.iter_mut() {
                *v = v.clamp(min, max);
            }
        }
    }
}
