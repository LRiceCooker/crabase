use tauri::Manager;

/// Decode a PNG from embedded bytes and set it as the app icon on the main window.
pub fn set_icon(is_dark: bool, app_handle: &tauri::AppHandle) -> Result<(), String> {
    let png_bytes: &[u8] = if is_dark {
        include_bytes!("../icons/dark/icon.png")
    } else {
        include_bytes!("../icons/light/icon.png")
    };

    // Decode PNG to RGBA using the png crate
    let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("PNG decode error: {}", e))?;
    let info = reader.info().clone();
    let mut buf = vec![0u8; info.raw_bytes()];
    reader
        .next_frame(&mut buf)
        .map_err(|e| format!("PNG frame error: {}", e))?;

    let icon = tauri::image::Image::new_owned(buf, info.width, info.height);

    // Set icon on the main window
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.set_icon(icon);
    }
    Ok(())
}
