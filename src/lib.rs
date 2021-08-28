#![warn(clippy::all)]
#![allow(clippy::single_match)]

// Re-export dependencies.
pub use egui;
use fltk::{prelude::*, *};
pub use gl;

mod painter;

pub use painter::Painter;

use egui::*;

mod clipboard;

use clipboard::{
    ClipboardContext, // TODO: remove
    ClipboardProvider,
};

pub struct EguiInputState {
    pub pointer_pos: Pos2,
    pub clipboard: Option<ClipboardContext>,
    pub input: RawInput,
    pub modifiers: Modifiers,
}

impl EguiInputState {
    pub fn new(input: RawInput) -> Self {
        EguiInputState {
            pointer_pos: Pos2::new(0f32, 0f32),
            clipboard: init_clipboard(),
            input,
            modifiers: Modifiers::default(),
        }
    }
}

pub fn is_printable_char(chr: char) -> bool {
    #![allow(clippy::manual_range_contains)]

    let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
        || '\u{f0000}' <= chr && chr <= '\u{ffffd}'
        || '\u{100000}' <= chr && chr <= '\u{10fffd}';

    !is_in_private_use_area && !chr.is_ascii_control()
}

pub fn input_to_egui(
    win: &mut window::GlutWindow,
    event: enums::Event,
    state: &mut EguiInputState,
) {
    let (x, y) = app::event_coords();
    match event {
        //Only the window resize event is handled
        enums::Event::Resize => {
            state.input.screen_rect = Some(Rect::from_min_size(
                Pos2::new(0f32, 0f32),
                egui::vec2(win.w() as f32, win.h() as f32) / state.input.pixels_per_point.unwrap(),
            ))
        }

        //MouseButonLeft pressed is the only one needed by egui
        enums::Event::Push => state.input.events.push(egui::Event::PointerButton {
            pos: state.pointer_pos,
            button: match app::event_mouse_button() {
                app::MouseButton::Left => egui::PointerButton::Primary,
                app::MouseButton::Right => egui::PointerButton::Secondary,
                app::MouseButton::Middle => egui::PointerButton::Middle,
                _ => unreachable!(),
            },
            pressed: true,
            modifiers: state.modifiers,
        }),

        //MouseButonLeft pressed is the only one needed by egui
        enums::Event::Released => state.input.events.push(egui::Event::PointerButton {
            pos: state.pointer_pos,
            button: match app::event_mouse_button() {
                app::MouseButton::Left => egui::PointerButton::Primary,
                app::MouseButton::Right => egui::PointerButton::Secondary,
                app::MouseButton::Middle => egui::PointerButton::Middle,
                _ => unreachable!(),
            },
            pressed: false,
            modifiers: state.modifiers,
        }),

        enums::Event::Move => {
            state.pointer_pos = pos2(
                x as f32 / state.input.pixels_per_point.unwrap(),
                y as f32 / state.input.pixels_per_point.unwrap(),
            );
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
                    command: (keymod & enums::EventState::Ctrl == enums::EventState::Ctrl),
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
            //
            let event_state = app::event_state();
            if let Some(c) = app::event_text().chars().nth(0) {
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
                    command: (keymod & enums::EventState::Ctrl == enums::EventState::Ctrl),
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
    let space: enums::Key = enums::Key::from_char(' ');
    let ret: enums::Key = enums::Key::from_char('\n');

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

    let matched = if matched.is_none() {
        if key == space {
            Some(egui::Key::Space)
        } else if key == ret {
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
    };
    matched
}

pub fn translate_cursor(cursor_icon: egui::CursorIcon) -> enums::Cursor {
    match cursor_icon {
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
        CursorIcon::Grab | CursorIcon::Grabbing => enums::Cursor::Hand,

        _ => enums::Cursor::Arrow,
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

pub fn copy_to_clipboard(egui_state: &mut EguiInputState, copy_text: String) {
    if let Some(clipboard) = egui_state.clipboard.as_mut() {
        let result = clipboard.set_contents(copy_text);
        if result.is_err() {
            dbg!("Unable to set clipboard content.");
        }
    }
}
