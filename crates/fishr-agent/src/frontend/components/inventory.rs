use leptos::prelude::*;

#[component]
pub fn InventoryPanel() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold text-gray-800">"📦 Inventario"</h1>
            <p class="text-gray-500">"Gestión de contenedores y pescados."</p>
        </div>
    }
}
