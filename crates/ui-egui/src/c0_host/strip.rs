//! The tool bar's host modes, applied to upstream's hotbar (`hotbar.rs`), which already docks,
//! floats and drags: hidden or shown, docked along the left side or floating, one or two columns.
//! The hotbar keeps drawing every button, flyout and tool option; this module only sets where it
//! sits and how many columns it uses.
use crate::PdfCraftApp;
use egui::{Context, Layout, Rect, Ui, UiBuilder, vec2};

/// The gap between two columns, the hotbar's own item spacing.
const GAP: f32 = 4.0;

/// Move the hotbar where the host's floating switch says: floating as upstream's own card, or
/// docked along the left side of the document area. Called when the host sets the switch.
pub(crate) fn place(app: &mut PdfCraftApp) {
    if app.chrome.toolbar_floating {
        // Upstream's own floating card, back at its default spot in the page margin.
        app.hotbar.dock = None;
        app.hotbar.full_side = false;
        app.hotbar.vertical = true;
        app.hotbar.position = [0.0, 0.0];
        app.hotbar.placed = false;
    } else {
        app.hotbar.full_side = true;
        // `place` accepts every side listed in its match; "left" is the host's docked side.
        let _ = app.hotbar.place("left");
    }
}

/// Before the hotbar is drawn: whether it is shown at all. A dock or float the user chose in the
/// hotbar itself flows back into the switch, so the host's Panels section shows what is on screen.
pub(crate) fn sync(app: &mut PdfCraftApp, _ctx: &Context) -> bool {
    app.chrome.toolbar_floating = app.hotbar.dock.is_none();
    app.chrome.toolbar
}

/// The width of a vertical bar: one button, or two side by side.
pub(crate) fn width(app: &PdfCraftApp, cell: f32) -> f32 {
    if app.chrome.toolbar_double { 2.0 * cell + GAP } else { cell }
}

/// Lay a vertical button list out in `layout`, wrapped into a second column when `double`. The
/// column height comes from the list's length in one column, measured on a frame where it fit in
/// one: half of it, rounded up to whole buttons. A list that spills into a third column (a tool
/// added its options) grows the estimate until the next one-column frame measures it again.
pub(crate) fn wrap(ui: &mut Ui, double: bool, cell: f32, layout: Layout, add: impl FnOnce(&mut Ui)) {
    if !double {
        ui.with_layout(layout, add);
        return;
    }
    let step = cell + GAP;
    let key = egui::Id::unique("c0-strip-one-column-length");
    let total = ui.ctx().data(|d| d.get_temp::<f32>(key));
    let height = total.map_or(f32::MAX / 4.0, |t| (t / 2.0 / step).ceil() * step + cell);
    let room = ui.available_rect_before_wrap();
    // One button wide: egui opens the second column beside it when the first is full.
    let rect = Rect::from_min_size(room.min, vec2(cell, height));
    // A centred column does not wrap; two columns line up from the left edge instead.
    let wrapped = Layout::top_down(egui::Align::Min).with_main_wrap(true);
    let used = ui.scope_builder(UiBuilder::new().layout(wrapped).max_rect(rect), add).response.rect;
    let columns = ((used.width() + GAP) / step).round().max(1.0);
    let next = if columns <= 1.0 {
        Some(used.height())
    } else if columns > 2.0 {
        Some(columns * height.min(used.height()))
    } else {
        total
    };
    if next != total {
        ui.ctx().data_mut(|d| d.insert_temp(key, next.unwrap_or(used.height())));
        ui.ctx().request_repaint();
    }
}

/// What `c0.inspect` reports under `panes.strip`: the bar as drawn now, not only the switches.
pub(crate) fn state(app: &PdfCraftApp) -> serde_json::Value {
    serde_json::json!({
        "visible": app.chrome.toolbar && app.mode != crate::Mode::Read && app.active.is_some(),
        "columns": if app.chrome.toolbar_double { 2 } else { 1 },
        "floating": app.hotbar.dock.is_none(),
        "dock": app.hotbar.dock,
        "pos": app.hotbar.position,
    })
}
