use leptos::prelude::*;
use crate::frontend::components::auth::AuthContext;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();

    view! {
        <div class="max-w-2xl mx-auto space-y-6">
            <h1 class="text-2xl font-bold text-gray-800">
                <i class="fas fa-gear text-slate-600 mr-2"></i>"Configuración"
            </h1>

            <div class="bg-white rounded-xl shadow-sm border border-gray-200">
                <div class="px-6 py-4 border-b border-gray-100">
                    <h2 class="font-semibold text-gray-700">
                        <i class="fas fa-user-circle mr-2"></i>"Usuario"
                    </h2>
                </div>
                <div class="p-6 space-y-4">
                    <div class="flex items-center gap-4">
                        <div class="w-12 h-12 rounded-full bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center text-white font-bold text-lg">
                            {move || auth.username.get().chars().next().map(|c| c.to_string()).unwrap_or_default()}
                        </div>
                        <div>
                            <div class="font-medium text-gray-800">{move || auth.username.get()}</div>
                            <div class="text-sm text-gray-500">"Sesión activa"</div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="bg-white rounded-xl shadow-sm border border-gray-200">
                <div class="px-6 py-4 border-b border-gray-100">
                    <h2 class="font-semibold text-gray-700">
                        <i class="fas fa-info-circle mr-2"></i>"Información del Sistema"
                    </h2>
                </div>
                <div class="p-6 space-y-3 text-sm">
                    <div class="flex justify-between">
                        <span class="text-gray-500">"Versión"</span>
                        <span class="text-gray-800">"0.1.0"</span>
                    </div>
                    <div class="flex justify-between">
                        <span class="text-gray-500">"Entorno"</span>
                        <span class="text-gray-800">"Offline-first"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
