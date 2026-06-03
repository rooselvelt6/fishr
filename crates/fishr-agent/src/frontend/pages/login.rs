use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::frontend::components::auth::AuthContext;

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = auth.error;
    let loading = RwSignal::new(false);

    let do_login = {
        let auth = auth.clone();
        let username = username;
        let password = password;
        move |_| {
            if loading.get() {
                return;
            }
            let user = username.get();
            let pass = password.get();
            loading.set(true);
            let auth = auth.clone();
            spawn_local(async move {
                auth.login(&user, &pass).await;
                loading.set(false);
            });
        }
    };

    fn handle_enter(
        loading: RwSignal<bool>,
        username: RwSignal<String>,
        password: RwSignal<String>,
        auth: AuthContext,
        ev: leptos::ev::KeyboardEvent,
    ) {
        if ev.key() == "Enter" && !loading.get() {
            let user = username.get();
            let pass = password.get();
            loading.set(true);
            let auth = auth.clone();
            spawn_local(async move {
                auth.login(&user, &pass).await;
                loading.set(false);
            });
        }
    }

    view! {
        <div class="min-h-screen flex items-center justify-center relative overflow-hidden"
            style="background: linear-gradient(135deg, #0c4a6e 0%, #0d9488 50%, #0891b2 100%);"
        >
            // Animated waves background
            <div class="absolute inset-0 overflow-hidden pointer-events-none">
                <div class="absolute -bottom-2 left-0 right-0 h-32 opacity-20"
                    style="background: repeating-linear-gradient(90deg, transparent, transparent 20px, rgba(255,255,255,0.1) 20px, rgba(255,255,255,0.1) 40px);"
                >
                    <div class="wave wave1"></div>
                    <div class="wave wave2"></div>
                </div>
            </div>

            // Floating fish decorations
            <i class="fas fa-fish absolute text-white/5 text-8xl top-20 left-20 animate-pulse"></i>
            <i class="fas fa-fish absolute text-white/5 text-6xl bottom-40 right-20 animate-bounce" style="animation-duration: 3s;"></i>
            <i class="fas fa-water absolute text-white/5 text-4xl top-40 right-40"></i>
            <i class="fas fa-ship absolute text-white/5 text-5xl bottom-20 left-40" style="animation: sway 4s ease-in-out infinite;"></i>

            // Login card
            <div class="relative w-full max-w-md mx-4">
                <div class="bg-white/95 backdrop-blur-lg rounded-3xl shadow-2xl p-8 md:p-10 border border-white/20">
                    // Logo section
                    <div class="text-center mb-8">
                        <div class="inline-flex items-center justify-center w-20 h-20 rounded-full
                            bg-gradient-to-br from-cyan-500 to-blue-600 shadow-lg shadow-cyan-500/30 mb-4">
                            <i class="fas fa-fish text-4xl text-white"></i>
                        </div>
                        <h1 class="text-3xl font-bold text-gray-800">"Fishr"</h1>
                        <p class="text-gray-500 mt-1 text-sm">"Sistema de Gestión de Pescadería"</p>
                    </div>

                    // Error message
                    {move || error.get().map(|msg| {
                        view! {
                            <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-xl flex items-center gap-2 text-red-700 text-sm">
                                <i class="fas fa-exclamation-circle text-red-500"></i>
                                <span>{msg}</span>
                            </div>
                        }
                    })}

                    // Login form
                    <div class="space-y-4">
                        <div class="relative">
                            <span class="absolute inset-y-0 left-0 flex items-center pl-4 text-gray-400">
                                <i class="fas fa-user"></i>
                            </span>
                            <input
                                type="text"
                                placeholder="Usuario"
                                class="w-full pl-11 pr-4 py-3 bg-gray-50 border border-gray-200 rounded-xl
                                    focus:outline-none focus:ring-2 focus:ring-cyan-500 focus:border-transparent
                                    placeholder:text-gray-400 text-gray-700 transition-all"
                                prop:value=username
                                on:input=move |ev| username.set(event_target_value(&ev))
                                on:keydown={
                                    let auth = auth.clone();
                                    move |ev| handle_enter(loading, username, password, auth.clone(), ev)
                                }
                                disabled=move || loading.get()
                            />
                        </div>

                        <div class="relative">
                            <span class="absolute inset-y-0 left-0 flex items-center pl-4 text-gray-400">
                                <i class="fas fa-lock"></i>
                            </span>
                            <input
                                type="password"
                                placeholder="Contraseña"
                                class="w-full pl-11 pr-4 py-3 bg-gray-50 border border-gray-200 rounded-xl
                                    focus:outline-none focus:ring-2 focus:ring-cyan-500 focus:border-transparent
                                    placeholder:text-gray-400 text-gray-700 transition-all"
                                prop:value=password
                                on:input=move |ev| password.set(event_target_value(&ev))
                                on:keydown={
                                    let auth = auth.clone();
                                    move |ev| handle_enter(loading, username, password, auth.clone(), ev)
                                }
                                disabled=move || loading.get()
                            />
                        </div>

                        <button
                            on:click=do_login
                            disabled=move || loading.get()
                            class="w-full py-3 px-6 bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-600 hover:to-blue-700
                                text-white font-medium rounded-xl shadow-lg shadow-cyan-500/25
                                hover:shadow-xl hover:shadow-cyan-500/30 transition-all duration-200
                                active:scale-[0.98] flex items-center justify-center gap-2
                                disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            {move || if loading.get() {
                                view! { <i class="fas fa-spinner fa-spin"></i> }.into_any()
                            } else {
                                view! { <i class="fas fa-right-to-bracket"></i> }.into_any()
                            }}
                            {move || if loading.get() { "Ingresando..." } else { "Iniciar Sesión" }}
                        </button>
                    </div>

                    // Footer
                    <div class="mt-8 text-center text-xs text-gray-400">
                        <i class="fas fa-fish mr-1"></i>
                        "Fishr v1.0 — "
                        <i class="fas fa-map-pin mr-1"></i>
                        "Venezuela"
                    </div>
                </div>
            </div>
        </div>
    }
}
