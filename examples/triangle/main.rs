use egui_backend::{
    egui::{vec2, Color32, Image},
    fltk::{enums::*, prelude::*, *},
    gl, DpiScaling,
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
    let a = app::App::default();
    let mut win = window::GlutWindow::new(100, 100, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, None);
    win.set_mode(Mode::Opengl3);
    win.end();
    win.make_resizable(true);
    win.show();
    win.make_current();

    let (painter, egui_input_state) = egui_backend::with_fltk(&mut win, DpiScaling::Default);
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
    let mut srgba: Vec<Color32> = Vec::new();

    //For now we will just set everything to black, because
    //we will be updating it dynamically later. However, this could just as
    //easily have been some actual picture data loaded in.
    for _ in 0..PIC_HEIGHT {
        for _ in 0..PIC_WIDTH {
            srgba.push(Color32::BLACK);
        }
    }

    //The user texture is what allows us to mix Egui and GL rendering contexts.
    //Egui just needs the texture id, as the actual texture is managed by the backend.
    let chip8_tex_id = painter.borrow_mut().new_user_texture(
        (PIC_WIDTH as usize, PIC_HEIGHT as usize),
        &srgba,
        false,
    );

    //We will draw a crisp white triangle using OpenGL.
    let triangle = triangle::Triangle::new();

    //Some variables to help draw a sine wave
    let mut sine_shift = 0f32;
    let mut amplitude: f32 = 50f32;
    let mut test_str: String =
        "A text box to write in. Cut, copy, paste commands are available.".to_owned();
    let mut quit = false;

    while a.wait() {
        let mut state = state.borrow_mut();
        let mut painter = painter.borrow_mut();
        state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(state.input.take());

        //An example of how OpenGL can be used to draw custom stuff with egui
        //overlaying it:
        //First clear the background to something nice.
        unsafe {
            // Clear the screen to black
            gl::ClearColor(0.3, 0.6, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        //Then draw our triangle.
        triangle.draw();

        let mut srgba: Vec<Color32> = Vec::new();
        let mut angle = 0f32;
        //Draw a cool sine wave in a buffer.
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

        //This updates the previously initialized texture with new data.
        //If we weren't updating the texture, this call wouldn't be required.
        painter.update_user_texture_data(chip8_tex_id, &srgba);

        egui::Window::new("Egui with FLTK and GL").show(&egui_ctx, |ui| {
            //Image just needs a texture id reference, so we just pass it the texture id that was returned to us
            //when we previously initialized the texture.
            ui.add(Image::new(chip8_tex_id, vec2(PIC_WIDTH as f32, PIC_HEIGHT as f32)));
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

        let (egui_output, paint_cmds) = egui_ctx.end_frame();
        state.fuse_output(&mut win, &egui_output);

        let paint_jobs = egui_ctx.tessellate(paint_cmds);

        //Note: passing a bg_color to paint_jobs will clear any previously drawn stuff.
        //Use this only if egui is being used for all drawing and you aren't mixing your own Open GL
        //drawing calls with it.
        //Since we are custom drawing an OpenGL Triangle we don't need egui to clear the background.
        painter.paint_jobs(None, paint_jobs, &egui_ctx.texture());

        win.swap_buffers();
        win.flush();
        app::sleep(0.006);
        app::awake();
        if quit {
            break;
        }
    }
}
