use egui_backend::{
    egui::{self, Label},
    fltk::{prelude::*, *},
    glow, EguiImageConvertible, EguiSvgConvertible,
};
use fltk::{
    enums::Mode,
    image::{JpegImage, SvgImage},
};
use fltk_egui as egui_backend;
use std::rc::Rc;
use std::{cell::RefCell, time::Instant};
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() {
    let fltk_app = app::App::default();
    let mut win = window::GlWindow::new(100, 100, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, None);
    win.set_mode(Mode::Opengl3);
    win.end();
    win.make_resizable(true);
    win.show();
    win.make_current();

    //Init backend
    let (gl, mut painter, egui_state) = egui_backend::with_fltk(&mut win);

    //Init egui ctx
    let egui_ctx = egui::Context::default();

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

    let retained_egui_image = JpegImage::load("screenshots/egui.jpg")
        .unwrap()
        .egui_image("egui.jpg")
        .unwrap();
    let retained_egui_image_svg = SvgImage::load("screenshots/crates.svg")
        .unwrap()
        .egui_svg_image("crates.svg")
        .unwrap();

    let start_time = Instant::now();
    let mut quit = false;

    while fltk_app.wait() {
        // Clear the screen to dark red
        draw_background(&gl);

        let mut state = state.borrow_mut();
        state.input.time = Some(start_time.elapsed().as_secs_f64());
        let egui_output = egui_ctx.run(state.input.take(), |ctx| {
            egui::CentralPanel::default().show(&ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(Label::new("this is crates.svg badge"));
                    retained_egui_image_svg.show(ui);
                    ui.add(Label::new("this is egui.jpg screenshot"));
                    retained_egui_image.show(ui);
                    if ui
                        .button("Quit?")
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        quit = true;
                    }
                });
            });
        });

        state.fuse_output(&mut win, egui_output.platform_output);

        //Draw egui texture
        let meshes = egui_ctx.tessellate(egui_output.shapes);
        painter.paint_and_update_textures(
            &gl,
            state.canvas_size,
            state.pixels_per_point,
            meshes,
            &egui_output.textures_delta,
        );
        win.swap_buffers();
        win.flush();

        if egui_output.needs_repaint || state.window_resized() {
            app::awake();
        } else if quit {
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
