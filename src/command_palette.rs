use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn CommandPalette(
    show: ReadSignal<bool>,
    set_show: WriteSignal<bool>,
) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let commands: Vec<(&'static str, &'static str)> = vec![
        ("Restore Backup", "Restore a .tar.gz PostgreSQL backup"),
    ];

    // Focus input when palette opens, clear query when it closes
    Effect::new(move |_| {
        if show.get() {
            if let Some(el) = input_ref.get() {
                let _ = el.focus();
            }
        } else {
            set_query.set(String::new());
        }
    });

    move || {
        if show.get() {
            let q = query.get().to_lowercase();
            let filtered: Vec<_> = commands
                .iter()
                .filter(|(name, _)| q.is_empty() || name.to_lowercase().contains(&q))
                .collect();

            Some(view! {
                <div class="fixed inset-0 z-50 flex justify-center items-start pt-[15vh]">
                    // Backdrop
                    <div
                        class="absolute inset-0 bg-black/50"
                        on:click=move |_| set_show.set(false)
                    ></div>
                    // Palette
                    <div class="relative z-10 w-full max-w-lg bg-base-100 rounded-lg shadow-2xl overflow-hidden">
                        <div class="p-3 border-b border-base-300">
                            <input
                                type="text"
                                node_ref=input_ref
                                placeholder="Type a command..."
                                class="input input-bordered w-full"
                                prop:value=move || query.get()
                                on:input=move |ev| set_query.set(event_target_value(&ev))
                                on:keydown=move |ev| {
                                    let ev: &web_sys::KeyboardEvent = ev.unchecked_ref();
                                    if ev.key() == "Escape" {
                                        set_show.set(false);
                                    }
                                }
                            />
                        </div>
                        <ul class="menu menu-sm p-2 max-h-64 overflow-y-auto">
                            {filtered.into_iter().map(|(name, desc)| {
                                view! {
                                    <li>
                                        <a class="flex flex-col items-start py-2">
                                            <span class="font-medium">{name.to_string()}</span>
                                            <span class="text-xs text-base-content/50">{desc.to_string()}</span>
                                        </a>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
                    </div>
                </div>
            })
        } else {
            None
        }
    }
}
