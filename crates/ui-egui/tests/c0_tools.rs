//! The host's tool list (`c0.tools`), opening one by id (`c0.openTool`) and the tool open now
//! (`c0.inspect().tool`). Opening runs the same code a click on the All tools panel runs.
use egui_kittest::{Harness, kittest::Queryable};
use pdfcraft_engine::catalog::TOOL_GROUPS;
use pdfcraft_ui_egui::{
    Dialog, LeftPanel, PdfCraftApp,
    control::{ControlClient, Reply},
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

type H = Harness<'static, PdfCraftApp>;

fn harness(pages: usize, setup: impl FnOnce(&mut PdfCraftApp) + 'static) -> (H, ControlClient) {
    let slot = Arc::new(Mutex::new(None));
    let copy = slot.clone();
    let mut h = Harness::builder().with_size(egui::vec2(1280.0, 900.0)).build_eframe(move |cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        if pages > 0 {
            let bytes = app.session.create_blank(200.0, 300.0, pages).unwrap();
            app.open_bytes("original.pdf", None, bytes.as_ref().clone()).unwrap();
        }
        setup(&mut app);
        *copy.lock().unwrap() = Some(app.attach_control(&cc.egui_ctx));
        app
    });
    h.run_steps(4);
    (h, slot.lock().unwrap().take().unwrap())
}

fn call(h: &mut H, client: &ControlClient, method: &str, params: Value) -> Reply {
    let rx = client.send(method, params);
    for _ in 0..40 {
        h.step();
        if let Ok(reply) = rx.try_recv() {
            return reply;
        }
    }
    panic!("no reply for {method}");
}

fn tools(h: &mut H, client: &ControlClient) -> Value {
    call(h, client, "c0.tools", json!({})).unwrap()
}

fn open(h: &mut H, client: &ControlClient, id: &str) -> Value {
    call(h, client, "c0.openTool", json!({"id": id})).unwrap()
}

fn tool_now(h: &mut H, client: &ControlClient) -> Value {
    call(h, client, "c0.inspect", json!({})).unwrap()["tool"].clone()
}

fn set_tools_panel(h: &mut H, client: &ControlClient, shown: bool) {
    call(h, client, "c0.set", json!({"panels": {"tools_panel": shown}})).unwrap();
    h.run_steps(2);
}

/// Top edges of the drawn nodes with this label in the Tools column (the left edge of the window).
fn column_tops(h: &H, label: &str) -> Vec<f32> {
    h.query_all_by_label(label).map(|node| node.rect()).filter(|r| r.min.x < 300.0).map(|r| r.min.y).collect()
}

#[test]
fn tools_lists_the_panel_entries_in_order_with_unique_ids() {
    let (mut h, client) = harness(1, |_| {});
    let reply = tools(&mut h, &client);
    let rows = reply["tools"].as_array().unwrap();
    let ids: Vec<&str> = rows.iter().map(|r| r["id"].as_str().unwrap()).collect();
    let expected: Vec<&str> = TOOL_GROUPS.iter().map(|g| g.id).collect();
    assert_eq!(ids, expected, "every catalogue group, in the panel's order");
    let mut unique = ids.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), ids.len(), "ids are unique");
    assert_eq!(reply["active"], Value::Null);

    // Every group shows as a row of the expanded panel, in the same top-to-bottom order.
    h.get_by_label("View more").click();
    h.run_steps(3);
    let mut last = f32::NEG_INFINITY;
    for row in rows {
        let label = row["label"].as_str().unwrap();
        // The first drawn row with this label below the row before it.
        let top = column_tops(&h, label).into_iter().filter(|y| *y > last).fold(f32::INFINITY, f32::min);
        assert!(top.is_finite(), "the panel shows no row below the previous one labelled {label:?}");
        last = top;
    }
    // The optional group hint is the expanded view's category.
    assert_eq!(rows[0]["group"], "Create & Edit", "Export a PDF is filed under Create & Edit");
    assert!(rows.iter().all(|r| r["enabled"] == true && r["active"] == false));
}

#[test]
fn open_tool_matches_clicking_the_panel_row() {
    let (mut clicked, client_a) = harness(1, |_| {});
    clicked.get_by_label("Compress a PDF").click();
    clicked.run_steps(3);
    let (mut opened, client_b) = harness(1, |_| {});
    assert_eq!(open(&mut opened, &client_b, "compress"), json!({"opened": "compress"}));
    opened.run_steps(3);

    assert_eq!(format!("{:?}", clicked.state().left), format!("{:?}", opened.state().left));
    assert_eq!(clicked.state().left_open, opened.state().left_open);
    assert!(opened.state().left_open, "the Tools pane is open for the tool");
    assert_eq!(tool_now(&mut opened, &client_b), json!("compress"));
    assert_eq!(tool_now(&mut clicked, &client_a), json!("compress"), "the click reports the same tool");
    // The tool's own rows appear, as they do after a click.
    assert!(opened.query_by_label("Reduce file size").is_some());
    assert!(clicked.query_by_label("Reduce file size").is_some());
}

#[test]
fn open_tool_works_while_the_tools_panel_is_hidden() {
    let (mut h, client) = harness(1, |_| {});
    set_tools_panel(&mut h, &client, false);
    assert_eq!(call(&mut h, &client, "c0.inspect", json!({})).unwrap()["panes"]["tools"], false);

    // Prepare a form: the field editing it turns on is state the page draws, not the panel.
    assert_eq!(open(&mut h, &client, "form"), json!({"opened": "form"}));
    h.run_steps(2);
    assert!(h.state().is_preparing(), "fields are editable with the Tools pane hidden");
    assert_eq!(tool_now(&mut h, &client), json!("form"));
    assert!(h.query_by_label("Compress a PDF").is_none(), "the hidden pane is not drawn");
    assert_eq!(call(&mut h, &client, "c0.inspect", json!({})).unwrap()["panes"]["tools"], false, "still hidden");

    // Organize pages: the page grid and its toolbar come up on the canvas.
    assert_eq!(open(&mut h, &client, "organize"), json!({"opened": "organize"}));
    h.run_steps(2);
    assert!(h.state().views[0].organize, "the page grid is open");
    assert!(h.query_by_label("Save pages").is_some(), "the grid's toolbar is drawn");
    assert_eq!(tool_now(&mut h, &client), json!("organize"));
}

#[test]
fn unknown_or_missing_tool_ids_are_refused_and_change_nothing() {
    let (mut h, client) = harness(1, |_| {});
    for params in [json!({"id": "no_such_tool"}), json!({"id": "Compress a PDF"}), json!({"id": 7}), json!({})] {
        let reply = call(&mut h, &client, "c0.openTool", params.clone());
        let error = reply.expect_err(&format!("{params} is refused"));
        assert!(!error.is_empty() && !error.contains('<'), "a plain-text error: {error:?}");
    }
    assert_eq!(tool_now(&mut h, &client), Value::Null);
    assert_eq!(h.state().left, LeftPanel::AllTools, "the panel did not move");
}

#[test]
fn open_tool_is_refused_while_a_dialog_is_open() {
    let (mut h, client) = harness(1, |app| app.dialog = Some(Dialog::WordCount));
    let error = call(&mut h, &client, "c0.openTool", json!({"id": "compress"})).expect_err("refused behind a dialog");
    assert!(error.contains("dialog"), "{error}");
    assert_eq!(tool_now(&mut h, &client), Value::Null);
}

#[test]
fn inspect_reports_the_open_tool_and_the_flag_follows_it() {
    let (mut h, client) = harness(1, |_| {});
    assert_eq!(tool_now(&mut h, &client), Value::Null);
    open(&mut h, &client, "export");
    assert_eq!(tool_now(&mut h, &client), json!("export"));
    let reply = tools(&mut h, &client);
    assert_eq!(reply["active"], "export");
    let active: Vec<&Value> = reply["tools"].as_array().unwrap().iter().filter(|r| r["active"] == true).collect();
    assert_eq!(active.len(), 1, "exactly one row is active");
    assert_eq!(active[0]["id"], "export");

    open(&mut h, &client, "comment");
    assert_eq!(tool_now(&mut h, &client), json!("comment"));
    assert_eq!(tools(&mut h, &client)["active"], "comment");
}
