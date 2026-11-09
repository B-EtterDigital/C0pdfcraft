//! The "All tools" panel's entries for the host: `c0.tools` lists them and `c0.openTool` opens one.
//! Both read the panel's own catalogue, and opening calls the same method a row click calls, so
//! the host's tab and the panel cannot disagree. Hiding the Tools pane changes neither.

use crate::{LeftPanel, PdfCraftApp};
use pdfcraft_engine::catalog::{self, TOOL_CATEGORIES, TOOL_GROUPS};
use serde_json::{Value, json};

/// The category a group is filed under in the expanded view. It is only a hint for the host:
/// the sidebar itself lists the groups flat.
fn category(id: &str) -> Option<&'static str> {
    TOOL_CATEGORIES.iter().find(|c| c.groups.contains(&id)).map(|c| c.label)
}

/// The tool open now, as its catalogue id. The organize grid of the active document counts
/// whether or not the Tools pane is open; otherwise the open left panel must be a known tool.
/// Hiding the pane (`tools_panel`) never changes this: the tool stays open for the host's tab.
pub(super) fn active(app: &PdfCraftApp) -> Option<&'static str> {
    if app.active.and_then(|i| app.views.get(i)).is_some_and(|v| v.organize) {
        return Some("organize");
    }
    match app.left {
        LeftPanel::Tool(id) if app.left_open => catalog::group(id).map(|g| g.id),
        _ => None,
    }
}

/// `c0.tools`: every entry of the All tools panel, in its order (View more included), with the
/// tool open now as `active` and in each row.
pub(super) fn list(app: &PdfCraftApp) -> Value {
    let open = active(app);
    let tools: Vec<Value> = TOOL_GROUPS
        .iter()
        .map(|g| {
            let mut row = json!({
                "id": g.id,
                "label": crate::i18n::t(g.label),
                "icon": g.icon,
                // The panel greys out no group row: planned and AI groups still open, with
                // their milestone hint on hover.
                "enabled": true,
                "active": open == Some(g.id),
            });
            if let Some(group) = category(g.id) {
                row["group"] = json!(crate::i18n::t(group));
            }
            row
        })
        .collect();
    json!({"tools": tools, "active": open})
}

/// `c0.openTool {id}`: the row click's own code path (`select_tool`), so the tool's panel shows
/// in the Tools pane when it is shown, and its canvas and dialog state changes either way.
pub(super) fn open(app: &mut PdfCraftApp, params: &Value) -> Result<Value, String> {
    let id = params.get("id").and_then(Value::as_str).ok_or("id must be a tool id string; c0.tools lists them")?;
    let group = catalog::group(id).ok_or_else(|| format!("unknown tool id {id}; c0.tools lists the ids"))?;
    if app.modal_open() {
        return Err("dismiss the current dialog before opening a tool".into());
    }
    app.select_tool(group.id);
    Ok(json!({"opened": group.id}))
}
