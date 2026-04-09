use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::icons::{IconArrowLeft, IconDatabase, IconAlertTriangle, IconLoader};

#[component]
pub fn ConnectionForm(
    form_host: RwSignal<String>,
    form_port: RwSignal<String>,
    form_user: RwSignal<String>,
    form_password: RwSignal<String>,
    form_dbname: RwSignal<String>,
    form_schema: RwSignal<String>,
    form_ssl: RwSignal<bool>,
    save_connection: RwSignal<bool>,
    save_name: RwSignal<String>,
    available_schemas: ReadSignal<Vec<String>>,
    loading_schemas: ReadSignal<bool>,
    error_message: ReadSignal<Option<String>>,
    connecting: ReadSignal<bool>,
    on_connect: Callback<()>,
    on_back: Callback<()>,
) -> impl IntoView {
    let input_class = "bg-white border border-gray-200 rounded-md px-3 py-1.5 text-[13px] w-full focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-colors duration-100";
    let select_class = "bg-white border border-gray-200 rounded-md px-3 py-1.5 text-[13px] w-full focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-colors duration-100";
    let label_class = "text-[13px] font-normal text-gray-700";

    view! {
        <main class="min-h-screen bg-gray-50 flex items-center justify-center p-4">
            <div class="bg-white rounded-lg shadow-xl border border-gray-200 w-full max-w-lg">
                <div class="px-6 py-8">
                    <div class="flex flex-col items-center gap-1 mb-6">
                        <div class="flex items-center gap-2">
                            <IconDatabase class="w-5 h-5 text-indigo-500" />
                            <h1 class="text-base font-semibold text-gray-900">"crabase"</h1>
                        </div>
                        <p class="text-[13px] text-gray-500">"Connection details"</p>
                    </div>

                    <div class="grid grid-cols-2 gap-3">
                        // Host
                        <div class="flex flex-col gap-1.5 col-span-2 sm:col-span-1">
                            <label class=label_class>"Host"</label>
                            <input
                                type="text"
                                class=input_class
                                prop:value=move || form_host.get()
                                on:input=move |ev| form_host.set(event_target_value(&ev))
                            />
                        </div>
                        // Port
                        <div class="flex flex-col gap-1.5 col-span-2 sm:col-span-1">
                            <label class=label_class>"Port"</label>
                            <input
                                type="text"
                                class=input_class
                                prop:value=move || form_port.get()
                                on:input=move |ev| form_port.set(event_target_value(&ev))
                            />
                        </div>
                        // User
                        <div class="flex flex-col gap-1.5 col-span-2 sm:col-span-1">
                            <label class=label_class>"User"</label>
                            <input
                                type="text"
                                class=input_class
                                prop:value=move || form_user.get()
                                on:input=move |ev| form_user.set(event_target_value(&ev))
                            />
                        </div>
                        // Password
                        <div class="flex flex-col gap-1.5 col-span-2 sm:col-span-1">
                            <label class=label_class>"Password"</label>
                            <input
                                type="password"
                                class=input_class
                                prop:value=move || form_password.get()
                                on:input=move |ev| form_password.set(event_target_value(&ev))
                            />
                        </div>
                        // Database
                        <div class="flex flex-col gap-1.5 col-span-2 sm:col-span-1">
                            <label class=label_class>"Database"</label>
                            <input
                                type="text"
                                class=input_class
                                prop:value=move || form_dbname.get()
                                on:input=move |ev| form_dbname.set(event_target_value(&ev))
                            />
                        </div>
                        // Schema
                        <div class="flex flex-col gap-1.5 col-span-2 sm:col-span-1">
                            <label class=label_class>"Schema"</label>
                            {move || {
                                let schemas = available_schemas.get();
                                if schemas.is_empty() && loading_schemas.get() {
                                    view! {
                                        <select class=select_class disabled=true>
                                            <option>"Loading schemas..."</option>
                                        </select>
                                    }.into_any()
                                } else if schemas.is_empty() {
                                    view! {
                                        <select class=select_class>
                                            <option selected=true>"public"</option>
                                        </select>
                                    }.into_any()
                                } else {
                                    let current = form_schema.get();
                                    view! {
                                        <select
                                            class=select_class
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
                        <div class="col-span-2 flex items-center gap-3 mt-1">
                            <input
                                type="checkbox"
                                class="w-4 h-4 rounded border-gray-300 text-indigo-500 focus:ring-indigo-500/20"
                                prop:checked=move || form_ssl.get()
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    form_ssl.set(checked);
                                }
                            />
                            <label class="text-[13px] text-gray-700 cursor-pointer">"SSL (require)"</label>
                        </div>
                        // Save connection toggle
                        <div class="col-span-2 flex items-center gap-3 mt-1">
                            <input
                                type="checkbox"
                                class="w-4 h-4 rounded border-gray-300 text-indigo-500 focus:ring-indigo-500/20"
                                prop:checked=move || save_connection.get()
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    save_connection.set(checked);
                                }
                            />
                            <label class="text-[13px] text-gray-700 cursor-pointer">"Save connection"</label>
                        </div>
                        // Connection name (shown when save is checked)
                        {move || save_connection.get().then(|| view! {
                            <div class="col-span-2 flex flex-col gap-1.5">
                                <label class=label_class>"Connection name"</label>
                                <input
                                    type="text"
                                    class=input_class
                                    placeholder="e.g. Production DB"
                                    prop:value=move || save_name.get()
                                    on:input=move |ev| save_name.set(event_target_value(&ev))
                                />
                            </div>
                        })}
                    </div>

                    {move || error_message.get().map(|msg| view! {
                        <div class="flex items-center gap-2 mt-3 px-3 py-2 bg-red-50 border border-red-200 rounded-md">
                            <IconAlertTriangle class="w-4 h-4 text-red-500 shrink-0" />
                            <span class="text-[13px] text-red-700">{msg}</span>
                        </div>
                    })}

                    <div class="mt-4 flex gap-2">
                        <button
                            class="text-gray-500 hover:bg-gray-100 hover:text-gray-900 text-[13px] font-medium px-3 py-1.5 rounded-md flex-1 transition-colors duration-100 flex items-center justify-center gap-1.5"
                            on:click=move |_| on_back.run(())
                        >
                            <IconArrowLeft class="w-4 h-4" />
                            "Back"
                        </button>
                        <button
                            class="bg-indigo-500 hover:bg-indigo-600 text-white text-[13px] font-medium px-3 py-1.5 rounded-md flex-1 transition-colors duration-100 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=move || connecting.get()
                            on:click=move |_| on_connect.run(())
                        >
                            {move || if connecting.get() {
                                view! {
                                    <span class="flex items-center justify-center gap-2">
                                        <IconLoader class="w-4 h-4 animate-spin" />
                                        "Connecting..."
                                    </span>
                                }.into_any()
                            } else {
                                view! { <span>"Connect"</span> }.into_any()
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
