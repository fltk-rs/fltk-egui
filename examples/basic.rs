use egui_backend::{
    egui,
    fltk::{prelude::*, *},
    glow,
};
use fltk::enums::Mode;
use fltk_egui as egui_backend;
use std::rc::Rc;
use std::{cell::RefCell, time::Instant};
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() {
    let fltk_app = app::App::default();
    let mut win = window::GlWindow::new(100, 100, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, None)
        .center_screen();
    win.set_mode(Mode::Opengl3);
    win.end();
    win.make_resizable(true);
    win.show();
    win.make_current();

    // Init backend
    let (mut painter, mut egui_state) = egui_backend::with_fltk(&mut win);
    // Set visual scale or egui display scaling
    egui_state.set_visual_scale(1.5);
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

    let egui_ctx = egui::Context::default();
    let start_time = Instant::now();
    let mut quit = false;
    let mut age: i32 = 17;
    let mut name: String = "".to_string();

    while fltk_app.wait() {
        // Clear the screen to dark red
        let gl = painter.gl().as_ref();
        draw_background(gl);

        let mut state = state.borrow_mut();
        state.input.time = Some(start_time.elapsed().as_secs_f64());
        let egui_output = egui_ctx.run(state.take_input(), |ctx| {
            egui::CentralPanel::default().show(&ctx, |ui| {
                ui.heading("My egui Application");
                ui.horizontal(|ui| {
                    ui.label("Your name: ");
                    ui.text_edit_singleline(&mut name);
                });
                ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
                if ui.button("Click each year").clicked() {
                    age += 1;
                }
                ui.label(format!("Hello '{}', age {}", name, age));
                ui.separator();
                if ui
                    .button("Quit?")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
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

    painter.destroy();
}

fn draw_background<GL: glow::HasContext>(gl: &GL) {
    unsafe {
        gl.clear_color(0.6, 0.3, 0.3, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
    }
}
