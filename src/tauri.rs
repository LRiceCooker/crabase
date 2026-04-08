use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

pub async fn connect_db(connection_string: &str) -> Result<String, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        connection_string: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args { connection_string })
        .map_err(|e| e.to_string())?;

    let result = invoke("connect_db", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Connection failed".to_string()))?;

    result
        .as_string()
        .ok_or_else(|| "Invalid response from backend".to_string())
}
