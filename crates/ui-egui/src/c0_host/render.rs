//! `ui.render`: a page as a PNG from the engine's renderer. No window, frame or presented pixels.
use super::MAX_FILE_BYTES;
use crate::PdfCraftApp;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};

/// Points per inch: `scale` 1 is 72 dpi, one pixel per point.
const POINTS_PER_INCH: f64 = 72.0;
/// The engine clamps resolution to 18..=1200 dpi.
const SCALES: std::ops::RangeInclusive<f64> = 0.25..=16.0;

pub(super) fn render(app: &PdfCraftApp, params: &Value) -> Result<Value, String> {
    if let Some(unknown) = params.as_object().and_then(|o| o.keys().find(|k| !matches!(k.as_str(), "page" | "scale"))) {
        return Err(format!("unknown parameter {unknown:?}"));
    }
    let (index, id) = app.active_ids().ok_or("no PDF open")?;
    let doc = app.session.get(id).ok_or("no PDF open")?;
    let count = doc.info.pages.len();
    let page = match params.get("page") {
        None | Some(Value::Null) => app.views.get(index).map_or(0, |view| view.current),
        Some(page) => page.as_u64().and_then(|v| usize::try_from(v).ok()).ok_or("page must be a zero-based integer")?,
    };
    if page >= count {
        return Err(format!("page must be less than {count}"));
    }
    let scale = match params.get("scale") {
        None | Some(Value::Null) => 1.0,
        Some(scale) => scale.as_f64().filter(|s| SCALES.contains(s)).ok_or("scale must be a number from 0.25 to 16")?,
    };
    let png = pdfcraft_engine::export::Exporter::new(doc).png(page, scale * POINTS_PER_INCH)?;
    if png.len() > MAX_FILE_BYTES {
        return Err("the page image exceeds the 72 MiB file limit".into());
    }
    // IHDR follows the 8-byte signature and the chunk header: width, then height, big-endian.
    let dimension = |at: usize| png.get(at..at + 4).and_then(|b| <[u8; 4]>::try_from(b).ok()).map(u32::from_be_bytes);
    let (width, height) = dimension(16).zip(dimension(20)).ok_or("the renderer produced no image")?;
    Ok(json!({"page": page, "pageCount": count, "scale": scale, "width": width, "height": height, "png_base64": STANDARD.encode(png)}))
}
