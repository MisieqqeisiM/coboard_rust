use std::{borrow::Borrow, cell::RefCell, ops::Deref, rc::Rc};

use common::entities::{Line, Position};
use itertools::Itertools;
use leptos::{
    component, create_effect, create_node_ref, create_signal, document, ev::resize, logging::log,
    store_value, svg::line, view, window, IntoView, ReadSignal, SignalGet, SignalGetUntracked,
    SignalSet, SignalUpdate, SignalWith,
};
use leptos_use::use_event_listener;
use nalgebra::Point2;
use web_sys::{
    js_sys, wasm_bindgen::JsCast, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader,
    WebGlVertexArrayObject,
};

use crate::{line_drawing::line_into_triangle_strip, webgl_utils::program::Program, Client};

use super::camera::Camera;

struct LineBuffer {
    vao: WebGlVertexArrayObject,
    vbo: WebGlBuffer,
    vertex_count: i32,
}

impl LineBuffer {
    fn new(context: &WebGl2RenderingContext) -> Self {
        let vbo = context.create_buffer().unwrap();
        let vao = context.create_vertex_array().unwrap();
        context.bind_vertex_array(Some(&vao));
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
        context.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(0);
        Self {
            vao,
            vbo,
            vertex_count: 0,
        }
    }

    fn draw(&self, context: &WebGl2RenderingContext) {
        context.bind_vertex_array(Some(&self.vao));
        context.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, self.vertex_count);
    }

    fn set_line(&mut self, context: &WebGl2RenderingContext, line: &Line) {
        let vertices = line_into_triangle_strip(
            line.points
                .iter()
                .map(|Position { x, y }| Point2::new(x.to_owned() as f64, y.to_owned() as f64))
                .collect(),
            line.width as f64,
        )
        .into_iter()
        .flat_map(|p| [p.x as f32, p.y as f32])
        .collect_vec();

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vbo));
        unsafe {
            let positions_array_buf_view = js_sys::Float32Array::view(&vertices);
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &positions_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        context.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(0);
        self.vertex_count = (vertices.len() / 2) as i32;
    }
}

#[component]
pub fn Canvas(tmp_line: ReadSignal<Line>, camera: ReadSignal<Camera>) -> impl IntoView {
    let canvas = create_node_ref::<leptos::html::Canvas>();
    let (context, set_context) = create_signal::<Option<WebGl2RenderingContext>>(None);
    let line_buffer = store_value::<Option<LineBuffer>>(None);
    let (program, set_program) = create_signal::<Option<Program>>(None);

    let draw = move || {
        let Some(context) = context.get_untracked() else {
            return;
        };
        context.clear_color(1.0, 1.0, 1.0, 1.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        line_buffer.with_value(|line_buffer| {
            line_buffer
                .as_ref()
                .map(|line_buffer| line_buffer.draw(&context));
        });
    };

    let resize_canvas = move || {
        let Some(canvas) = canvas.get_untracked() else {
            return;
        };
        let Some(context) = context.get_untracked() else {
            return;
        };
        let Some(program) = program.get_untracked() else {
            return;
        };

        let width = canvas.client_width();
        let height = canvas.client_height();
        program.set_resolution(&context, width as u32, height as u32);
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);
        context.viewport(0, 0, width, height);
    };

    create_effect(move |_| {
        let Some(canvas) = canvas.get() else {
            return;
        };
        let canvas = canvas.deref();
        let context = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();
        let program = Program::new(&context).unwrap();
        program.use_program(&context);

        line_buffer.set_value(Some(LineBuffer::new(&context)));
        set_program.set(Some(program));
        set_context.set(Some(context));

        resize_canvas();
        draw();
    });

    create_effect(move |_| {
        let Some(context) = context.get() else {
            return;
        };
        let line = tmp_line.get();
        line_buffer.update_value(move |line_buffer| {
            line_buffer
                .as_mut()
                .map(|line_buffer| line_buffer.set_line(&context, &line));
        });
        draw();
    });

    let _ = use_event_listener(window(), resize, move |_| {
        resize_canvas();
        draw();
    });

    create_effect(move |_| {
        let Some(context) = context.get_untracked() else {
            return;
        };
        let Some(program) = program.get_untracked() else {
            return;
        };
        program.set_camera(&context, camera.get());
        draw();
    });

    view! { <canvas _ref=canvas width=500 height=500></canvas> }
}
