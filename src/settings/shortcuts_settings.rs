use leptos::prelude::*;

use crate::shortcuts::{use_shortcuts, ShortcutAction};
use super::shortcut_input::ShortcutInput;

/// Settings section that lists all configurable keyboard shortcuts grouped by category,
/// with click-to-rebind via ShortcutInput and a "Reset to defaults" button.
#[component]
pub fn ShortcutsSetting() -> impl IntoView {
    let sc = use_shortcuts();

    // Group actions by category, preserving display order.
    let groups = build_groups();

    let on_reset = move |_| {
        sc.reset_defaults();
    };

    view! {
        <div class="flex flex-col gap-1.5">
            <label class="text-[13px] font-semibold text-gray-700 dark:text-zinc-300">"Keyboard Shortcuts"</label>
            <p class="text-[13px] text-gray-500 dark:text-zinc-400 mb-2">"Click a shortcut to rebind it. Press Escape to cancel, Backspace to clear."</p>

            <div class="flex flex-col gap-4">
                {groups.into_iter().map(|(category, actions)| {
                    view! {
                        <div class="flex flex-col gap-1">
                            <span class="text-[11px] font-medium text-gray-400 dark:text-zinc-500 uppercase tracking-wider">{category}</span>
                            <div class="flex flex-col">
                                {actions.into_iter().map(|action| {
                                    view! {
                                        <div class="flex items-center justify-between py-1.5">
                                            <span class="text-[13px] text-gray-700 dark:text-zinc-300">{action.label()}</span>
                                            <ShortcutInput action=action />
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // Reset to defaults button
            <div class="mt-2">
                <button
                    class="text-[13px] text-gray-500 dark:text-zinc-400 hover:text-gray-700 dark:hover:text-zinc-200 transition-colors duration-100"
                    on:click=on_reset
                >
                    "Reset to defaults"
                </button>
            </div>
        </div>
    }
}

/// Build ordered groups of (category_name, actions) from ShortcutAction::all().
fn build_groups() -> Vec<(&'static str, Vec<ShortcutAction>)> {
    let mut groups: Vec<(&str, Vec<ShortcutAction>)> = Vec::new();
    for &action in ShortcutAction::all() {
        let cat = action.category();
        if let Some(group) = groups.iter_mut().find(|(c, _)| *c == cat) {
            group.1.push(action);
        } else {
            groups.push((cat, vec![action]));
        }
    }
    groups
}
