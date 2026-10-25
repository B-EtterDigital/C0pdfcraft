//! The page's tool-strip modes (`toolbar`, `toolbar_double`, `toolbar_floating`), the right dock's
//! fold (`right_dock_collapsed`) and the frameless `ui.render`. Defaults are upstream's.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use egui_kittest::{Harness, kittest::Queryable};
use pdfcraft_ui_egui::{
    Mode, PdfCraftApp, RightPanel, c0_host,
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

fn set(h: &mut H, client: &ControlClient, key: &str, value: Value) -> Value {
    call(h, client, "c0.set", json!({"panels": {key: value}})).unwrap()
}

fn panes(h: &mut H, client: &ControlClient) -> Value {
    call(h, client, "c0.inspect", json!({})).unwrap()["panes"].clone()
}

const KEYS: [&str; 4] = ["toolbar", "toolbar_double", "toolbar_floating", "right_dock_collapsed"];

#[test]
fn defaults_are_upstreams_and_inspect_reports_every_key() {
    let (mut h, client) = harness(2, |_| {});
    let panes = panes(&mut h, &client);
    let chrome = &panes["chrome"];
    assert_eq!(chrome["toolbar"], true);
    for key in ["toolbar_double", "toolbar_floating", "right_dock_collapsed"] {
        assert_eq!(chrome[key], false, "{key}");
    }
    for key in ["title_bar", "app_bar", "tab_strip", "start_screen", "community_links", "right_rail", "tools_panel"] {
        assert_eq!(chrome[key], true, "{key} keeps its default");
    }
    assert_eq!(panes["strip"], json!({"visible": true, "columns": 1, "floating": false, "pos": null}));
    // Upstream's place: 14 points in from the document area's corner, one column, no grip.
    let bar = h.ctx.memory(|m| m.area_rect(egui::Id::unique("quick-bar"))).expect("the strip is drawn");
    assert!(h.query_by_label("Move toolbar").is_none());
    let hand = h.get_by_label("Hand (H)").rect();
    let select = h.get_by_label("Select (V)").rect();
    assert_eq!(select.min.x, hand.min.x, "one column");
    assert!(hand.min.y > select.min.y);
    assert!(bar.min.x >= 14.0 && bar.min.y >= 14.0);
}

#[test]
fn every_key_sets_reads_back_and_is_idempotent() {
    let (mut h, client) = harness(1, |_| {});
    for key in KEYS {
        let want = key != "toolbar";
        for _ in 0..2 {
            let reply = set(&mut h, &client, key, json!(want));
            assert_eq!(reply, json!({"applied": [format!("panels.{key}")], "unknown": [], "invalid": []}));
            assert_eq!(panes(&mut h, &client)["chrome"][key], want, "{key}");
        }
        set(&mut h, &client, key, json!(!want));
        assert_eq!(panes(&mut h, &client)["chrome"][key], !want, "{key} back to its default");
    }
    // All at once, with one bad value and one unknown key reported rather than applied.
    let reply = call(
        &mut h,
        &client,
        "c0.set",
        json!({"panels": {"toolbar_double": true, "toolbar_floating": true, "right_dock_collapsed": "yes", "toolbar_triple": true}}),
    )
    .unwrap();
    assert_eq!(reply["applied"], json!(["panels.toolbar_double", "panels.toolbar_floating"]));
    assert_eq!(reply["invalid"], json!(["panels.right_dock_collapsed"]));
    assert_eq!(reply["unknown"], json!(["panels.toolbar_triple"]));
    let chrome = panes(&mut h, &client)["chrome"].clone();
    assert_eq!(
        (chrome["toolbar_double"].clone(), chrome["toolbar_floating"].clone(), chrome["right_dock_collapsed"].clone()),
        (json!(true), json!(true), json!(false))
    );
}

#[test]
fn toolbar_switch_hides_and_shows_the_strip() {
    let (mut h, client) = harness(1, |_| {});
    assert!(h.query_by_label("Select (V)").is_some());
    set(&mut h, &client, "toolbar", json!(false));
    h.run_steps(2);
    assert!(h.query_by_label("Select (V)").is_none() && h.query_by_label("Hand (H)").is_none());
    assert_eq!(panes(&mut h, &client)["strip"]["visible"], false);
    set(&mut h, &client, "toolbar", json!(true));
    h.run_steps(2);
    assert!(h.query_by_label("Select (V)").is_some());
    assert_eq!(panes(&mut h, &client)["strip"]["visible"], true);
}

fn columns_of(h: &H) -> usize {
    let select = h.get_by_label("Select (V)").rect();
    let strip = h.query_all_by_label("Fill & Sign").map(|n| n.rect()).filter(|r| r.min.y > select.min.y && (r.min.x - select.min.x).abs() < 80.0);
    let mut xs: Vec<i32> = [select, h.get_by_label("Hand (H)").rect()].into_iter().chain(strip).map(|r| r.min.x.round() as i32).collect();
    xs.sort_unstable();
    xs.dedup();
    xs.len()
}

#[test]
fn double_puts_the_strip_in_two_columns_and_back() {
    let (mut h, client) = harness(1, |_| {});
    assert_eq!(columns_of(&h), 1);
    let first = h.get_by_label("Select (V)").rect().min;
    let single_height = h.ctx.memory(|m| m.area_rect(egui::Id::unique("quick-bar"))).unwrap().height();
    set(&mut h, &client, "toolbar_double", json!(true));
    h.run_steps(3);
    assert_eq!(columns_of(&h), 2, "buttons wrap into a second column");
    let bar = h.ctx.memory(|m| m.area_rect(egui::Id::unique("quick-bar"))).unwrap();
    assert!(bar.height() < single_height, "the two columns are shorter than one");
    assert_eq!(h.get_by_label("Select (V)").rect().min, first, "the first button stays where it was");
    assert_eq!(panes(&mut h, &client)["strip"]["columns"], 2);
    set(&mut h, &client, "toolbar_double", json!(false));
    h.run_steps(3);
    assert_eq!(columns_of(&h), 1);
}

#[test]
fn floating_strip_has_a_grip_that_drags_it_within_the_document_area() {
    let (mut h, client) = harness(1, |_| {});
    let home = h.get_by_label("Select (V)").rect().min;
    set(&mut h, &client, "toolbar_floating", json!(true));
    h.run_steps(3);
    assert_eq!(h.get_by_label("Select (V)").rect().min, home, "floating starts where the docked strip sat");
    let grip = h.get_by_label("Move toolbar").rect();
    let reply = call(
        &mut h,
        &client,
        "ui.drag",
        json!({"from": [grip.center().x, grip.center().y], "to": [grip.center().x + 80.0, grip.center().y + 60.0], "steps": 4}),
    )
    .unwrap();
    assert!(reply.is_object());
    h.run_steps(3);
    let moved = h.get_by_label("Select (V)").rect().min;
    assert!((moved.x - home.x - 80.0).abs() < 2.0 && (moved.y - home.y - 60.0).abs() < 2.0, "{home:?} -> {moved:?}");
    let pos = panes(&mut h, &client)["strip"]["pos"].clone();
    assert!(pos[0].as_f64().is_some_and(|x| (x - 80.0).abs() < 2.0), "{pos}");
    // Far past the corner: kept inside the document area.
    let grip = h.get_by_label("Move toolbar").rect();
    call(
        &mut h,
        &client,
        "ui.drag",
        json!({"from": [grip.center().x, grip.center().y], "to": [grip.center().x + 5000.0, grip.center().y + 5000.0], "steps": 4}),
    )
    .unwrap();
    h.run_steps(3);
    let low = h.get_by_label("Select (V)").rect().min;
    assert!(low.x < 1280.0 && low.y < 900.0);
    // Back to docked: the strip returns to its corner, the place is kept for the next float.
    set(&mut h, &client, "toolbar_floating", json!(false));
    h.run_steps(3);
    assert_eq!(h.get_by_label("Select (V)").rect().min, home);
    assert!(h.query_by_label("Move toolbar").is_none());
    assert!(panes(&mut h, &client)["strip"]["pos"].is_array());
}

#[test]
fn floating_and_double_combine() {
    let (mut h, client) = harness(1, |_| {});
    call(&mut h, &client, "c0.set", json!({"panels": {"toolbar_floating": true, "toolbar_double": true}})).unwrap();
    h.run_steps(3);
    assert_eq!(columns_of(&h), 2);
    assert!(h.query_by_label("Move toolbar").is_some());
}

fn pages_panel_shown(h: &H) -> bool {
    h.query_all_by_label("Pages").count() > 0
}

#[test]
fn right_dock_folds_to_the_rail_and_an_icon_opens_it_again() {
    let (mut h, client) = harness(2, |app| app.right = Some(RightPanel::Pages));
    assert!(pages_panel_shown(&h), "the Pages panel is open");
    let saved = h.state().side_panels;
    assert_eq!(set(&mut h, &client, "right_dock_collapsed", json!(true))["applied"], json!(["panels.right_dock_collapsed"]));
    h.run_steps(3);
    assert!(!pages_panel_shown(&h), "folded");
    assert!(h.query_by_label("Bookmarks").is_some(), "the rail keeps its icons");
    assert_eq!(h.state().right, Some(RightPanel::Pages), "the open pane is remembered");
    assert_eq!(h.state().side_panels, saved, "the saved layout is untouched");
    assert_eq!(panes(&mut h, &client)["chrome"]["right_dock_collapsed"], true);
    // A different icon opens its pane and unfolds.
    h.get_by_label("Bookmarks").click();
    h.run_steps(4);
    assert_eq!(h.state().right, Some(RightPanel::Bookmarks));
    assert_eq!(panes(&mut h, &client)["chrome"]["right_dock_collapsed"], false);
    // Folded again: the icon of the pane that is open opens it rather than closing it.
    set(&mut h, &client, "right_dock_collapsed", json!(true));
    h.run_steps(3);
    assert_eq!(h.state().right, Some(RightPanel::Bookmarks));
    h.get_by_label("Bookmarks").click();
    h.run_steps(4);
    assert_eq!(h.state().right, Some(RightPanel::Bookmarks));
    assert_eq!(panes(&mut h, &client)["chrome"]["right_dock_collapsed"], false);
}

#[test]
fn folding_keeps_the_document_view_and_read_mode_side_panel() {
    let (mut h, client) = harness(3, |app| {
        app.right = Some(RightPanel::Pages);
        app.mode = Mode::Read;
    });
    assert!(pages_panel_shown(&h), "Read mode shows the side panel");
    set(&mut h, &client, "right_dock_collapsed", json!(true));
    h.run_steps(3);
    assert!(!pages_panel_shown(&h), "folded in Read mode too");
    assert_eq!(call(&mut h, &client, "c0.inspect", json!({})).unwrap()["pageCount"], 3);
    set(&mut h, &client, "right_dock_collapsed", json!(false));
    h.run_steps(3);
    assert!(pages_panel_shown(&h));
}

fn decode(reply: &Value) -> image::RgbaImage {
    let bytes = STANDARD.decode(reply["png_base64"].as_str().unwrap()).unwrap();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    image::load_from_memory(&bytes).unwrap().to_rgba8()
}

#[test]
fn render_returns_a_page_png_without_presenting_a_frame() {
    let (mut h, client) = harness(3, |_| {});
    let before = call(&mut h, &client, "c0.inspect", json!({})).unwrap();
    // Straight through the handler: no frame is stepped, none is presented.
    let ctx = h.ctx.clone();
    let reply = c0_host::handle(h.state_mut(), &ctx, "ui.render", &json!({})).unwrap();
    assert_eq!((reply["page"].clone(), reply["pageCount"].clone(), reply["scale"].clone()), (json!(0), json!(3), json!(1.0)));
    let image = decode(&reply);
    assert_eq!((image.width(), image.height()), (200, 300));
    assert_eq!((reply["width"].clone(), reply["height"].clone()), (json!(200), json!(300)));
    assert_eq!(image.get_pixel(100, 150).0, [255, 255, 255, 255], "a blank page is white, with no window chrome round it");
    // Through the control channel, with page and scale.
    let reply = call(&mut h, &client, "ui.render", json!({"page": 2, "scale": 2.0})).unwrap();
    assert_eq!((reply["page"].clone(), reply["width"].clone(), reply["height"].clone()), (json!(2), json!(400), json!(600)));
    assert_eq!(decode(&reply).dimensions(), (400, 600));
    // The current page is the default.
    call(&mut h, &client, "c0.goTo", json!({"page": 1})).unwrap();
    assert_eq!(call(&mut h, &client, "ui.render", json!({"scale": 0.5})).unwrap()["page"], 1);
    // Nothing about the document or the interface changed.
    let mut after = call(&mut h, &client, "c0.inspect", json!({})).unwrap();
    after["currentPage"] = json!(0);
    assert_eq!(before, after);
}

#[test]
fn render_refuses_bad_requests() {
    let (mut h, client) = harness(2, |_| {});
    for params in [
        json!({"page": 2}),
        json!({"page": -1}),
        json!({"page": 0.5}),
        json!({"page": "0"}),
        json!({"scale": 0}),
        json!({"scale": 0.1}),
        json!({"scale": 17}),
        json!({"scale": "big"}),
        json!({"pages": [0]}),
    ] {
        assert!(call(&mut h, &client, "ui.render", params.clone()).is_err(), "{params}");
    }
    let (mut empty, client) = harness(0, |_| {});
    assert_eq!(call(&mut empty, &client, "ui.render", json!({})).unwrap_err(), "no PDF open");
    assert!(call(&mut empty, &client, "ui.nonexistent", json!({})).is_err());
}
