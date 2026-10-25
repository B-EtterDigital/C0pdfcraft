//! The right dock's fold. The drawn panes drop the side panel (see `project_panes`); the rail
//! keeps its icons. Pressing one is the only
//! way the right pane changes while folded, so a change of `app.right` unfolds the dock.
use crate::PdfCraftApp;

/// Once per logic pass.
pub(super) fn step(app: &mut PdfCraftApp) {
    let seen = app.chrome.dock_seen;
    if app.chrome.right_dock_collapsed && app.right != seen {
        // The rail toggles the open pane off when its own icon is pressed again; the press was
        // meant to open that pane, so it comes back rather than closing.
        app.right = app.right.or(seen);
        app.chrome.right_dock_collapsed = false;
    }
    app.chrome.dock_seen = app.right;
}
