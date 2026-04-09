use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn ConnectionForm(
    form_host: RwSignal<String>,
    form_port: RwSignal<String>,
    form_user: RwSignal<String>,
    form_password: RwSignal<String>,
    form_dbname: RwSignal<String>,
    form_schema: RwSignal<String>,
    form_ssl: RwSignal<bool>,
    available_schemas: ReadSignal<Vec<String>>,
    loading_schemas: ReadSignal<bool>,
    error_message: ReadSignal<Option<String>>,
    connecting: ReadSignal<bool>,
    on_connect: Callback<()>,
    on_back: Callback<()>,
) -> impl IntoView {
    view! {
        <main class="min-h-screen bg-base-200 flex items-center justify-center p-4">
            <div class="card bg-base-100 shadow-xl w-full max-w-lg">
                <div class="card-body">
                    <h1 class="text-2xl font-bold text-center">"crabase"</h1>
                    <p class="text-base-content/70 text-center mb-4">"Connection details"</p>

                    <div class="grid grid-cols-2 gap-3">
                        // Host
                        <div class="form-control col-span-2 sm:col-span-1">
                            <label class="label"><span class="label-text">"Host"</span></label>
                            <input
                                type="text"
                                class="input input-bordered w-full"
                                prop:value=move || form_host.get()
                                on:input=move |ev| form_host.set(event_target_value(&ev))
                            />
                        </div>
                        // Port
                        <div class="form-control col-span-2 sm:col-span-1">
                            <label class="label"><span class="label-text">"Port"</span></label>
                            <input
                                type="text"
                                class="input input-bordered w-full"
                                prop:value=move || form_port.get()
                                on:input=move |ev| form_port.set(event_target_value(&ev))
                            />
                        </div>
                        // User
                        <div class="form-control col-span-2 sm:col-span-1">
                            <label class="label"><span class="label-text">"User"</span></label>
                            <input
                                type="text"
                                class="input input-bordered w-full"
                                prop:value=move || form_user.get()
                                on:input=move |ev| form_user.set(event_target_value(&ev))
                            />
                        </div>
                        // Password
                        <div class="form-control col-span-2 sm:col-span-1">
                            <label class="label"><span class="label-text">"Password"</span></label>
                            <input
                                type="password"
                                class="input input-bordered w-full"
                                prop:value=move || form_password.get()
                                on:input=move |ev| form_password.set(event_target_value(&ev))
                            />
                        </div>
                        // Database
                        <div class="form-control col-span-2 sm:col-span-1">
                            <label class="label"><span class="label-text">"Database"</span></label>
                            <input
                                type="text"
                                class="input input-bordered w-full"
                                prop:value=move || form_dbname.get()
                                on:input=move |ev| form_dbname.set(event_target_value(&ev))
                            />
                        </div>
                        // Schema
                        <div class="form-control col-span-2 sm:col-span-1">
                            <label class="label"><span class="label-text">"Schema"</span></label>
                            {move || {
                                let schemas = available_schemas.get();
                                if schemas.is_empty() && loading_schemas.get() {
                                    view! {
                                        <select class="select select-bordered w-full" disabled=true>
                                            <option>"Loading schemas..."</option>
                                        </select>
                                    }.into_any()
                                } else if schemas.is_empty() {
                                    view! {
                                        <select class="select select-bordered w-full">
                                            <option selected=true>"public"</option>
                                        </select>
                                    }.into_any()
                                } else {
                                    let current = form_schema.get();
                                    view! {
                                        <select
                                            class="select select-bordered w-full"
                                            on:change=move |ev| form_schema.set(event_target_value(&ev))
                                        >
                                            {schemas.into_iter().map(|s| {
                                                let selected = s == current;
                                                let s2 = s.clone();
                                                view! { <option value={s} selected=selected>{s2}</option> }
                                            }).collect::<Vec<_>>()}
                                        </select>
                                    }.into_any()
                                }
                            }}
                        </div>
                        // SSL toggle
                        <div class="form-control col-span-2">
                            <label class="label cursor-pointer justify-start gap-3">
                                <input
                                    type="checkbox"
                                    class="toggle toggle-primary"
                                    prop:checked=move || form_ssl.get()
                                    on:change=move |ev| {
                                        let checked = event_target_checked(&ev);
                                        form_ssl.set(checked);
                                    }
                                />
                                <span class="label-text">"SSL (require)"</span>
                            </label>
                        </div>
                    </div>

                    {move || error_message.get().map(|msg| view! {
                        <div class="alert alert-error mt-3">
                            <span>{msg}</span>
                        </div>
                    })}

                    <div class="card-actions mt-4 flex gap-2">
                        <button
                            class="btn btn-ghost flex-1"
                            on:click=move |_| on_back.run(())
                        >
                            "Back"
                        </button>
                        <button
                            class="btn btn-primary flex-1"
                            disabled=move || connecting.get()
                            on:click=move |_| on_connect.run(())
                        >
                            {move || if connecting.get() {
                                "Connexion..."
                            } else {
                                "Se connecter"
                            }}
                        </button>
                    </div>
                </div>
            </div>
        </main>
    }
}

fn event_target_checked(ev: &leptos::ev::Event) -> bool {
    ev.target()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>()
        .checked()
}
