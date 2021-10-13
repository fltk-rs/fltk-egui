use egui_backend::{
    egui,
    epi::{
        backend::{AppOutput, FrameBuilder},
        App, IntegrationInfo,
    },
    fltk::{enums::*, prelude::*, *},
    get_frame_time, get_seconds_since_midnight, gl, DpiScaling, Signal,
};

use fltk_egui as egui_backend;
use std::{cell::RefCell, time::Instant};
use std::{rc::Rc, sync::Arc};

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
    let repaint_signal = Arc::new(Signal::default());
    let demo_windows = Rc::new(RefCell::new(egui_demo_lib::WrapApp::default()));
    let app_output = Rc::new(RefCell::new(AppOutput::default()));
    let egui_ctx = Rc::new(RefCell::new(egui::CtxRef::default()));

    // Redraw while window being resized (requires on windows platform),
    // (using win.draw() for correctness, using win.resize_callback() the output looks weird).
    win.draw({
        let state = state.clone();
        let painter = painter.clone();
        let demo_windows = demo_windows.clone();
        let app_output = app_output.clone();
        let egui_ctx = egui_ctx.clone();
        let repaint_signal = repaint_signal.clone();
        move |win| {
            // And here also using "if let ..." for safety.
            if let Ok(mut state) = state.try_borrow_mut() {
                if state.window_resized() {
                    if let Ok(mut painter) = painter.try_borrow_mut() {
                        win.clear_damage();
                        let mut demo_windows = demo_windows.borrow_mut();
                        let mut app_output = app_output.borrow_mut();
                        let mut egui_ctx = egui_ctx.borrow_mut();
                        state.input.time = Some(start_time.elapsed().as_secs_f64());
                        egui_ctx.begin_frame(state.input.take());

                        // Draw background color.
                        draw_color();

                        let mut frame = FrameBuilder {
                            info: IntegrationInfo {
                                web_info: None,
                                cpu_usage: Some(get_frame_time(start_time)),
                                seconds_since_midnight: Some(get_seconds_since_midnight()),
                                native_pixels_per_point: Some(painter.pixels_per_point),
                                prefer_dark_mode: None,
                            },
                            tex_allocator: &mut *painter,
                            output: &mut app_output,
                            repaint_signal: repaint_signal.clone(),
                        }
                        .build();

                        demo_windows.update(&egui_ctx, &mut frame);

                        let (egui_output, shapes) = egui_ctx.end_frame();
                        state.fuse_output(win, &egui_output);

                        let meshes = egui_ctx.tessellate(shapes);

                        //Draw egui texture
                        painter.paint_jobs(None, meshes, &egui_ctx.texture());
                        win.swap_buffers();
                        win.flush();
                        app::awake()
                    }
                }
            }
        }
    });

    while a.wait() {
        let mut state = state.borrow_mut();
        let mut painter = painter.borrow_mut();
        let mut demo_windows = demo_windows.borrow_mut();
        let mut app_output = app_output.borrow_mut();
        let mut egui_ctx = egui_ctx.borrow_mut();
        state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(state.input.take());

        // Draw background color.
        draw_color();

        let mut frame = FrameBuilder {
            info: IntegrationInfo {
                web_info: None,
                cpu_usage: Some(get_frame_time(start_time)),
                seconds_since_midnight: Some(get_seconds_since_midnight()),
                native_pixels_per_point: Some(painter.pixels_per_point),
                prefer_dark_mode: None,
            },
            tex_allocator: &mut *painter,
            output: &mut app_output,
            repaint_signal: repaint_signal.clone(),
        }
        .build();

        demo_windows.update(&egui_ctx, &mut frame);

        let (egui_output, shapes) = egui_ctx.end_frame();

        if app_output.quit {
            break;
        }

        let window_resized = state.window_resized();
        if window_resized {
            win.clear_damage()
        }

        if egui_output.needs_repaint || window_resized {
            //Draw egui texture
            state.fuse_output(&mut win, &egui_output);
            let meshes = egui_ctx.tessellate(shapes);
            painter.paint_jobs(None, meshes, &egui_ctx.texture());
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
