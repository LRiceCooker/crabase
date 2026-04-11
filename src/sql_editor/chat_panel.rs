use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::icons::{IconLoader, IconX};
use crate::tauri;

/// A single chat message.
#[derive(Clone, Debug)]
struct ChatMessage {
    role: &'static str, // "user" or "assistant"
    content: String,
}

/// Side panel for AI chat (Cmd+I). Slides in from the right.
#[component]
pub fn ChatPanel(
    /// Whether the panel is visible
    visible: ReadSignal<bool>,
    /// Callback to close the panel
    on_close: Callback<()>,
    /// Callback to get current SQL editor content
    get_sql: Callback<(), String>,
) -> impl IntoView {
    let (messages, set_messages) = signal(Vec::<ChatMessage>::new());
    let (input, set_input) = signal(String::new());
    let (sending, set_sending) = signal(false);
    let (claude_installed, set_claude_installed) = signal(Option::<bool>::None);
    let input_ref = NodeRef::<leptos::html::Textarea>::new();

    // Check if Claude is installed when panel opens
    Effect::new(move |_| {
        if visible.get() {
            // Reset chat on each open
            set_messages.set(Vec::new());
            set_input.set(String::new());
            set_sending.set(false);

            spawn_local(async move {
                let installed = tauri::check_claude_installed().await;
                set_claude_installed.set(Some(installed));
            });

            // Focus input
            if let Some(el) = input_ref.get() {
                let _ = el.focus();
            }
        }
    });

    let send_message = move || {
        let msg = input.get_untracked().trim().to_string();
        if msg.is_empty() || sending.get_untracked() {
            return;
        }

        // Add user message
        set_messages.update(|msgs| {
            msgs.push(ChatMessage { role: "user", content: msg.clone() });
        });
        set_input.set(String::new());
        set_sending.set(true);

        // Add empty assistant message to stream into
        set_messages.update(|msgs| {
            msgs.push(ChatMessage { role: "assistant", content: String::new() });
        });

        let sql_content = get_sql.run(());

        spawn_local(async move {
            // Get full schema context
            let schema = tauri::get_full_schema_for_chat().await.unwrap_or_default();

            // Build the prompt with context
            let prompt = format!(
                "You are a PostgreSQL expert assistant. The user is working in a SQL editor.\n\n\
                 --- Database Schema ---\n{}\n\n\
                 --- Current SQL in Editor ---\n{}\n\n\
                 --- User Message ---\n{}",
                schema, sql_content, msg
            );

            // Set up listener for streaming responses
            let set_messages_clone = set_messages;
            let _ = tauri::listen_chat_response(move |text| {
                set_messages_clone.update(|msgs| {
                    if let Some(last) = msgs.last_mut() {
                        if last.role == "assistant" {
                            last.content.push_str(&text);
                        }
                    }
                });
            }).await;

            let set_sending_clone = set_sending;
            let _ = tauri::listen_chat_done(move |_| {
                set_sending_clone.set(false);
            }).await;

            // Fire the chat command
            if let Err(e) = tauri::chat_with_claude(&prompt).await {
                set_messages.update(|msgs| {
                    if let Some(last) = msgs.last_mut() {
                        if last.role == "assistant" && last.content.is_empty() {
                            last.content = format!("Error: {}", e);
                        }
                    }
                });
                set_sending.set(false);
            }
        });
    };

    view! {
        <div
            class="shrink-0 border-l border-gray-200 dark:border-zinc-800 bg-white dark:bg-[#111113] flex flex-col h-full"
            class:hidden=move || !visible.get()
            style="width: 384px;"
        >
            // Header
            <div class="h-10 flex items-center justify-between px-3 border-b border-gray-200 dark:border-zinc-800 shrink-0">
                <span class="text-[13px] font-semibold text-gray-900 dark:text-neutral-50">"AI Chat"</span>
                <button
                    class="text-gray-400 dark:text-zinc-500 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 p-1 rounded-md transition-colors duration-100"
                    on:click=move |_| on_close.run(())
                >
                    <IconX class="w-4 h-4" />
                </button>
            </div>

            // Messages area
            <div class="flex-1 overflow-y-auto p-3 flex flex-col gap-3">
                {move || {
                    if claude_installed.get() == Some(false) {
                        return view! {
                            <div class="flex-1 flex items-center justify-center text-[13px] text-gray-500 dark:text-zinc-400 text-center px-4">
                                <p>"Claude Code is not installed. Install it from claude.ai/code to use the AI assistant."</p>
                            </div>
                        }.into_any();
                    }

                    let msgs = messages.get();
                    if msgs.is_empty() {
                        return view! {
                            <div class="flex-1 flex items-center justify-center text-[13px] text-gray-400 dark:text-zinc-500 text-center">
                                <p>"Ask a question about your database or SQL..."</p>
                            </div>
                        }.into_any();
                    }

                    view! {
                        <div class="flex flex-col gap-3">
                            {msgs.iter().map(|msg| {
                                let is_user = msg.role == "user";
                                let bubble_class = if is_user {
                                    "self-end bg-indigo-500 dark:bg-indigo-600 text-white rounded-lg px-3 py-2 text-[13px] max-w-[85%]"
                                } else {
                                    "self-start bg-gray-100 dark:bg-zinc-800 text-gray-900 dark:text-neutral-50 rounded-lg px-3 py-2 text-[13px] max-w-[85%] font-mono whitespace-pre-wrap"
                                };
                                let content = msg.content.clone();
                                view! {
                                    <div class=bubble_class>{content}</div>
                                }
                            }).collect::<Vec<_>>()}
                            {move || if sending.get() {
                                Some(view! {
                                    <div class="self-start flex items-center gap-1.5 text-[11px] text-gray-400 dark:text-zinc-500">
                                        <IconLoader class="w-3 h-3 animate-spin" />
                                        "Thinking..."
                                    </div>
                                })
                            } else {
                                None
                            }}
                        </div>
                    }.into_any()
                }}
            </div>

            // Input area
            <div class="border-t border-gray-200 dark:border-zinc-800 p-3 shrink-0">
                <div class="flex gap-2">
                    <textarea
                        node_ref=input_ref
                        class="flex-1 bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-700 rounded-md px-3 py-2 text-[13px] text-gray-900 dark:text-neutral-50 placeholder-gray-400 dark:placeholder-zinc-500 resize-none focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:focus:ring-indigo-500/60 focus:border-indigo-500"
                        rows="2"
                        placeholder="Ask about your database..."
                        disabled=move || claude_installed.get() == Some(false) || sending.get()
                        prop:value=move || input.get()
                        on:input=move |ev| set_input.set(event_target_value(&ev))
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            if ev.key() == "Enter" && !ev.shift_key() {
                                ev.prevent_default();
                                send_message();
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
