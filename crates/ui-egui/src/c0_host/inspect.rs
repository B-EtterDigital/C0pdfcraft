use crate::PdfCraftApp;
use serde_json::{Value, json};

fn outline(app: &PdfCraftApp) -> Vec<Value> {
    let Some(doc) = app.active_ids().and_then(|(_, id)| app.session.get(id)) else { return Vec::new() };
    let mut pending: Vec<_> = doc.info.outline.iter().rev().map(|item| (0usize, item)).collect();
    let mut rows = Vec::new();
    // The engine has already bounded outline traversal; also bound the host response.
    while let Some((level, item)) = pending.pop() {
        if rows.len() >= 10_000 {
            break;
        }
        rows.push(json!({"level": level, "title": item.title, "page": item.page}));
        if level < 64 {
            pending.extend(item.children.iter().rev().map(|child| (level.saturating_add(1), child)));
        }
    }
    rows
}

pub(super) fn inspect(app: &PdfCraftApp, ctx: &egui::Context) -> Value {
    let view = app.active.and_then(|i| app.views.get(i));
    let doc = view.and_then(|view| app.session.get(view.id));
    let dirty = app.active.is_some_and(|i| app.has_unsaved_work(i));
    let documents: Vec<_> = app
        .views
        .iter()
        .enumerate()
        .filter_map(|(i, view)| {
            let doc = app.session.get(view.id)?;
            Some(json!({"name": doc.name, "dirty": app.has_unsaved_work(i), "active": app.active == Some(i)}))
        })
        .collect();
    json!({
        "name": doc.map(|d| d.name.as_str()), "revision": doc.map_or(0, |d| d.edit_generation()),
        "dirty": dirty, "canUndo": doc.is_some_and(|d| d.can_undo().is_some()),
        "canRedo": doc.is_some_and(|d| d.can_redo().is_some()),
        "pageCount": doc.map_or(0, |d| d.info.pages.len()), "currentPage": view.map(|v| v.current),
        "outline": outline(app), "annotationCount": doc.map_or(0, |d| d.info.annotations.len()),
        "formFieldCount": doc.map_or(0, |d| d.info.fields.len()), "documents": documents,
        "zoom": view.map_or(1.0, |v| v.zoom), "theme": if app.theme == crate::theme::ThemeKind::Dark {"dark"} else {"light"},
        "panes": {"chrome": app.chrome, "tools": app.chrome.tools_panel && app.left_open,
            "left": if app.left_open {Some(format!("{:?}", app.left))} else {None}, "right": app.right.map(|p| format!("{p:?}"))},
        "pendingReplacement": super::files::replacement_pending(ctx), "fileDialog": super::files::picker_pending(ctx),
        "needsConfirmation": app.close_request.is_some()
    })
}

pub(super) fn go_to(app: &mut PdfCraftApp, ctx: &egui::Context, params: &Value) -> Result<Value, String> {
    if app.modal_open() {
        return Err("dismiss the current dialog before navigating".into());
    }
    let (index, id) = app.active_ids().ok_or("no PDF open")?;
    let page = match (params.get("page"), params.get("outline")) {
        (Some(page), None) => page.as_u64().and_then(|v| usize::try_from(v).ok()).ok_or("page must be a zero-based integer")?,
        (None, Some(row)) => {
            let row = row.as_u64().and_then(|v| usize::try_from(v).ok()).ok_or("outline must be a zero-based integer")?;
            outline(app)
                .get(row)
                .and_then(|v| v.get("page"))
                .and_then(Value::as_u64)
                .and_then(|v| usize::try_from(v).ok())
                .ok_or("outline entry has no page or does not exist")?
        }
        _ => return Err("provide exactly one of page or outline".into()),
    };
    let doc = app.session.get(id).ok_or("no PDF open")?;
    if page >= doc.info.pages.len() {
        return Err(format!("page must be less than {}", doc.info.pages.len()));
    }
    if !app.process_pending_edits() {
        return Err("the current edit could not be committed".into());
    }
    let view = app.views.get_mut(index).ok_or("document view closed")?;
    view.go_to_page(page);
    ctx.request_repaint();
    Ok(json!({"page": page}))
}
