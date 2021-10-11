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
    let a = app::App::default().with_scheme(app::Scheme::Gtk);
    app::get_system_colors();
    app::set_font_size(20);
    let mut main_win = window::Window::new(100, 100, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, None);
    let mut gl_win = window::GlWindow::new(5, 5, main_win.w() - 200, main_win.h() - 10, None);
    gl_win.set_mode(Mode::Opengl3);
    gl_win.end();
    let mut col = group::Flex::default()
        .column()
        .with_size(185, 590)
        .right_of(&gl_win, 5);
    col.set_frame(FrameType::DownBox);
    let mut frm = frame::Frame::default();
    frm.set_color(Color::Red.inactive());
    frm.set_frame(FrameType::FlatBox);
    let mut slider = valuator::Slider::default().with_type(valuator::SliderType::HorizontalFill);
    slider.clear_visible_focus();
    slider.set_slider_frame(FrameType::RFlatBox);
    slider.set_slider_size(0.20);
    slider.set_color(Color::Blue.inactive());
    slider.set_selection_color(Color::Red);
    col.set_size(&mut slider, 20);
    col.end();
    main_win.end();
    main_win.make_resizable(true);
    main_win.show();
    gl_win.make_current();

    let (painter, egui_input_state) =
        egui_backend::with_fltk(&mut gl_win, DpiScaling::Custom(1.5));
    let mut egui_ctx = egui::CtxRef::default();

    let state = Rc::from(RefCell::from(egui_input_state));
    let painter = Rc::from(RefCell::from(painter));

    main_win.handle({
        let state = state.clone();
        let painter = painter.clone();
        let mut w = gl_win.clone();
        move |_, ev| match ev {
            enums::Event::Push
            | enums::Event::Released
            | enums::Event::KeyDown
            | enums::Event::KeyUp
            | enums::Event::MouseWheel
            | enums::Event::Resize
            | enums::Event::Move
            | enums::Event::Drag => {
                let mut state = state.borrow_mut();
                state.fuse_input(&mut w, ev, &mut painter.borrow_mut());
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
        frm.set_label(&format!("Hello {}", &name));
        slider.set_value(age as f64 / 120.);

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

        let (egui_output, shapes) = egui_ctx.end_frame();
        state.fuse_output(&mut gl_win, &egui_output);

        let meshes = egui_ctx.tessellate(shapes);

        //Draw egui texture
        painter.paint_jobs(None, meshes, &egui_ctx.texture());

        gl_win.swap_buffers();
        gl_win.flush();
        app::sleep(0.006);
        app::awake();
        if quit {
            break;
        }
    }
}
