use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::connection::connection_form::ConnectionForm;
use crate::connection::connection_screen::ConnectionScreen;
use crate::main_layout::MainLayout;
use crate::tauri::{self, build_connection_string_js, ConnectionInfo, SavedConnection};
use crate::theme;

#[component]
pub fn App() -> impl IntoView {
    // Initialize theme system (provides ThemeCtx via Leptos context)
    theme::provide_theme();

    // 3 states: "input" -> "form" -> "connected"
    let (screen, set_screen) = signal(String::from("input"));
    let (connection_string, set_connection_string) = signal(String::new());
    let (error_message, set_error_message) = signal(Option::<String>::None);
    let (parsing, set_parsing) = signal(false);
    let (connecting, set_connecting) = signal(false);

    // Form fields (populated after parsing)
    let form_host = RwSignal::new(String::new());
    let form_port = RwSignal::new(String::from("5432"));
    let form_user = RwSignal::new(String::new());
    let form_password = RwSignal::new(String::new());
    let form_dbname = RwSignal::new(String::new());
    let form_schema = RwSignal::new(String::from("public"));
    let form_ssl = RwSignal::new(false);
    let (available_schemas, set_available_schemas) = signal(Vec::<String>::new());
    let (loading_schemas, set_loading_schemas) = signal(false);
    let save_connection = RwSignal::new(false);
    let save_name = RwSignal::new(String::new());

    // Step 1 -> Step 2: parse connection string, fetch schemas, show form
    let on_parse = Callback::new(move |_: ()| {
        let cs = connection_string.get();
        set_error_message.set(None);
        set_parsing.set(true);
        spawn_local(async move {
            match tauri::parse_connection_string(&cs).await {
                Ok(info) => {
                    form_host.set(info.host);
                    form_port.set(info.port.to_string());
                    form_user.set(info.user);
                    form_password.set(info.password);
                    form_dbname.set(info.dbname);
                    form_schema.set(info.schema);
                    form_ssl.set(info.sslmode == "require");
                    set_screen.set("form".to_string());

                    // Fetch available schemas in background
                    let cs2 = cs.clone();
                    set_loading_schemas.set(true);
                    match tauri::list_schemas(&cs2).await {
                        Ok(schemas) => {
                            if !schemas.is_empty() {
                                set_available_schemas.set(schemas);
                            }
                        }
                        Err(_) => {
                            // Silently fail — user can still type manually
                        }
                    }
                    set_loading_schemas.set(false);
                }
                Err(e) => set_error_message.set(Some(e)),
            }
            set_parsing.set(false);
        });
    });

    // Step 2 -> Step 3: connect with form fields
    let on_connect = Callback::new(move |_: ()| {
        let info = ConnectionInfo {
            host: form_host.get(),
            port: form_port.get().parse().unwrap_or(5432),
            user: form_user.get(),
            password: form_password.get(),
            dbname: form_dbname.get(),
            schema: form_schema.get(),
            sslmode: if form_ssl.get() { "require".to_string() } else { "disable".to_string() },
        };
        let should_save = save_connection.get();
        let name = save_name.get();
        set_error_message.set(None);
        set_connecting.set(true);
        spawn_local(async move {
            match tauri::connect_db(&info).await {
                Ok(_) => {
                    if should_save && !name.trim().is_empty() {
                        let _ = tauri::save_connection(&name, &info).await;
                    }
                    set_screen.set("connected".to_string());
                }
                Err(e) => set_error_message.set(Some(e)),
            }
            set_connecting.set(false);
        });
    });

    // Select a saved connection: fill form fields and jump to form screen
    let on_select_saved = Callback::new(move |saved: SavedConnection| {
        let info = saved.info;
        form_host.set(info.host.clone());
        form_port.set(info.port.to_string());
        form_user.set(info.user.clone());
        form_password.set(info.password.clone());
        form_dbname.set(info.dbname.clone());
        form_schema.set(info.schema.clone());
        form_ssl.set(info.sslmode == "require");
        set_error_message.set(None);
        set_screen.set("form".to_string());

        // Fetch schemas in background
        let cs = build_connection_string_js(&info);
        set_connection_string.set(cs.clone());
        set_loading_schemas.set(true);
        spawn_local(async move {
            if let Ok(schemas) = tauri::list_schemas(&cs).await {
                if !schemas.is_empty() {
                    set_available_schemas.set(schemas);
                }
            }
            set_loading_schemas.set(false);
        });
    });

    // Back to step 1
    let on_back = Callback::new(move |_: ()| {
        set_error_message.set(None);
        set_screen.set("input".to_string());
    });

    move || {
        let current_screen = screen.get();
        match current_screen.as_str() {
            "connected" => {
                let on_disconnect = Callback::new(move |_: ()| {
                    set_screen.set("input".to_string());
                });
                view! { <MainLayout on_disconnect=on_disconnect /> }.into_any()
            }
            "form" => {
                view! {
                    <ConnectionForm
                        form_host=form_host
                        form_port=form_port
                        form_user=form_user
                        form_password=form_password
                        form_dbname=form_dbname
                        form_schema=form_schema
                        form_ssl=form_ssl
                        save_connection=save_connection
                        save_name=save_name
                        available_schemas=available_schemas
                        loading_schemas=loading_schemas
                        error_message=error_message
                        connecting=connecting
                        on_connect=on_connect
                        on_back=on_back
                    />
                }.into_any()
            }
            _ => {
                view! {
                    <ConnectionScreen
                        connection_string=connection_string
                        set_connection_string=set_connection_string
                        error_message=error_message
                        parsing=parsing
                        on_parse=on_parse
                        on_select_saved=on_select_saved
                    />
                }.into_any()
            }
        }
    }
}
