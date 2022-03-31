// Draws a simple white triangle
// based on the example from:
// https://github.com/brendanzab/gl-rs/blob/master/gl/examples/triangle.rs

use egui_backend::{
    cast_slice,
    glow::{HasContext, Program, Shader},
};
use egui_glow::glow;
use fltk_egui as egui_backend;
use std::str;

const VS_SRC: &str = "
#version 150
in vec2 position;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}";

const FS_SRC: &str = "
#version 150
out vec4 out_color;
void main() {
    out_color = vec4(1.0, 1.0, 1.0, 1.0);
}";

static VERTEX_DATA: [f32; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];

pub struct Triangle {
    pub program: glow::Program,
    pub vao: glow::VertexArray,
    pub vbo: glow::Buffer,
}

pub fn compile_shader(gl: &glow::Context, src: &str, ty: u32) -> Shader {
    let shader;
    unsafe {
        shader = gl.create_shader(ty).unwrap();
        // Attempt to compile the shader
        gl.shader_source(shader, src);
        gl.compile_shader(shader);

        // Fail on error
        if !gl.get_shader_compile_status(shader) {
            panic!("{}", gl.get_shader_info_log(shader));
        }
    }
    shader
}

pub fn link_program(gl: &glow::Context, vs: Shader, fs: Shader) -> Program {
    unsafe {
        let program = gl.create_program().unwrap();
        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);
        // Get the link status
        let ok = gl.get_program_link_status(program);

        gl.detach_shader(program, vs);
        gl.detach_shader(program, fs);
        gl.delete_shader(vs);
        gl.delete_shader(fs);

        // Fail on error
        if !ok {
            panic!("{}", gl.get_program_info_log(program));
        }
        program
    }
}

impl Triangle {
    pub fn new(gl: &glow::Context) -> Self {
        let vs = compile_shader(gl, VS_SRC, glow::VERTEX_SHADER);
        let fs = compile_shader(gl, FS_SRC, glow::FRAGMENT_SHADER);
        let program = link_program(gl, vs, fs);

        let vao = unsafe { gl.create_vertex_array().unwrap() };
        let vbo = unsafe { gl.create_buffer().unwrap() };
        Triangle { program, vao, vbo }
    }
    pub fn draw(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));

            // Create a Vertex Buffer Object and copy the vertex data to it

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                cast_slice(&VERTEX_DATA),
                glow::STATIC_DRAW,
            );

            // Use shader program
            gl.use_program(Some(self.program));
            gl.bind_frag_data_location(self.program, 0, "out_color");

            // Specify the layout of the vertex data
            let pos_attr = gl.get_attrib_location(self.program, "position").unwrap();
            gl.enable_vertex_attrib_array(pos_attr);
            gl.vertex_attrib_pointer_f32(pos_attr, 2, glow::FLOAT, false, 0, 0);

            // Draw a triangle from the 3 vertices
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }

    pub fn free(self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_buffer(self.vbo);
            gl.delete_vertex_array(self.vao);
        }
    }
}
