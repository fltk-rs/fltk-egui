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
    let mut win = window::GlutWindow::new(100, 100, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, None);
    win.set_mode(Mode::Opengl3);
    win.end();
    win.make_resizable(true);
    win.show();
    win.make_current();

    let (painter, egui_input_state) = egui_backend::with_fltk(&mut win, DpiScaling::Custom(1.5));
    let mut egui_ctx = egui::CtxRef::default();

    let state = Rc::from(RefCell::from(egui_input_state));
    let painter = Rc::from(RefCell::from(painter));

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
                let mut state = state.borrow_mut();
                state.fuse_input(win, ev, &mut painter.borrow_mut());
                true
            }
            _ => false,
        }
    });

    let start_time = Instant::now();
    let mut name = String::new();
    let mut age = 0;
    let mut quit = false;

    while a.wait() {
        let mut state = state.borrow_mut();
        let mut painter = painter.borrow_mut();
        state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(state.input.take());

        unsafe {
            // Clear the screen to black
            gl::ClearColor(0.6, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        egui::CentralPanel::default().show(&egui_ctx, |ui| {
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

        let (egui_output, paint_cmds) = egui_ctx.end_frame();
        state.fuse_output(&mut win, &egui_output);

        let paint_jobs = egui_ctx.tessellate(paint_cmds);

        //Draw egui texture
        painter.paint_jobs(None, paint_jobs, &egui_ctx.texture());

        win.swap_buffers();
        win.flush();

        if egui_output.needs_repaint {
            app::awake()
        } else if quit {
            break;
        }
    }
}
