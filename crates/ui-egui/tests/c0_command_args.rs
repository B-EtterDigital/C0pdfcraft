//! `ui.command` forwards its `params` to the command: `page.rotate` takes `degrees`, a command that
//! takes no arguments runs with or without `params`, and a value it cannot use is refused.

use egui_kittest::Harness;
use pdfcraft_ui_egui::{
    PdfCraftApp,
    control::{ControlClient, Reply},
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

fn harness(pages: usize) -> (Harness<'static, PdfCraftApp>, ControlClient) {
    let slot = Arc::new(Mutex::new(None));
    let copy = slot.clone();
    let mut h = Harness::builder().with_size(egui::vec2(1280.0, 900.0)).build_eframe(move |cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        let bytes = app.session.create_blank(200.0, 300.0, pages).unwrap();
        app.open_bytes("original.pdf", None, bytes.as_ref().clone()).unwrap();
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

fn command(h: &mut Harness<'static, PdfCraftApp>, client: &ControlClient, params: Value) -> Reply {
    call(h, client, "ui.command", params)
}

/// The rotation of each page of the first document, in degrees.
fn rotations(h: &Harness<'static, PdfCraftApp>) -> Vec<i64> {
    let app = h.state();
    app.session.get(app.views[0].id).unwrap().info.pages.iter().map(|p| i64::from(p.rotation)).collect()
}

#[test]
fn page_rotate_takes_degrees_from_params() {
    let (mut h, client) = harness(3);
    assert!(command(&mut h, &client, json!({"id": "page.rotate", "params": {"degrees": 180}})).is_ok());
    assert_eq!(rotations(&h), [180, 0, 0], "the current page turns by the requested 180");

    assert!(command(&mut h, &client, json!({"id": "page.rotate", "params": {"degrees": 90}})).is_ok());
    assert_eq!(rotations(&h), [270, 0, 0], "a second request adds its own 90 to the page");

    assert!(command(&mut h, &client, json!({"id": "page.rotate", "params": {"degrees": -90}})).is_ok());
    assert_eq!(rotations(&h), [180, 0, 0], "negative degrees turn counter-clockwise");
}

#[test]
fn page_rotate_without_params_still_turns_a_quarter_clockwise() {
    let (mut h, client) = harness(3);
    assert!(command(&mut h, &client, json!({"id": "page.rotate"})).is_ok());
    assert_eq!(rotations(&h), [90, 0, 0]);
    assert!(command(&mut h, &client, json!({"id": "page.rotate", "params": null})).is_ok());
    assert_eq!(rotations(&h), [180, 0, 0]);
    assert!(command(&mut h, &client, json!({"id": "page.rotate", "params": {}})).is_ok());
    assert_eq!(rotations(&h), [270, 0, 0]);
}

#[test]
fn page_rotate_refuses_a_value_it_cannot_use_and_changes_nothing() {
    let (mut h, client) = harness(3);
    let bad = [
        json!({"id": "page.rotate", "params": {"degrees": 45}}),
        json!({"id": "page.rotate", "params": {"degrees": "180"}}),
        json!({"id": "page.rotate", "params": {"degrees": 90.5}}),
        json!({"id": "page.rotate", "params": {"degrees": null}}),
        json!({"id": "page.rotate", "params": {"angle": 90}}),
        json!({"id": "page.rotate", "params": {"degrees": 90, "pages": [2]}}),
        json!({"id": "page.rotate", "params": 180}),
    ];
    for request in bad {
        let err = command(&mut h, &client, request.clone()).unwrap_err();
        assert!(err.contains("page.rotate"), "{request}: {err}");
        assert_eq!(rotations(&h), [0, 0, 0], "{request} must not fall back to the default turn");
    }
}

#[test]
fn commands_without_arguments_run_with_or_without_empty_params() {
    let (mut h, client) = harness(3);
    let pages = |h: &Harness<'static, PdfCraftApp>| {
        let app = h.state();
        app.session.get(app.views[0].id).unwrap().info.pages.len()
    };
    assert_eq!(pages(&h), 3);
    assert!(command(&mut h, &client, json!({"id": "page.insert_blank"})).is_ok());
    assert_eq!(pages(&h), 4);
    assert!(command(&mut h, &client, json!({"id": "page.insert_blank", "params": {}})).is_ok());
    assert_eq!(pages(&h), 5);
    assert!(command(&mut h, &client, json!({"id": "page.insert_blank", "params": null})).is_ok());
    assert_eq!(pages(&h), 6);
}

#[test]
fn commands_without_arguments_refuse_params_they_cannot_use() {
    let (mut h, client) = harness(3);
    let err = command(&mut h, &client, json!({"id": "page.rotate_ccw", "params": {"degrees": 180}})).unwrap_err();
    assert!(err.contains("page.rotate_ccw takes no parameters"), "{err}");
    assert_eq!(rotations(&h), [0, 0, 0], "the refused command did not run");

    let err = command(&mut h, &client, json!({"id": "page.insert_blank", "params": {"count": 2}})).unwrap_err();
    assert!(err.contains("takes no parameters"), "{err}");
    let app = h.state();
    assert_eq!(app.session.get(app.views[0].id).unwrap().info.pages.len(), 3);
}

#[test]
fn params_must_be_an_object() {
    let (mut h, client) = harness(3);
    let err = command(&mut h, &client, json!({"id": "page.rotate", "params": "180"})).unwrap_err();
    assert!(err.contains("params must be an object"), "{err}");
    assert_eq!(rotations(&h), [0, 0, 0]);
}
