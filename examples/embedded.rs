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
    let fltk_app = app::App::default().with_scheme(app::Scheme::Gtk);
    app::get_system_colors();
    app::set_font_size(20);
    let mut main_win =
        window::Window::new(100, 100, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, None).center_screen();
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

    //Init backend
    let (gl, mut painter, egui_state) = egui_backend::with_fltk(&mut gl_win);

    //Init egui ctx
    let egui_ctx = egui::Context::default();

    let state = Rc::from(RefCell::from(egui_state));

    main_win.handle({
        let state = state.clone();
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
                // Using "if let ..." for safety.
                if let Ok(mut state) = state.try_borrow_mut() {
                    state.fuse_input(&mut w, ev);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    });

    // Set visual scale or egui display scaling
    state.borrow_mut().set_visual_scale(1.5);

    let start_time = Instant::now();
    let mut name = String::new();
    let mut age: i32 = 0;
    let mut quit = false;

    while fltk_app.wait() {
        // Clear the screen to dark red
        draw_background(&*gl);

        let mut state = state.borrow_mut();
        state.input.time = Some(start_time.elapsed().as_secs_f64());
        frm.set_label(&format!("Hello {}", &name));
        slider.set_value(age as f64 / 120.);
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

        state.fuse_output(&mut gl_win, egui_output.platform_output);

        let meshes = egui_ctx.tessellate(egui_output.shapes);

        //Draw egui texture
        painter.paint_and_update_textures(
            state.canvas_size,
            state.pixels_per_point(),
            &meshes,
            &egui_output.textures_delta,
        );

        gl_win.swap_buffers();
        gl_win.flush();
        app::sleep(0.006);
        app::awake();
        if quit {
            break;
        }
    }
}

fn draw_background<GL: glow::HasContext>(gl: &GL) {
    unsafe {
        gl.clear_color(0.6, 0.3, 0.3, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
    }
}
