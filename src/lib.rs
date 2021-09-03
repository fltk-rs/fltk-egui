#![warn(clippy::all)]

// Re-export dependencies.
pub use egui;
use fltk::*;
use fltk::{prelude::*, window::GlutWindow};
pub mod painter;
use egui::*;
pub use gl;
pub use painter::Painter;
mod clipboard;

use clipboard::{
    ClipboardContext, // TODO: remove
    ClipboardProvider,
};

pub fn with_fltk(
    win: &mut fltk::window::GlutWindow,
    scale: DpiScaling,
) -> (Painter, EguiInputState) {
    let scale = match scale {
        DpiScaling::Default => win.pixels_per_unit(),
        DpiScaling::Custom(custom) => custom,
    };
    let painter = painter::Painter::new(win, scale);
    EguiInputState::new(painter)
}

pub enum DpiScaling {
    /// Default DPI Scale by fltk, usually 1.0
    Default,
    /// Custome DPI scaling, e.g: 1.5, 2.0 and so fort.
    Custom(f32),
}

pub struct FusedCursor {
    pub cursor_icon: fltk::enums::Cursor,
}

const ARROW: enums::Cursor = enums::Cursor::Arrow;

impl FusedCursor {
    pub fn new() -> Self {
        Self { cursor_icon: ARROW }
    }
}
pub struct EguiInputState {
    pub fuse_cursor: FusedCursor,
    pub pointer_pos: Pos2,
    pub clipboard: Option<ClipboardContext>,
    pub input: RawInput,
    pub modifiers: Modifiers,
}

impl EguiInputState {
    pub fn new(painter: Painter) -> (Painter, EguiInputState) {
        let _self = EguiInputState {
            fuse_cursor: FusedCursor::new(),
            pointer_pos: Pos2::new(0f32, 0f32),
            clipboard: init_clipboard(),
            input: egui::RawInput {
                screen_rect: Some(painter.screen_rect),
                pixels_per_point: Some(painter.pixels_per_point),
                ..Default::default()
            },
            modifiers: Modifiers::default(),
        };
        (painter, _self)
    }
}

// copied from https://github.com/not-fl3/egui-miniquad/blob/842e127d05e4c921da5ae1e797e28e848a220bea/src/input.rs#L25
pub fn is_printable_char(chr: char) -> bool {
    #![allow(clippy::manual_range_contains)]

    let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
        || '\u{f0000}' <= chr && chr <= '\u{ffffd}'
        || '\u{100000}' <= chr && chr <= '\u{10fffd}';

    !is_in_private_use_area && !chr.is_ascii_control()
}

pub fn input_to_egui(
    win: &mut fltk::window::GlutWindow,
    event: enums::Event,
    state: &mut EguiInputState,
    painter: &mut Painter,
) {
    let (x, y) = app::event_coords();
    let pixels_per_point = painter.pixels_per_point;
    match event {
        enums::Event::Resize => {
            let (w, h) = (win.width(), win.height());
            painter.update_screen_rect((w, h));
            state.input.screen_rect = Some(painter.screen_rect);
        }
        //MouseButonLeft pressed is the only one needed by egui
        enums::Event::Push => {
            let mouse_btn = match app::event_mouse_button() {
                app::MouseButton::Left => Some(egui::PointerButton::Primary),
                app::MouseButton::Middle => Some(egui::PointerButton::Middle),
                app::MouseButton::Right => Some(egui::PointerButton::Secondary),
                _ => None,
            };
            if let Some(pressed) = mouse_btn {
                state.input.events.push(egui::Event::PointerButton {
                    pos: state.pointer_pos,
                    button: pressed,
                    pressed: true,
                    modifiers: state.modifiers,
                })
            }
        }

        //MouseButonLeft pressed is the only one needed by egui
        enums::Event::Released => {
            // fix unreachable, we can use Option.
            let mouse_btn = match app::event_mouse_button() {
                app::MouseButton::Left => Some(egui::PointerButton::Primary),
                app::MouseButton::Middle => Some(egui::PointerButton::Middle),
                app::MouseButton::Right => Some(egui::PointerButton::Secondary),
                _ => None,
            };
            if let Some(released) = mouse_btn {
                state.input.events.push(egui::Event::PointerButton {
                    pos: state.pointer_pos,
                    button: released,
                    pressed: false,
                    modifiers: state.modifiers,
                })
            }
        }

        enums::Event::Move | enums::Event::Drag => {
            state.pointer_pos = pos2(x as f32 / pixels_per_point, y as f32 / pixels_per_point);
            state
                .input
                .events
                .push(egui::Event::PointerMoved(state.pointer_pos))
        }

        enums::Event::KeyUp => {
            if let Some(key) = translate_virtual_key_code(app::event_key()) {
                let keymod = app::event_state();
                state.modifiers = Modifiers {
                    alt: (keymod & enums::EventState::Alt == enums::EventState::Alt),
                    ctrl: (keymod & enums::EventState::Ctrl == enums::EventState::Ctrl),
                    shift: (keymod & enums::EventState::Shift == enums::EventState::Shift),
                    mac_cmd: keymod & enums::EventState::Meta == enums::EventState::Meta,

                    //TOD: Test on both windows and mac
                    command: (keymod & enums::EventState::Command == enums::EventState::Command),
                };

                if state.modifiers.command && key == Key::C {
                    println!("copy event");
                    state.input.events.push(Event::Copy)
                } else if state.modifiers.command && key == Key::X {
                    println!("cut event");
                    state.input.events.push(Event::Cut)
                } else if state.modifiers.command && key == Key::V {
                    println!("paste");
                    if let Some(clipboard) = state.clipboard.as_mut() {
                        match clipboard.get_contents() {
                            Ok(contents) => {
                                state.input.events.push(Event::Text(contents));
                            }
                            Err(err) => {
                                eprintln!("Paste error: {}", err);
                            }
                        }
                    }
                } else {
                    state.input.events.push(Event::Key {
                        key,
                        pressed: false,
                        modifiers: state.modifiers,
                    });
                }
            }
        }

        enums::Event::KeyDown => {
            let event_state = app::event_state();
            if let Some(c) = app::event_text().chars().next() {
                if is_printable_char(c)
                    && event_state != enums::EventState::Ctrl
                    && event_state != enums::EventState::Meta
                {
                    state.input.events.push(Event::Text(app::event_text()));
                }
            }
            if let Some(key) = translate_virtual_key_code(app::event_key()) {
                let keymod = app::event_state();
                state.modifiers = Modifiers {
                    alt: (keymod & enums::EventState::Alt == enums::EventState::Alt),
                    ctrl: (keymod & enums::EventState::Ctrl == enums::EventState::Ctrl),
                    shift: (keymod & enums::EventState::Shift == enums::EventState::Shift),
                    mac_cmd: keymod & enums::EventState::Meta == enums::EventState::Meta,

                    //TOD: Test on both windows and mac
                    command: (keymod & enums::EventState::Command == enums::EventState::Command),
                };

                state.input.events.push(Event::Key {
                    key,
                    pressed: true,
                    modifiers: state.modifiers,
                });
            }
        }

        enums::Event::MouseWheel => {
            state.input.scroll_delta = vec2(app::event_x() as f32, app::event_y() as f32);
        }

        _ => {
            //dbg!(event);
        }
    }
}

pub fn translate_virtual_key_code(key: enums::Key) -> Option<egui::Key> {
    let matched = match key {
        enums::Key::Left => Some(egui::Key::ArrowLeft),
        enums::Key::Up => Some(egui::Key::ArrowUp),
        enums::Key::Right => Some(egui::Key::ArrowRight),
        enums::Key::Down => Some(egui::Key::ArrowDown),

        enums::Key::Escape => Some(egui::Key::Escape),
        enums::Key::Tab => Some(egui::Key::Tab),
        enums::Key::BackSpace => Some(egui::Key::Backspace),

        enums::Key::Insert => Some(egui::Key::Insert),
        enums::Key::Home => Some(egui::Key::Home),
        enums::Key::Delete => Some(egui::Key::Delete),
        enums::Key::End => Some(egui::Key::End),
        enums::Key::PageDown => Some(egui::Key::PageDown),
        enums::Key::PageUp => Some(egui::Key::PageUp),
        _ => None,
    };

    if matched.is_none() {
        if key == enums::Key::from_char(' ') {
            Some(egui::Key::Space)
        } else if key == enums::Key::from_char('\n') {
            Some(egui::Key::Enter)
        } else if key == enums::Key::from_char('a') {
            Some(egui::Key::A)
        } else if key == enums::Key::from_char('b') {
            Some(egui::Key::B)
        } else if key == enums::Key::from_char('c') {
            Some(egui::Key::C)
        } else if key == enums::Key::from_char('d') {
            Some(egui::Key::D)
        } else if key == enums::Key::from_char('e') {
            Some(egui::Key::E)
        } else if key == enums::Key::from_char('f') {
            Some(egui::Key::F)
        } else if key == enums::Key::from_char('g') {
            Some(egui::Key::G)
        } else if key == enums::Key::from_char('h') {
            Some(egui::Key::H)
        } else if key == enums::Key::from_char('i') {
            Some(egui::Key::I)
        } else if key == enums::Key::from_char('j') {
            Some(egui::Key::J)
        } else if key == enums::Key::from_char('k') {
            Some(egui::Key::K)
        } else if key == enums::Key::from_char('l') {
            Some(egui::Key::L)
        } else if key == enums::Key::from_char('m') {
            Some(egui::Key::M)
        } else if key == enums::Key::from_char('n') {
            Some(egui::Key::N)
        } else if key == enums::Key::from_char('o') {
            Some(egui::Key::O)
        } else if key == enums::Key::from_char('p') {
            Some(egui::Key::P)
        } else if key == enums::Key::from_char('q') {
            Some(egui::Key::Q)
        } else if key == enums::Key::from_char('r') {
            Some(egui::Key::R)
        } else if key == enums::Key::from_char('s') {
            Some(egui::Key::S)
        } else if key == enums::Key::from_char('t') {
            Some(egui::Key::T)
        } else if key == enums::Key::from_char('u') {
            Some(egui::Key::U)
        } else if key == enums::Key::from_char('v') {
            Some(egui::Key::V)
        } else if key == enums::Key::from_char('w') {
            Some(egui::Key::W)
        } else if key == enums::Key::from_char('x') {
            Some(egui::Key::X)
        } else if key == enums::Key::from_char('y') {
            Some(egui::Key::Y)
        } else if key == enums::Key::from_char('z') {
            Some(egui::Key::Z)
        } else if key == enums::Key::from_char('0') {
            Some(egui::Key::Num0)
        } else if key == enums::Key::from_char('1') {
            Some(egui::Key::Num1)
        } else if key == enums::Key::from_char('2') {
            Some(egui::Key::Num2)
        } else if key == enums::Key::from_char('3') {
            Some(egui::Key::Num3)
        } else if key == enums::Key::from_char('4') {
            Some(egui::Key::Num4)
        } else if key == enums::Key::from_char('5') {
            Some(egui::Key::Num5)
        } else if key == enums::Key::from_char('6') {
            Some(egui::Key::Num6)
        } else if key == enums::Key::from_char('7') {
            Some(egui::Key::Num7)
        } else if key == enums::Key::from_char('8') {
            Some(egui::Key::Num8)
        } else if key == enums::Key::from_char('9') {
            Some(egui::Key::Num9)
        } else {
            None
        }
    } else {
        matched
    }
}

pub fn translate_cursor(
    win: &mut GlutWindow,
    fused: &mut FusedCursor,
    cursor_icon: egui::CursorIcon,
) {
    let tmp_icon = match cursor_icon {
        CursorIcon::Default => enums::Cursor::Arrow,
        CursorIcon::PointingHand => enums::Cursor::Hand,
        CursorIcon::ResizeHorizontal => enums::Cursor::WE,
        CursorIcon::ResizeNeSw => enums::Cursor::NESW,
        CursorIcon::ResizeNwSe => enums::Cursor::NWSE,
        CursorIcon::ResizeVertical => enums::Cursor::NS,
        CursorIcon::Text => enums::Cursor::Insert,
        CursorIcon::Crosshair => enums::Cursor::Cross,
        CursorIcon::NotAllowed | CursorIcon::NoDrop => enums::Cursor::Wait,
        CursorIcon::Wait => enums::Cursor::Wait,
        //There doesn't seem to be a suitable SDL equivalent...
        CursorIcon::Grab | CursorIcon::Grabbing => enums::Cursor::Move,

        _ => enums::Cursor::Arrow,
    };

    if tmp_icon != fused.cursor_icon {
        fused.cursor_icon = tmp_icon;
        win.set_cursor(tmp_icon)
    }
}

pub fn init_clipboard() -> Option<ClipboardContext> {
    match ClipboardContext::new() {
        Ok(clipboard) => Some(clipboard),
        Err(err) => {
            eprintln!("Failed to initialize clipboard: {}", err);
            None
        }
    }
}

pub fn copy_to_clipboard(clipboard: &mut Option<ClipboardContext>, copy_text: String) {
    if let Some(clipboard) = clipboard.as_mut() {
        let result = clipboard.set_contents(copy_text);
        if result.is_err() {
            dbg!("Unable to set clipboard content.");
        }
    }
}
