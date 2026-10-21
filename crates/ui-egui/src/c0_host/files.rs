//! Bounded byte snapshots, staged replacement and cancellable browser Open.
use super::MAX_FILE_BYTES;
use crate::PdfCraftApp;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

struct Prepared {
    name: String,
    size: usize,
    document: pdfcraft_engine::PreparedDocument,
    conversion_source: Option<Arc<Vec<u8>>>,
}

#[derive(Clone)]
struct Replacement {
    old: Option<pdfcraft_engine::DocId>,
    // None means upstream's blank-document operation.
    prepared: Arc<Mutex<Option<Prepared>>>,
}

fn replacement_id() -> egui::Id {
    egui::Id::new("pdfcraft-host-replacement")
}
fn picker_id() -> egui::Id {
    egui::Id::new("pdfcraft-host-picker")
}
fn enabled_id() -> egui::Id {
    egui::Id::new("pdfcraft-page-control")
}

pub fn enable_page(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(enabled_id(), true));
}
pub fn replacement_pending(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<Replacement>(replacement_id())).is_some()
}
pub fn picker_pending(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<Arc<AtomicBool>>(picker_id())).is_some_and(|flag| flag.load(Ordering::Relaxed))
}
pub fn cancel_picker(ctx: &egui::Context) -> bool {
    ctx.data_mut(|d| d.remove_temp::<Arc<AtomicBool>>(picker_id())).is_some_and(|flag| flag.swap(false, Ordering::Relaxed))
}

pub(super) fn document_bytes(app: &PdfCraftApp, params: &Value) -> Result<Value, String> {
    if params.get("format").is_some_and(|v| v.as_str() != Some("pdf")) {
        return Err("format must be pdf or omitted".into());
    }
    let (_, id) = app.active_ids().ok_or("no PDF open")?;
    let doc = app.session.get(id).ok_or("no PDF open")?;
    let bytes = app.session.save_bytes(id).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err("document exceeds the 72 MiB file limit".into());
    }
    let name = if doc.name.to_ascii_lowercase().ends_with(".pdf") { doc.name.clone() } else { format!("{}.pdf", doc.name) };
    Ok(json!({"name": name, "format": "pdf", "bytes": STANDARD.encode(bytes.as_slice()), "revision": doc.edit_generation()}))
}

pub(super) fn open(app: &mut PdfCraftApp, ctx: &egui::Context, params: &Value) -> Result<Value, String> {
    let name = params.get("name").and_then(Value::as_str).filter(|s| !s.trim().is_empty()).ok_or("name is required")?;
    if name.len() > 255 || name.contains(['/', '\\', '\0']) || matches!(name, "." | "..") {
        return Err("name must be a file basename of at most 255 bytes".into());
    }
    let text = params.get("bytes").and_then(Value::as_str).ok_or("bytes must be standard padded base64")?;
    if text.len() > MAX_FILE_BYTES.div_ceil(3).saturating_mul(4) {
        return Err("file exceeds the 72 MiB file limit".into());
    }
    let bytes = STANDARD.decode(text).map_err(|_| "bytes must be standard padded base64")?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err("file exceeds the 72 MiB file limit".into());
    }
    let bytes = Arc::new(bytes);
    let conversion_source =
        matches!(pdfcraft_engine::source_kind(name, &bytes), Some(pdfcraft_engine::SourceKind::Image | pdfcraft_engine::SourceKind::Text))
            .then(|| bytes.clone());
    let bytes = app.session.as_pdf(name, &bytes).map_err(|e| e.to_string())?;
    let document = app.session.prepare_open(name, None, bytes.clone(), None).map_err(|e| e.to_string())?;
    // Preparation succeeds before we ask about or close the existing document. Encrypted
    // documents without an empty password fail here and remain available through normal Open.
    replace(app, ctx, Some(Prepared { name: name.to_owned(), size: bytes.len(), document, conversion_source }))
}

pub(super) fn new_document(app: &mut PdfCraftApp, ctx: &egui::Context, params: &Value) -> Result<Value, String> {
    if params.get("template").is_some_and(|v| !matches!(v.as_str(), Some("" | "blank"))) {
        return Err("template must be blank or omitted".into());
    }
    replace(app, ctx, None)
}

fn replace(app: &mut PdfCraftApp, ctx: &egui::Context, prepared: Option<Prepared>) -> Result<Value, String> {
    if replacement_pending(ctx) {
        return Err("a replacement is awaiting Save Changes".into());
    }
    if app.modal_open() {
        return Err("dismiss the current dialog before replacing the document".into());
    }
    if !app.process_pending_edits() {
        return Err("the current edit could not be committed".into());
    }
    cancel_picker(ctx);
    let old = app.active_ids().map(|(_, id)| id);
    let replacement = Replacement { old, prepared: Arc::new(Mutex::new(prepared)) };
    if let Some(index) = app.active {
        app.request_close_tab(index);
        if app.close_request.is_some() {
            ctx.data_mut(|d| d.insert_temp(replacement_id(), replacement));
            ctx.request_repaint();
            return Ok(json!({"pending": true, "needsConfirmation": true}));
        }
    }
    finish(app, replacement)?;
    ctx.request_repaint();
    Ok(json!({"opened": true, "pending": false}))
}

fn finish(app: &mut PdfCraftApp, replacement: Replacement) -> Result<(), String> {
    let prepared = replacement.prepared.lock().map_err(|_| "replacement lock is unavailable")?.take();
    app.startup_superseded = true;
    if let Some(prepared) = prepared {
        if let Some(bytes) = prepared.conversion_source {
            // Upstream's conversion path names the resulting PDF and marks it unsaved.
            app.open_bytes(&prepared.name, None, bytes.as_ref().clone())?;
        } else {
            app.finish_open(&prepared.name, None, prepared.size, prepared.document, true)?;
        }
    } else {
        let before = app.views.len();
        app.create_blank();
        if app.views.len() == before {
            return Err("blank document creation failed; see the notice".into());
        }
    }
    Ok(())
}

/// Runs on logic passes too. Cancel keeps the old id; Discard/Save removes it.
pub fn poll(app: &mut PdfCraftApp, ctx: &egui::Context) {
    let Some(replacement) = ctx.data(|d| d.get_temp::<Replacement>(replacement_id())) else { return };
    if app.close_request.is_some() {
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    if app.pickers.pending() || app.pending_overwrite.is_some() {
        return;
    }
    ctx.data_mut(|d| d.remove::<Replacement>(replacement_id()));
    if replacement.old.is_none_or(|id| app.session.get(id).is_none()) {
        if let Err(error) = finish(app, replacement) {
            app.notify_fmt("Couldn't open document: {e}", &[("e", &error)]);
        }
        ctx.request_repaint();
    }
}

/// Intercept Open only for the opt-in page, preserving the upstream picker elsewhere.
pub fn page_picker(app: &mut PdfCraftApp) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = app;
        false
    }
    #[cfg(target_arch = "wasm32")]
    {
        let Some(ctx) = app.ctx.clone().filter(|ctx| ctx.data(|d| d.get_temp::<bool>(enabled_id())).unwrap_or(false)) else { return false };
        cancel_picker(&ctx);
        let flag = Arc::new(AtomicBool::new(true));
        ctx.data_mut(|d| d.insert_temp(picker_id(), flag.clone()));
        let inbox = app.inbox.clone();
        let failed = app.failed_inbox.clone();
        app.startup_superseded = true;
        wasm_bindgen_futures::spawn_local(async move {
            let selected = rfd::AsyncFileDialog::new()
                .add_filter("PDF", &["pdf"])
                .add_filter("Images and text", &crate::create_ui::CONVERTIBLE)
                .pick_file()
                .await;
            if let Some(file) = selected {
                // Check before and after reading; cancelled late results never enter the inbox.
                if flag.load(Ordering::Relaxed) {
                    let result = if file.inner().size() > MAX_FILE_BYTES as f64 {
                        Err("file exceeds the 72 MiB file limit".to_owned())
                    } else {
                        wasm_bindgen_futures::JsFuture::from(file.inner().array_buffer())
                            .await
                            .map(|buffer| js_sys::Uint8Array::new(&buffer).to_vec())
                            .map_err(|e| format!("file read failed: {e:?}"))
                    };
                    if flag.swap(false, Ordering::Relaxed) {
                        match result {
                            Ok(bytes) if bytes.len() <= MAX_FILE_BYTES => {
                                if let Ok(mut queue) = inbox.lock() {
                                    queue.push((file.file_name(), bytes));
                                }
                            }
                            Ok(_) => {
                                if let Ok(mut queue) = failed.lock() {
                                    queue.push((file.file_name(), "file exceeds the 72 MiB file limit".into()));
                                }
                            }
                            Err(error) => {
                                if let Ok(mut queue) = failed.lock() {
                                    queue.push((file.file_name(), error));
                                }
                            }
                        }
                    }
                }
            }
            flag.store(false, Ordering::Relaxed);
            ctx.request_repaint();
        });
        true
    }
}
