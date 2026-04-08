use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Fuzzy match: checks if all characters in `pattern` appear in order in `text`.
/// Returns a score (higher = better match) or None if no match.
/// Bonuses: consecutive matches, word-boundary matches, prefix matches.
fn fuzzy_score(pattern: &str, text: &str) -> Option<i32> {
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
) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let commands: Vec<(&'static str, &'static str)> = vec![
        ("Restore Backup", "Restore a .tar.gz PostgreSQL backup"),
    ];

    // Focus input when palette opens, clear query when it closes
    Effect::new(move |_| {
        if show.get() {
            if let Some(el) = input_ref.get() {
                let _ = el.focus();
            }
        } else {
            set_query.set(String::new());
        }
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
            let filtered: Vec<_> = scored.into_iter().map(|(cmd, _)| cmd).collect();

            Some(view! {
                <div class="fixed inset-0 z-50 flex justify-center items-start pt-[15vh]">
                    // Backdrop
                    <div
                        class="absolute inset-0 bg-black/50"
                        on:click=move |_| set_show.set(false)
                    ></div>
                    // Palette
                    <div class="relative z-10 w-full max-w-lg bg-base-100 rounded-lg shadow-2xl overflow-hidden">
                        <div class="p-3 border-b border-base-300">
                            <input
                                type="text"
                                node_ref=input_ref
                                placeholder="Type a command..."
                                class="input input-bordered w-full"
                                prop:value=move || query.get()
                                on:input=move |ev| set_query.set(event_target_value(&ev))
                                on:keydown=move |ev| {
                                    let ev: &web_sys::KeyboardEvent = ev.unchecked_ref();
                                    if ev.key() == "Escape" {
                                        set_show.set(false);
                                    }
                                }
                            />
                        </div>
                        <ul class="menu menu-sm p-2 max-h-64 overflow-y-auto">
                            {filtered.into_iter().map(|(name, desc)| {
                                view! {
                                    <li>
                                        <a class="flex flex-col items-start py-2">
                                            <span class="font-medium">{name.to_string()}</span>
                                            <span class="text-xs text-base-content/50">{desc.to_string()}</span>
                                        </a>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
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
