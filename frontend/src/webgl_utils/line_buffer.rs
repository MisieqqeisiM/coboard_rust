use std::{collections::HashMap, mem::swap};

use common::entities::Line;
use itertools::Itertools;
use leptos::logging::log;
use web_sys::{js_sys::Float32Array, WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

use crate::line_drawing::line_into_triangle_strip;

type GL = WebGl2RenderingContext;

#[derive(Clone)]
struct Location {
    start: u32,
    size: u32,
}

#[derive(Clone)]
pub struct LineBuffer {
    vao: WebGlVertexArrayObject,
    vbo: WebGlBuffer,
    copy_buffer: WebGlBuffer,
    vertex_count: u32,
    capacity: u32,
    location_by_id: HashMap<i64, Location>,
}

impl LineBuffer {
    pub fn new(context: &WebGl2RenderingContext) -> Self {
        let capacity: u32 = 1024;
        let vbo = context.create_buffer().unwrap();
        let copy_buffer = context.create_buffer().unwrap();
        let vao = context.create_vertex_array().unwrap();
        context.bind_vertex_array(Some(&vao));
        context.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(0);
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&copy_buffer));
        context.buffer_data_with_i32(
            WebGl2RenderingContext::ARRAY_BUFFER,
            capacity as i32,
            WebGl2RenderingContext::DYNAMIC_COPY,
        );
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
        context.buffer_data_with_i32(
            WebGl2RenderingContext::ARRAY_BUFFER,
            capacity as i32,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
        Self {
            vao,
            vbo,
            copy_buffer,
            vertex_count: 0,
            capacity,
            location_by_id: HashMap::new(),
        }
    }

    fn realloc(&mut self, gl: &GL) {
        gl.bind_buffer(GL::COPY_READ_BUFFER, Some(&self.vbo));
        gl.bind_buffer(GL::COPY_WRITE_BUFFER, Some(&self.copy_buffer));
        gl.buffer_data_with_i32(
            GL::COPY_WRITE_BUFFER,
            2 * self.capacity as i32,
            GL::DYNAMIC_DRAW,
        );
        gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
            GL::COPY_READ_BUFFER,
            GL::COPY_WRITE_BUFFER,
            0,
            0,
            self.capacity as i32,
        );
        gl.buffer_data_with_i32(
            GL::COPY_READ_BUFFER,
            2 * self.capacity as i32,
            GL::DYNAMIC_COPY,
        );
        swap(&mut self.vbo, &mut self.copy_buffer);
        self.capacity *= 2;
        log!("{} {}", self.capacity, self.vertex_count * 8);
    }

    pub fn draw(&self, gl: &GL) {
        gl.bind_vertex_array(Some(&self.vao));
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vbo));
        gl.vertex_attrib_pointer_with_i32(0, 2, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);
        gl.draw_arrays(
            WebGl2RenderingContext::TRIANGLE_STRIP,
            0,
            self.vertex_count as i32,
        );
    }

    pub fn add_line(&mut self, gl: &GL, line: &Line) {
        let vertices = line_into_triangle_strip(line)
            .into_iter()
            .flat_map(|p| [p.x as f32, p.y as f32])
            .collect_vec();
        let location = Location {
            start: self.vertex_count,
            size: vertices.len() as u32 / 2,
        };
        self.vertex_count += vertices.len() as u32 / 2;
        while self.capacity < self.vertex_count * 8 {
            self.realloc(gl);
        }
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vbo));
        unsafe {
            let view = Float32Array::view(&vertices);
            gl.buffer_sub_data_with_i32_and_array_buffer_view(
                GL::ARRAY_BUFFER,
                location.start as i32 * 8,
                &view,
            );
        }
        self.location_by_id.insert(line.id, location);
    }

    pub fn clear(&mut self) {
        self.location_by_id.clear();
        self.vertex_count = 0;
    }
}
