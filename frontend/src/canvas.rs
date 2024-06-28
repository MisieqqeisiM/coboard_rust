use std::ops::Deref;

use itertools::Itertools;
use leptos::{component, create_effect, create_node_ref, logging::log, view, IntoView};
use nalgebra::Point2;
use web_sys::{js_sys, wasm_bindgen::JsCast, WebGl2RenderingContext, WebGlProgram, WebGlShader};

use crate::{line_drawing::line_into_triangles, Client};

#[component]
pub fn Canvas(client: Client) -> impl IntoView {
    let canvas = create_node_ref::<leptos::html::Canvas>();

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
        let program = create_program(&context).unwrap();
        context.use_program(Some(&program));
        context.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        context.clear_color(0.7, 0.7, 0.7, 1.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        let vertices = line_into_triangles(
            vec![
                Point2::new(0.0, 0.0),
                Point2::new(100.0, 0.0),
                Point2::new(0.0, 100.0),
                Point2::new(-100.0, 50.0),
                Point2::new(-110.0, 70.0),
            ],
            30.0,
        )
        .into_iter()
        .flat_map(|p| [p.x as f32, p.y as f32])
        .collect_vec();

        let position_attribute_location = context.get_attrib_location(&program, "position");
        let buffer = context.create_buffer().unwrap();
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        unsafe {
            let positions_array_buf_view = js_sys::Float32Array::view(&vertices);
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &positions_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        let vao = context.create_vertex_array().unwrap();
        context.bind_vertex_array(Some(&vao));
        context.vertex_attrib_pointer_with_i32(
            position_attribute_location as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        context.enable_vertex_attrib_array(position_attribute_location as u32);
        let vert_count = (vertices.len() / 2) as i32;
        context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vert_count);
    });
    view! { <canvas _ref=canvas width=500 height=500></canvas> }
}

const VERTEX_SHADER: &'static str = include_str!("shaders/vertex_shader.glsl");
const FRAGMENT_SHADER: &'static str = include_str!("shaders/fragment_shader.glsl");

fn create_program(context: &WebGl2RenderingContext) -> Result<WebGlProgram, String> {
    let vertex_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        VERTEX_SHADER,
    )?;
    let fragment_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        &FRAGMENT_SHADER,
    )?;
    link_program(&context, &vertex_shader, &fragment_shader)
}

fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or("Unable to create shader object")?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);
    let compiled = context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false);
    if compiled {
        Ok(shader)
    } else {
        let error = context
            .get_shader_info_log(&shader)
            .unwrap_or("Unknown error when compiling shader".to_owned());
        Err(error)
    }
}

fn link_program(
    context: &WebGl2RenderingContext,
    vertex_shader: &WebGlShader,
    fragment_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or("Unable to create shader object")?;
    context.attach_shader(&program, vertex_shader);
    context.attach_shader(&program, fragment_shader);
    context.link_program(&program);
    let linked = context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false);
    if linked {
        Ok(program)
    } else {
        let error = context
            .get_program_info_log(&program)
            .unwrap_or("Unknown error when linking program".to_owned());
        Err(error)
    }
}
