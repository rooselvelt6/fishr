use crate::fuzzy::sets::*;

#[derive(Debug, Clone)]
pub struct FuzzyRule {
    pub antecedents: Vec<Clause>,
    pub consequent: SuggestionType,
    pub weight: f64,
}

impl FuzzyRule {
    pub fn evaluate(&self, input: &FuzzyInput) -> Option<Suggestion> {
        if self.antecedents.is_empty() {
            return None;
        }
        let mut strength = 1.0f64;
        for clause in &self.antecedents {
            let m = clause.eval(input);
            strength = strength.min(m);
            if strength == 0.0 {
                return None;
            }
        }
        let confidence = strength * self.weight;
        if confidence <= 0.0 {
            return None;
        }
        Some(Suggestion {
            suggestion_type: self.consequent.clone(),
            confidence,
        })
    }
}

pub struct FuzzyEngine {
    pub rules: Vec<FuzzyRule>,
}

impl FuzzyEngine {
    pub fn new(rules: Vec<FuzzyRule>) -> Self {
        Self { rules }
    }

    pub fn evaluate(&self, input: &FuzzyInput) -> Vec<Suggestion> {
        let mut results: Vec<Suggestion> = self
            .rules
            .iter()
            .filter_map(|rule| rule.evaluate(input))
            .collect();
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    pub fn evaluate_top(&self, input: &FuzzyInput, n: usize) -> Vec<Suggestion> {
        let mut results = self.evaluate(input);
        results.truncate(n);
        results
    }
}
