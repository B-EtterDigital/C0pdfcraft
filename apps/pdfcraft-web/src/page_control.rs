//! Opt-in page transport; native tests compile the same request validation (K-41).
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use serde_json::{Value, json};
#[cfg(test)]
mod tests;

#[cfg(target_arch = "wasm32")]
mod bridge;
#[cfg(target_arch = "wasm32")]
pub use bridge::{attach, runner};

pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;
pub const MAX_INLINE_PASSES: usize = 40;

pub fn parse(text: &str) -> Result<(String, Value), String> {
    if text.len() > pdfcraft_ui_egui::c0_host::MAX_REQUEST_BYTES {
        return Err("request exceeds 100 MiB".into());
    }
    let value: Value = serde_json::from_str(text).map_err(|e| format!("request is not JSON: {e}"))?;
    let method = value.get("method").and_then(Value::as_str).ok_or("request has no method")?;
    if !method.starts_with("c0.") && text.len() > MAX_REQUEST_BYTES {
        return Err("request exceeds 1 MiB".into());
    }
    let params = value.get("params").filter(|v| !v.is_null()).cloned().unwrap_or(json!({}));
    if !params.is_object() {
        return Err("params must be an object".into());
    }
    if method == "ui.drag" && params.get("steps").and_then(Value::as_u64).is_some_and(|n| n > 32) {
        return Err("steps exceeds 32".into());
    }
    Ok((method.to_owned(), params))
}

pub fn reads_only(method: &str) -> bool {
    matches!(method, "c0.inspect" | "c0.selection" | "c0.documentBytes" | "ui.render" | "ui.state" | "ui.inspect" | "ui.commands")
}

pub fn encode(reply: pdfcraft_ui_egui::control::Reply) -> String {
    match reply {
        Ok(result) => json!({"ok": true, "result": result}).to_string(),
        Err(error) => json!({"ok": false, "error": error}).to_string(),
    }
}
