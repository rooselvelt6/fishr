use leptos::prelude::*;

#[component]
pub fn ReportsPanel() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold text-gray-800">"📈 Reportes"</h1>
            <p class="text-gray-500">"Reportes de ventas y rendimiento."</p>
        </div>
    }
}
