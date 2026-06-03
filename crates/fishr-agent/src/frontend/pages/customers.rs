use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use crate::frontend::components::auth::AuthContext;

#[derive(Clone, Serialize, Deserialize)]
struct Customer {
    id: String,
    name: String,
    doc_type: String,
    doc_number: String,
    phone: String,
    email: String,
}

#[cfg(target_arch = "wasm32")]
async fn fetch_customers(token: &str) -> Result<Vec<Customer>, String> {
    let resp = gloo_net::http::Request::get("/api/customers")
        .header("x-session-token", token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<Vec<Customer>>().await.map_err(|e| e.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_customers(_token: &str) -> Result<Vec<Customer>, String> { Err("N/A".into()) }

#[component]
pub fn CustomersPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let customers = RwSignal::new(Vec::<Customer>::new());

    let load = move || {
        let a = auth.clone();
        spawn_local(async move {
            if let Some(t) = a.token.get() {
                if let Ok(list) = fetch_customers(&t).await {
                    customers.set(list);
                }
            }
        });
    };
    load();

    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold text-gray-800">
                <i class="fas fa-users text-indigo-600 mr-2"></i>"Clientes"
            </h1>

            <div class="bg-white rounded-xl shadow-sm border border-gray-200">
                <div class="px-6 py-4 border-b border-gray-100">
                    <h2 class="font-semibold text-gray-700">
                        <i class="fas fa-list mr-2"></i>"Listado de Clientes"
                    </h2>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full text-sm">
                        <thead class="bg-gray-50 text-gray-600">
                            <tr>
                                <th class="text-left px-6 py-3 font-medium">"Nombre"</th>
                                <th class="text-left px-6 py-3 font-medium">"Doc. Identidad"</th>
                                <th class="text-left px-6 py-3 font-medium">"Teléfono"</th>
                                <th class="text-left px-6 py-3 font-medium">"Email"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-100">
                            {move || customers.get().iter().cloned().map(|c| {
                                view! {
                                    <tr class="hover:bg-gray-50 transition-colors">
                                        <td class="px-6 py-3 font-medium text-gray-800">{c.name}</td>
                                        <td class="px-6 py-3 text-gray-600">{c.doc_type}": "{c.doc_number}</td>
                                        <td class="px-6 py-3 text-gray-600">{c.phone}</td>
                                        <td class="px-6 py-3 text-gray-600">{c.email}</td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>()}
                        </tbody>
                    </table>
                </div>
                {move || if customers.get().is_empty() {
                    view! { <p class="text-center text-gray-400 py-8">"No hay clientes registrados"</p> }.into_any()
                } else { "".into_any() }}
            </div>
        </div>
    }
}
