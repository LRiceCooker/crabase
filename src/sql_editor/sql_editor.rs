use leptos::prelude::*;

use crate::shortcuts::{self, ShortcutAction};

/// SQL text editor with line numbers and monospace font.
/// Supports Cmd+/ to toggle SQL comments on selected lines.
#[component]
pub fn SqlEditor(
    sql: RwSignal<String>,
) -> impl IntoView {
    let line_count = move || {
        let text = sql.get();
        text.lines().count().max(1)
    };

    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();

    let sc = shortcuts::use_shortcuts();
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        // Toggle comment via shortcuts registry
        if sc.matches(ShortcutAction::ToggleComment, &ev) {
            ev.prevent_default();

            if let Some(el) = textarea_ref.get() {
                let el: &web_sys::HtmlTextAreaElement = el.as_ref();
                let text = el.value();
                let start = el.selection_start().ok().flatten().unwrap_or(0) as usize;
                let end = el.selection_end().ok().flatten().unwrap_or(0) as usize;

                let (new_text, new_start, new_end) = toggle_comment(&text, start, end);
                sql.set(new_text.clone());
                el.set_value(&new_text);
                let _ = el.set_selection_start(Some(new_start as u32));
                let _ = el.set_selection_end(Some(new_end as u32));
            }
        }
    };

    view! {
        <div class="flex flex-1 overflow-hidden">
            // Line number gutter
            <div class="bg-gray-50 dark:bg-[#0F0F11] text-gray-400 dark:text-zinc-500 text-right pr-2 pl-2 select-none border-r border-gray-100 dark:border-[#1F1F23] font-mono text-[13px] leading-relaxed pt-2 overflow-hidden shrink-0">
                {move || {
                    (1..=line_count()).map(|n| {
                        view! { <div>{n}</div> }
                    }).collect::<Vec<_>>()
                }}
            </div>
            // Editor textarea
            <textarea
                class="flex-1 bg-white dark:bg-[#0D0D0F] font-mono text-[13px] leading-relaxed p-2 resize-none focus:outline-none text-gray-900 dark:text-zinc-200 placeholder-gray-400 dark:placeholder-zinc-500"
                spellcheck="false"
                autocomplete="off"
                prop:value=move || sql.get()
                on:input=move |ev| sql.set(event_target_value(&ev))
                on:keydown=on_keydown
                node_ref=textarea_ref
                placeholder="Write your SQL query here..."
            />
        </div>
    }
}

/// Toggle `-- ` comment prefix on lines covered by the selection range.
/// Returns (new_text, new_selection_start, new_selection_end).
fn toggle_comment(text: &str, sel_start: usize, sel_end: usize) -> (String, usize, usize) {
    let lines: Vec<&str> = text.lines().collect();

    // Find which lines the selection covers
    let mut offset = 0;
    let mut start_line = 0;
    let mut end_line = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_end = offset + line.len();
        if offset <= sel_start && sel_start <= line_end {
            start_line = i;
        }
        if offset <= sel_end && sel_end <= line_end {
            end_line = i;
        }
        offset = line_end + 1; // +1 for newline
    }

    // Check if all selected lines are already commented
    let all_commented = lines[start_line..=end_line]
        .iter()
        .all(|l| l.starts_with("-- "));

    let mut result_lines = lines.iter().map(|l| l.to_string()).collect::<Vec<_>>();
    let mut offset_delta: i64 = 0;

    for i in start_line..=end_line {
        if all_commented {
            // Remove comment prefix
            if result_lines[i].starts_with("-- ") {
                result_lines[i] = result_lines[i][3..].to_string();
                offset_delta -= 3;
            }
        } else {
            // Add comment prefix
            result_lines[i] = format!("-- {}", result_lines[i]);
            offset_delta += 3;
        }
    }

    let new_text = result_lines.join("\n");
    // Handle trailing newline if original text had one
    let new_text = if text.ends_with('\n') && !new_text.ends_with('\n') {
        format!("{}\n", new_text)
    } else {
        new_text
    };

    let new_end = (sel_end as i64 + offset_delta).max(0) as usize;
    let new_start = if sel_start == sel_end {
        new_end
    } else {
        sel_start
    };

    (new_text, new_start, new_end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_comment_single_line() {
        let (result, _, _) = toggle_comment("SELECT * FROM users", 0, 0);
        assert_eq!(result, "-- SELECT * FROM users");
    }

    #[test]
    fn test_toggle_uncomment_single_line() {
        let (result, _, _) = toggle_comment("-- SELECT * FROM users", 0, 0);
        assert_eq!(result, "SELECT * FROM users");
    }

    #[test]
    fn test_toggle_comment_multiple_lines() {
        let text = "SELECT *\nFROM users\nWHERE id = 1";
        let (result, _, _) = toggle_comment(text, 0, text.len());
        assert_eq!(result, "-- SELECT *\n-- FROM users\n-- WHERE id = 1");
    }

    #[test]
    fn test_toggle_uncomment_multiple_lines() {
        let text = "-- SELECT *\n-- FROM users\n-- WHERE id = 1";
        let (result, _, _) = toggle_comment(text, 0, text.len());
        assert_eq!(result, "SELECT *\nFROM users\nWHERE id = 1");
    }
}
