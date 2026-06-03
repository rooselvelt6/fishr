use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use crate::frontend::components::auth::AuthContext;

#[derive(Clone, Serialize, Deserialize)]
struct AvailableFish {
    id: String,
    fish_type_name: String,
    container_id: String,
    container_name: String,
    weight_grams: i32,
    price_per_kg: f64,
}

#[derive(Clone, Serialize, Deserialize)]
struct PaymentMethod {
    name: String,
    description: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct ConfirmSaleRequest {
    items: Vec<ConfirmSaleItem>,
    payment_method: String,
    customer_id: Option<String>,
    cash_amount: Option<f64>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ConfirmSaleItem {
    fish_item_id: String,
    container_id: String,
    weight_grams: i32,
    preparation_id: Option<String>,
    preparation_name: Option<String>,
    preparation_fee: f64,
}

#[cfg(target_arch = "wasm32")]
async fn fetch_available_fish(token: &str) -> Result<Vec<AvailableFish>, String> {
    let resp = gloo_net::http::Request::get("/api/fish/available")
        .header("x-session-token", token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<Vec<AvailableFish>>().await.map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn fetch_payment_methods(token: &str) -> Result<Vec<PaymentMethod>, String> {
    let resp = gloo_net::http::Request::get("/api/pos/payment-methods")
        .header("x-session-token", token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<Vec<PaymentMethod>>().await.map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn post_confirm_sale(token: &str, req: &ConfirmSaleRequest) -> Result<serde_json::Value, String> {
    let body = serde_json::to_string(req).map_err(|e| e.to_string())?;
    let resp = gloo_net::http::Request::post("/api/pos/confirm")
        .header("Content-Type", "application/json")
        .header("x-session-token", token)
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<serde_json::Value>().await.map_err(|e| e.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_available_fish(_token: &str) -> Result<Vec<AvailableFish>, String> { Err("N/A".into()) }
#[cfg(not(target_arch = "wasm32"))]
async fn fetch_payment_methods(_token: &str) -> Result<Vec<PaymentMethod>, String> { Err("N/A".into()) }
#[cfg(not(target_arch = "wasm32"))]
async fn post_confirm_sale(_token: &str, _req: &ConfirmSaleRequest) -> Result<serde_json::Value, String> { Err("N/A".into()) }

fn refresh_data(auth: &AuthContext, fish: RwSignal<Vec<AvailableFish>>, methods: RwSignal<Vec<PaymentMethod>>) {
    let a = auth.clone();
    spawn_local(async move {
        if let Some(t) = a.token.get() {
            if let Ok(list) = fetch_available_fish(&t).await {
                fish.set(list);
            }
            if let Ok(list) = fetch_payment_methods(&t).await {
                methods.set(list);
            }
        }
    });
}

#[derive(Clone)]
struct CartItem {
    fish_item_id: String,
    fish_type_name: String,
    container_id: String,
    weight_grams: i32,
    price_per_kg: f64,
    line_total: f64,
}

fn compute_total(cart: &[CartItem]) -> f64 {
    cart.iter().map(|ci| ci.line_total).sum()
}

#[component]
pub fn PosPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let fish = RwSignal::new(Vec::<AvailableFish>::new());
    let methods = RwSignal::new(Vec::<PaymentMethod>::new());
    let cart = RwSignal::new(Vec::<CartItem>::new());
    let selected_method = RwSignal::new(String::new());
    let scale_weight = RwSignal::new(0.0f64);
    let loading = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);
    let success_msg = RwSignal::new(Option::<String>::None);

    refresh_data(&auth, fish, methods);

    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold text-gray-800">
                <i class="fas fa-cash-register text-blue-600 mr-2"></i> "Punto de Venta"
            </h1>

            {move || error_msg.get().map(|e| view! {
                <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded-lg">{e}</div>
            })}
            {move || success_msg.get().map(|s| view! {
                <div class="bg-green-100 border border-green-400 text-green-700 px-4 py-3 rounded-lg">{s}</div>
            })}

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div class="lg:col-span-2 space-y-4">
                    <div class="bg-white rounded-xl shadow-sm border border-gray-200">
                        <div class="px-6 py-4 border-b border-gray-100">
                            <h2 class="font-semibold text-gray-700">
                                <i class="fas fa-fish text-cyan-500 mr-2"></i>"Pescados Disponibles"
                            </h2>
                        </div>
                        <div class="p-4 grid grid-cols-1 sm:grid-cols-2 gap-3">
                            {move || fish.get().into_iter().map(|f| {
                                let ff = f.clone();
                                view! {
                                    <button
                                        on:click=move |_| {
                                            let mut c = cart.get();
                                            c.push(CartItem {
                                                fish_item_id: ff.id.clone(),
                                                fish_type_name: ff.fish_type_name.clone(),
                                                container_id: ff.container_id.clone(),
                                                weight_grams: ff.weight_grams,
                                                price_per_kg: ff.price_per_kg,
                                                line_total: (ff.weight_grams as f64 / 1000.0) * ff.price_per_kg,
                                            });
                                            cart.set(c);
                                        }
                                        class="text-left p-3 rounded-lg border border-gray-200 hover:border-blue-400 hover:bg-blue-50 transition-all"
                                    >
                                        <div class="font-medium text-gray-800">{f.fish_type_name}</div>
                                        <div class="text-sm text-gray-500">{f.container_name}" - "{f.weight_grams}"g"</div>
                                        <div class="text-sm font-semibold text-green-600">
                                            "Bs. " {format!("{:.2}", f.price_per_kg)} "/kg"
                                        </div>
                                        <div class="text-xs text-gray-400">
                                            <i class="fas fa-weight-hanging mr-1"></i>{f.weight_grams}"g disp."
                                        </div>
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>

                    <div class="bg-white rounded-xl shadow-sm border border-gray-200">
                        <div class="px-6 py-4 border-b border-gray-100">
                            <h2 class="font-semibold text-gray-700">
                                <i class="fas fa-weight-scale text-blue-500 mr-2"></i>"Báscula"
                            </h2>
                        </div>
                        <div class="p-6 text-center">
                            <div class="text-4xl font-bold text-blue-600">
                                {move || format!("{:.1} g", scale_weight.get())}
                            </div>
                            <p class="text-sm text-gray-400 mt-1">"Peso registrado por la báscula"</p>
                        </div>
                    </div>
                </div>

                <div class="space-y-4">
                    <div class="bg-white rounded-xl shadow-sm border border-gray-200">
                        <div class="px-6 py-4 border-b border-gray-100">
                            <h2 class="font-semibold text-gray-700">
                                <i class="fas fa-shopping-cart text-orange-500 mr-2"></i>"Ticket"
                            </h2>
                        </div>
                        <div class="p-4 space-y-2 max-h-96 overflow-y-auto">
                            {move || if cart.get().is_empty() {
                                view! { <p class="text-gray-400 text-sm text-center py-8">"Agregue pescados a la venta"</p> }.into_any()
                            } else {
                                let total = compute_total(&cart.get());
                                let items: Vec<_> = cart.get().into_iter().map(|ci| {
                                    view! {
                                        <div class="flex justify-between items-center py-2 border-b border-gray-100 last:border-0">
                                            <div>
                                                <div class="text-sm font-medium">{ci.fish_type_name}</div>
                                                <div class="text-xs text-gray-400">{ci.weight_grams}"g × Bs. " {format!("{:.2}", ci.price_per_kg)}"/kg"</div>
                                            </div>
                                            <div class="text-sm font-semibold">"Bs. " {format!("{:.2}", ci.line_total)}</div>
                                        </div>
                                    }
                                }).collect();
                                let total_view = view! {
                                    <div class="px-4 py-3 border-t border-gray-100">
                                        <div class="flex justify-between items-center text-lg font-bold">
                                            <span>"Total"</span>
                                            <span class="text-blue-600">"Bs. " {format!("{:.2}", total)}</span>
                                        </div>
                                    </div>
                                };
                                (items, total_view).into_any()
                            }}
                        </div>
                    </div>

                    <div class="bg-white rounded-xl shadow-sm border border-gray-200">
                        <div class="px-6 py-4 border-b border-gray-100">
                            <h2 class="font-semibold text-gray-700">
                                <i class="fas fa-credit-card text-purple-500 mr-2"></i>"Método de Pago"
                            </h2>
                        </div>
                        <div class="p-4 space-y-2">
                            {move || methods.get().into_iter().map(|m| {
                                let m_name = m.name.clone();
                                let is_sel = selected_method.get() == m.name;
                                view! {
                                    <button
                                        on:click=move |_| selected_method.set(m_name.clone())
                                        class=format!("w-full text-left px-4 py-3 rounded-lg border transition-all {}",
                                            if is_sel { "border-blue-500 bg-blue-50 text-blue-700" } else { "border-gray-200 hover:border-gray-300" }
                                        )
                                    >
                                        <div class="font-medium">{m.name}</div>
                                        <div class="text-xs text-gray-500">{m.description}</div>
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>

                    <button
                        on:click=move |_| {
                            let items = cart.get();
                            let pm = selected_method.get();
                            if items.is_empty() || pm.is_empty() { return; }
                            loading.set(true);
                            error_msg.set(None);
                            success_msg.set(None);
                            let a = auth.clone();
                            spawn_local(async move {
                                if let Some(t) = a.token.get() {
                                    let req = ConfirmSaleRequest {
                                        items: items.iter().map(|ci| ConfirmSaleItem {
                                            fish_item_id: ci.fish_item_id.clone(),
                                            container_id: ci.container_id.clone(),
                                            weight_grams: ci.weight_grams,
                                            preparation_id: None,
                                            preparation_name: None,
                                            preparation_fee: 0.0,
                                        }).collect(),
                                        payment_method: pm.clone(),
                                        customer_id: None,
                                        cash_amount: None,
                                    };
                                    match post_confirm_sale(&t, &req).await {
                                        Ok(resp) => {
                                            loading.set(false);
                                            success_msg.set(Some(format!("Venta confirmada #{}", resp["sale_id"].as_str().unwrap_or(""))));
                                            cart.set(Vec::new());
                                            selected_method.set(String::new());
                                            refresh_data(&a, fish, methods);
                                        }
                                        Err(e) => {
                                            loading.set(false);
                                            error_msg.set(Some(e));
                                        }
                                    }
                                }
                            });
                        }
                        disabled=move || cart.get().is_empty() || selected_method.get().is_empty() || loading.get()
                        class="w-full py-3 rounded-xl font-bold text-white transition-all disabled:opacity-50 disabled:cursor-not-allowed
                            bg-gradient-to-r from-blue-600 to-cyan-600 hover:from-blue-700 hover:to-cyan-700"
                    >
                        {move || if loading.get() {
                            view! { <span><i class="fas fa-spinner fa-spin mr-2"></i>"Procesando..."</span> }
                        } else {
                            view! { <span><i class="fas fa-check-circle mr-2"></i>"Confirmar Venta"</span> }
                        }}
                    </button>
                </div>
            </div>
        </div>
    }
}
