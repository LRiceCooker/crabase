use leptos::prelude::*;

/// SQL text editor with line numbers and monospace font.
#[component]
pub fn SqlEditor(
    sql: RwSignal<String>,
) -> impl IntoView {
    let line_count = move || {
        let text = sql.get();
        text.lines().count().max(1)
    };

    view! {
        <div class="flex flex-1 overflow-hidden">
            // Line number gutter
            <div class="bg-gray-50 text-gray-400 text-right pr-2 pl-2 select-none border-r border-gray-100 font-mono text-[13px] leading-relaxed pt-2 overflow-hidden shrink-0">
                {move || {
                    (1..=line_count()).map(|n| {
                        view! { <div>{n}</div> }
                    }).collect::<Vec<_>>()
                }}
            </div>
            // Editor textarea
            <textarea
                class="flex-1 bg-white font-mono text-[13px] leading-relaxed p-2 resize-none focus:outline-none text-gray-900"
                spellcheck="false"
                autocomplete="off"
                prop:value=move || sql.get()
                on:input=move |ev| sql.set(event_target_value(&ev))
                placeholder="Write your SQL query here..."
            />
        </div>
    }
}
