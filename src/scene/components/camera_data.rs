use nalgebra_glm::{Mat4, Vec3};
use specs::{Component, DenseVecStorage};

pub struct CameraData {
    pub active: bool,
    pub projection: Mat4,
    pub view: Mat4,

    fov: f32,
    z_near: f32,
    z_far: f32,
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            active: false,
            projection: Mat4::identity(),
            view: Mat4::identity(),
            fov: 70.0,
            z_near: 0.1,
            z_far: 100.0,
        }
    }
}

impl CameraData {
    pub fn new_perspective(fov: f32, width: f32, height: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            projection: nalgebra_glm::perspective_fov_lh_no(fov, width, height, z_near, z_far),
            view: Mat4::identity(),
            active: true,
            fov,
            z_near,
            z_far,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.projection =
            nalgebra_glm::perspective_fov_lh_no(self.fov, width, height, self.z_near, self.z_far);
    }

    pub fn update_view(&mut self, eye: Vec3, at: Vec3, up: Vec3) {
        self.view = nalgebra_glm::look_at_lh(&eye, &at, &up);
    }

    pub fn get_matrix(&self) -> Mat4 {
        self.projection * self.view
    }
}

impl Component for CameraData {
    type Storage = DenseVecStorage<Self>;
}
