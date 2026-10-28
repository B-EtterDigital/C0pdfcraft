use base64::{Engine as _, engine::general_purpose::STANDARD};
use egui_kittest::{Harness, kittest::Queryable};
use pdfcraft_ui_egui::{
    PdfCraftApp, c0_host,
    control::{ControlClient, Reply},
    theme::{self, ThemeKind},
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

fn harness(pages: usize) -> (Harness<'static, PdfCraftApp>, ControlClient) {
    let slot = Arc::new(Mutex::new(None));
    let copy = slot.clone();
    let mut h = Harness::builder().with_size(egui::vec2(1280.0, 900.0)).build_eframe(move |cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        if pages > 0 {
            let bytes = app.session.create_blank(200.0, 300.0, pages).unwrap();
            app.open_bytes("original.pdf", None, bytes.as_ref().clone()).unwrap();
        }
        *copy.lock().unwrap() = Some(app.attach_control(&cc.egui_ctx));
        app
    });
    h.run_steps(3);
    let client = slot.lock().unwrap().take().unwrap();
    (h, client)
}

fn call(h: &mut Harness<'static, PdfCraftApp>, client: &ControlClient, method: &str, params: Value) -> Reply {
    let rx = client.send(method, params);
    for _ in 0..8 {
        h.step();
        if let Ok(reply) = rx.try_recv() {
            return reply;
        }
    }
    panic!("no reply for {method}");
}

#[test]
fn all_host_replies_empty_state_and_invalid_requests() {
    let (mut h, client) = harness(0);
    let state = call(&mut h, &client, "c0.inspect", json!({})).unwrap();
    assert_eq!(state["pageCount"], 0);
    assert_eq!(state["currentPage"], Value::Null);
    assert_eq!(state["documents"], json!([]));
    assert_eq!(state["revision"], 0);
    for key in ["dirty", "canUndo", "canRedo"] {
        assert_eq!(state[key], false);
    }
    assert_eq!(state["theme"], "light");
    for (method, params) in [
        ("c0.goTo", json!({"page": 0})),
        ("c0.documentBytes", json!({})),
        ("c0.noSuchMethod", json!({})),
        ("c0.newDocument", json!({"template": "missing"})),
        ("c0.openFile", json!({"name": "../a.pdf", "bytes": ""})),
        ("c0.openFile", json!({"name": "a.pdf", "bytes": "nonsense"})),
        ("c0.openFile", json!({"name": "a.pdf", "bytes": "eA=="})),
    ] {
        assert!(call(&mut h, &client, method, params).is_err(), "{method}");
    }
    assert_eq!(call(&mut h, &client, "c0.cancelFileDialog", json!({})).unwrap(), json!({"cancelled": false}));
    assert_eq!(call(&mut h, &client, "c0.newDocument", json!({})).unwrap()["opened"], true);
    let state = call(&mut h, &client, "c0.inspect", json!({})).unwrap();
    assert_eq!(state["pageCount"], 1);
    assert_eq!(state["dirty"], true);
    assert_eq!(state["documents"][0]["name"], "Untitled.pdf");
}

#[test]
fn bytes_round_trip_preserves_dirty_revision_history_and_other_tabs() {
    let (mut h, client) = harness(2);
    assert!(h.state_mut().apply_edit(pdfcraft_engine::Edit::AddBookmark {
        parent: vec![],
        index: 0,
        title: "Chapter".into(),
        page: 1,
        left: None,
        top: None
    }));
    let before = call(&mut h, &client, "c0.inspect", json!({})).unwrap();
    assert_eq!(before["canUndo"], true);
    assert_eq!(before["canRedo"], false);
    let exported = call(&mut h, &client, "c0.documentBytes", json!({"format": "pdf"})).unwrap();
    assert!(STANDARD.decode(exported["bytes"].as_str().unwrap()).unwrap().starts_with(b"%PDF-"));
    assert_eq!(before, call(&mut h, &client, "c0.inspect", json!({})).unwrap());
    h.state_mut().undo();
    assert_eq!(call(&mut h, &client, "c0.inspect", json!({})).unwrap()["canRedo"], true);
    h.state_mut().redo();
    assert!(call(&mut h, &client, "c0.documentBytes", json!({"format": "png"})).is_err());
    let before_id = h.state().views[0].id;
    // A second document stays open while the host replaces only its active tab.
    let bytes = h.state().session.create_blank(200.0, 300.0, 1).unwrap();
    h.state_mut().open_bytes("other.pdf", None, bytes.as_ref().clone()).unwrap();
    let result = call(&mut h, &client, "c0.openFile", json!({"name": "roundtrip.pdf", "bytes": exported["bytes"]})).unwrap();
    assert_eq!(result["pending"], false);
    assert_eq!(h.state().views.len(), 2);
    assert!(h.state().session.get(before_id).is_some());
    let state = call(&mut h, &client, "c0.inspect", json!({})).unwrap();
    assert_eq!(state["pageCount"], 2);
    assert_eq!(state["outline"], json!([{"level": 0, "title": "Chapter", "page": 1}]));
    assert_eq!(state["annotationCount"], 0);
    assert_eq!(state["formFieldCount"], 0);
    assert_eq!(state["dirty"], false);
}

#[test]
fn navigation_is_zero_based_validated_and_outline_aware() {
    let (mut h, client) = harness(3);
    h.state_mut().apply_edit(pdfcraft_engine::Edit::AddBookmark { parent: vec![], index: 0, title: "End".into(), page: 2, left: None, top: None });
    assert_eq!(call(&mut h, &client, "c0.goTo", json!({"page": 1})).unwrap()["page"], 1);
    assert_eq!(call(&mut h, &client, "c0.goTo", json!({"outline": 0})).unwrap()["page"], 2);
    for params in [json!({}), json!({"page": -1}), json!({"page": 3}), json!({"page": 1.5}), json!({"outline": 99}), json!({"page": 0,"outline": 0})]
    {
        assert!(call(&mut h, &client, "c0.goTo", params).is_err());
        assert_eq!(h.state().views[0].current, 2);
    }
}

#[test]
fn replacement_uses_existing_save_changes_cancel_discard_and_save() {
    for choice in [None, Some(false), Some(true)] {
        let (mut h, client) = harness(1);
        h.state_mut().apply_edit(pdfcraft_engine::Edit::AddBookmark {
            parent: vec![],
            index: 0,
            title: "Keep".into(),
            page: 0,
            left: None,
            top: None,
        });
        let id = h.state().views[0].id;
        let path = std::env::temp_dir().join(format!("c0pdf-save-{}-{}.pdf", std::process::id(), choice == Some(true)));
        h.state_mut().save_override = Some(path.to_string_lossy().into_owned());
        let result = call(&mut h, &client, "c0.newDocument", json!({"template": "blank"})).unwrap();
        assert_eq!(result["pending"], true);
        assert!(call(&mut h, &client, "c0.newDocument", json!({})).is_err());
        assert!(h.state().session.get(id).unwrap().dirty);
        h.get_by_label(match choice {
            None => "Cancel",
            Some(false) => "Don't save",
            Some(true) => "Save",
        })
        .click();
        h.run_steps(3);
        assert!(!c0_host::files::replacement_pending(&h.ctx));
        if choice.is_none() {
            assert_eq!(h.state().views[0].id, id);
            assert!(h.state().session.get(id).unwrap().dirty);
        } else {
            assert!(h.state().session.get(id).is_none());
            assert_eq!(h.state().session.get(h.state().views[0].id).unwrap().name, "Untitled.pdf");
        }
        if choice == Some(true) {
            assert!(std::fs::read(&path).unwrap().starts_with(b"%PDF-"));
            std::fs::remove_file(path).unwrap();
        }
    }
}

#[test]
fn panel_defaults_settings_validation_and_no_persistence() {
    let (mut h, client) = harness(0);
    let defaults = serde_json::to_value(h.state().chrome).unwrap();
    // Every switch that hides upstream chrome defaults to shown; the mode switches default to upstream's layout.
    for (key, value) in defaults.as_object().unwrap() {
        let off = matches!(key.as_str(), "toolbar_double" | "toolbar_floating" | "right_dock_collapsed");
        assert_eq!(*value, if key == "toolbar_pos" { Value::Null } else { json!(!off) }, "{key}");
    }
    assert!(h.query_by_label("Welcome to PdfCraft").is_some());
    let result = call(
        &mut h,
        &client,
        "c0.set",
        json!({"panels": {"start_screen": false, "tools_panel": false, "community_links": false, "wat": true, "right_rail": "no"}}),
    )
    .unwrap();
    assert_eq!(result["unknown"], json!(["panels.wat"]));
    assert_eq!(result["invalid"], json!(["panels.right_rail"]));
    assert!(h.query_by_label("Welcome to PdfCraft").is_none());
    let saved = h.state().persist();
    let mut app = PdfCraftApp::new();
    app.restore(&saved);
    assert_eq!(serde_json::to_value(app.chrome).unwrap(), defaults);
    let before = serde_json::to_value(h.state().chrome).unwrap();
    assert!(call(&mut h, &client, "c0.set", json!({"panels": {"start_screen": true}, "themeTokens": 3})).is_err());
    assert_eq!(serde_json::to_value(h.state().chrome).unwrap(), before);
}

#[test]
fn community_switch_removes_home_about_menu_palette_and_command_links() {
    let (mut h, client) = harness(0);
    assert!(h.query_by_label("Join the ArtCraft community").is_some());
    call(&mut h, &client, "c0.set", json!({"panels": {"community_links": false}})).unwrap();
    assert!(h.query_by_label("Join the ArtCraft community").is_none());
    assert!(h.query_by_label("Discord").is_none());
    for link in pdfcraft_engine::links::LINKS {
        assert!(call(&mut h, &client, "ui.command", json!({"id":link.command})).is_err());
    }
    h.get_by_label("Menu").click();
    h.run_steps(2);
    h.get_by_label("Help ⏵").hover();
    h.run_steps(3);
    for link in pdfcraft_engine::links::LINKS {
        assert!(h.query_by_label_contains(link.label).is_none());
    }
    call(&mut h, &client, "ui.key", json!({"key":"Escape"})).unwrap();
    call(&mut h, &client, "ui.key", json!({"key":"Escape"})).unwrap();
    h.state_mut().dialog = Some(pdfcraft_ui_egui::Dialog::About);
    h.run_steps(2);
    assert!(h.query_by_label("ArtCraft").is_none());
    assert!(h.query_by_label("PdfCraft").is_some());
    assert!(h.query_by_label("By the PdfCraft contributors (see Contributors).").is_some());
    for link in pdfcraft_engine::links::LINKS {
        assert!(h.query_by_label(link.label).is_none());
    }
    h.state_mut().dialog = None;
    h.state_mut().palette_open = true;
    h.state_mut().palette_query = "Discord".into();
    h.run_steps(2);
    assert!(h.query_by_label("Join the ArtCraft Discord").is_none());
}

#[test]
fn theme_all_twenty_colours_survive_mode_changes_and_clear() {
    let (mut h, client) = harness(0);
    let keys = [
        "titlebar",
        "chrome",
        "panel",
        "pasteboard",
        "card",
        "border",
        "divider",
        "text",
        "text_muted",
        "text_faint",
        "icon",
        "hover",
        "pressed",
        "selected",
        "accent",
        "accent_text",
        "accent_soft",
        "field",
        "badge_new",
        "page_shadow",
    ];
    let mut tokens: serde_json::Map<String, Value> = keys.iter().map(|k| (k.to_string(), json!("#123456"))).collect();
    tokens.insert("radius".into(), json!("#123456"));
    tokens.insert("nonexistent".into(), json!("#123456"));
    let result = call(&mut h, &client, "c0.set", json!({"themeTokens": tokens})).unwrap();
    assert_eq!(result["applied"].as_array().unwrap().len(), 20);
    assert_eq!(result["unknown"].as_array().unwrap().len(), 2);
    for kind in [ThemeKind::Dark, ThemeKind::Light] {
        theme::apply(&h.ctx, kind);
        assert_eq!(theme::Tokens::get(&h.ctx).accent, egui::Color32::from_rgb(0x12, 0x34, 0x56));
        assert_eq!(h.ctx.global_style().visuals.panel_fill, egui::Color32::from_rgb(0x12, 0x34, 0x56));
    }
    let result = call(&mut h, &client, "c0.set", json!({"themeTokens": {"accent": "#xyz", "card": "#01020380"}})).unwrap();
    assert_eq!(result["invalid"], json!(["themeTokens.accent"]));
    assert_eq!(theme::Tokens::get(&h.ctx).card, egui::Color32::from_rgba_unmultiplied(1, 2, 3, 128));
    call(&mut h, &client, "c0.set", json!({"themeTokens": null})).unwrap();
    assert_eq!(theme::Tokens::get(&h.ctx).accent, theme::Tokens::for_kind(h.state().theme).accent);
}

#[test]
fn every_request_can_be_answered_by_unpresented_pass() {
    let (mut h, client) = harness(1);
    let rx = client.send("c0.inspect", json!({}));
    // Harness logic/layout has no renderer or presentation; replies need no screenshot/frame.
    h.step();
    assert_eq!(rx.try_recv().unwrap().unwrap()["pageCount"], 1);
}

#[test]
fn text_conversion_uses_upstream_unsaved_document_semantics() {
    let (mut h, client) = harness(0);
    call(&mut h, &client, "c0.openFile", json!({"name":"notes.txt", "bytes":STANDARD.encode(b"Original synthetic text")})).unwrap();
    let state = call(&mut h, &client, "c0.inspect", json!({})).unwrap();
    assert_eq!(state["name"], "notes.pdf");
    assert_eq!(state["dirty"], true);
    assert_eq!(state["pageCount"], 1);
    let before = state.clone();
    assert!(call(&mut h, &client, "c0.openFile", json!({"name":"bad.pdf", "bytes":"eA=="})).is_err());
    assert_eq!(call(&mut h, &client, "c0.inspect", json!({})).unwrap(), before);
}
