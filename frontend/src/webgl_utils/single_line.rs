use super::line_buffer::LineBuffer;
use common::entities::{Line, Position};
use web_sys::WebGl2RenderingContext;

type GL = WebGl2RenderingContext;

#[derive(Clone)]
pub struct SingleLine {
    buffer: LineBuffer,
    line: Line,
}

impl SingleLine {
    pub fn new(context: &GL) -> Self {
        Self {
            buffer: LineBuffer::new(context),
            line: Line {
                id: 0,
                points: vec![],
                width: 0.0,
            },
        }
    }

    pub fn draw(&self, gl: &GL) {
        self.buffer.draw(gl);
    }

    pub fn add_point(&mut self, gl: &GL, point: Position) {
        // TODO: update only the tip of the line
        self.line.points.push(point);
        self.buffer.clear();
        self.buffer.add_line(gl, &self.line);
    }

    pub fn set_line(&mut self, gl: &GL, line: Line) {
        self.line = line;
        self.buffer.clear();
        self.buffer.add_line(gl, &self.line);
    }
}
