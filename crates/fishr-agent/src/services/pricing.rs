use rust_decimal::Decimal;
use fishr_core::models::{Preparation, CostType};

pub fn calculate_item_price(
    weight_grams: i32,
    price_per_kg: Decimal,
    preparation: Option<&Preparation>,
) -> (Decimal, Decimal) {
    let weight_kg = Decimal::new(weight_grams as i64, 3);
    let subtotal = weight_kg * price_per_kg;

    let prep_fee = match preparation {
        Some(prep) => match prep.cost_type {
            CostType::Fixed => prep.additional_cost,
            CostType::Percentage => {
                subtotal * prep.additional_cost / Decimal::new(100, 0)
            }
        },
        None => Decimal::ZERO,
    };

    (subtotal, prep_fee)
}
