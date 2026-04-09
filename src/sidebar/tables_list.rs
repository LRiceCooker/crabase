use leptos::prelude::*;

#[component]
pub fn TablesList(
    tables: ReadSignal<Vec<String>>,
) -> impl IntoView {
    view! {
        <div class="p-3">
            <h2 class="text-sm font-semibold text-base-content/50 uppercase tracking-wider mb-2">"Tables"</h2>
            <ul class="menu menu-sm">
                {move || tables.get().into_iter().map(|name| view! {
                    <li><a>{name}</a></li>
                }).collect::<Vec<_>>()}
            </ul>
            {move || if tables.get().is_empty() {
                Some(view! {
                    <p class="text-sm text-base-content/40 italic">"No tables found"</p>
                })
            } else {
                None
            }}
        </div>
    }
}
