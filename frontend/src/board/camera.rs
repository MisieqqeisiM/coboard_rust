#[derive(Clone)]
pub struct Camera {
    x: f32,
    y: f32,
    zoom: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn to_board_coords(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        (screen_x / self.zoom + self.x, screen_y / self.zoom + self.y)
    }

    pub fn change_zoom(&mut self, screen_x: f32, screen_y: f32, amount: f32) {
        self.x += screen_x * (1.0 - 1.0 / amount) / self.zoom;
        self.y += screen_y * (1.0 - 1.0 / amount) / self.zoom;
        self.zoom *= amount;
    }

    pub fn change_position(&mut self, screen_dx: f32, screen_dy: f32) {
        self.x += screen_dx / self.zoom;
        self.y += screen_dy / self.zoom;
    }
}
