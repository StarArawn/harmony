use nalgebra_glm::Mat4;

/// A viewing region for displaying computer graphics.
#[derive(Debug)]
pub struct Viewport {
    width: u32,
    height: u32,
    transformation: Mat4,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            transformation: Mat4::identity(),
        }
    }
}

impl Viewport {
    /// Creates a new [`Viewport`] with the given dimensions.
    pub fn new(width: u32, height: u32) -> Viewport {
        let opengl_to_wgpu_matrix: Mat4 = Mat4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
        );

        Viewport {
            width,
            height,
            transformation: nalgebra_glm::ortho_lh(
                0.0,
                width as f32,
                height as f32,
                0.0,
                -1.0,
                1.0,
            ) * opengl_to_wgpu_matrix,
        }
    }

    /// Returns the dimensions of the [`Viewport`].
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn transformation(&self) -> Mat4 {
        self.transformation
    }
}
