use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::tauri;

/// User's theme preference as stored in settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

impl ThemePreference {
    fn from_tauri(theme: &tauri::Theme) -> Self {
        match theme {
            tauri::Theme::Light => Self::Light,
            tauri::Theme::Dark => Self::Dark,
            tauri::Theme::System => Self::System,
        }
    }

    fn to_tauri(self) -> tauri::Theme {
        match self {
            Self::Light => tauri::Theme::Light,
            Self::Dark => tauri::Theme::Dark,
            Self::System => tauri::Theme::System,
        }
    }
}

/// Theme context provided at the app root.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct ThemeCtx {
    /// The user's chosen preference (Light / Dark / System).
    pub preference: RwSignal<ThemePreference>,
    /// Whether the OS currently prefers dark mode.
    system_is_dark: RwSignal<bool>,
    /// Derived: whether dark mode is currently active.
    pub is_dark: Memo<bool>,
}

impl ThemeCtx {
    /// Set theme preference and persist to backend settings.
    pub fn set_theme(&self, pref: ThemePreference) {
        self.preference.set(pref);
        let theme = pref.to_tauri();
        spawn_local(async move {
            let _ = tauri::save_settings(&tauri::Settings { theme }).await;
        });
    }

    /// Toggle between light and dark. If currently on System, switches to the opposite of what's active.
    pub fn toggle(&self) {
        let next = if self.is_dark.get() {
            ThemePreference::Light
        } else {
            ThemePreference::Dark
        };
        self.set_theme(next);
    }
}

/// Initialize the theme system. Call once at the app root.
/// Provides `ThemeCtx` via Leptos context.
pub fn provide_theme() {
    let preference = RwSignal::new(ThemePreference::Light);
    let system_is_dark = RwSignal::new(query_system_dark());

    let is_dark = Memo::new(move |_| match preference.get() {
        ThemePreference::Dark => true,
        ThemePreference::Light => false,
        ThemePreference::System => system_is_dark.get(),
    });

    let ctx = ThemeCtx {
        preference,
        system_is_dark,
        is_dark,
    };
    provide_context(ctx);

    // Load saved settings from backend
    spawn_local(async move {
        if let Ok(settings) = tauri::load_settings().await {
            preference.set(ThemePreference::from_tauri(&settings.theme));
        }
    });

    // Listen for OS theme changes (for "System" preference)
    listen_system_theme(system_is_dark);

    // Apply/remove `dark` class on <html> reactively + switch app icon
    Effect::new(move |_| {
        let dark = is_dark.get();
        apply_dark_class(dark);
        // Switch the window icon to match the theme
        spawn_local(async move {
            let _ = tauri::set_app_icon(dark).await;
        });
    });
}

/// Retrieve the theme context from Leptos context.
pub fn use_theme() -> ThemeCtx {
    expect_context::<ThemeCtx>()
}

/// Query the OS for current dark mode preference.
fn query_system_dark() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
        .map(|mq| mq.matches())
        .unwrap_or(false)
}

/// Add or remove the `dark` class on `<html>`.
fn apply_dark_class(dark: bool) {
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let class_list = el.class_list();
        if dark {
            let _ = class_list.add_1("dark");
        } else {
            let _ = class_list.remove_1("dark");
        }
    }
}

/// Listen for OS `prefers-color-scheme` changes and update the signal.
fn listen_system_theme(system_is_dark: RwSignal<bool>) {
    let mq = web_sys::window()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten());

    if let Some(mq) = mq {
        let closure = Closure::<dyn FnMut(JsValue)>::new(move |_event: JsValue| {
            system_is_dark.set(query_system_dark());
        });
        let _ = mq.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
        closure.forget(); // App-lifetime: theme provider lives for the entire session
    }
}
