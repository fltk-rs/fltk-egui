/*!
    # fltk-egui

    An FLTK backend for Egui using a GlWindow. The code is largely based on https://github.com/ArjunNair/egui_sdl2_gl modified for fltk-rs.

    ## Usage
    Add to your Cargo.toml:
    ```toml
    [dependencies]
    fltk-egui = "0.3"
    ```

    The basic premise is that egui is an immediate mode gui, while FLTK is retained.
    To be able to run Egui code, events and redrawing would need to be handled/done in the FLTK event loop.
    The events are those of the GlWindow, which are sent to egui's event handlers.
    Other FLTK widgets can function also normally since there is no interference from Egui.

    ## Examples
    To run the examples, just run:
    ```
    $ cargo run --example demo_windows
    $ cargo run --example triangle
    $ cargo run --example basic
    $ cargo run --example embedded
    ```
*/

#![warn(clippy::all)]

use std::time::Instant;

// Re-export dependencies.
pub use egui;
use egui::{pos2, CursorIcon, Event, Key, Modifiers, Pos2, RawInput, Vec2};
pub use fltk;
use fltk::{
    app, enums,
    prelude::{FltkError, GroupExt, ImageExt, InputExt, WidgetExt, WindowExt},
    window::GlWindow,
};
pub use gl;
mod painter;
pub use painter::Painter;

/// Construct the backend.
/// Requires the DpiScaling, which can be Default or Custom(f32)
pub fn with_fltk(win: &mut GlWindow, scale: DpiScaling) -> (Painter, EguiInputState) {
    let scale = match scale {
        DpiScaling::Default => win.pixels_per_unit(),
        DpiScaling::Custom(custom) => custom,
    };
    let inp = fltk::input::Input::default();
    let painter = Painter::new(win, scale);
    win.add(&inp);
    EguiInputState::new(painter)
}

/// Frame time for FPS.
pub fn get_frame_time(start_time: Instant) -> f32 {
    (Instant::now() - start_time).as_secs_f64() as f32
}

/// The scaling factors of the app
pub enum DpiScaling {
    /// Default DPI Scale by fltk, usually 1.0
    Default,
    /// Custome DPI scaling, e.g: 1.5, 2.0 and so fort.
    Custom(f32),
}

/// The default cursor
pub struct FusedCursor {
    pub cursor_icon: fltk::enums::Cursor,
}

const ARROW: enums::Cursor = enums::Cursor::Arrow;

impl FusedCursor {
    /// Construct a new cursor
    pub fn new() -> Self {
        Self { cursor_icon: ARROW }
    }
}

impl Default for FusedCursor {
    fn default() -> Self {
        Self::new()
    }
}

/// Shuttles FLTK's input and events to Egui
pub struct EguiInputState {
    pub fuse_cursor: FusedCursor,
    pub pointer_pos: Pos2,
    pub input: RawInput,
    pub modifiers: Modifiers,
    /// Internal use case for fn window_resized()
    pub _window_resized: bool,
}

impl EguiInputState {
    /// Construct a new state
    pub fn new(painter: Painter) -> (Painter, EguiInputState) {
        let _self = EguiInputState {
            fuse_cursor: FusedCursor::new(),
            pointer_pos: Pos2::new(0f32, 0f32),
            input: egui::RawInput {
                screen_rect: Some(painter.screen_rect),
                pixels_per_point: Some(painter.pixels_per_point),
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            _window_resized: false,
        };
        (painter, _self)
    }

    /// Check if current window being resized.
    pub fn window_resized(&mut self) -> bool {
        let tmp = self._window_resized;
        self._window_resized = false;
        tmp
    }

    /// Conveniece method bundling the necessary components for input/event handling
    pub fn fuse_input(&mut self, win: &mut GlWindow, event: enums::Event, painter: &mut Painter) {
        input_to_egui(win, event, self, painter);
    }

    /// Convenience method for outputting what egui emits each frame
    pub fn fuse_output(&mut self, win: &mut GlWindow, egui_output: &egui::Output) {
        if !egui_output.copied_text.is_empty() {
            app::copy(&egui_output.copied_text);
        }
        translate_cursor(win, &mut self.fuse_cursor, egui_output.cursor_icon);
    }
}

/// Handles input/events from FLTK
pub fn input_to_egui(
    win: &mut GlWindow,
    event: enums::Event,
    state: &mut EguiInputState,
    painter: &mut Painter,
) {
    let (x, y) = app::event_coords();
    let pixels_per_point = painter.pixels_per_point;
    match event {
        enums::Event::Resize => {
            painter.update_screen_rect((win.width(), win.height()));
            state.input.screen_rect = Some(painter.screen_rect);
            state._window_resized = true;
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
                if state.modifiers.command && key == Key::V {
                    let inp: fltk::input::Input = unsafe { win.child(0).unwrap().into_widget() };
                    app::paste(&inp);
                    state.input.events.push(Event::Text(inp.value()));
                }
            }
        }

        enums::Event::KeyDown => {
            if let Some(c) = app::event_text().chars().next() {
                if let Some(del) = app::compose() {
                    state.input.events.push(Event::Text(c.to_string()));
                    if del != 0 {
                        app::compose_reset();
                    }
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

                if state.modifiers.command && key == Key::C {
                    // println!("copy event");
                    state.input.events.push(Event::Copy)
                } else if state.modifiers.command && key == Key::X {
                    // println!("cut event");
                    state.input.events.push(Event::Cut)
                } else {
                    state.input.events.push(Event::Key {
                        key,
                        pressed: false,
                        modifiers: state.modifiers,
                    })
                }
            }
        }

        enums::Event::MouseWheel => {
            if app::is_event_ctrl() {
                let zoom_factor = 1.2;
                match app::event_dy() {
                    app::MouseWheel::Up => {
                        state.input.events.push(Event::Zoom(zoom_factor * -1.0));
                    }
                    app::MouseWheel::Down => {
                        state.input.events.push(Event::Zoom(zoom_factor));
                    }
                    _ => (),
                }
            } else {
                let scroll_factor = 15.0;
                match app::event_dy() {
                    app::MouseWheel::Up => {
                        state.input.events.push(Event::Scroll(Vec2 {
                            x: scroll_factor,
                            y: 0.,
                        }));
                    }
                    app::MouseWheel::Down => {
                        state.input.events.push(Event::Scroll(Vec2 {
                            x: 0.,
                            y: scroll_factor,
                        }));
                    }
                    _ => (),
                }
            }
        }

        _ => {
            //dbg!(event);
        }
    }
}

/// Translates key codes
pub fn translate_virtual_key_code(key: enums::Key) -> Option<egui::Key> {
    match key {
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
        enums::Key::Enter => Some(egui::Key::Enter),
        _ => {
            if let Some(k) = key.to_char() {
                match k {
                    ' ' => Some(egui::Key::Space),
                    'a' => Some(egui::Key::A),
                    'b' => Some(egui::Key::B),
                    'c' => Some(egui::Key::C),
                    'd' => Some(egui::Key::D),
                    'e' => Some(egui::Key::E),
                    'f' => Some(egui::Key::F),
                    'g' => Some(egui::Key::G),
                    'h' => Some(egui::Key::H),
                    'i' => Some(egui::Key::I),
                    'j' => Some(egui::Key::J),
                    'k' => Some(egui::Key::K),
                    'l' => Some(egui::Key::L),
                    'm' => Some(egui::Key::M),
                    'n' => Some(egui::Key::N),
                    'o' => Some(egui::Key::O),
                    'p' => Some(egui::Key::P),
                    'q' => Some(egui::Key::Q),
                    'r' => Some(egui::Key::R),
                    's' => Some(egui::Key::S),
                    't' => Some(egui::Key::T),
                    'u' => Some(egui::Key::U),
                    'v' => Some(egui::Key::V),
                    'w' => Some(egui::Key::W),
                    'x' => Some(egui::Key::X),
                    'y' => Some(egui::Key::Y),
                    'z' => Some(egui::Key::Z),
                    '0' => Some(egui::Key::Num0),
                    '1' => Some(egui::Key::Num1),
                    '2' => Some(egui::Key::Num2),
                    '3' => Some(egui::Key::Num3),
                    '4' => Some(egui::Key::Num4),
                    '5' => Some(egui::Key::Num5),
                    '6' => Some(egui::Key::Num6),
                    '7' => Some(egui::Key::Num7),
                    '8' => Some(egui::Key::Num8),
                    '9' => Some(egui::Key::Num9),
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

/// Translates FLTK cursor to Egui cursors
pub fn translate_cursor(
    win: &mut GlWindow,
    fused: &mut FusedCursor,
    cursor_icon: egui::CursorIcon,
) {
    let tmp_icon = match cursor_icon {
        CursorIcon::None => enums::Cursor::None,
        CursorIcon::Default => enums::Cursor::Arrow,
        CursorIcon::Help => enums::Cursor::Help,
        CursorIcon::PointingHand => enums::Cursor::Hand,
        CursorIcon::ResizeHorizontal => enums::Cursor::WE,
        CursorIcon::ResizeNeSw => enums::Cursor::NESW,
        CursorIcon::ResizeNwSe => enums::Cursor::NWSE,
        CursorIcon::ResizeVertical => enums::Cursor::NS,
        CursorIcon::Text => enums::Cursor::Insert,
        CursorIcon::Crosshair => enums::Cursor::Cross,
        CursorIcon::NotAllowed | CursorIcon::NoDrop => enums::Cursor::Wait,
        CursorIcon::Wait => enums::Cursor::Wait,
        CursorIcon::Progress => enums::Cursor::Wait,
        CursorIcon::Grab => enums::Cursor::Hand,
        CursorIcon::Grabbing => enums::Cursor::Move,
        CursorIcon::Move => enums::Cursor::Move,

        _ => enums::Cursor::Arrow,
    };

    if tmp_icon != fused.cursor_icon {
        fused.cursor_icon = tmp_icon;
        win.set_cursor(tmp_icon)
    }
}

pub trait EguiImageConvertible<I>
where
    I: ImageExt,
{
    fn to_egui_image(
        self,
        painter: &mut Painter,
        new_size: (u32, u32),
        filtering: bool,
    ) -> Result<(egui::Image, egui::TextureId), FltkError>;
}

impl<I> EguiImageConvertible<I> for I
where
    I: ImageExt,
{
    /// Return (egui::Image, egui::TextureId)
    fn to_egui_image(
        self,
        painter: &mut Painter,
        new_size: (u32, u32),
        filtering: bool,
    ) -> Result<(egui::Image, egui::TextureId), FltkError> {
        let size = (self.data_w() as usize, self.data_h() as usize);
        let texture_id = painter.new_user_texture_rgba8(
            size,
            self.to_rgb()?
                .convert(enums::ColorDepth::Rgba8)?
                .to_rgb_data(),
            filtering,
        );
        let image = egui::Image::new(texture_id, egui::vec2(new_size.0 as _, new_size.1 as _));
        Ok((image, texture_id))
    }
}

pub trait EguiSvgConvertible {
    fn to_egui_image(
        self,
        painter: &mut Painter,
        new_size: (u32, u32),
        filtering: bool,
    ) -> Result<(egui::Image, egui::TextureId), FltkError>;
}

impl EguiSvgConvertible for fltk::image::SvgImage {
    /// Return (egui::Image, egui::TextureId)
    fn to_egui_image(
        mut self,
        painter: &mut Painter,
        new_size: (u32, u32),
        filtering: bool,
    ) -> Result<(egui::Image, egui::TextureId), FltkError> {
        self.normalize();
        let size = (self.data_w() as usize, self.data_h() as usize);
        let texture_id = painter.new_user_texture_rgba8(
            size,
            self.to_rgb()?
                .convert(enums::ColorDepth::Rgba8)?
                .to_rgb_data(),
            filtering,
        );
        let image = egui::Image::new(texture_id, egui::vec2(new_size.0 as _, new_size.1 as _));
        Ok((image, texture_id))
    }
}
