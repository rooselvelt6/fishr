use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AuthContext {
    pub is_authenticated: RwSignal<bool>,
    pub username: RwSignal<String>,
    pub error: RwSignal<Option<String>>,
    pub token: RwSignal<Option<String>>,
}

impl AuthContext {
    pub fn new() -> Self {
        Self {
            is_authenticated: RwSignal::new(false),
            username: RwSignal::new(String::new()),
            error: RwSignal::new(None),
            token: RwSignal::new(None),
        }
    }

    pub async fn login(&self, user: &str, pass: &str) -> bool {
        if user.trim().is_empty() || pass.is_empty() {
            self.error.set(Some("Ingrese usuario y contraseña".into()));
            return false;
        }

        login_request(user, pass).await.map_or_else(
            |e| {
                self.error.set(Some(e));
                false
            },
            |resp| {
                self.token.set(Some(resp.token));
                self.is_authenticated.set(true);
                self.username.set(resp.user.display_name);
                self.error.set(None);
                true
            },
        )
    }

    pub async fn check_session(&self) -> bool {
        let token = match self.token.get() {
            Some(t) => t,
            None => return false,
        };
        check_session_request(&token).await.map_or(false, |info| {
            self.is_authenticated.set(true);
            self.username.set(info.display_name);
            true
        })
    }

    pub async fn logout(&self) {
        let token = self.token.get();
        if let Some(t) = &token {
            let _ = logout_request(t).await;
        }
        self.is_authenticated.set(false);
        self.username.set(String::new());
        self.error.set(None);
        self.token.set(None);
    }
}

#[derive(Serialize, Deserialize)]
struct LoginResponse {
    token: String,
    user: UserInfo,
}

#[derive(Serialize, Deserialize)]
struct UserInfo {
    username: String,
    display_name: String,
    role: String,
}

// --- WASM HTTP helpers ---

#[cfg(target_arch = "wasm32")]
async fn login_request(user: &str, pass: &str) -> Result<LoginResponse, String> {
    let body = serde_json::json!({"username": user, "password": pass});
    let resp = gloo_net::http::Request::post("/api/auth/login")
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.json::<LoginResponse>()
            .await
            .map_err(|e| format!("Error: {}", e))
    } else {
        Err("Usuario o contraseña incorrectos".into())
    }
}

#[cfg(target_arch = "wasm32")]
async fn check_session_request(token: &str) -> Result<UserInfo, String> {
    let resp = gloo_net::http::Request::get("/api/auth/me")
        .header("x-session-token", token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.json::<UserInfo>()
            .await
            .map_err(|e| format!("Error: {}", e))
    } else {
        Err("Sesión inválida".into())
    }
}

#[cfg(target_arch = "wasm32")]
async fn logout_request(token: &str) -> Result<(), String> {
    let resp = gloo_net::http::Request::post("/api/auth/logout")
        .header("x-session-token", token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.ok() {
        Ok(())
    } else {
        Err("Error al cerrar sesión".into())
    }
}

// --- Server-side stubs ---

#[cfg(not(target_arch = "wasm32"))]
async fn login_request(_user: &str, _pass: &str) -> Result<LoginResponse, String> {
    Err("No disponible en el servidor".into())
}

#[cfg(not(target_arch = "wasm32"))]
async fn check_session_request(_token: &str) -> Result<UserInfo, String> {
    Err("No disponible en el servidor".into())
}

#[cfg(not(target_arch = "wasm32"))]
async fn logout_request(_token: &str) -> Result<(), String> {
    Err("No disponible en el servidor".into())
}

#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    let auth = AuthContext::new();
    provide_context(auth);
    children()
}
