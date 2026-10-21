use crate::{PdfCraftApp, theme::host};
use serde_json::{Value, json};

pub(super) fn set(app: &mut PdfCraftApp, ctx: &egui::Context, params: &Value) -> Result<Value, String> {
    for key in ["panels", "themeTokens"] {
        if params.get(key).is_some_and(|v| !v.is_object() && !(key == "themeTokens" && v.is_null())) {
            return Err(format!("{key} must be an object{}", if key == "themeTokens" { " or null" } else { "" }));
        }
    }
    let mut applied = Vec::new();
    let mut unknown: Vec<String> =
        params.as_object().into_iter().flatten().filter(|(k, _)| !matches!(k.as_str(), "panels" | "themeTokens")).map(|(k, _)| k.clone()).collect();
    let mut invalid = Vec::new();
    if let Some(tokens) = params.get("themeTokens") {
        let report = host::set(ctx, tokens)?;
        applied.extend(report.applied.into_iter().map(|k| format!("themeTokens.{k}")));
        unknown.extend(report.unknown.into_iter().map(|k| format!("themeTokens.{k}")));
        invalid.extend(report.invalid.into_iter().map(|k| format!("themeTokens.{k}")));
    }
    for (key, value) in params.get("panels").and_then(Value::as_object).into_iter().flatten() {
        let target = match key.as_str() {
            "title_bar" => &mut app.chrome.title_bar,
            "app_bar" => &mut app.chrome.app_bar,
            "tab_strip" => &mut app.chrome.tab_strip,
            "start_screen" => &mut app.chrome.start_screen,
            "community_links" => &mut app.chrome.community_links,
            "right_rail" => &mut app.chrome.right_rail,
            "tools_panel" => &mut app.chrome.tools_panel,
            _ => {
                unknown.push(format!("panels.{key}"));
                continue;
            }
        };
        if let Some(value) = value.as_bool() {
            *target = value;
            applied.push(format!("panels.{key}"));
        } else {
            invalid.push(format!("panels.{key}"));
        }
    }
    ctx.request_repaint();
    Ok(json!({"applied": applied, "unknown": unknown, "invalid": invalid}))
}
