//! Borrow an unpainted logic pass from the actual WebRunner, as in GridCraft.
use eframe::wasm_bindgen::{JsCast as _, JsValue, closure::Closure};
use pdfcraft_ui_egui::{
    PdfCraftApp,
    control::{ControlClient, Reply},
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc::{Receiver, TryRecvError};

thread_local! {
    static RUNNER: RefCell<Option<eframe::WebRunner>> = const {RefCell::new(None)};
    static PUMP: RefCell<Option<Pump>> = const {RefCell::new(None)};
}

pub fn runner(runner: &eframe::WebRunner) {
    RUNNER.with(|slot| *slot.borrow_mut() = Some(runner.clone()));
}

#[derive(Clone)]
struct Pump {
    target: web_sys::EventTarget,
    ran: Rc<Cell<bool>>,
}

impl Pump {
    fn get() -> Option<Self> {
        if let Some(pump) = PUMP.with(|slot| slot.borrow().clone()) {
            return Some(pump);
        }
        let runner = RUNNER.with(|slot| slot.borrow().clone())?;
        let pump = Self { target: web_sys::EventTarget::new().ok()?, ran: Rc::default() };
        let ran = pump.ran.clone();
        runner
            .add_event_listener::<web_sys::Event>(&pump.target, "pdfcraft-control-pass", move |_, runner| {
                runner.logic();
                ran.set(true);
            })
            .ok()?;
        PUMP.with(|slot| *slot.borrow_mut() = Some(pump.clone()));
        Some(pump)
    }
    fn pass(&self) -> bool {
        self.ran.set(false);
        if let Ok(event) = web_sys::Event::new("pdfcraft-control-pass") {
            let _ = self.target.dispatch_event(&event);
        }
        self.ran.get()
    }
}

pub fn attach(app: &mut PdfCraftApp, ctx: &egui::Context) {
    pdfcraft_ui_egui::c0_host::files::enable_page(ctx);
    app.chrome.title_bar = false;
    app.chrome.tab_strip = false;
    app.chrome.start_screen = false;
    app.chrome.community_links = false;
    let client = app.attach_control(ctx);
    install(client, ctx.clone());
}

fn install(client: ControlClient, ctx: egui::Context) {
    let Some(window) = web_sys::window() else { return };
    let call = Closure::<dyn FnMut(JsValue) -> js_sys::Promise>::new(move |text: JsValue| {
        let request = text.as_string().ok_or_else(|| "request must be JSON text".to_owned()).and_then(|text| super::parse(&text));
        let (method, params) = match request {
            Ok(request) => request,
            Err(error) => return js_sys::Promise::resolve(&JsValue::from_str(&super::encode(Err(error)))),
        };
        let rx = client.send(&method, params);
        let pump = Pump::get();
        if pump.as_ref().is_some_and(Pump::pass) {
            if matches!(method.as_str(), "ui.click" | "ui.focus" | "ui.key" | "ui.type" | "ui.drag" | "ui.move") {
                for _ in 0..super::MAX_INLINE_PASSES {
                    if !pump.as_ref().is_some_and(Pump::pass) {
                        break;
                    }
                }
            }
            if !super::reads_only(&method) {
                ctx.request_repaint();
            }
        }
        wasm_bindgen_futures::future_to_promise(wait(rx, pump))
    });
    if js_sys::Reflect::set(&window, &JsValue::from_str("__pdfcraftControl"), call.as_ref().unchecked_ref()).is_ok() {
        call.forget();
    }
}

async fn wait(rx: Receiver<Reply>, pump: Option<Pump>) -> Result<JsValue, JsValue> {
    loop {
        match rx.try_recv() {
            Ok(reply) => return Ok(JsValue::from_str(&super::encode(reply))),
            Err(TryRecvError::Disconnected) => return Ok(JsValue::from_str(&super::encode(Err("PdfCraft dropped the request".into())))),
            Err(TryRecvError::Empty) => {
                let promise = js_sys::Promise::new(&mut |resolve, _| {
                    if let Some(window) = web_sys::window()
                        && window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 16).is_ok()
                    {
                        return;
                    }
                    let _ = resolve.call0(&JsValue::NULL);
                });
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                if let Some(pump) = &pump {
                    pump.pass();
                }
            }
        }
    }
}
