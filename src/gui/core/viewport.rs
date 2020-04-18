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
            transformation: Self::create_ortho(
                0.0,
                width as f32,
                0.0,
                height as f32,
                -1.0,
                1.0,
            ), // * opengl_to_wgpu_matrix,
        }
    }

    pub fn create_ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
        let c0r0 = 2.0 / (right - left);
        let c0r1 = 0.0;
        let c0r2 = 0.0;
        let c0r3 = 0.0;
    
        let c1r0 = 0.0;
        let c1r1 = 2.0 / (bottom - top);
        let c1r2 = 0.0;
        let c1r3 = 0.0;
    
        let c2r0 = 0.0;
        let c2r1 = 0.0;
        let c2r2 = 1.0 / (far - near);
        let c2r3 = 0.0;
    
        let c3r0 = -(right + left) / (right - left);
        let c3r1 = -(bottom + top) / (bottom - top);
        let c3r2 = near / (near - far);
        let c3r3 = 1.0;
    
        Mat4::new(
            c0r0, c1r0, c2r0, c3r0,
            c0r1, c1r1, c2r1, c3r1,
            c0r2, c1r2, c2r2, c3r2,
            c0r3, c1r3, c2r3, c3r3,
        )
    }
    

    /// Returns the dimensions of the [`Viewport`].
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn transformation(&self) -> Mat4 {
        self.transformation
    }
}
