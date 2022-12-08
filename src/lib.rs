#![doc = include_str!("../README.md")]
#![warn(clippy::all)]

use std::{sync::Arc, time::Instant};

// Re-export dependencies.
pub use egui;
use egui::{pos2, vec2, CursorIcon, Event, Key, Modifiers, Pos2, RawInput, Rect, Vec2};
pub use egui_glow;
use egui_glow::{glow, Painter};
pub use egui_image::RetainedEguiImage;
pub use fltk;
use fltk::{
    app, enums,
    prelude::{FltkError, ImageExt, WidgetExt, WindowExt},
    window::GlWindow,
};

mod clipboard;
mod egui_image;
use clipboard::Clipboard;

/// Construct the backend.
pub fn with_fltk(win: &mut GlWindow) -> (Painter, EguiState) {
    app::set_screen_scale(win.screen_num(), 1.);
    app::keyboard_screen_scaling(false);
    let gl = unsafe { glow::Context::from_loader_function(|s| win.get_proc_address(s) as _) };
    let painter = Painter::new(Arc::from(gl), "", None)
        .unwrap_or_else(|error| panic!("some OpenGL error occurred {}\n", error));
    let max_texture_side = painter.max_texture_side();
    (painter, EguiState::new(&win, max_texture_side))
}

/// Frame time for FPS.
pub fn get_frame_time(start_time: Instant) -> f32 {
    (Instant::now() - start_time).as_secs_f64() as f32
}

/// Casting slice to another type of slice
pub fn cast_slice<T, D>(s: &[T]) -> &[D] {
    unsafe {
        std::slice::from_raw_parts(s.as_ptr() as *const D, s.len() * std::mem::size_of::<T>())
    }
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
pub struct EguiState {
    pub canvas_size: [u32; 2],
    pub clipboard: Clipboard,
    pub fuse_cursor: FusedCursor,
    /// Use state.input.take() use this fn instead (to avoid pixels per point miscalculation).
    pub input: RawInput,
    _pixels_per_point: f32,
    pub pointer_pos: Pos2,
    /// default value is 12.0
    pub scroll_factor: f32,
    /// default value is 8.0
    pub zoom_factor: f32,
    /// Internal use case for fn window_resized()
    _window_resized: bool,
    pub max_texture_side: usize,
}

impl EguiState {
    /// Construct a new state
    pub fn new(win: &GlWindow, max_texture_side: usize) -> EguiState {
        let ppu = win.pixels_per_unit();
        let (width, height) = (win.width(), win.height());
        let rect = vec2(width as f32, height as f32) / ppu;
        let screen_rect = Rect::from_min_size(Pos2::new(0f32, 0f32), rect);
        EguiState {
            canvas_size: [width as u32, height as u32],
            clipboard: Clipboard::default(),
            fuse_cursor: FusedCursor::new(),
            input: egui::RawInput {
                screen_rect: Some(screen_rect),
                pixels_per_point: Some(ppu),
                max_texture_side: Some(max_texture_side),
                ..Default::default()
            },
            max_texture_side,
            _pixels_per_point: ppu,
            pointer_pos: Pos2::new(0f32, 0f32),
            scroll_factor: 12.0,
            zoom_factor: 8.0,
            _window_resized: false,
        }
    }

    pub fn take_input(&mut self) -> egui::RawInput {
        self.input.max_texture_side = Some(self.max_texture_side);
        let pixels_per_point = self.input.pixels_per_point;
        let take = self.input.take();
        self.input.pixels_per_point = pixels_per_point;
        if let Some(ppp) = pixels_per_point {
            self._pixels_per_point = ppp;
        }
        take
    }

    pub fn pixels_per_point(&self) -> f32 {
        self._pixels_per_point
    }

    /// Check if current window being resized.
    pub fn window_resized(&mut self) -> bool {
        let tmp = self._window_resized;
        self._window_resized = false;
        tmp
    }

    /// Conveniece method bundling the necessary components for input/event handling
    pub fn fuse_input(&mut self, win: &mut GlWindow, event: enums::Event) {
        input_to_egui(win, event, self);
    }

    /// Convenience method for outputting what egui emits each frame
    pub fn fuse_output(&mut self, win: &mut GlWindow, egui_output: egui::PlatformOutput) {
        if !egui_output.copied_text.is_empty() {
            self.clipboard.set(egui_output.copied_text);
        }
        if win.damage() {
            win.clear_damage();
        }
        translate_cursor(win, &mut self.fuse_cursor, egui_output.cursor_icon);
    }

    /// Convenience method for outputting what egui emits each frame (borrow PlatformOutput)
    pub fn fuse_output_borrow(&mut self, win: &mut GlWindow, egui_output: &egui::PlatformOutput) {
        if !egui_output.copied_text.is_empty() {
            app::copy(&egui_output.copied_text);
        }
        if win.damage() {
            win.clear_damage();
        }
        translate_cursor(win, &mut self.fuse_cursor, egui_output.cursor_icon);
    }

    /// Set visual scale, e.g: 0.8, 1.5, 2.0 .etc (default is 1.0)
    pub fn set_visual_scale(&mut self, size: f32) {
        // have to be setted the pixels_per_point of both the inner (input) and the state.
        self.input.pixels_per_point = Some(size);
        self._pixels_per_point = size;

        // resize rect with canvas.
        let canvas_size = self.canvas_size;
        let rect = vec2(canvas_size[0] as f32, canvas_size[1] as f32) / size;
        self.input.screen_rect = Some(Rect::from_min_size(Default::default(), rect));
    }
}

/// Handles input/events from FLTK
pub fn input_to_egui(
    win: &mut GlWindow,
    event: enums::Event,
    state: &mut EguiState,
    // painter: &mut Painter,
) {
    match event {
        enums::Event::Resize => {
            state.canvas_size = [win.width() as u32, win.height() as u32];
            state.set_visual_scale(state.pixels_per_point());
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
                    modifiers: state.input.modifiers,
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
                    modifiers: state.input.modifiers,
                })
            }
        }

        enums::Event::Move | enums::Event::Drag => {
            let ppp = state.pixels_per_point();
            let (x, y) = app::event_coords();
            state.pointer_pos = pos2(x as f32 / ppp, y as f32 / ppp);
            state
                .input
                .events
                .push(egui::Event::PointerMoved(state.pointer_pos))
        }

        enums::Event::KeyUp => {
            if let Some(key) = translate_virtual_key_code(app::event_key()) {
                let keymod = app::event_state();
                state.input.modifiers = Modifiers {
                    alt: (keymod & enums::EventState::Alt == enums::EventState::Alt),
                    ctrl: (keymod & enums::EventState::Ctrl == enums::EventState::Ctrl),
                    shift: (keymod & enums::EventState::Shift == enums::EventState::Shift),
                    mac_cmd: keymod & enums::EventState::Meta == enums::EventState::Meta,

                    //TOD: Test on both windows and mac
                    command: (keymod & enums::EventState::Command == enums::EventState::Command),
                };
                if state.input.modifiers.command && key == Key::V {
                    if let Some(value) = state.clipboard.get() {
                        state.input.events.push(egui::Event::Text(value));
                    }
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
                state.input.modifiers = Modifiers {
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
                    modifiers: state.input.modifiers,
                });

                if state.input.modifiers.command && key == Key::C {
                    // println!("copy event");
                    state.input.events.push(Event::Copy)
                } else if state.input.modifiers.command && key == Key::X {
                    // println!("cut event");
                    state.input.events.push(Event::Cut)
                } else {
                    state.input.events.push(Event::Key {
                        key,
                        pressed: false,
                        modifiers: state.input.modifiers,
                    })
                }
            }
        }

        enums::Event::MouseWheel => {
            if app::is_event_ctrl() {
                let zoom_factor = state.zoom_factor;
                match app::event_dy() {
                    app::MouseWheel::Up => {
                        let delta = egui::vec2(1., -1.) * zoom_factor;

                        // Treat as zoom in:
                        state
                            .input
                            .events
                            .push(Event::Zoom((delta.y / 200.0).exp()));
                    }
                    app::MouseWheel::Down => {
                        let delta = egui::vec2(-1., 1.) * zoom_factor;

                        // Treat as zoom out:
                        state
                            .input
                            .events
                            .push(Event::Zoom((delta.y / 200.0).exp()));
                    }
                    _ => (),
                }
            } else {
                let scroll_factor = state.scroll_factor;
                match app::event_dy() {
                    app::MouseWheel::Up => {
                        state.input.events.push(Event::Scroll(Vec2 {
                            x: 0.,
                            y: -scroll_factor,
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
    fn egui_image(
        self,
        debug_name: &str,
        options: egui::TextureOptions,
    ) -> Result<RetainedEguiImage, FltkError>;
}

impl<I> EguiImageConvertible<I> for I
where
    I: ImageExt,
{
    /// Return (egui_extras::RetainedImage)
    fn egui_image(
        self,
        debug_name: &str,
        options: egui::TextureOptions,
    ) -> Result<RetainedEguiImage, FltkError> {
        let size = [self.data_w() as usize, self.data_h() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            size,
            &self
                .to_rgb()?
                .convert(enums::ColorDepth::Rgba8)?
                .to_rgb_data(),
        );

        Ok(RetainedEguiImage::from_color_image(
            debug_name,
            color_image,
            options,
        ))
    }
}

pub trait EguiSvgConvertible {
    fn egui_svg_image(
        self,
        debug_name: &str,
        options: egui::TextureOptions,
    ) -> Result<RetainedEguiImage, FltkError>;
}

impl EguiSvgConvertible for fltk::image::SvgImage {
    /// Return (egui_extras::RetainedEguiImage)
    fn egui_svg_image(
        mut self,
        debug_name: &str,
        options: egui::TextureOptions,
    ) -> Result<RetainedEguiImage, FltkError> {
        self.normalize();
        let size = [self.data_w() as usize, self.data_h() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            size,
            &self
                .to_rgb()?
                .convert(enums::ColorDepth::Rgba8)?
                .to_rgb_data(),
        );

        Ok(RetainedEguiImage::from_color_image(
            debug_name,
            color_image,
            options,
        ))
    }
}

/// egui::ColorImage Extender.
pub trait ColorImageExt {
    fn from_vec_color32(size: [usize; 2], vec: Vec<egui::Color32>) -> Self;

    fn from_color32_slice(size: [usize; 2], slice: &[egui::Color32]) -> Self;
}

impl ColorImageExt for egui::ColorImage {
    fn from_vec_color32(size: [usize; 2], vec: Vec<egui::Color32>) -> Self {
        let mut pixels: Vec<u8> = Vec::with_capacity(vec.len() * 4);
        vec.into_iter().for_each(|x| {
            pixels.push(x[0]);
            pixels.push(x[1]);
            pixels.push(x[2]);
            pixels.push(x[3]);
        });
        egui::ColorImage::from_rgba_unmultiplied(size, &pixels)
    }

    fn from_color32_slice(size: [usize; 2], slice: &[egui::Color32]) -> Self {
        let mut pixels: Vec<u8> = Vec::with_capacity(slice.len() * 4);
        slice.into_iter().for_each(|x| {
            pixels.push(x[0]);
            pixels.push(x[1]);
            pixels.push(x[2]);
            pixels.push(x[3]);
        });
        egui::ColorImage::from_rgba_unmultiplied(size, &pixels)
    }
}

/// egui::TextureHandle Extender.
pub trait TextureHandleExt {
    /// egui::TextureHandle from Vec u8
    fn from_vec_u8(
        ctx: &egui::Context,
        debug_name: &str,
        size: [usize; 2],
        vec: Vec<u8>,
        options: egui::TextureOptions,
    ) -> egui::TextureHandle;

    fn from_u8_slice(
        ctx: &egui::Context,
        debug_name: &str,
        size: [usize; 2],
        slice: &[u8],
        options: egui::TextureOptions,
    ) -> egui::TextureHandle;

    fn from_vec_color32(
        ctx: &egui::Context,
        debug_name: &str,
        size: [usize; 2],
        vec: Vec<egui::Color32>,
        options: egui::TextureOptions,
    ) -> egui::TextureHandle;

    fn from_color32_slice(
        ctx: &egui::Context,
        debug_name: &str,
        size: [usize; 2],
        slice: &[egui::Color32],
        options: egui::TextureOptions,
    ) -> egui::TextureHandle;
}

impl TextureHandleExt for egui::TextureHandle {
    fn from_vec_u8(
        ctx: &egui::Context,
        debug_name: &str,
        size: [usize; 2],
        vec: Vec<u8>,
        options: egui::TextureOptions,
    ) -> Self {
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &vec);
        drop(vec);
        ctx.load_texture(debug_name, color_image, options)
    }

    fn from_u8_slice(
        ctx: &egui::Context,
        debug_name: &str,
        size: [usize; 2],
        slice: &[u8],
        options: egui::TextureOptions,
    ) -> Self {
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, slice);
        ctx.load_texture(debug_name, color_image, options)
    }

    fn from_vec_color32(
        ctx: &egui::Context,
        debug_name: &str,
        size: [usize; 2],
        vec: Vec<egui::Color32>,
        options: egui::TextureOptions,
    ) -> Self {
        let mut pixels: Vec<u8> = Vec::with_capacity(vec.len() * 4);
        vec.into_iter().for_each(|x| {
            pixels.push(x[0]);
            pixels.push(x[1]);
            pixels.push(x[2]);
            pixels.push(x[3]);
        });
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        ctx.load_texture(debug_name, color_image, options)
    }

    fn from_color32_slice(
        ctx: &egui::Context,
        debug_name: &str,
        size: [usize; 2],
        slice: &[egui::Color32],
        options: egui::TextureOptions,
    ) -> Self {
        let mut pixels: Vec<u8> = Vec::with_capacity(slice.len() * 4);
        slice.into_iter().for_each(|x| {
            pixels.push(x[0]);
            pixels.push(x[1]);
            pixels.push(x[2]);
            pixels.push(x[3]);
        });
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        ctx.load_texture(debug_name, color_image, options)
    }
}
