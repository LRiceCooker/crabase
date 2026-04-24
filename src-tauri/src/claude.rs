use std::io::BufRead;
use tauri::Emitter;

/// Get the user's default shell, falling back to /bin/zsh.
pub fn user_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
}

/// Check if claude is available via user's login shell.
/// Uses -l (login) AND -i (interactive) to ensure ~/.zshrc is sourced,
/// which is where tools like mise/nvm/volta configure PATH.
pub fn is_installed() -> bool {
    let shell = user_shell();
    std::process::Command::new(&shell)
        .args(["-l", "-i", "-c", "command -v claude"])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run claude CLI with --print mode and stream output line-by-line via Tauri events.
/// Emits "chat-response" for each line and "chat-done" when finished.
pub fn run_streaming(prompt: &str, app_handle: &tauri::AppHandle) -> Result<(), String> {
    // Escape single quotes in the prompt for shell
    let escaped_prompt = prompt.replace('\'', "'\\''");
    let shell_cmd = format!(
        "claude --print -p '{}' --dangerously-skip-permissions",
        escaped_prompt
    );
    let shell = user_shell();

    let mut child = std::process::Command::new(&shell)
        .args(["-l", "-i", "-c", &shell_cmd])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn claude: {e}"))?;

    if let Some(stdout) = child.stdout.take() {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            let Ok(line) = line else { continue };
            let _ = app_handle.emit("chat-response", format!("{line}\n"));
        }
    }

    let _ = child.wait();
    let _ = app_handle.emit("chat-done", ());
    Ok(())
}
