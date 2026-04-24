use std::collections::HashMap;

use leptos::prelude::*;

/// All configurable shortcut actions in the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    /// Open command palette (global)
    CommandPalette,
    /// Open table/query finder (global)
    TableFinder,
    /// Save current context (dirty table changes or SQL query)
    Save,
    // Editor shortcuts (handled by CodeMirror, shown in Settings)
    ToggleComment,
    ToggleBlockComment,
    Find,
    FindReplace,
    SelectNextOccurrence,
    CopyLineDown,
    MoveLineUp,
    MoveLineDown,
    DeleteLine,
    GoToLine,
    IndentMore,
    IndentLess,
    // AI
    OpenAIChat,
}

impl ShortcutAction {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::CommandPalette => "Command Palette",
            Self::TableFinder => "Table / Query Finder",
            Self::Save => "Save",
            Self::ToggleComment => "Toggle Line Comment",
            Self::ToggleBlockComment => "Toggle Block Comment",
            Self::Find => "Find",
            Self::FindReplace => "Find & Replace",
            Self::SelectNextOccurrence => "Select Next Occurrence",
            Self::CopyLineDown => "Copy Line Down",
            Self::MoveLineUp => "Move Line Up",
            Self::MoveLineDown => "Move Line Down",
            Self::DeleteLine => "Delete Line",
            Self::GoToLine => "Go to Line",
            Self::IndentMore => "Indent",
            Self::IndentLess => "Outdent",
            Self::OpenAIChat => "Open AI Chat",
        }
    }

    /// Category for grouping in settings UI.
    pub fn category(self) -> &'static str {
        match self {
            Self::CommandPalette | Self::TableFinder | Self::Save => "General",
            Self::OpenAIChat => "AI",
            _ => "Editor",
        }
    }

    /// All actions in display order.
    pub fn all() -> &'static [ShortcutAction] {
        &[
            Self::CommandPalette,
            Self::TableFinder,
            Self::Save,
            Self::ToggleComment,
            Self::ToggleBlockComment,
            Self::Find,
            Self::FindReplace,
            Self::SelectNextOccurrence,
            Self::CopyLineDown,
            Self::MoveLineUp,
            Self::MoveLineDown,
            Self::DeleteLine,
            Self::GoToLine,
            Self::IndentMore,
            Self::IndentLess,
            Self::OpenAIChat,
        ]
    }
}

/// A key binding: modifier flags + physical key code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding {
    /// Whether Cmd (macOS) or Ctrl (other) must be held.
    pub cmd_or_ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    /// Physical key code from `KeyboardEvent.code()`, e.g. "KeyP", "Slash".
    pub code: String,
}

impl KeyBinding {
    pub fn new(cmd_or_ctrl: bool, shift: bool, alt: bool, code: &str) -> Self {
        Self {
            cmd_or_ctrl,
            shift,
            alt,
            code: code.to_string(),
        }
    }

    /// Check if a keyboard event matches this binding (strict: unspecified modifiers must be off).
    pub fn matches(&self, ev: &web_sys::KeyboardEvent) -> bool {
        let cmd_ok = if self.cmd_or_ctrl {
            ev.meta_key() || ev.ctrl_key()
        } else {
            !ev.meta_key() && !ev.ctrl_key()
        };
        cmd_ok
            && self.shift == ev.shift_key()
            && self.alt == ev.alt_key()
            && self.code == ev.code()
    }

    /// Human-readable display string, e.g. "⌘⇧P".
    pub fn display(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.cmd_or_ctrl {
            parts.push("\u{2318}".to_string()); // ⌘
        }
        if self.shift {
            parts.push("\u{21E7}".to_string()); // ⇧
        }
        if self.alt {
            parts.push("\u{2325}".to_string()); // ⌥
        }
        parts.push(code_to_display(&self.code));
        parts.join("")
    }
}

/// Convert a `KeyboardEvent.code()` value to a short human-readable label.
fn code_to_display(code: &str) -> String {
    if let Some(letter) = code.strip_prefix("Key") {
        return letter.to_string();
    }
    if let Some(digit) = code.strip_prefix("Digit") {
        return digit.to_string();
    }
    match code {
        "Slash" => "/",
        "Backslash" => "\\",
        "BracketLeft" => "[",
        "BracketRight" => "]",
        "Comma" => ",",
        "Period" => ".",
        "Semicolon" => ";",
        "Quote" => "'",
        "Backquote" => "`",
        "Minus" => "-",
        "Equal" => "=",
        "Enter" => "\u{21B5}",   // ↵
        "Escape" => "Esc",
        "Space" => "Space",
        "Tab" => "Tab",
        "ArrowUp" => "\u{2191}",    // ↑
        "ArrowDown" => "\u{2193}",  // ↓
        "ArrowLeft" => "\u{2190}",  // ←
        "ArrowRight" => "\u{2192}", // →
        "Backspace" => "\u{232B}",  // ⌫
        "Delete" => "\u{2326}",     // ⌦
        other => return other.to_string(),
    }
    .to_string()
}

/// Build the default key bindings for all actions.
fn default_bindings() -> HashMap<ShortcutAction, KeyBinding> {
    [
        // General
        (ShortcutAction::CommandPalette, KeyBinding::new(true, true, false, "KeyP")),
        (ShortcutAction::TableFinder, KeyBinding::new(true, false, false, "KeyP")),
        (ShortcutAction::Save, KeyBinding::new(true, false, false, "KeyS")),
        // Editor
        (ShortcutAction::ToggleComment, KeyBinding::new(true, false, false, "Slash")),
        (ShortcutAction::ToggleBlockComment, KeyBinding::new(true, true, false, "KeyA")),
        (ShortcutAction::Find, KeyBinding::new(true, false, false, "KeyF")),
        (ShortcutAction::FindReplace, KeyBinding::new(true, false, true, "KeyF")),
        (ShortcutAction::SelectNextOccurrence, KeyBinding::new(true, false, false, "KeyD")),
        (ShortcutAction::CopyLineDown, KeyBinding::new(true, true, false, "KeyD")),
        (ShortcutAction::MoveLineUp, KeyBinding::new(false, false, true, "ArrowUp")),
        (ShortcutAction::MoveLineDown, KeyBinding::new(false, false, true, "ArrowDown")),
        (ShortcutAction::DeleteLine, KeyBinding::new(true, true, false, "KeyK")),
        (ShortcutAction::GoToLine, KeyBinding::new(true, false, false, "KeyG")),
        (ShortcutAction::IndentMore, KeyBinding::new(false, false, false, "Tab")),
        (ShortcutAction::IndentLess, KeyBinding::new(false, true, false, "Tab")),
        // AI
        (ShortcutAction::OpenAIChat, KeyBinding::new(true, false, false, "KeyI")),
    ]
    .into_iter()
    .collect()
}

/// Shortcuts context provided at the app root.
/// Holds the current bindings (customizable) and provides matching helpers.
#[derive(Clone, Copy)]
pub struct ShortcutsCtx {
    bindings: RwSignal<HashMap<ShortcutAction, KeyBinding>>,
}

impl ShortcutsCtx {
    /// Check if a keyboard event matches the binding for `action`.
    pub fn matches(&self, action: ShortcutAction, ev: &web_sys::KeyboardEvent) -> bool {
        self.bindings
            .read_untracked()
            .get(&action)
            .map(|b| b.matches(ev))
            .unwrap_or(false)
    }

    /// Get a clone of the current binding for an action (if any).
    pub fn get_binding(&self, action: ShortcutAction) -> Option<KeyBinding> {
        self.bindings.read_untracked().get(&action).cloned()
    }

    /// Get the display string for an action's binding, or empty string if unbound.
    pub fn display(&self, action: ShortcutAction) -> String {
        self.get_binding(action)
            .map(|b| b.display())
            .unwrap_or_default()
    }

    /// Override the binding for an action.
    pub fn set_binding(&self, action: ShortcutAction, binding: KeyBinding) {
        self.bindings.update(|map| {
            map.insert(action, binding);
        });
    }

    /// Remove the binding for an action (unbind).
    pub fn clear_binding(&self, action: ShortcutAction) {
        self.bindings.update(|map| {
            map.remove(&action);
        });
    }

    /// Reset all bindings to defaults.
    pub fn reset_defaults(&self) {
        self.bindings.set(default_bindings());
    }

    /// Read-only access to all current bindings (reactive).
    pub fn bindings(&self) -> RwSignal<HashMap<ShortcutAction, KeyBinding>> {
        self.bindings
    }
}

/// Initialize the shortcuts system. Call once at the app root.
/// Provides `ShortcutsCtx` via Leptos context.
pub fn provide_shortcuts() {
    let bindings = RwSignal::new(default_bindings());
    let ctx = ShortcutsCtx { bindings };
    provide_context(ctx);
}

/// Retrieve the shortcuts context.
pub fn use_shortcuts() -> ShortcutsCtx {
    expect_context::<ShortcutsCtx>()
}

/// A global save trigger. Bump the counter to request a save from the active view.
#[derive(Clone, Copy)]
pub struct SaveTrigger {
    counter: RwSignal<u64>,
}

impl SaveTrigger {
    /// Request a save (increments the counter, notifying listeners).
    pub fn request(&self) {
        self.counter.update(|c| *c += 1);
    }

    /// Get the reactive counter signal (watch for changes).
    pub fn counter(&self) -> RwSignal<u64> {
        self.counter
    }
}

/// Initialize the save trigger. Call once at the app root.
pub fn provide_save_trigger() {
    let trigger = SaveTrigger {
        counter: RwSignal::new(0),
    };
    provide_context(trigger);
}

/// Retrieve the save trigger context.
pub fn use_save_trigger() -> SaveTrigger {
    expect_context::<SaveTrigger>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bindings_cover_all_actions() {
        let defaults = default_bindings();
        for action in ShortcutAction::all() {
            assert!(
                defaults.contains_key(action),
                "Missing default binding for {action:?}"
            );
        }
    }

    #[test]
    fn display_command_palette() {
        let binding = KeyBinding::new(true, true, false, "KeyP");
        let d = binding.display();
        assert!(d.contains('P'), "display should contain key letter: {d}");
        assert!(
            d.contains('\u{2318}'),
            "display should contain Cmd symbol: {d}"
        );
        assert!(
            d.contains('\u{21E7}'),
            "display should contain Shift symbol: {d}"
        );
    }

    #[test]
    fn display_toggle_comment() {
        let binding = KeyBinding::new(true, false, false, "Slash");
        let d = binding.display();
        assert_eq!(d, "\u{2318}/");
    }

    #[test]
    fn code_to_display_keys() {
        assert_eq!(code_to_display("KeyA"), "A");
        assert_eq!(code_to_display("Digit5"), "5");
        assert_eq!(code_to_display("Slash"), "/");
        assert_eq!(code_to_display("Enter"), "\u{21B5}");
    }

    #[test]
    fn action_labels_not_empty() {
        for action in ShortcutAction::all() {
            assert!(!action.label().is_empty());
            assert!(!action.category().is_empty());
        }
    }
}
