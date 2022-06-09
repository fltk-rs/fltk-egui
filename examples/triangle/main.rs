use egui_backend::{
    egui::{Color32, ColorImage, Image, TextureHandle},
    fltk::{enums::*, prelude::*, *},
    glow, ColorImageExt, TextureHandleExt,
};

use fltk_egui as egui_backend;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
mod triangle;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const PIC_WIDTH: i32 = 320;
const PIC_HEIGHT: i32 = 192;

fn main() {
    let fltk_app = app::App::default();
    let mut win = window::GlWindow::new(100, 100, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, None)
        .center_screen();
    win.set_mode(Mode::MultiSample);
    win.end();
    win.make_resizable(true);
    win.show();
    win.make_current();

    // Init backend
    let (mut painter, egui_state) = egui_backend::with_fltk(&mut win);
    let state = Rc::from(RefCell::from(egui_state));

    win.handle({
        let state = state.clone();
        move |win, ev| match ev {
            enums::Event::Push
            | enums::Event::Released
            | enums::Event::KeyDown
            | enums::Event::KeyUp
            | enums::Event::MouseWheel
            | enums::Event::Resize
            | enums::Event::Move
            | enums::Event::Focus
            | enums::Event::Drag => {
                // Using "if let ..." for safety.
                if let Ok(mut state) = state.try_borrow_mut() {
                    state.fuse_input(win, ev);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    });

    // We will draw a crisp white triangle using Glow OpenGL.
    let triangle = triangle::Triangle::new(painter.gl().as_ref());

    // Some variables to help draw a sine wave
    let mut sine_shift = 0f32;
    let mut amplitude: f32 = 50f32;
    let mut texture: Option<TextureHandle> = None;

    let egui_ctx = egui::Context::default();
    let start_time = Instant::now();
    let mut test_str: String =
        "A text box to write in. Cut, copy, paste commands are available.".to_owned();
    let mut quit = false;

    while fltk_app.wait() {
        // Clear the screen to dark red
        let gl = painter.gl().as_ref();
        draw_background(gl);

        let mut state = state.borrow_mut();
        state.input.time = Some(start_time.elapsed().as_secs_f64());
        let egui_output = egui_ctx.run(state.take_input(), |ctx| {
            // Draw our triangle.
            triangle.draw(gl);

            egui::Window::new("Egui with FLTK and GL").show(ctx, |ui| {
                // Compose sine wave in a buffer.
                let mut srgba: Vec<Color32> = Vec::new();
                let mut angle = 0f32;
                for y in 0..PIC_HEIGHT {
                    for x in 0..PIC_WIDTH {
                        srgba.push(Color32::BLACK);
                        if y == PIC_HEIGHT - 1 {
                            let y = amplitude * (angle * std::f32::consts::PI / 180f32 + sine_shift).sin();
                            let y = PIC_HEIGHT as f32 / 2f32 - y;
                            srgba[(y as i32 * PIC_WIDTH + x) as usize] = Color32::YELLOW;
                            angle += 360f32 / PIC_WIDTH as f32;
                        }
                    }
                }

                sine_shift += 0.1f32;

                match &mut texture {
                    Some(texture) => {
                        // and then set new color image.
                        let new_color_image = ColorImage::from_color32_slice(texture.size(), &srgba);
                        texture.set(new_color_image);
                    }
                    _ => {
                        // We just need to Initialize egui::TextureHandle and create texture id once.
                        let new_texture = TextureHandle::from_color32_slice(ctx, "sinewave", [PIC_WIDTH as usize, PIC_HEIGHT as usize], &srgba);
                        texture = Some(new_texture);
                    }
                }

                if let Some(texture) = &texture {
                    //Draw sine wave texture
                    ui.add(Image::new(texture.id(), texture.size_vec2()));
                    // repaint
                    ctx.request_repaint();
                }
                ui.separator();
                ui.label("A simple sine wave plotted onto a GL texture then blitted to an egui managed Image.");
                ui.label(" ");
                ui.text_edit_multiline(&mut test_str);
                ui.label(" ");
                ui.add(egui::Slider::new(&mut amplitude, 0.0..=50.0).text("Amplitude"));
                ui.label(" ");
                if ui.button("Quit").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    quit = true;
                }
            });
        });

        if egui_output.needs_repaint || state.window_resized() {
            state.fuse_output(&mut win, egui_output.platform_output);
            let meshes = egui_ctx.tessellate(egui_output.shapes);
            painter.paint_and_update_textures(
                state.canvas_size,
                state.pixels_per_point(),
                &meshes,
                &egui_output.textures_delta,
            );

            win.swap_buffers();
            win.flush();
            app::awake();
        }

        if quit {
            break;
        }
    }

    triangle.free(painter.gl().as_ref());
    painter.destroy();
}

fn draw_background<GL: glow::HasContext>(gl: &GL) {
    unsafe {
        gl.clear_color(0.6, 0.3, 0.3, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
    }
}
