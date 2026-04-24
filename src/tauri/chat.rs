use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use super::{invoke, listen};

/// Checks whether the Claude CLI is installed on the system.
pub async fn check_claude_installed() -> bool {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap_or(JsValue::UNDEFINED);
    match invoke("check_claude_installed", args).await {
        Ok(val) => val.as_bool().unwrap_or(false),
        Err(_) => false,
    }
}

/// Sends a prompt to Claude for AI-assisted chat.
pub async fn chat_with_claude(prompt: &str) -> Result<(), String> {
    #[derive(Serialize)]
    struct Args<'a> {
        prompt: &'a str,
    }
    let args = serde_wasm_bindgen::to_value(&Args { prompt }).map_err(|e| e.to_string())?;
    invoke("chat_with_claude", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Chat failed".to_string()))?;
    Ok(())
}

/// Gets the full database schema text for use as Claude context.
pub async fn get_full_schema_for_chat() -> Result<String, String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).map_err(|e| e.to_string())?;
    let result = invoke("get_full_schema_for_chat", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to get schema".to_string()))?;
    Ok(result.as_string().unwrap_or_default())
}

/// Listens for streaming chat response events. Returns a JS function to call to unlisten.
pub async fn listen_chat_response(
    callback: impl Fn(String) + 'static,
) -> Result<js_sys::Function, String> {
    let closure = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
        if let Ok(payload) = js_sys::Reflect::get(&event, &JsValue::from_str("payload")) {
            if let Some(text) = payload.as_string() {
                callback(text);
            }
        }
    });

    let unlisten = listen("chat-response", closure.as_ref())
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to listen".to_string()))?;

    closure.forget();
    Ok(unlisten.unchecked_into::<js_sys::Function>())
}

/// Listens for chat completion events. Returns a JS function to call to unlisten.
pub async fn listen_chat_done(
    callback: impl Fn(()) + 'static,
) -> Result<js_sys::Function, String> {
    let closure = Closure::<dyn FnMut(JsValue)>::new(move |_event: JsValue| {
        callback(());
    });

    let unlisten = listen("chat-done", closure.as_ref())
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to listen".to_string()))?;

    closure.forget();
    Ok(unlisten.unchecked_into::<js_sys::Function>())
}
