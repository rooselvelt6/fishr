use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MembershipFn {
    Triangular { a: f64, b: f64, c: f64 },
    Trapezoidal { a: f64, b: f64, c: f64, d: f64 },
    LeftShoulder { a: f64, b: f64 },
    RightShoulder { a: f64, b: f64 },
}

impl MembershipFn {
    pub fn eval(&self, x: f64) -> f64 {
        match self {
            MembershipFn::Triangular { a, b, c } => {
                if x <= *a || x >= *c {
                    0.0
                } else if x == *b {
                    1.0
                } else if x < *b {
                    (x - a) / (b - a)
                } else {
                    (c - x) / (c - b)
                }
            }
            MembershipFn::Trapezoidal { a, b, c, d } => {
                if x <= *a || x >= *d {
                    0.0
                } else if x >= *b && x <= *c {
                    1.0
                } else if x < *b {
                    (x - a) / (b - a)
                } else {
                    (d - x) / (d - c)
                }
            }
            MembershipFn::LeftShoulder { a, b } => {
                if x <= *a {
                    1.0
                } else if x >= *b {
                    0.0
                } else {
                    (b - x) / (b - a)
                }
            }
            MembershipFn::RightShoulder { a, b } => {
                if x <= *a {
                    0.0
                } else if x >= *b {
                    1.0
                } else {
                    (x - a) / (b - a)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FuzzyVariable {
    StockLevel,
    TimeOfDay,
    Popularity,
    CustomerLoyalty,
    HourlyDemand,
    DaysSincePriceChange,
    SeasonalDemand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clause {
    pub var: FuzzyVariable,
    pub set: MembershipFn,
}

impl Clause {
    pub fn eval(&self, input: &FuzzyInput) -> f64 {
        let val = match self.var {
            FuzzyVariable::StockLevel => input.stock_pct,
            FuzzyVariable::TimeOfDay => input.hour,
            FuzzyVariable::Popularity => input.popularity_pct,
            FuzzyVariable::CustomerLoyalty => input.customer_visits_pct,
            FuzzyVariable::HourlyDemand => input.hourly_demand_pct,
            FuzzyVariable::DaysSincePriceChange => input.days_since_price_change,
            FuzzyVariable::SeasonalDemand => input.seasonal_demand_pct,
        };
        self.set.eval(val)
    }
}

#[derive(Debug, Clone)]
pub struct FuzzyInput {
    pub stock_pct: f64,
    pub hour: f64,
    pub popularity_pct: f64,
    pub customer_visits_pct: f64,
    pub hourly_demand_pct: f64,
    pub days_since_price_change: f64,
    pub seasonal_demand_pct: f64,
}

impl FuzzyInput {
    pub fn new_pos(stock_pct: f64, hour: f64, popularity_pct: f64, customer_visits_pct: f64, hourly_demand_pct: f64) -> Self {
        Self {
            stock_pct, hour, popularity_pct, customer_visits_pct, hourly_demand_pct,
            days_since_price_change: 0.0,
            seasonal_demand_pct: 50.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    SuggestPreparation { preparation_id: String, reason: String },
    SuggestDiscount { max_discount_pct: f64, reason: String },
    SuggestPromotion { message: String, reason: String },
    SuggestUpsell { message: String, reason: String },
    PriceFactor { factor: f64, reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub suggestion_type: SuggestionType,
    pub confidence: f64,
}
