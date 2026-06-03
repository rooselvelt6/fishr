use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Router, Routes, Route};
use leptos_router::path;

#[component]
pub fn CentralApp() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html attr:lang="es" />
        <Title text="Fishr Central - Gestión" />
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <Router>
            <div class="flex h-screen bg-gray-100">
                <aside class="w-64 bg-gray-900 text-white flex flex-col">
                    <div class="p-4 text-xl font-bold border-b border-gray-800">
                        "🐟 Fishr Central"
                    </div>
                    <nav class="flex-1 p-4 space-y-2">
                        <NavLink href="/" label="📊 Dashboard" />
                        <NavLink href="/branches" label="🏪 Sucursales" />
                        <NavLink href="/reports" label="📈 Reportes Globales" />
                    </nav>
                </aside>
                <main class="flex-1 overflow-y-auto p-6">
                    <Routes fallback=|| "Página no encontrada">
                        <Route path=path!("") view=|| view! { <DashboardView /> } />
                        <Route path=path!("/branches") view=|| view! { <BranchesView /> } />
                        <Route path=path!("/reports") view=|| view! { <ReportsView /> } />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

#[component]
fn NavLink(href: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <a href=href
            class="flex items-center px-3 py-2 hover:bg-gray-800 rounded-lg transition-colors">
            {label}
        </a>
    }
}

#[component]
fn DashboardView() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold">"Dashboard Central"</h1>
            <p class="text-gray-500">"Resumen de todas las sucursales."</p>
        </div>
    }
}

#[component]
fn BranchesView() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold">"🏪 Sucursales"</h1>
            <p class="text-gray-500">"Listado de sucursales activas."</p>
        </div>
    }
}

#[component]
fn ReportsView() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold">"📈 Reportes Globales"</h1>
            <p class="text-gray-500">"Reportes consolidados de todas las sucursales."</p>
        </div>
    }
}
