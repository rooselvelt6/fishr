use crate::fuzzy::engine::{FuzzyEngine, FuzzyRule};
use crate::fuzzy::sets::*;

pub fn build_pos_engine() -> FuzzyEngine {
    FuzzyEngine::new(vec![
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::StockLevel, set: MembershipFn::RightShoulder { a: 50.0, b: 85.0 } },
                Clause { var: FuzzyVariable::TimeOfDay, set: MembershipFn::RightShoulder { a: 17.0, b: 19.0 } },
            ],
            consequent: SuggestionType::SuggestDiscount { max_discount_pct: 15.0, reason: "Stock alto y próximo al cierre".into() },
            weight: 0.9,
        },
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::StockLevel, set: MembershipFn::RightShoulder { a: 70.0, b: 95.0 } },
            ],
            consequent: SuggestionType::SuggestPromotion { message: "Ofrece descuento por volumen — inventario elevado".into(), reason: "Nivel de inventario alto".into() },
            weight: 0.7,
        },
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::TimeOfDay, set: MembershipFn::Triangular { a: 6.0, b: 9.0, c: 12.0 } },
                Clause { var: FuzzyVariable::Popularity, set: MembershipFn::RightShoulder { a: 30.0, b: 60.0 } },
            ],
            consequent: SuggestionType::SuggestUpsell { message: "Sugerir preparación — horario matutino".into(), reason: "Clientes prefieren pescado preparado por la mañana".into() },
            weight: 0.6,
        },
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::StockLevel, set: MembershipFn::LeftShoulder { a: 10.0, b: 30.0 } },
                Clause { var: FuzzyVariable::Popularity, set: MembershipFn::RightShoulder { a: 50.0, b: 80.0 } },
            ],
            consequent: SuggestionType::SuggestUpsell { message: "Alta demanda — sugiere preparación premium".into(), reason: "Stock bajo + alta popularidad".into() },
            weight: 0.8,
        },
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::CustomerLoyalty, set: MembershipFn::RightShoulder { a: 30.0, b: 70.0 } },
                Clause { var: FuzzyVariable::HourlyDemand, set: MembershipFn::LeftShoulder { a: 30.0, b: 60.0 } },
            ],
            consequent: SuggestionType::SuggestPreparation { preparation_id: String::new(), reason: "Cliente frecuente — ofrece preparación especial".into() },
            weight: 0.5,
        },
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::StockLevel, set: MembershipFn::Triangular { a: 30.0, b: 60.0, c: 80.0 } },
                Clause { var: FuzzyVariable::HourlyDemand, set: MembershipFn::RightShoulder { a: 50.0, b: 80.0 } },
            ],
            consequent: SuggestionType::SuggestUpsell { message: "Hora pico — prioriza preparaciones rápidas".into(), reason: "Demanda alta en hora pico".into() },
            weight: 0.4,
        },
    ])
}

pub fn match_preparation_suggestion(
    suggestions: &[Suggestion],
    fish_type_name: &str,
    category: &str,
    preparations: &[(&str, &str)],
) -> Vec<Suggestion> {
    suggestions.iter().filter_map(|s| {
        match &s.suggestion_type {
            SuggestionType::SuggestPreparation { preparation_id: _, reason } => {
                let matched = match_preparation(fish_type_name, category, preparations);
                matched.map(|(pid, _pname)| Suggestion {
                    suggestion_type: SuggestionType::SuggestPreparation {
                        preparation_id: pid.to_string(),
                        reason: reason.clone(),
                    },
                    confidence: s.confidence,
                })
            }
            SuggestionType::SuggestUpsell { message, reason } => {
                let matched = match_preparation(fish_type_name, category, preparations);
                match matched {
                    Some((pid, _pname)) => Some(Suggestion {
                        suggestion_type: SuggestionType::SuggestPreparation {
                            preparation_id: pid.to_string(),
                            reason: format!("{}: {}", reason, message),
                        },
                        confidence: s.confidence,
                    }),
                    None => Some(s.clone()),
                }
            }
            _ => Some(s.clone()),
        }
    }).collect()
}

pub fn build_pricing_engine() -> FuzzyEngine {
    FuzzyEngine::new(vec![
        // Stock alto → reducir precio para mover inventario
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::StockLevel, set: MembershipFn::RightShoulder { a: 60.0, b: 90.0 } },
            ],
            consequent: SuggestionType::PriceFactor { factor: 0.85, reason: "Precio reducido: inventario alto".into() },
            weight: 0.8,
        },
        // Stock muy bajo → aumentar precio
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::StockLevel, set: MembershipFn::LeftShoulder { a: 5.0, b: 20.0 } },
            ],
            consequent: SuggestionType::PriceFactor { factor: 1.15, reason: "Precio incrementado: inventario bajo".into() },
            weight: 0.8,
        },
        // Stock alto + hora cierre → gran descuento
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::StockLevel, set: MembershipFn::RightShoulder { a: 50.0, b: 80.0 } },
                Clause { var: FuzzyVariable::TimeOfDay, set: MembershipFn::RightShoulder { a: 17.0, b: 19.0 } },
            ],
            consequent: SuggestionType::PriceFactor { factor: 0.75, reason: "Liquidación: stock alto al cierre".into() },
            weight: 0.9,
        },
        // Alta demanda horaria + stock moderado → precio normal+
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::HourlyDemand, set: MembershipFn::RightShoulder { a: 60.0, b: 85.0 } },
                Clause { var: FuzzyVariable::StockLevel, set: MembershipFn::Triangular { a: 20.0, b: 50.0, c: 70.0 } },
            ],
            consequent: SuggestionType::PriceFactor { factor: 1.05, reason: "Demanda alta en hora pico".into() },
            weight: 0.5,
        },
        // Muchos días sin cambio de precio + stock alto → rebaja progresiva
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::DaysSincePriceChange, set: MembershipFn::RightShoulder { a: 5.0, b: 14.0 } },
                Clause { var: FuzzyVariable::StockLevel, set: MembershipFn::RightShoulder { a: 40.0, b: 70.0 } },
            ],
            consequent: SuggestionType::PriceFactor { factor: 0.80, reason: "Precio estancado + inventario alto".into() },
            weight: 0.6,
        },
        // Alta popularidad + stock bajo → premium pricing
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::Popularity, set: MembershipFn::RightShoulder { a: 50.0, b: 80.0 } },
                Clause { var: FuzzyVariable::StockLevel, set: MembershipFn::LeftShoulder { a: 10.0, b: 30.0 } },
            ],
            consequent: SuggestionType::PriceFactor { factor: 1.20, reason: "Premium: alta demanda + stock limitado".into() },
            weight: 0.7,
        },
        // Demanda estacional alta → incremento
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::SeasonalDemand, set: MembershipFn::RightShoulder { a: 60.0, b: 85.0 } },
            ],
            consequent: SuggestionType::PriceFactor { factor: 1.10, reason: "Temporada alta: demanda estacional elevada".into() },
            weight: 0.4,
        },
        // Demanda estacional baja + stock alto → oferta
        FuzzyRule {
            antecedents: vec![
                Clause { var: FuzzyVariable::SeasonalDemand, set: MembershipFn::LeftShoulder { a: 15.0, b: 35.0 } },
                Clause { var: FuzzyVariable::StockLevel, set: MembershipFn::RightShoulder { a: 40.0, b: 70.0 } },
            ],
            consequent: SuggestionType::PriceFactor { factor: 0.80, reason: "Fuera de temporada + excedente".into() },
            weight: 0.5,
        },
    ])
}

pub fn compute_price_factor(
    engine: &FuzzyEngine,
    input: &FuzzyInput,
) -> (f64, Vec<String>) {
    let results = engine.evaluate(input);
    let mut total_weight = 0.0f64;
    let mut weighted_sum = 0.0f64;
    let mut reasons = Vec::new();

    for s in &results {
        if let SuggestionType::PriceFactor { factor, reason } = &s.suggestion_type {
            let w = s.confidence;
            weighted_sum += factor * w;
            total_weight += w;
            reasons.push(reason.clone());
        }
    }

    if total_weight == 0.0 {
        return (1.0, vec!["Precio base sin ajuste".into()]);
    }

    let factor = (weighted_sum / total_weight).clamp(0.7, 1.3);
    (factor, reasons)
}

fn match_preparation<'a>(
    fish_type_name: &str,
    category: &str,
    preparations: &[(&'a str, &'a str)],
) -> Option<(&'a str, &'a str)> {
    let fish_lower = fish_type_name.to_lowercase();
    let cat_lower = category.to_lowercase();

    // Prefer fileteado for white fish
    if cat_lower == "white" {
        if let Some(p) = preparations.iter().find(|(name, _)| name.to_lowercase().contains("filete")) {
            return Some(*p);
        }
    }
    // Limpieza for crustaceans/shellfish
    if cat_lower == "crustacean" || cat_lower == "shellfish" {
        if let Some(p) = preparations.iter().find(|(name, _)| name.to_lowercase().contains("limpieza")) {
            return Some(*p);
        }
    }
    // General: any preparation matching by keyword
    let keywords = if fish_lower.contains("merluza") || fish_lower.contains("lenguado") || fish_lower.contains("pargo") {
        vec!["filete"]
    } else if fish_lower.contains("camar") || fish_lower.contains("pulpeta") {
        vec!["limpieza"]
    } else {
        return None;
    };
    for kw in keywords {
        if let Some(p) = preparations.iter().find(|(name, _)| name.to_lowercase().contains(kw)) {
            return Some(*p);
        }
    }
    None
}
