use egui_backend::{
    egui,
    fltk::{enums::*, prelude::*, *},
    gl, DpiScaling,
};

use fltk_egui as egui_backend;
use std::rc::Rc;
use std::{cell::RefCell, time::Instant};

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() {
    let a = app::App::default();
    let mut win = window::GlWindow::new(100, 100, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, None)
        .center_screen();
    win.set_mode(Mode::Opengl3);
    win.end();
    win.make_resizable(true);
    win.show();
    win.make_current();
    let (painter, egui_input_state) = egui_backend::with_fltk(&mut win, DpiScaling::Custom(1.25));
    let state = Rc::new(RefCell::new(egui_input_state));
    let painter = Rc::new(RefCell::new(painter));

    win.handle({
        let state = state.clone();
        let painter = painter.clone();
        move |win, ev| match ev {
            enums::Event::Push
            | enums::Event::Released
            | enums::Event::KeyDown
            | enums::Event::KeyUp
            | enums::Event::MouseWheel
            | enums::Event::Resize
            | enums::Event::Move
            | enums::Event::Drag => {
                let mut handled = false;
                // Using "if let ..." for safety.
                if let Ok(mut state) = state.try_borrow_mut() {
                    if let Ok(mut painter) = painter.try_borrow_mut() {
                        state.fuse_input(win, ev, &mut painter);
                        handled = true;
                    }
                }
                handled
            }
            _ => false,
        }
    });

    let start_time = Instant::now();
    let mut demo_windows = egui_demo_lib::DemoWindows::default();
    let mut egui_ctx = egui::CtxRef::default();

    while a.wait() {
        let mut state = state.borrow_mut();
        let mut painter = painter.borrow_mut();
        state.input.time = Some(start_time.elapsed().as_secs_f64());
        let (egui_output, shapes) = egui_ctx.run(state.input.take(), |ctx| {
            // Draw background color.
            draw_color();

            demo_windows.ui(&ctx);
        });

        let window_resized = state.window_resized();
        if window_resized {
            win.clear_damage()
        }

        if egui_output.needs_repaint || window_resized {
            //Draw egui texture
            state.fuse_output(&mut win, &egui_output);
            let meshes = egui_ctx.tessellate(shapes);
            painter.paint_jobs(None, meshes, &egui_ctx.font_image());
            win.swap_buffers();
            win.flush();
            app::awake()
        }
    }
}

fn draw_color() {
    unsafe {
        // Clear the screen to dark red
        gl::ClearColor(0.6, 0.3, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}
