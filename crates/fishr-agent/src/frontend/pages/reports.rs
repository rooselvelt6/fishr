use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;
use crate::frontend::components::auth::AuthContext;

#[derive(Clone, Deserialize)]
struct Sale {
    id: String,
    payment_method_name: String,
    total: String,
    created_at: String,
}

#[derive(Clone, Deserialize)]
struct DailyReport {
    total_sales: i64,
    total_revenue: String,
    average_ticket: String,
}

#[cfg(target_arch = "wasm32")]
async fn fetch_sales(token: &str) -> Result<Vec<Sale>, String> {
    let resp = gloo_net::http::Request::get("/api/sales")
        .header("x-session-token", token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<Vec<Sale>>().await.map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn fetch_daily_report(token: &str) -> Result<DailyReport, String> {
    let resp = gloo_net::http::Request::get("/api/reports/daily")
        .header("x-session-token", token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<DailyReport>().await.map_err(|e| e.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_sales(_token: &str) -> Result<Vec<Sale>, String> { Err("N/A".into()) }

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_daily_report(_token: &str) -> Result<DailyReport, String> { Err("N/A".into()) }

#[component]
pub fn ReportsPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let sales = RwSignal::new(Vec::<Sale>::new());
    let daily = RwSignal::new(Option::<DailyReport>::None);

    let load = move || {
        let a = auth.clone();
        spawn_local(async move {
            if let Some(t) = a.token.get() {
                if let Ok(r) = fetch_daily_report(&t).await {
                    daily.set(Some(r));
                }
                if let Ok(list) = fetch_sales(&t).await {
                    sales.set(list);
                }
            }
        });
    };
    load();

    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold text-gray-800">
                <i class="fas fa-chart-bar text-rose-600 mr-2"></i>"Reportes"
            </h1>

            {move || daily.get().map(|d| view! {
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
                        <div class="text-sm text-gray-500">
                            <i class="fas fa-receipt mr-1"></i>"Ventas Hoy"
                        </div>
                        <div class="text-3xl font-bold text-gray-800 mt-1">{d.total_sales}</div>
                    </div>
                    <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
                        <div class="text-sm text-gray-500">
                            <i class="fas fa-dollar-sign mr-1"></i>"Ingresos Hoy"
                        </div>
                        <div class="text-3xl font-bold text-green-600 mt-1">
                            "Bs. " {d.total_revenue}
                        </div>
                    </div>
                    <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
                        <div class="text-sm text-gray-500">
                            <i class="fas fa-receipt mr-1"></i>"Ticket Promedio"
                        </div>
                        <div class="text-3xl font-bold text-blue-600 mt-1">
                            "Bs. " {d.average_ticket}
                        </div>
                    </div>
                </div>
            })}

            <div class="bg-white rounded-xl shadow-sm border border-gray-200">
                <div class="px-6 py-4 border-b border-gray-100">
                    <h2 class="font-semibold text-gray-700">
                        <i class="fas fa-clock-rotate-left mr-2"></i>"Ventas Recientes"
                    </h2>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full text-sm">
                        <thead class="bg-gray-50 text-gray-600">
                            <tr>
                                <th class="text-left px-6 py-3 font-medium">"ID"</th>
                                <th class="text-left px-6 py-3 font-medium">"Método"</th>
                                <th class="text-right px-6 py-3 font-medium">"Total"</th>
                                <th class="text-right px-6 py-3 font-medium">"Fecha"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-100">
                            {move || sales.get().iter().cloned().map(|s| {
                                view! {
                                    <tr class="hover:bg-gray-50 transition-colors">
                                        <td class="px-6 py-3 font-mono text-xs text-gray-500">{s.id}</td>
                                        <td class="px-6 py-3">{s.payment_method_name}</td>
                                        <td class="px-6 py-3 text-right font-semibold">"Bs. " {s.total}</td>
                                        <td class="px-6 py-3 text-right text-gray-500">{s.created_at}</td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>()}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
