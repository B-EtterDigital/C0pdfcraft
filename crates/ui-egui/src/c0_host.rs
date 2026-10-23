//! C0VIBE host policy and requests. Defaults retain the standalone interface.

pub(crate) mod community;
mod dock;
pub mod files;
mod inspect;
mod panels;
mod render;
mod settings;
pub(crate) mod strip;
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
    /// The quick-action strip (select, hand, comment, fill and sign) is drawn at all.
    pub toolbar: bool,
    /// The strip in two columns instead of one.
    pub toolbar_double: bool,
    /// The strip is a palette to drag within the document area instead of fixed in its corner.
    pub toolbar_floating: bool,
    /// The right dock folds away; the rail's icons remain and open it again.
    pub right_dock_collapsed: bool,
    /// Where the dragged strip sits, in points from the document area's corner.
    pub toolbar_pos: Option<[f32; 2]>,
    /// The right pane at the last logic pass: an icon pressed while folded shows as a change.
    #[serde(skip)]
    pub(crate) dock_seen: Option<crate::RightPanel>,
}

impl Default for Chrome {
    fn default() -> Self {
        Self {
            title_bar: true,
            app_bar: true,
            tab_strip: true,
            start_screen: true,
            community_links: true,
            right_rail: true,
            tools_panel: true,
            toolbar: true,
            toolbar_double: false,
            toolbar_floating: false,
            right_dock_collapsed: false,
            toolbar_pos: None,
            dock_seen: None,
        }
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
        "ui.render" => render::render(app, params),
        _ => Err(format!("unknown host method {method:?}")),
    })
    .map_err(|e| format!("host request stopped: {e}"))?
}

/// Run `draw` when `on`; the page's switches gate upstream's chrome in one line each.
pub(crate) fn when(on: bool, draw: impl FnOnce()) {
    if on {
        draw();
    }
}

/// The rendered dock differs from the saved one: Tools hidden by the host, or the right dock folded.
pub(crate) fn trims_dock(app: &PdfCraftApp) -> bool {
    !app.chrome.tools_panel || app.chrome.right_dock_collapsed
}

/// Project host visibility into the rendered layout, retaining the saved docking tree.
pub(crate) fn project_panels(app: &PdfCraftApp, layout: &mut craft_ui::docking::Layout<crate::docking::Panel>) {
    use crate::docking::Panel;
    use craft_ui::docking::Action;
    if !app.chrome.tools_panel && layout.contains(&Panel::Tools) {
        let _ = layout.apply(Action::Close { panel: Panel::Tools });
    }
    if app.chrome.right_dock_collapsed && layout.contains(&Panel::Inspector) {
        let _ = layout.apply(Action::Close { panel: Panel::Inspector });
    }
}
