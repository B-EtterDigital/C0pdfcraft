//! The quick-action strip's host modes: hidden, in two columns, or dragged about the document
//! area. Upstream's `quick_bar` draws the buttons, flyouts and tool options as they are; this
//! module decides whether, where and how many columns wide.
use crate::{PdfCraftApp, QuickTool};
use egui::{Align, CornerRadius, Id, Layout, Rect, Sense, Stroke, Ui, UiBuilder, Vec2, vec2};

/// Upstream's area id for the strip (`canvas.rs`, `quick_bar`).
const BAR: &str = "quick-bar";
/// Upstream's offset of the strip from the document area's corner.
const MARGIN: f32 = 14.0;
/// A button's side: the width of one column, so the next column starts right beside it.
const BUTTON: f32 = 32.0;
/// A button and the 2-point gap below it.
const CELL: f32 = 34.0;
/// Height of the grip above a floating strip.
const GRIP: f32 = 12.0;

/// The buttons the strip shows now, and what it appends below them.
fn items(app: &PdfCraftApp) -> usize {
    // Select, Hand, three comment groups, Fill & Sign, and Edit text in the Edit workspace.
    let tools = 6 + usize::from(app.mode == crate::Mode::Edit || app.quick_tool == QuickTool::EditText);
    tools
        + match app.quick_tool {
            // Pin, colour, opacity and, for some tools, fill and thickness.
            QuickTool::Comment(tool) => 3 + usize::from(tool.has_fill()) + usize::from(tool.has_width()),
            // The font-size label and field.
            QuickTool::Fill(crate::fill_sign::FillTool::Text | crate::fill_sign::FillTool::Date) => 2,
            _ => 0,
        }
}

/// What `c0.inspect` reports under `panes.strip`: the strip as drawn now, not only the switches.
pub(crate) fn state(app: &PdfCraftApp) -> serde_json::Value {
    let chrome = &app.chrome;
    serde_json::json!({
        "visible": chrome.toolbar && app.mode != crate::Mode::Read && app.active.is_some(),
        "columns": if chrome.toolbar_double { 2 } else { 1 },
        "floating": chrome.toolbar_floating,
        "pos": chrome.toolbar_pos,
    })
}

/// What `quick_bar` lays its buttons out in: one column as upstream, or two when the switch says
/// so, by wrapping the column at half its height.
pub(crate) fn columns(ui: &Ui, app: &PdfCraftApp) -> UiBuilder {
    let layout = Layout::top_down(Align::Min);
    if !app.chrome.toolbar_double {
        return UiBuilder::new().layout(layout);
    }
    let room = ui.available_rect_before_wrap();
    let height = items(app).div_ceil(2) as f32 * CELL + 4.0;
    UiBuilder::new().layout(layout.with_main_wrap(true)).max_rect(Rect::from_min_size(room.min, vec2(BUTTON, height)))
}

/// Draw the strip the way the switches say. `draw` is upstream's `quick_bar`.
pub(crate) fn show(app: &mut PdfCraftApp, area: Rect, ui: &mut Ui, draw: fn(&mut PdfCraftApp, Rect, &mut Ui)) {
    if !app.chrome.toolbar {
        return;
    }
    if !app.chrome.toolbar_floating {
        return draw(app, area, ui);
    }
    let ctx = ui.ctx().clone();
    let size = ctx.memory(|m| m.area_rect(Id::unique(BAR))).map_or(vec2(46.0, 220.0), |r| r.size());
    let room = (area.size() - size - vec2(2.0 * MARGIN, 2.0 * MARGIN)).max(Vec2::ZERO);
    let at = app.chrome.toolbar_pos.map_or(Vec2::ZERO, |p| vec2(p[0], p[1]).clamp(Vec2::ZERO, room));
    draw(app, area.translate(at), ui);
    let t = crate::theme::Tokens::get(&ctx);
    let corner = area.min + at + vec2(MARGIN, MARGIN - GRIP);
    egui::Area::new(Id::unique("quick-bar-grip")).order(egui::Order::Middle).fixed_pos(corner).show(&ctx, |ui| {
        let (rect, resp) = ui.allocate_exact_size(vec2(size.x, GRIP), Sense::drag());
        ui.painter().rect_filled(rect, CornerRadius::same(6), t.card);
        ui.painter().rect_stroke(rect, CornerRadius::same(6), Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
        for k in [-1.0, 0.0, 1.0] {
            ui.painter().circle_filled(rect.center() + vec2(k * 6.0, 0.0), 1.2, t.text_faint);
        }
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "Move toolbar"));
        let resp = resp.on_hover_cursor(egui::CursorIcon::Grab);
        if resp.dragged() {
            let to = (at + resp.drag_delta()).clamp(Vec2::ZERO, room);
            app.chrome.toolbar_pos = Some([to.x, to.y]);
        }
    });
}
