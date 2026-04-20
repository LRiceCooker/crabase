use leptos::prelude::*;

/// A single context menu item.
#[derive(Clone)]
pub struct ContextMenuItem {
    pub label: &'static str,
    pub danger: bool,
    pub action: Callback<()>,
}

/// A positioned right-click context menu.
#[component]
pub fn ContextMenu(
    /// Screen X coordinate
    x: i32,
    /// Screen Y coordinate
    y: i32,
    /// Menu items to show
    items: Vec<ContextMenuItem>,
    /// Called when menu should close
    on_close: Callback<()>,
) -> impl IntoView {
    // Close on click outside
    let backdrop_click = move |_: web_sys::MouseEvent| {
        on_close.run(());
    };

    let style = format!("left: {x}px; top: {y}px;");

    view! {
        // Invisible backdrop to catch clicks outside menu
        <div
            class="fixed inset-0 z-40"
            on:mousedown=backdrop_click
            on:contextmenu=move |ev: web_sys::MouseEvent| {
                ev.prevent_default();
                on_close.run(());
            }
        />
        <div
            class="fixed z-50 min-w-[180px] bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-800 dark:ring-1 dark:ring-white/[0.06] rounded-md shadow-xl py-1"
            style=style
        >
            {items.into_iter().map(|item| {
                let action = item.action;
                let text_class = if item.danger {
                    "px-4 py-2 text-[13px] text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/30 cursor-pointer transition-colors duration-100 select-none"
                } else {
                    "px-4 py-2 text-[13px] text-gray-900 dark:text-neutral-50 hover:bg-gray-50 dark:hover:bg-white/[0.06] cursor-pointer transition-colors duration-100 select-none"
                };
                view! {
                    <div
                        class=text_class
                        on:mousedown=move |ev: web_sys::MouseEvent| {
                            ev.stop_propagation();
                            action.run(());
                            on_close.run(());
                        }
                    >
                        {item.label}
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
