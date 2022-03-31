use egui_backend::{
    egui,
    fltk::{enums::*, prelude::*, *},
    glow,
};

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

    //Init backend
    let (gl, mut painter, egui_state) = egui_backend::with_fltk(&mut win);

    //Init egui ctx
    let egui_ctx = egui::Context::default();

    let state = Rc::new(RefCell::new(egui_state));

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

    let start_time = Instant::now();
    let mut demo_windows = egui_demo_lib::DemoWindows::default();

    while fltk_app.wait() {
        // Clear the screen to dark red
        draw_background(&gl);

        let mut state = state.borrow_mut();
        state.input.time = Some(start_time.elapsed().as_secs_f64());
        let egui_output = egui_ctx.run(state.take_input(), |ctx| {
            demo_windows.ui(&ctx);
        });

        let window_resized = state.window_resized();
        if window_resized {
            win.clear_damage();
        }

        if egui_output.needs_repaint || window_resized {
            //Draw egui texture
            state.fuse_output(&mut win, egui_output.platform_output);
            let meshes = egui_ctx.tessellate(egui_output.shapes);
            painter.paint_and_update_textures(
                &gl,
                state.canvas_size,
                state.pixels_per_point(),
                meshes,
                &egui_output.textures_delta,
            );
            win.swap_buffers();
            win.flush();
            app::awake();
        }
    }
}

fn draw_background<GL: glow::HasContext>(gl: &GL) {
    unsafe {
        gl.clear_color(0.6, 0.3, 0.3, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
    }
}
