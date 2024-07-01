use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use crate::board::camera::Camera;

const VERTEX_SHADER: &'static str = include_str!("../shaders/vertex_shader.glsl");
const FRAGMENT_SHADER: &'static str = include_str!("../shaders/fragment_shader.glsl");

#[derive(Clone)]
pub struct Program {
    program: WebGlProgram,
    resolution_location: Option<WebGlUniformLocation>,
    camera_pos_location: Option<WebGlUniformLocation>,
    camera_zoom_location: Option<WebGlUniformLocation>,
}

impl Program {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<Self, String> {
        let program = create_program(&gl)?;
        let resolution_location = gl.get_uniform_location(&program, "resolution");
        let camera_pos_location = gl.get_uniform_location(&program, "camera_pos");
        let camera_zoom_location = gl.get_uniform_location(&program, "camera_zoom");

        Ok(Self {
            program,
            resolution_location,
            camera_pos_location,
            camera_zoom_location,
        })
    }

    pub fn use_program(&self, gl: &WebGl2RenderingContext) {
        gl.use_program(Some(&self.program));
    }

    pub fn set_resolution(&self, gl: &WebGl2RenderingContext, width: u32, height: u32) {
        self.use_program(gl);
        gl.uniform2f(
            self.resolution_location.as_ref(),
            width as f32,
            height as f32,
        );
    }

    pub fn set_camera(&self, gl: &WebGl2RenderingContext, camera: Camera) {
        self.use_program(gl);
        gl.uniform2f(self.camera_pos_location.as_ref(), camera.x(), camera.y());
        gl.uniform1f(self.camera_zoom_location.as_ref(), camera.zoom());
    }
}

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
