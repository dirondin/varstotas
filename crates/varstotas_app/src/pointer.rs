//! What the pointer is (a mouse, a finger, a pen): eframe does not say, so the web build
//! listens to the page's pointer events itself, ahead of eframe (on the window, capturing). The
//! pen's hover and its eraser come from here too: eframe passes neither on. Elsewhere it stays
//! a mouse (touch screens give `egui::Event::Touch`, pens move the mouse); the UI scripts set
//! it with `raw pen`.

use std::cell::{Cell, RefCell};

use egui::Pos2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Mouse,
    Touch,
    Pen,
}

/// What the pen did (screen points). The view takes the pen from here, not from eframe's mouse
/// (in a browser that follows the first touch: a palm resting on the screen).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PenEvent {
    Down(Pos2),
    Move(Pos2),
    Up(Pos2),
    /// Above the screen, not touching it.
    Hover(Pos2),
    /// Gone out of reach.
    Leave,
    /// The eraser end.
    EraseDown(Pos2),
    EraseMove(Pos2),
    EraseUp(Pos2),
}

thread_local! {
    static KIND: Cell<Kind> = const { Cell::new(Kind::Mouse) };
    /// Where the pen touches the screen (screen points).
    static PEN_DOWN: Cell<Option<Pos2>> = const { Cell::new(None) };
    static PEN_EVENTS: RefCell<Vec<PenEvent>> = const { RefCell::new(Vec::new()) };
}

/// The kind of the pointer last pressed or moved.
pub fn kind() -> Kind {
    KIND.get()
}

/// Where the pen touches the screen, while it does.
pub fn pen_down() -> Option<Pos2> {
    PEN_DOWN.get()
}

/// The pen's hover and eraser since the last call.
pub fn take_pen_events() -> Vec<PenEvent> {
    PEN_EVENTS.with_borrow_mut(std::mem::take)
}

pub fn set_kind(kind: Kind) {
    KIND.set(kind);
}

pub fn set_pen_down(at: Option<Pos2>) {
    PEN_DOWN.set(at);
}

pub fn push_pen(e: PenEvent) {
    PEN_EVENTS.with_borrow_mut(|v| v.push(e));
}

/// Listen to the page's pointer events, ahead of eframe's own listeners.
#[cfg(target_arch = "wasm32")]
pub fn install(ctx: egui::Context) {
    use wasm_bindgen::JsCast;
    let Some(window) = web_sys::window() else { return };
    let canvas = window.document().and_then(|d| d.get_element_by_id("varstotas"));
    let on_pointer = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |e: web_sys::PointerEvent| {
        let kind = match e.pointer_type().as_str() {
            "pen" => Kind::Pen,
            "touch" => Kind::Touch,
            _ => Kind::Mouse,
        };
        let ty = e.type_();
        // a finger lifted says nothing of the next pointer; the others are what they were
        if ty != "pointerout" {
            KIND.set(kind);
        }
        if kind != Kind::Pen {
            return;
        }
        // as eframe places the mouse: from the canvas's corner, in points
        let pos = {
            let (x, y) = canvas.as_ref().map_or((0.0, 0.0), |c| {
                let r = c.get_bounding_client_rect();
                (r.left(), r.top())
            });
            let zoom = ctx.zoom_factor();
            Pos2::new((e.client_x() as f64 - x) as f32 / zoom, (e.client_y() as f64 - y) as f32 / zoom)
        };
        const ERASER: u16 = 32;
        let erasing = e.buttons() & ERASER != 0 || e.button() == 5;
        match ty.as_str() {
            "pointerdown" if erasing => push_pen(PenEvent::EraseDown(pos)),
            "pointerdown" => {
                PEN_DOWN.set(Some(pos));
                push_pen(PenEvent::Down(pos));
            }
            "pointermove" if erasing => push_pen(PenEvent::EraseMove(pos)),
            "pointermove" if e.buttons() == 0 => push_pen(PenEvent::Hover(pos)),
            "pointermove" => {
                PEN_DOWN.set(Some(pos));
                push_pen(PenEvent::Move(pos));
            }
            "pointerup" | "pointercancel" if e.button() == 5 => push_pen(PenEvent::EraseUp(pos)),
            "pointerup" | "pointercancel" => {
                PEN_DOWN.set(None);
                push_pen(PenEvent::Up(pos));
            }
            "pointerout" => push_pen(PenEvent::Leave),
            _ => {}
        }
        ctx.request_repaint();
    });
    let options = web_sys::AddEventListenerOptions::new();
    options.set_capture(true);
    options.set_passive(true);
    for ty in ["pointerdown", "pointermove", "pointerup", "pointercancel", "pointerout"] {
        let _ = window.add_event_listener_with_callback_and_add_event_listener_options(ty, on_pointer.as_ref().unchecked_ref(), &options);
    }
    on_pointer.forget();
}
