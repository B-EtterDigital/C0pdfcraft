//! Empty layout slots for upstream's trademark-only images, which forks cannot ship.
//! Preserve the previous blank SVGs' image sizing without loading a texture.

use egui::{ImageSize, Response, Role, Sense, Ui, Vec2, WidgetInfo, vec2};

/// Retain the wordmark's layout slot without bundling an image.
pub fn artcraft_logo(ui: &mut Ui, height: f32) -> Response {
    let sizing = ImageSize { max_size: vec2(f32::INFINITY, height), ..Default::default() };
    let size = sizing.calc_size(ui.available_size(), vec2(1.0, 1.0));
    empty_image(ui, size)
}

/// Retain the square mark's layout slot.
pub fn artcraft_mark(ui: &mut Ui, size: f32) -> Response {
    empty_image(ui, vec2(size, size))
}

fn empty_image(ui: &mut Ui, size: Vec2) -> Response {
    let response = ui.allocate_exact_size(size, Sense::hover()).1;
    // Keep upstream's accessible image role/alt text for the headless UI channel.
    response.widget_info(|| {
        let mut info = WidgetInfo::new(Role::Image);
        info.label = Some("ArtCraft".into());
        info
    });
    response
}
