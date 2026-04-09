use leptos::prelude::*;

#[component]
pub fn ConnectionScreen(
    connection_string: ReadSignal<String>,
    set_connection_string: WriteSignal<String>,
    error_message: ReadSignal<Option<String>>,
    parsing: ReadSignal<bool>,
    on_parse: Callback<()>,
) -> impl IntoView {
    view! {
        <main class="min-h-screen bg-base-200 flex items-center justify-center p-4">
            <div class="card bg-base-100 shadow-xl w-full max-w-md">
                <div class="card-body">
                    <h1 class="text-3xl font-bold text-center">"crabase"</h1>
                    <p class="text-base-content/70 text-center mb-6">"PostgreSQL Desktop Client"</p>

                    <div class="form-control w-full">
                        <label class="label">
                            <span class="label-text">"Connection string"</span>
                        </label>
                        <input
                            type="text"
                            placeholder="postgresql://user:password@host:port/dbname"
                            class="input input-bordered w-full"
                            prop:value=move || connection_string.get()
                            on:input=move |ev| {
                                set_connection_string.set(event_target_value(&ev));
                            }
                        />
                    </div>

                    {move || error_message.get().map(|msg| view! {
                        <div class="alert alert-error mt-4">
                            <span>{msg}</span>
                        </div>
                    })}

                    <div class="card-actions mt-4">
                        <button
                            class="btn btn-primary w-full"
                            disabled=move || connection_string.with(|s| s.is_empty()) || parsing.get()
                            on:click=move |_| on_parse.run(())
                        >
                            {move || if parsing.get() {
                                "Parsing..."
                            } else {
                                "Suivant"
                            }}
                        </button>
                    </div>
                </div>
            </div>
        </main>
    }
}
