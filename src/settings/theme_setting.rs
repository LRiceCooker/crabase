use leptos::ev;
use leptos::prelude::*;

use crate::theme::{use_theme, ThemePreference};

/// Theme preference selector with Light/Dark/System options.
#[component]
pub fn ThemeSetting() -> impl IntoView {
    let theme_ctx = use_theme();

    view! {
        <div class="flex flex-col gap-1.5">
            <label class="text-[13px] font-semibold text-gray-700 dark:text-zinc-300">"Theme"</label>
            <p class="text-[13px] text-gray-500 dark:text-zinc-400 mb-2">"Choose how crabase looks."</p>
            <div class="flex items-center gap-2">
                <ThemeButton
                    label="Light"
                    active=Signal::derive(move || theme_ctx.preference.get() == ThemePreference::Light)
                    on_click=Callback::new(move |_| theme_ctx.set_theme(ThemePreference::Light))
                />
                <ThemeButton
                    label="Dark"
                    active=Signal::derive(move || theme_ctx.preference.get() == ThemePreference::Dark)
                    on_click=Callback::new(move |_| theme_ctx.set_theme(ThemePreference::Dark))
                />
                <ThemeButton
                    label="System"
                    active=Signal::derive(move || theme_ctx.preference.get() == ThemePreference::System)
                    on_click=Callback::new(move |_| theme_ctx.set_theme(ThemePreference::System))
                />
            </div>
        </div>
    }
}

/// A single selectable theme option button (Light, Dark, or System).
#[component]
fn ThemeButton(
    label: &'static str,
    active: Signal<bool>,
    on_click: Callback<ev::MouseEvent>,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                if active.get() {
                    "bg-indigo-500 text-white text-[13px] font-medium px-3 py-1.5 rounded-md transition-colors duration-100"
                } else {
                    "bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-800 text-gray-700 dark:text-zinc-300 hover:bg-gray-50 dark:hover:bg-white/[0.03] text-[13px] font-medium px-3 py-1.5 rounded-md transition-colors duration-100"
                }
            }
            on:click=move |ev| on_click.run(ev)
        >
            {label}
        </button>
    }
}
