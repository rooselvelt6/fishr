use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;
use crate::frontend::components::auth::AuthContext;

#[derive(Clone, Deserialize)]
struct Sale {
    #[allow(dead_code)]
    id: String,
    payment_method_name: String,
    total: String,
    created_at: String,
}

#[cfg(target_arch = "wasm32")]
async fn fetch_recent_sales(token: &str) -> Result<Vec<Sale>, String> {
    let resp = gloo_net::http::Request::get("/api/sales")
        .header("x-session-token", token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<Vec<Sale>>().await.map_err(|e| e.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_recent_sales(_token: &str) -> Result<Vec<Sale>, String> { Err("N/A".into()) }

#[component]
pub fn DashboardPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let sales = RwSignal::new(Vec::<Sale>::new());
    let today_count = RwSignal::new(0i64);
    let today_revenue = RwSignal::new(0.0f64);

    let auth_clone = auth.clone();
    let load = move || {
        let a = auth_clone.clone();
        spawn_local(async move {
            if let Some(t) = a.token.get() {
                if let Ok(list) = fetch_recent_sales(&t).await {
                    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                    let today_sales: Vec<_> = list.iter().filter(|s| s.created_at.starts_with(&today)).collect();
                    today_count.set(today_sales.len() as i64);
                    let rev: f64 = today_sales.iter()
                        .filter_map(|s| s.total.replace("Bs. ", "").parse::<f64>().ok())
                        .sum();
                    today_revenue.set(rev);
                    sales.set(list);
                }
            }
        });
    };
    load();

    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold text-gray-800">
                <i class="fas fa-chart-line text-blue-600 mr-2"></i>"Dashboard"
            </h1>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="bg-gradient-to-br from-blue-500 to-blue-700 rounded-xl shadow-sm p-6 text-white">
                    <div class="text-blue-100 text-sm">
                        <i class="fas fa-fish mr-1"></i>"Bienvenido"
                    </div>
                    <div class="text-xl font-bold mt-1">{move || auth.username.get()}</div>
                </div>
                <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
                    <div class="text-sm text-gray-500">
                        <i class="fas fa-receipt mr-1"></i>"Ventas Hoy"
                    </div>
                    <div class="text-3xl font-bold text-gray-800 mt-1">{move || today_count.get()}</div>
                </div>
                <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
                    <div class="text-sm text-gray-500">
                        <i class="fas fa-dollar-sign mr-1"></i>"Ingresos Hoy"
                    </div>
                    <div class="text-3xl font-bold text-green-600 mt-1">
                        "Bs. " {move || format!("{:.0}", today_revenue.get())}
                    </div>
                </div>
            </div>

            <div class="bg-white rounded-xl shadow-sm border border-gray-200">
                <div class="px-6 py-4 border-b border-gray-100">
                    <h2 class="font-semibold text-gray-700">
                        <i class="fas fa-clock-rotate-left mr-2"></i>"Actividad Reciente"
                    </h2>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full text-sm">
                        <thead class="bg-gray-50 text-gray-600">
                            <tr>
                                <th class="text-left px-6 py-3 font-medium">"Método"</th>
                                <th class="text-right px-6 py-3 font-medium">"Total"</th>
                                <th class="text-right px-6 py-3 font-medium">"Fecha"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-100">
                            {move || sales.get().iter().cloned().take(10).map(|s| {
                                view! {
                                    <tr class="hover:bg-gray-50 transition-colors">
                                        <td class="px-6 py-3">{s.payment_method_name}</td>
                                        <td class="px-6 py-3 text-right font-semibold">"Bs. " {s.total}</td>
                                        <td class="px-6 py-3 text-right text-gray-500">{s.created_at}</td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>()}
                        </tbody>
                    </table>
                </div>
                {move || if sales.get().is_empty() {
                    view! { <p class="text-center text-gray-400 py-8">"No hay ventas recientes"</p> }.into_any()
                } else { "".into_any() }}
            </div>
        </div>
    }
}
