/// Log an error message to the browser console.
pub fn log_error(msg: &str) {
    web_sys::console::error_1(&msg.into());
}
