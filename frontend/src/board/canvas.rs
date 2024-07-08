use super::camera::Camera;
use crate::webgl_utils::{line_buffer::LineBuffer, program::Program, single_line::SingleLine};
use common::entities::{Line, Position};
use leptos::{
    component, create_effect, create_node_ref, create_signal, ev::resize, store_value, view,
    window, HtmlElement, IntoView, ReadSignal, SignalGet, SignalSet, SignalWith,
};
use leptos_use::use_event_listener;
use std::ops::Deref;
use web_sys::{wasm_bindgen::JsCast, WebGl2RenderingContext};

#[derive(Clone)]
pub struct LineControls {
    pub set: ReadSignal<Line>,
    pub add: ReadSignal<Position>,
}

#[derive(Clone)]
pub struct CanvasControls {
    pub camera: ReadSignal<Camera>,
    pub tmp_line: LineControls,
    pub add_line: ReadSignal<Line>,
}

#[component]
pub fn Canvas(controls: CanvasControls) -> impl IntoView {
    let (canvas_context, set_canvas_context) = create_signal::<Option<CanvasContext>>(None);
    let canvas = create_node_ref::<leptos::html::Canvas>();

    create_effect(move |_| {
        let Some(canvas) = canvas.get() else {
            return;
        };
        let gl = canvas
            .deref()
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();
        let program = Program::new(&gl).unwrap();
        program.use_program(&gl);

        let tmp_line = SingleLine::new(&gl);
        let lines = LineBuffer::new(&gl);

        let ctx = CanvasContext {
            gl,
            program,
            canvas,
            tmp_line,
            lines,
        };

        ctx.fix_resolution();

        set_canvas_context.set(Some(ctx));
    });

    let behavior = move || {
        canvas_context.with(|ctx| {
            if let Some(ctx) = ctx {
                view! { <CanvasBehavior ctx=ctx.clone() controls=controls.clone()/> }
            } else {
                ().into_view()
            }
        })
    };

    view! {
        {behavior}

        <canvas _ref=canvas width=500 height=500></canvas>
    }
}

#[derive(Clone)]
struct CanvasContext {
    gl: WebGl2RenderingContext,
    tmp_line: SingleLine,
    lines: LineBuffer,
    program: Program,
    canvas: HtmlElement<leptos::html::Canvas>,
}

impl CanvasContext {
    fn draw(&self) {
        self.gl.clear_color(1.0, 1.0, 1.0, 1.0);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        self.lines.draw(&self.gl);
        self.tmp_line.draw(&self.gl);
    }

    fn fix_resolution(&self) {
        let width = self.canvas.client_width();
        let height = self.canvas.client_height();
        self.program
            .set_resolution(&self.gl, width as u32, height as u32);
        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);
        self.gl.viewport(0, 0, width, height);
    }

    fn set_camera(&self, camera: Camera) {
        self.program.set_camera(&self.gl, camera);
    }

    fn add_line(&mut self, line: Line) {
        self.lines.add_line(&self.gl, &line);
    }

    fn set_tmp_line(&mut self, line: Line) {
        self.tmp_line.set_line(&self.gl, line);
    }

    fn grow_tmp_line(&mut self, point: Position) {
        self.tmp_line.add_point(&self.gl, point);
    }
}

#[component]
fn CanvasBehavior(ctx: CanvasContext, controls: CanvasControls) -> impl IntoView {
    let ctx = store_value(ctx);

    create_effect(move |_| {
        let line = controls.tmp_line.set.get();
        ctx.update_value(move |ctx| {
            ctx.set_tmp_line(line);
            ctx.draw();
        });
    });

    create_effect(move |_| {
        let point = controls.tmp_line.add.get();
        ctx.update_value(move |ctx| {
            ctx.grow_tmp_line(point);
            ctx.draw();
        });
    });

    create_effect(move |_| {
        let line = controls.add_line.get();
        ctx.update_value(move |ctx| {
            ctx.add_line(line);
            ctx.draw();
        });
    });

    create_effect(move |_| {
        let camera = controls.camera.get();
        ctx.update_value(move |ctx| {
            ctx.set_camera(camera);
            ctx.draw();
        });
    });

    let _ = use_event_listener(window(), resize, move |_| {
        ctx.update_value(move |ctx| {
            ctx.fix_resolution();
            ctx.draw();
        });
    });
}
