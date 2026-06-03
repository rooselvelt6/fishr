use leptos::prelude::*;

#[component]
pub fn PosPanel() -> impl IntoView {
    let _selected_payment = RwSignal::new(String::new());
    let scale_weight = RwSignal::new(0.0f64);

    view! {
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div class="lg:col-span-2 bg-white rounded-xl shadow-sm border p-6">
                <h2 class="text-lg font-bold mb-4">"🛒 Punto de Venta"</h2>
                <p class="text-gray-500">"Seleccione pescados disponibles para iniciar una venta."</p>
            </div>
            <div class="space-y-4">
                <div class="bg-white rounded-xl shadow-sm border p-6 text-center">
                    <h3 class="text-sm text-gray-500 mb-1">"⚖️ Báscula"</h3>
                    <div class="text-3xl font-bold text-blue-600">
                        {move || format!("{:.0} g", scale_weight.get())}
                    </div>
                </div>
                <div class="bg-white rounded-xl shadow-sm border p-6">
                    <h3 class="font-medium mb-2">"Método de Pago"</h3>
                    <p class="text-gray-400 text-sm">"Seleccione un método de pago"</p>
                </div>
            </div>
        </div>
    }
}
