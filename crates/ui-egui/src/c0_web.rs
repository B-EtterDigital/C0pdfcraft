//! Browser fixes independent of the opt-in host transport. Suitable upstream as is.

#[path = "c0_clock.rs"]
mod clock;
pub(crate) use clock::system_time;
#[cfg(target_arch = "wasm32")]
use clock::{epoch_seconds, utc_offset};

pub fn session() -> pdfcraft_engine::Session {
    let session = pdfcraft_engine::Session::new();
    #[cfg(target_arch = "wasm32")]
    let session = session.with_wall_clock(epoch_seconds, utc_offset);
    session
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn mime(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap_or("").to_ascii_lowercase().as_str() {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "txt" => "text/plain;charset=utf-8",
        "csv" => "text/csv;charset=utf-8",
        "html" | "htm" => "text/html;charset=utf-8",
        "xml" | "xfdf" => "application/xml",
        "fdf" => "application/vnd.fdf",
        "json" => "application/json",
        "zip" => "application/zip",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }
}
