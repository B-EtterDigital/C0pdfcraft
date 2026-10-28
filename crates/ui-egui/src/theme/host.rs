//! Context-local palette overrides. Replacing the palette never changes the saved theme.
use super::Tokens;
use egui::Color32;
use serde_json::{Map, Value};
use std::sync::Arc;

#[derive(Default)]
pub struct Report {
    pub applied: Vec<String>,
    pub unknown: Vec<String>,
    pub invalid: Vec<String>,
}

fn store() -> egui::Id {
    egui::Id::unique("pdfcraft-host-theme")
}

pub fn set(ctx: &egui::Context, value: &Value) -> Result<Report, String> {
    if !value.is_object() && !value.is_null() {
        return Err("themeTokens must be an object or null".into());
    }
    let mut report = Report::default();
    let mut clean = Map::new();
    let mut scratch = Tokens::for_kind(super::ThemeKind::Light);
    for (key, value) in value.as_object().into_iter().flatten() {
        if !assign(&mut scratch, key, Color32::TRANSPARENT) {
            report.unknown.push(key.clone());
        } else if colour(value).is_some() {
            clean.insert(key.clone(), value.clone());
            report.applied.push(key.clone());
        } else {
            report.invalid.push(key.clone());
        }
    }
    ctx.data_mut(|data| {
        data.remove::<Arc<Value>>(store());
        if !clean.is_empty() {
            data.insert_temp(store(), Arc::new(Value::Object(clean)));
        }
    });
    super::apply(ctx, Tokens::get(ctx).kind);
    Ok(report)
}

pub(super) fn resolve(ctx: &egui::Context, mut tokens: Tokens) -> Tokens {
    if let Some(value) = ctx.data(|data| data.get_temp::<Arc<Value>>(store())) {
        for (key, value) in value.as_object().into_iter().flatten() {
            if let Some(color) = colour(value) {
                assign(&mut tokens, key, color);
            }
        }
    }
    tokens
}

fn colour(value: &Value) -> Option<Color32> {
    let hex = value.as_str()?.strip_prefix('#')?;
    if !matches!(hex.len(), 6 | 8) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |at| hex.get(at..at + 2).and_then(|s| u8::from_str_radix(s, 16).ok());
    Some(Color32::from_rgba_unmultiplied(byte(0)?, byte(2)?, byte(4)?, if hex.len() == 8 { byte(6)? } else { 255 }))
}

fn assign(tokens: &mut Tokens, key: &str, color: Color32) -> bool {
    macro_rules! colors {
        ($($field:ident),* $(,)?) => { match key { $(stringify!($field) => { tokens.$field = color; true },)* _ => false } };
    }
    colors!(
        titlebar,
        chrome,
        panel,
        pasteboard,
        card,
        border,
        divider,
        text,
        text_muted,
        text_faint,
        icon,
        hover,
        pressed,
        selected,
        accent,
        accent_text,
        accent_soft,
        field,
        badge_new,
        page_shadow
    )
}
