use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::*;
use leptos_router::components::{Router, Routes, Route};
use leptos_router::path;
use crate::frontend::components::auth::{AuthProvider, AuthContext};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html attr:lang="es" attr:dir="ltr" />
        <Title text="Fishr - Pescadería" />
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <Link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.7.2/css/all.min.css" />

        <style>
            "
            .wave {
                position: absolute;
                bottom: 0;
                left: 0;
                width: 200%;
                height: 100px;
                background: url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1440 320'%3E%3Cpath fill='%23ffffff' fill-opacity='0.15' d='M0,160L48,176C96,192,192,224,288,213.3C384,203,480,149,576,133.3C672,117,768,139,864,154.7C960,171,1056,181,1152,170.7C1248,160,1344,128,1392,112L1440,96L1440,320L1392,320C1344,320,1248,320,1152,320C1056,320,960,320,864,320C768,320,672,320,576,320C480,320,384,320,288,320C192,320,96,320,48,320L0,320Z'%3E%3C/path%3E%3C/svg%3E\") repeat-x;
                animation: waveAnim 8s linear infinite;
                transform: translateX(0);
            }
            .wave2 {
                animation-direction: reverse;
                animation-duration: 12s;
                opacity: 0.5;
                bottom: -10px;
            }
            @keyframes waveAnim {
                0% { transform: translateX(0); }
                100% { transform: translateX(-50%); }
            }
            @keyframes sway {
                0%, 100% { transform: translateX(0) rotate(0deg); }
                25% { transform: translateX(10px) rotate(3deg); }
                75% { transform: translateX(-10px) rotate(-3deg); }
            }
            "
        </style>

        <AuthProvider>
            <Router>
                <AppContent />
            </Router>
        </AuthProvider>
    }
}

#[component]
fn AppContent() -> impl IntoView {
    let auth = expect_context::<AuthContext>();

    // Check for existing session on client mount
    let auth_check = auth.clone();
    spawn_local(async move {
        auth_check.check_session().await;
    });

    move || {
        if auth.is_authenticated.get() {
            view! {
                <div class="flex h-screen bg-gray-100">
                    <Sidebar />
                    <main class="flex-1 overflow-y-auto p-6">
                        <Routes fallback=|| "Página no encontrada">
                            <Route path=path!("") view=|| view! { <pages::dashboard::DashboardPage /> } />
                            <Route path=path!("/pos") view=|| view! { <pages::pos::PosPage /> } />
                            <Route path=path!("/inventory") view=|| view! { <pages::inventory::InventoryPage /> } />
                            <Route path=path!("/customers") view=|| view! { <pages::customers::CustomersPage /> } />
                            <Route path=path!("/reports") view=|| view! { <pages::reports::ReportsPage /> } />
                            <Route path=path!("/settings") view=|| view! { <pages::settings::SettingsPage /> } />
                        </Routes>
                    </main>
                </div>
            }.into_any()
        } else {
            view! { <pages::login::LoginPage /> }.into_any()
        }
    }
}

use crate::frontend::pages;

#[component]
fn Sidebar() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let username = auth.username;

    view! {
        <aside class="w-64 bg-gradient-to-b from-slate-800 to-slate-900 text-white flex flex-col shadow-xl">
            <div class="p-5 text-xl font-bold border-b border-slate-700/50 flex items-center gap-2">
                <i class="fas fa-fish text-cyan-400"></i>
                "Fishr"
            </div>
            <nav class="flex-1 p-3 space-y-1">
                <SidebarLink href="/" icon="fa-chart-line" label="Dashboard" />
                <SidebarLink href="/pos" icon="fa-cash-register" label="Punto de Venta" />
                <SidebarLink href="/inventory" icon="fa-boxes-stacked" label="Inventario" />
                <SidebarLink href="/customers" icon="fa-users" label="Clientes" />
                <SidebarLink href="/reports" icon="fa-chart-bar" label="Reportes" />
                <SidebarLink href="/settings" icon="fa-gear" label="Configuración" />
            </nav>
            <div class="p-4 border-t border-slate-700/50">
                <div class="flex items-center gap-2 text-sm text-slate-400">
                    <i class="fas fa-store"></i>
                    <span>"Mi Pescadería"</span>
                </div>
                <div class="flex items-center gap-2 text-xs text-slate-500 mt-1">
                    <i class="fas fa-user"></i>
                    <span>{move || username.get()}</span>
                    <button
                        on:click=move |_| {
                            let a = auth.clone();
                            spawn_local(async move { a.logout().await; });
                        }
                        class="ml-auto text-slate-500 hover:text-red-400 transition-colors"
                    >
                        <i class="fas fa-right-from-bracket"></i>
                    </button>
                </div>
            </div>
        </aside>
    }
}

#[component]
fn SidebarLink(href: &'static str, icon: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <a
            href=href
            class="flex items-center gap-3 px-3 py-2.5 text-slate-300 hover:text-white hover:bg-slate-700/50 rounded-xl transition-all duration-150 text-sm"
        >
            <i class=format!("fas {} w-5 text-center text-cyan-400/70", icon)></i>
            {label}
        </a>
    }
}
