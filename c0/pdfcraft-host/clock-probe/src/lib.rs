//! Test-only browser probe. Never linked into or shipped with PdfCraft.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use wasm_bindgen::prelude::*;

#[allow(dead_code)]
#[path = "../../../../crates/ui-egui/src/c0_clock.rs"]
mod clock;

/// The exact unsupported call previously used by combine_ui::ago. The browser test
/// deliberately catches its wasm trap in this isolated test instance.
#[wasm_bindgen]
pub fn previous_system_time() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

#[wasm_bindgen]
pub fn fixed_system_time() -> f64 {
    clock::system_time().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}
