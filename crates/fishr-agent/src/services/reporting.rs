use rayon::prelude::*;
use rust_decimal::Decimal;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct ConsolidatedReport {
    pub total_sales: i64,
    pub total_revenue: Decimal,
    pub total_items_sold: i64,
    pub by_product: Vec<ProductStat>,
    pub by_hour: Vec<HourStat>,
    pub average_sale_value: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductStat {
    pub name: String,
    pub quantity: i64,
    pub revenue: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct HourStat {
    pub hour: u32,
    pub sales: i64,
    pub revenue: Decimal,
}

#[derive(Debug, Clone)]
pub struct SaleData {
    pub total: Decimal,
    pub item_count: i32,
    pub hour: u32,
    pub items: Vec<SaleItemData>,
}

#[derive(Debug, Clone)]
pub struct SaleItemData {
    pub name: String,
    pub quantity: i32,
    pub subtotal: Decimal,
}

pub fn consolidate_sales(items: &[SaleData]) -> ConsolidatedReport {
    let total_sales = items.len() as i64;

    let (total_revenue, total_items): (Decimal, i64) = items.par_iter()
        .map(|i| (i.total, i.item_count as i64))
        .reduce(|| (Decimal::ZERO, 0), |a, b| (a.0 + b.0, a.1 + b.1));

    // Product aggregation using parallel fold
    let product_map: HashMap<String, (i64, Decimal)> = items.par_iter()
        .flat_map(|s| &s.items)
        .fold(HashMap::new, |mut map, item| {
            let entry = map.entry(item.name.clone()).or_insert((0i64, Decimal::ZERO));
            entry.0 += item.quantity as i64;
            entry.1 += item.subtotal;
            map
        })
        .reduce(HashMap::new, |mut a, b| {
            for (k, v) in b {
                let e = a.entry(k).or_insert((0, Decimal::ZERO));
                e.0 += v.0;
                e.1 += v.1;
            }
            a
        });

    let by_product: Vec<ProductStat> = product_map
        .into_iter()
        .map(|(name, (quantity, revenue))| ProductStat { name, quantity, revenue })
        .collect();

    let by_hour: Vec<HourStat> = (0..24).map(|h| {
        let (sales, revenue) = items.par_iter()
            .filter(|s| s.hour == h)
            .map(|s| (1i64, s.total))
            .reduce(|| (0i64, Decimal::ZERO), |a, b| (a.0 + b.0, a.1 + b.1));
        HourStat { hour: h, sales, revenue }
    }).collect();

    let avg = if total_sales > 0 {
        total_revenue / Decimal::from(total_sales)
    } else {
        Decimal::ZERO
    };

    ConsolidatedReport {
        total_sales,
        total_revenue,
        total_items_sold: total_items,
        by_product,
        by_hour,
        average_sale_value: avg,
    }
}
