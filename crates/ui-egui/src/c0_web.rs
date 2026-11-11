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

/// The type a browser download of `name` is offered as. Zip is the one type this fork adds, as
/// upstream's table does not list it; every other name gets `upstream`'s answer (upstream's
/// `download_mime_type`, passed in so this module needs no crate root).
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn mime(name: &str, upstream: fn(&str) -> &'static str) -> &'static str {
    match name.rsplit('.').next().unwrap_or("").to_ascii_lowercase().as_str() {
        "zip" => "application/zip",
        _ => upstream(name),
    }
}
