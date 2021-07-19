use crate::{core::get_context, math::{Matrix, Vector2}};





pub struct Camera {
    pub zoom: f32
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            zoom: 1.
        }
    }
}


impl Camera {
    pub fn get_projection(&self) -> Matrix {
        let window_size: Vector2 = get_context().window_size.into();
        let mut proj = Matrix::ortho(0.0, window_size.x, window_size.y, 0.0, -100.0, 100.0);
        proj.scale(Vector2::new(1., 1.) * self.zoom);

        proj
    }
}