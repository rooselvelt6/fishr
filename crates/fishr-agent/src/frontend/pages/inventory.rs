use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use crate::frontend::components::auth::AuthContext;

#[derive(Clone, Serialize, Deserialize)]
struct Container {
    id: String,
    name: String,
    fish_type_name: String,
    capacity_kg: f64,
    current_kg: f64,
    is_active: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct MarketPrice {
    fish_type_name: String,
    price_per_kg: f64,
}

#[cfg(target_arch = "wasm32")]
async fn fetch_containers(token: &str) -> Result<Vec<Container>, String> {
    let resp = gloo_net::http::Request::get("/api/containers")
        .header("x-session-token", token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<Vec<Container>>().await.map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn fetch_market_prices(token: &str) -> Result<Vec<MarketPrice>, String> {
    let resp = gloo_net::http::Request::get("/api/market-prices")
        .header("x-session-token", token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<Vec<MarketPrice>>().await.map_err(|e| e.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_containers(_token: &str) -> Result<Vec<Container>, String> { Err("N/A".into()) }

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_market_prices(_token: &str) -> Result<Vec<MarketPrice>, String> { Err("N/A".into()) }

#[component]
pub fn InventoryPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let containers = RwSignal::new(Vec::<Container>::new());
    let prices = RwSignal::new(Vec::<MarketPrice>::new());

    let load = move || {
        let a = auth.clone();
        spawn_local(async move {
            if let Some(t) = a.token.get() {
                if let Ok(list) = fetch_containers(&t).await {
                    containers.set(list);
                }
                if let Ok(list) = fetch_market_prices(&t).await {
                    prices.set(list);
                }
            }
        });
    };
    load();

    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold text-gray-800">
                <i class="fas fa-boxes-stacked text-emerald-600 mr-2"></i>"Inventario"
            </h1>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div class="bg-white rounded-xl shadow-sm border border-gray-200">
                    <div class="px-6 py-4 border-b border-gray-100">
                        <h2 class="font-semibold text-gray-700">
                            <i class="fas fa-box text-amber-500 mr-2"></i>"Contenedores"
                        </h2>
                    </div>
                    <div class="p-4 space-y-2">
                        {move || containers.get().iter().cloned().map(|c| {
                            let pct = if c.capacity_kg > 0.0 { (c.current_kg / c.capacity_kg * 100.0) as i32 } else { 0 };
                            let bar_color = if pct > 90 { "bg-red-500" } else if pct > 70 { "bg-yellow-500" } else { "bg-emerald-500" };
                            view! {
                                <div class="p-3 rounded-lg border border-gray-100 hover:border-gray-200 transition-all">
                                    <div class="flex justify-between items-center">
                                        <div>
                                            <div class="font-medium text-gray-800">{c.name}</div>
                                            <div class="text-xs text-gray-500">{c.fish_type_name}</div>
                                        </div>
                                        <div class="text-sm font-semibold">
                                            {format!("{:.1}", c.current_kg)}" / "{format!("{:.1}", c.capacity_kg)}" kg"
                                        </div>
                                    </div>
                                    <div class="mt-2 w-full bg-gray-200 rounded-full h-2">
                                        <div class=format!("{} h-2 rounded-full transition-all", bar_color)
                                            style=format!("width: {}%", pct)></div>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                <div class="bg-white rounded-xl shadow-sm border border-gray-200">
                    <div class="px-6 py-4 border-b border-gray-100">
                        <h2 class="font-semibold text-gray-700">
                            <i class="fas fa-tags text-blue-500 mr-2"></i>"Precios de Mercado"
                        </h2>
                    </div>
                    <div class="p-4 space-y-2">
                        {move || prices.get().iter().cloned().map(|p| {
                            view! {
                                <div class="flex justify-between items-center py-2 border-b border-gray-100 last:border-0">
                                    <span class="text-gray-700">{p.fish_type_name}</span>
                                    <span class="font-semibold text-green-600">
                                        "Bs. " {format!("{:.2}", p.price_per_kg)}"/kg"
                                    </span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </div>
        </div>
    }
}
