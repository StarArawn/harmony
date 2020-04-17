use nalgebra_glm::{ Vec3, Mat4 };
use specs::{ Component, DenseVecStorage };

pub struct CameraData {
    pub active: bool,
    pub projection: Mat4,
    pub view: Mat4,
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            active: false,
            projection: Mat4::identity(),
            view: Mat4::identity(),
        }
    }
}

impl CameraData {
    pub fn new_perspective(fov: f32, width: f32, height: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            projection: nalgebra_glm::perspective_fov_lh_no(fov, width, height, z_near, z_far),
            view: Mat4::identity(),
            active: true,
        }
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