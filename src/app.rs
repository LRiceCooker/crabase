use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="min-h-screen bg-base-200 flex items-center justify-center">
            <div class="card bg-base-100 shadow-xl p-8">
                <h1 class="text-3xl font-bold">"crabase"</h1>
                <p class="text-base-content/70">"PostgreSQL Desktop Client"</p>
            </div>
        </main>
    }
}
