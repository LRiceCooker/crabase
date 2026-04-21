use leptos::prelude::*;

use crate::table_view::cell_editors::array_editor_modal::{ArrayEditRequest, ArrayEditorModal};
use crate::table_view::cell_editors::xml_editor_modal::{XmlEditRequest, XmlEditorModal};
use crate::table_view::change_tracker::ChangeTracker;
use crate::table_view::json_editor::{JsonEditRequest, JsonEditorModal};

/// Holds modal editor state (JSON, Array, XML) and provides open/save/cancel callbacks.
#[derive(Clone, Copy)]
pub struct ModalEditors {
    pub json_edit: ReadSignal<Option<JsonEditRequest>>,
    set_json_edit: WriteSignal<Option<JsonEditRequest>>,
    pub array_edit: ReadSignal<Option<ArrayEditRequest>>,
    set_array_edit: WriteSignal<Option<ArrayEditRequest>>,
    pub xml_edit: ReadSignal<Option<XmlEditRequest>>,
    set_xml_edit: WriteSignal<Option<XmlEditRequest>>,
    rows: RwSignal<Vec<Vec<serde_json::Value>>>,
    changes: ChangeTracker,
}

impl ModalEditors {
    /// Create new modal editor state tied to the given rows and change tracker.
    pub fn new(
        rows: RwSignal<Vec<Vec<serde_json::Value>>>,
        changes: ChangeTracker,
    ) -> Self {
        let (json_edit, set_json_edit) = signal(Option::<JsonEditRequest>::None);
        let (array_edit, set_array_edit) = signal(Option::<ArrayEditRequest>::None);
        let (xml_edit, set_xml_edit) = signal(Option::<XmlEditRequest>::None);
        Self {
            json_edit,
            set_json_edit,
            array_edit,
            set_array_edit,
            xml_edit,
            set_xml_edit,
            rows,
            changes,
        }
    }

    /// Callback to open the JSON editor modal.
    pub fn on_json_edit(&self) -> Callback<JsonEditRequest> {
        let set = self.set_json_edit;
        Callback::new(move |req| set.set(Some(req)))
    }

    /// Callback to open the array editor modal.
    pub fn on_array_edit(&self) -> Callback<ArrayEditRequest> {
        let set = self.set_array_edit;
        Callback::new(move |req| set.set(Some(req)))
    }

    /// Callback to open the XML editor modal.
    pub fn on_xml_edit(&self) -> Callback<XmlEditRequest> {
        let set = self.set_xml_edit;
        Callback::new(move |req| set.set(Some(req)))
    }

    /// Save a cell edit from any modal, track the change, update the row, close the modal.
    fn save_modal_edit(
        &self,
        row: usize,
        col: usize,
        val: serde_json::Value,
    ) {
        let original = self.rows.get()
            .get(row)
            .and_then(|r| r.get(col))
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        self.changes.track_cell_edit(row, col, original, &val);

        self.rows.update(|r| {
            if let Some(row_data) = r.get_mut(row) {
                if let Some(cell) = row_data.get_mut(col) {
                    *cell = val;
                }
            }
        });
    }

    /// Callback for saving from the JSON editor modal.
    pub fn on_json_save(&self) -> Callback<(usize, usize, serde_json::Value)> {
        let this = *self;
        Callback::new(move |(row, col, val)| {
            this.save_modal_edit(row, col, val);
            this.set_json_edit.set(None);
        })
    }

    /// Callback for cancelling the JSON editor modal.
    pub fn on_json_cancel(&self) -> Callback<()> {
        let set = self.set_json_edit;
        Callback::new(move |_| set.set(None))
    }

    /// Callback for saving from the array editor modal.
    pub fn on_array_save(&self) -> Callback<(usize, usize, serde_json::Value)> {
        let this = *self;
        Callback::new(move |(row, col, val)| {
            this.save_modal_edit(row, col, val);
            this.set_array_edit.set(None);
        })
    }

    /// Callback for cancelling the array editor modal.
    pub fn on_array_cancel(&self) -> Callback<()> {
        let set = self.set_array_edit;
        Callback::new(move |_| set.set(None))
    }

    /// Callback for saving from the XML editor modal.
    pub fn on_xml_save(&self) -> Callback<(usize, usize, serde_json::Value)> {
        let this = *self;
        Callback::new(move |(row, col, val)| {
            this.save_modal_edit(row, col, val);
            this.set_xml_edit.set(None);
        })
    }

    /// Callback for cancelling the XML editor modal.
    pub fn on_xml_cancel(&self) -> Callback<()> {
        let set = self.set_xml_edit;
        Callback::new(move |_| set.set(None))
    }

    /// Render all modal editor overlays.
    pub fn view(&self) -> impl IntoView {
        let json_edit = self.json_edit;
        let on_json_save = self.on_json_save();
        let on_json_cancel = self.on_json_cancel();
        let array_edit = self.array_edit;
        let on_array_save = self.on_array_save();
        let on_array_cancel = self.on_array_cancel();
        let xml_edit = self.xml_edit;
        let on_xml_save = self.on_xml_save();
        let on_xml_cancel = self.on_xml_cancel();

        view! {
            // JSON editor modal
            {move || {
                json_edit.get().map(|req| {
                    view! {
                        <JsonEditorModal
                            request=req
                            on_save=on_json_save
                            on_cancel=on_json_cancel
                        />
                    }
                })
            }}

            // Array editor modal
            {move || {
                array_edit.get().map(|req| {
                    view! {
                        <ArrayEditorModal
                            request=req
                            on_save=on_array_save
                            on_cancel=on_array_cancel
                        />
                    }
                })
            }}

            // XML editor modal
            {move || {
                xml_edit.get().map(|req| {
                    view! {
                        <XmlEditorModal
                            request=req
                            on_save=on_xml_save
                            on_cancel=on_xml_cancel
                        />
                    }
                })
            }}
        }
    }
}
