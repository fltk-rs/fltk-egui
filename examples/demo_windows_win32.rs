// Allow redraw while window being resized on windows platform (double buffered).

#[cfg(target_os = "windows")]
use egui_backend::{
    egui,
    fltk::{enums::*, prelude::*, *},
    glow,
};
#[cfg(target_os = "windows")]
use egui_demo_lib::DemoWindows;
#[cfg(target_os = "windows")]
use fltk::{app::App, window::GlutWindow};
#[cfg(target_os = "windows")]
use fltk_egui as egui_backend;
#[cfg(target_os = "windows")]
use std::rc::Rc;
#[cfg(target_os = "windows")]
use std::{cell::RefCell, time::Instant};
#[cfg(target_os = "windows")]
const SCREEN_WIDTH: u32 = 800;
#[cfg(target_os = "windows")]
const SCREEN_HEIGHT: u32 = 600;

#[cfg(target_os = "windows")]
fn main() {
    let fltk_app = app::App::default();
    let mut win = window::GlWindow::new(100, 100, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, None)
        .center_screen();
    win.set_mode(Mode::Opengl3);
    win.end();
    win.make_resizable(true);
    win.show();
    win.make_current();

    let demo = egui_demo_lib::DemoWindows::default();
    run_egui(fltk_app, win, demo);
}

#[cfg(target_os = "windows")]
fn run_egui(fltk_app: App, mut win: GlutWindow, demo: DemoWindows) {
    // Init backend
    let (egui_painter, egui_state) = egui_backend::with_fltk(&mut win);
    let painter = Rc::new(RefCell::new(egui_painter));
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

    let egui_ctx = Rc::new(egui::Context::default());
    let demo_windows = Rc::new(RefCell::new(demo));
    let start_time = Instant::now();

    win.draw({
        let painter = painter.clone();
        let state = state.clone();
        let egui_ctx = egui_ctx.clone();
        let demo_windows = demo_windows.clone();
        move |win_draw| {
            if let Ok(mut painter) = painter.try_borrow_mut() {
                // Also borrow required variables as mutable
                let mut state = state.borrow_mut();
                let mut demo_windows = demo_windows.borrow_mut();

                // Clear the screen to dark red
                let gl = painter.gl().as_ref();
                draw_background(gl);

                state.input.time = Some(start_time.elapsed().as_secs_f64());
                let egui_output = egui_ctx.run(state.take_input(), |ctx| {
                    demo_windows.ui(&ctx);
                });

                if egui_output.needs_repaint || state.window_resized() {
                    //Draw egui texture
                    state.fuse_output(win_draw, egui_output.platform_output);
                    let meshes = egui_ctx.tessellate(egui_output.shapes);
                    painter.paint_and_update_textures(
                        state.canvas_size,
                        state.pixels_per_point(),
                        &meshes,
                        &egui_output.textures_delta,
                    );
                    win_draw.swap_buffers();
                    win_draw.flush();
                    app::awake();
                }
            }
        }
    });

    while fltk_app.wait() {
        // Borrow required variables as mutable
        let mut state = state.borrow_mut();
        let mut painter = painter.borrow_mut();
        let mut demo_windows = demo_windows.borrow_mut();

        // Clear the screen to dark red
        let gl = painter.gl().as_ref();
        draw_background(gl);

        state.input.time = Some(start_time.elapsed().as_secs_f64());
        let egui_output = egui_ctx.run(state.take_input(), |ctx| {
            demo_windows.ui(&ctx);
        });

        if egui_output.needs_repaint || state.window_resized() {
            //Draw egui texture
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
    }

    painter.borrow_mut().destroy();
}

#[cfg(target_os = "windows")]
fn draw_background<GL: glow::HasContext>(gl: &GL) {
    unsafe {
        gl.clear_color(0.6, 0.3, 0.3, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("\nthis example for windows platform only.")
}
