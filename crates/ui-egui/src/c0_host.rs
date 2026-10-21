//! C0VIBE host policy and requests. Defaults retain the standalone interface.

pub(crate) mod community;
pub mod files;
mod inspect;
mod panels;
mod settings;
pub(crate) use panels::split_path;

use crate::PdfCraftApp;
use serde_json::{Value, json};

pub const MAX_FILE_BYTES: usize = 72 * 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = 100 * 1024 * 1024;

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct Chrome {
    pub title_bar: bool,
    pub app_bar: bool,
    pub tab_strip: bool,
    pub start_screen: bool,
    pub community_links: bool,
    pub right_rail: bool,
    pub tools_panel: bool,
}

impl Default for Chrome {
    fn default() -> Self {
        Self { title_bar: true, app_bar: true, tab_strip: true, start_screen: true, community_links: true, right_rail: true, tools_panel: true }
    }
}

pub fn handle(app: &mut PdfCraftApp, ctx: &egui::Context, method: &str, params: &Value) -> Result<Value, String> {
    if !params.is_object() {
        return Err("params must be an object".into());
    }
    pdfcraft_engine::guard(|| match method {
        "c0.inspect" => Ok(inspect::inspect(app, ctx)),
        "c0.goTo" => inspect::go_to(app, ctx, params),
        "c0.documentBytes" => files::document_bytes(app, params),
        "c0.openFile" => files::open(app, ctx, params),
        "c0.newDocument" => files::new_document(app, ctx, params),
        "c0.cancelFileDialog" => Ok(json!({"cancelled": files::cancel_picker(ctx)})),
        "c0.set" => settings::set(app, ctx, params),
        _ => Err(format!("unknown host method {method:?}")),
    })
    .map_err(|e| format!("host request stopped: {e}"))?
}

/// Project host visibility into the rendered layout, retaining the saved docking tree.
pub(crate) fn project_panels(app: &PdfCraftApp, layout: &mut craft_ui::docking::Layout<crate::docking::Panel>) {
    use crate::docking::Panel;
    use craft_ui::docking::Action;
    if !app.chrome.tools_panel && layout.contains(&Panel::Tools) {
        let _ = layout.apply(Action::Close { panel: Panel::Tools });
    }
}
