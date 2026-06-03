use leptos::prelude::*;

#[allow(dead_code)]
#[component]
pub fn Card(title: &'static str, children: ChildrenFn) -> impl IntoView {
    view! {
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
            <h2 class="text-lg font-semibold text-gray-800 mb-4">{title}</h2>
            {children()}
        </div>
    }
}
