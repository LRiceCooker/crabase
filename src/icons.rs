use leptos::prelude::*;

/// Icon size class. Defaults to "w-4 h-4" per design.md.
/// Pass `class` to override (e.g. "w-5 h-5" for toolbar icons).

/// Base SVG wrapper for all Lucide icons.
/// All Lucide icons use a 24x24 viewBox, stroke-based, 2px stroke width.
#[component]
fn LucideBase(
    #[prop(optional, default = "w-4 h-4")] class: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            {children()}
        </svg>
    }
}

// ── General ──────────────────────────────────────────────

#[component]
pub fn IconX(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! { <LucideBase class=class><path d="M18 6 6 18" /><path d="m6 6 12 12" /></LucideBase> }
}

#[component]
pub fn IconSearch(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! { <LucideBase class=class><circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" /></LucideBase> }
}

#[component]
pub fn IconPlus(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! { <LucideBase class=class><path d="M5 12h14" /><path d="M12 5v14" /></LucideBase> }
}

#[component]
pub fn IconRefreshCw(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
            <path d="M21 3v5h-5" />
            <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
            <path d="M8 16H3v5" />
        </LucideBase>
    }
}

#[component]
pub fn IconTrash2(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <path d="M3 6h18" />
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
            <line x1="10" x2="10" y1="11" y2="17" />
            <line x1="14" x2="14" y1="11" y2="17" />
        </LucideBase>
    }
}

#[component]
pub fn IconEdit(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" />
            <path d="m15 5 4 4" />
        </LucideBase>
    }
}

// ── Navigation ───────────────────────────────────────────

#[component]
pub fn IconChevronLeft(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! { <LucideBase class=class><path d="m15 18-6-6 6-6" /></LucideBase> }
}

#[component]
pub fn IconChevronRight(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! { <LucideBase class=class><path d="m9 18 6-6-6-6" /></LucideBase> }
}

#[component]
pub fn IconArrowLeft(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! { <LucideBase class=class><path d="m12 19-7-7 7-7" /><path d="M19 12H5" /></LucideBase> }
}

// ── Status ───────────────────────────────────────────────

#[component]
pub fn IconCheckCircle(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
            <path d="m9 11 3 3L22 4" />
        </LucideBase>
    }
}

#[component]
pub fn IconXCircle(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <circle cx="12" cy="12" r="10" />
            <path d="m15 9-6 6" />
            <path d="m9 9 6 6" />
        </LucideBase>
    }
}

#[component]
pub fn IconAlertTriangle(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3" />
            <path d="M12 9v4" />
            <path d="M12 17h.01" />
        </LucideBase>
    }
}

// ── Data / Database ──────────────────────────────────────

#[component]
pub fn IconDatabase(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <ellipse cx="12" cy="5" rx="9" ry="3" />
            <path d="M3 5V19A9 3 0 0 0 21 19V5" />
            <path d="M3 12A9 3 0 0 0 21 12" />
        </LucideBase>
    }
}

#[component]
pub fn IconTable(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <path d="M12 3v18" />
            <rect width="18" height="18" x="3" y="3" rx="2" />
            <path d="M3 9h18" />
            <path d="M3 15h18" />
        </LucideBase>
    }
}

// ── Actions ──────────────────────────────────────────────

#[component]
pub fn IconPlay(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! { <LucideBase class=class><polygon points="6 3 20 12 6 21 6 3" /></LucideBase> }
}

#[component]
pub fn IconSave(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <path d="M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z" />
            <path d="M17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7" />
            <path d="M7 3v4a1 1 0 0 0 1 1h7" />
        </LucideBase>
    }
}

#[component]
pub fn IconUpload(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="17 8 12 3 7 8" />
            <line x1="12" x2="12" y1="3" y2="15" />
        </LucideBase>
    }
}

#[component]
pub fn IconDownload(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" x2="12" y1="15" y2="3" />
        </LucideBase>
    }
}

#[component]
pub fn IconFile(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" />
            <path d="M14 2v4a2 2 0 0 0 2 2h4" />
        </LucideBase>
    }
}

// ── UI / Layout ──────────────────────────────────────────

#[component]
pub fn IconCommand(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <path d="M15 6.5a2.5 2.5 0 1 0 5 0 2.5 2.5 0 1 0-5 0" />
            <path d="M4 6.5a2.5 2.5 0 1 0 5 0 2.5 2.5 0 1 0-5 0" />
            <path d="M15 17.5a2.5 2.5 0 1 0 5 0 2.5 2.5 0 1 0-5 0" />
            <path d="M4 17.5a2.5 2.5 0 1 0 5 0 2.5 2.5 0 1 0-5 0" />
            <path d="M9 6.5h6" />
            <path d="M9 17.5h6" />
            <path d="M6.5 9v6" />
            <path d="M17.5 9v6" />
        </LucideBase>
    }
}

#[component]
pub fn IconTerminal(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <polyline points="4 17 10 11 4 5" />
            <line x1="12" x2="20" y1="19" y2="19" />
        </LucideBase>
    }
}

#[component]
pub fn IconLogOut(#[prop(optional, default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
            <polyline points="16 17 21 12 16 7" />
            <line x1="21" x2="9" y1="12" y2="12" />
        </LucideBase>
    }
}

#[component]
pub fn IconLoader(#[prop(optional, default = "w-4 h-4 animate-spin")] class: &'static str) -> impl IntoView {
    view! {
        <LucideBase class=class>
            <path d="M12 2v4" />
            <path d="m16.2 7.8 2.9-2.9" />
            <path d="M18 12h4" />
            <path d="m16.2 16.2 2.9 2.9" />
            <path d="M12 18v4" />
            <path d="m4.9 19.1 2.9-2.9" />
            <path d="M2 12h4" />
            <path d="m4.9 4.9 2.9 2.9" />
        </LucideBase>
    }
}
