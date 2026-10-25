//! `c0.selection` and `c0.place`: what the C0VIBE AI actions read from the PDF and put back.
//!
//! Not upstream's: part of the C0VIBE host shell (`c0/pdfcraft-host/README.md`). The placement goes
//! through the app's own `apply_edit` with the engine's `AddAnnotation` edit, so history, the dirty
//! flag and saving behave as for a user comment, and one Ctrl+Z removes all of it.
//!
//! | Method | Params | Result |
//! |---|---|---|
//! | `c0.selection` | `{}` | `{kind:"text", page, pageCount, selectedText?, pageText, documentTitle}` |
//! | `c0.place` | `{note, page?, title?}` | `{ok:true, placed:"note", page, id, index, text}` |

use crate::PdfCraftApp;
use serde_json::{Value, json};

/// The most page text a selection reply carries, in characters.
pub const MAX_PAGE_TEXT_CHARS: usize = 20_000;
const MAX_NOTE_CHARS: usize = 20_000;
const MAX_TITLE_CHARS: usize = 200;
/// Distance of a placed note from the page's top-left corner, in points.
const MARGIN: f32 = 18.0;

pub fn methods() -> [&'static str; 2] {
    ["c0.selection", "c0.place"]
}

fn page_text(app: &PdfCraftApp, view: usize, page: usize) -> String {
    let Some(v) = app.views.get(view) else { return String::new() };
    let text = match v.page_text(page) {
        Some(t) => Some(t.plain_text()),
        None => app.session.get(v.id).and_then(|doc| {
            let config = pdfcraft_render::RenderConfig { password: doc.password.as_deref().map(std::sync::Arc::from), ..Default::default() };
            let mut r = pdfcraft_render::PageRenderer::new(doc.bytes.clone(), config);
            let req = pdfcraft_render::RenderRequest { page, kind: pdfcraft_render::RequestKind::Text, tile: None, scale: 1.0, tag: 0, preview: false };
            r.render(req).text.map(|t| t.plain_text())
        }),
    };
    text.unwrap_or_default().trim().chars().take(MAX_PAGE_TEXT_CHARS).collect()
}

pub(super) fn selection(app: &PdfCraftApp) -> Result<Value, String> {
    let (index, id) = app.active_ids().ok_or("no PDF open")?;
    let doc = app.session.get(id).ok_or("no PDF open")?;
    let view = app.views.get(index).ok_or("no PDF open")?;
    let count = doc.info.pages.len();
    let page = view.current.min(count.saturating_sub(1));
    let title = doc.info.title.as_deref().map(str::trim).filter(|t| !t.is_empty()).map_or_else(|| doc.name.clone(), str::to_string);
    let mut out = json!({
        "kind": "text", "page": page, "pageCount": count,
        "pageText": page_text(app, index, page), "documentTitle": title,
    });
    if let Some(text) = view.selected_text() {
        out["selectedText"] = json!(text);
    }
    Ok(out)
}

fn text_param(p: &Value, key: &str, max: usize) -> Result<Option<String>, String> {
    match p.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) if s.trim().is_empty() => Err(format!("`{key}` is empty")),
        Some(Value::String(s)) if s.chars().count() > max => Err(format!("`{key}` is longer than {max} characters")),
        Some(Value::String(s)) => Ok(Some(s.trim().to_string())),
        Some(_) => Err(format!("`{key}` must be text")),
    }
}

pub(super) fn place(app: &mut PdfCraftApp, params: &Value) -> Result<Value, String> {
    let note = text_param(params, "note", MAX_NOTE_CHARS)?.ok_or("`note` must be text: give the words to place")?;
    let title = text_param(params, "title", MAX_TITLE_CHARS)?;
    if app.modal_open() {
        return Err("dismiss the current dialog before placing".into());
    }
    let (index, id) = app.active_ids().ok_or("no PDF open")?;
    let (count, current) = {
        let doc = app.session.get(id).ok_or("no PDF open")?;
        (doc.info.pages.len(), app.views.get(index).map_or(0, |v| v.current))
    };
    let page = match params.get("page") {
        None | Some(Value::Null) => current.min(count.saturating_sub(1)),
        Some(v) => match v.as_u64().and_then(|n| usize::try_from(n).ok()) {
            Some(n) if n < count => n,
            _ => return Err(format!("`page` must be a zero-based page number below {count}")),
        },
    };
    let (at, reason) = {
        let doc = app.session.get(id).ok_or("no PDF open")?;
        let info = doc.info.pages.get(page).ok_or("the document has no such page")?;
        let p = info.view_to_user(MARGIN, MARGIN);
        ([f64::from(p[0]), f64::from(p[1])], doc.read_only_reason.clone())
    };
    if let Some(reason) = reason {
        return Err(format!("this PDF cannot be edited: {reason}"));
    }
    if !app.process_pending_edits() {
        return Err("the current edit could not be committed".into());
    }
    let contents = title.as_ref().map_or_else(|| note.clone(), |t| format!("{t}\n\n{note}"));
    let shape = pdfcraft_engine::Shape::Note { at, icon: pdfcraft_engine::NoteIcon::Comment };
    let style = pdfcraft_engine::Style::default_for(&shape);
    let author = app.comment_prefs.author.clone();
    let edit = pdfcraft_engine::Edit::AddAnnotation(pdfcraft_engine::NewAnnotation { page, shape, style, contents: contents.clone(), author });
    if !app.apply_edit(edit) {
        return Err("the note could not be added (the PDF may be protected or read-only)".into());
    }
    let doc = app.session.get(id).ok_or("no PDF open")?;
    let added = doc.info.annotations.iter().filter(|a| a.page == page).max_by_key(|a| a.index).ok_or("the note was not found after adding it")?;
    Ok(json!({"ok": true, "placed": "note", "page": page, "id": added.name, "index": added.index, "text": contents}))
}
