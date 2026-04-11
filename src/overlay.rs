use leptos::prelude::*;

/// Which overlay is currently active. Only one can be open at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveOverlay {
    #[default]
    None,
    CommandPalette,
    TableFinder,
    FindBar,
    Restore,
    Settings,
}

/// Context for centralized overlay state management.
#[derive(Debug, Clone, Copy)]
pub struct OverlayCtx {
    pub active: RwSignal<ActiveOverlay>,
}

impl OverlayCtx {
    /// Open an overlay, closing any currently open one first.
    pub fn open(&self, overlay: ActiveOverlay) {
        self.active.set(overlay);
    }

    /// Close the active overlay (sets to None).
    pub fn close(&self) {
        self.active.set(ActiveOverlay::None);
    }

    /// Close only if the given overlay is currently active.
    pub fn close_if(&self, overlay: ActiveOverlay) {
        if self.active.get_untracked() == overlay {
            self.active.set(ActiveOverlay::None);
        }
    }

    /// Check if a specific overlay is currently active (reactive).
    pub fn is_open(&self, overlay: ActiveOverlay) -> bool {
        self.active.get() == overlay
    }
}

/// Provide the overlay context at the top level.
pub fn provide_overlay_ctx() -> OverlayCtx {
    let ctx = OverlayCtx {
        active: RwSignal::new(ActiveOverlay::None),
    };
    provide_context(ctx);
    ctx
}

/// Use the overlay context from any child component.
pub fn use_overlay() -> OverlayCtx {
    expect_context::<OverlayCtx>()
}
