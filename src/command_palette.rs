use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::icons::IconSearch;

/// Fuzzy match: checks if all characters in `pattern` appear in order in `text`.
/// Returns a score (higher = better match) or None if no match.
/// Bonuses: consecutive matches, word-boundary matches, prefix matches.
pub fn fuzzy_score(pattern: &str, text: &str) -> Option<i32> {
    if pattern.is_empty() {
        return Some(0);
    }

    let pattern_lower: Vec<char> = pattern.to_lowercase().chars().collect();
    let text_lower: Vec<char> = text.to_lowercase().chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    let mut score: i32 = 0;
    let mut pattern_idx = 0;
    let mut prev_match_idx: Option<usize> = None;

    for (i, &ch) in text_lower.iter().enumerate() {
        if pattern_idx < pattern_lower.len() && ch == pattern_lower[pattern_idx] {
            score += 1;

            // Bonus for consecutive matches
            if let Some(prev) = prev_match_idx {
                if i == prev + 1 {
                    score += 5;
                }
            }

            // Bonus for matching at word boundary (start, after space/separator)
            if i == 0 || matches!(text_chars.get(i.wrapping_sub(1)), Some(' ' | '_' | '-')) {
                score += 3;
            }

            prev_match_idx = Some(i);
            pattern_idx += 1;
        }
    }

    if pattern_idx == pattern_lower.len() {
        Some(score)
    } else {
        None
    }
}

#[component]
pub fn CommandPalette(
    show: ReadSignal<bool>,
    set_show: WriteSignal<bool>,
    on_command: Callback<String>,
) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (selected_idx, set_selected_idx) = signal(0usize);
    let input_ref = NodeRef::<leptos::html::Input>::new();

    // (name, description, shortcut)
    let commands: Vec<(&'static str, &'static str, &'static str)> = vec![
        ("New SQL Editor", "Open a new SQL editor tab", ""),
        ("Restore Backup", "Restore a .tar.gz PostgreSQL backup", ""),
        ("Settings", "Open application settings", ""),
    ];

    // Focus input when palette opens, clear query when it closes
    Effect::new(move |_| {
        if show.get() {
            if let Some(el) = input_ref.get() {
                let _ = el.focus();
            }
        } else {
            set_query.set(String::new());
            set_selected_idx.set(0);
        }
    });

    // Reset selection when query changes
    Effect::new(move |_| {
        let _ = query.get();
        set_selected_idx.set(0);
    });

    move || {
        if show.get() {
            let q = query.get();
            let mut scored: Vec<_> = commands
                .iter()
                .filter_map(|cmd| {
                    if q.is_empty() {
                        Some((cmd, 0))
                    } else {
                        fuzzy_score(&q, cmd.0).map(|s| (cmd, s))
                    }
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            let filtered: Vec<_> = scored.into_iter().map(|(cmd, _)| *cmd).collect();
            let count = filtered.len();

            // Clone for Enter key handler
            let filtered_for_enter = filtered.clone();

            Some(view! {
                <div class="fixed inset-0 z-50 flex justify-center items-start">
                    // Backdrop
                    <div
                        class="absolute inset-0 backdrop-blur-sm bg-black/30"
                        on:click=move |_| set_show.set(false)
                    ></div>
                    // Palette panel
                    <div class="relative z-10 w-[560px] max-h-[400px] bg-white dark:bg-zinc-900 rounded-xl shadow-2xl dark:shadow-black/40 border border-gray-200 dark:border-white/[0.08] overflow-hidden mt-[20vh] dark:ring-1 dark:ring-white/[0.06]">
                        // Search input
                        <div class="flex items-center gap-2 px-4 py-3 border-b border-gray-100 dark:border-[#1F1F23]">
                            <IconSearch class="w-4 h-4 text-gray-400 dark:text-zinc-500 shrink-0" />
                            <input
                                type="text"
                                node_ref=input_ref
                                placeholder="Type a command..."
                                class="text-base w-full focus:outline-none bg-transparent text-gray-900 dark:text-neutral-50 placeholder-gray-400 dark:placeholder-zinc-500"
                                prop:value=move || query.get()
                                on:input=move |ev| set_query.set(event_target_value(&ev))
                                on:keydown={
                                    let filtered = filtered_for_enter.clone();
                                    move |ev| {
                                        let ev: &web_sys::KeyboardEvent = ev.unchecked_ref();
                                        match ev.key().as_str() {
                                            "Escape" => set_show.set(false),
                                            "Enter" => {
                                                let idx = selected_idx.get();
                                                if let Some((name, _, _)) = filtered.get(idx) {
                                                    on_command.run(name.to_string());
                                                    set_show.set(false);
                                                }
                                            }
                                            "ArrowDown" => {
                                                ev.prevent_default();
                                                let idx = selected_idx.get();
                                                if idx + 1 < count {
                                                    set_selected_idx.set(idx + 1);
                                                }
                                            }
                                            "ArrowUp" => {
                                                ev.prevent_default();
                                                let idx = selected_idx.get();
                                                if idx > 0 {
                                                    set_selected_idx.set(idx - 1);
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            />
                        </div>
                        // Command group
                        <div class="text-[11px] font-medium text-gray-400 dark:text-zinc-500 uppercase px-4 py-2">"Commands"</div>
                        // Command list
                        <div class="pb-2 max-h-64 overflow-y-auto">
                            {filtered.into_iter().enumerate().map(|(idx, (name, desc, shortcut))| {
                                let cmd_name = name.to_string();
                                let is_selected = selected_idx.get() == idx;
                                let class = if is_selected {
                                    "px-4 py-2 flex items-center justify-between text-[13px] cursor-pointer bg-indigo-50 dark:bg-indigo-500/25 text-indigo-600 dark:text-indigo-400"
                                } else {
                                    "px-4 py-2 flex items-center justify-between text-[13px] cursor-pointer hover:bg-indigo-50 dark:hover:bg-indigo-500/25 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-100"
                                };
                                view! {
                                    <div
                                        class=class
                                        on:click=move |_| {
                                            on_command.run(cmd_name.clone());
                                            set_show.set(false);
                                        }
                                    >
                                        <div class="flex flex-col">
                                            <span class="font-medium text-gray-900 dark:text-neutral-50">{name.to_string()}</span>
                                            <span class="text-[11px] text-gray-400 dark:text-zinc-500">{desc.to_string()}</span>
                                        </div>
                                        {if !shortcut.is_empty() {
                                            Some(view! {
                                                <span class="text-[11px] text-gray-400 dark:text-zinc-500 font-mono">{shortcut.to_string()}</span>
                                            })
                                        } else {
                                            None
                                        }}
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                </div>
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_pattern_matches_everything() {
        assert_eq!(fuzzy_score("", "Restore Backup"), Some(0));
        assert_eq!(fuzzy_score("", ""), Some(0));
    }

    #[test]
    fn exact_match() {
        assert!(fuzzy_score("Restore Backup", "Restore Backup").is_some());
    }

    #[test]
    fn case_insensitive() {
        let lower = fuzzy_score("restore", "Restore Backup");
        let upper = fuzzy_score("RESTORE", "Restore Backup");
        assert!(lower.is_some());
        assert!(upper.is_some());
        assert_eq!(lower, upper);
    }

    #[test]
    fn fuzzy_subsequence_matches() {
        // "rb" matches "Restore Backup" (R...B...)
        assert!(fuzzy_score("rb", "Restore Backup").is_some());
        // "reb" matches "Restore Backup" (Re...B...)
        assert!(fuzzy_score("reb", "Restore Backup").is_some());
    }

    #[test]
    fn no_match_when_chars_missing() {
        assert!(fuzzy_score("xyz", "Restore Backup").is_none());
        assert!(fuzzy_score("rz", "Restore Backup").is_none());
    }

    #[test]
    fn no_match_when_pattern_longer_than_text() {
        assert!(fuzzy_score("abcdef", "abc").is_none());
    }

    #[test]
    fn consecutive_matches_score_higher() {
        // "res" is consecutive in "Restore" → higher score than "r_e_s" scattered
        let consecutive = fuzzy_score("res", "Restore Backup").unwrap();
        let scattered = fuzzy_score("reb", "Restore Backup").unwrap();
        assert!(consecutive > scattered);
    }

    #[test]
    fn word_boundary_bonus() {
        // "rb" matches at word boundaries (R of Restore, B of Backup)
        let score = fuzzy_score("rb", "Restore Backup").unwrap();
        // Both matches are at word boundaries, so should get boundary bonuses
        assert!(score > 2); // base 2 (1 per char) + bonuses
    }
}
