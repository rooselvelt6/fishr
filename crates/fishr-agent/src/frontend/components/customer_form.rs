use leptos::prelude::*;

#[component]
pub fn CustomerForm(
    #[prop(optional)] _show: RwSignal<bool>,
    #[prop(optional)] _on_save: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <p>"Formulario de registro de cliente."</p>
        </div>
    }
}
