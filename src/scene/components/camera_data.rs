use ultraviolet::{ Vec3, Mat4 };
use specs::{ Component, DenseVecStorage };

#[derive(Default)]
pub struct CameraData {
    pub active: bool,
    pub projection: Mat4,
    pub view: Mat4,
}

impl CameraData {
    pub fn new_perspective(fov: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            projection: ultraviolet::projection::perspective_vk(fov, aspect_ratio, z_near, z_far),
            view: Mat4::identity(),
            active: true,
        }
    }

    pub fn update_view(&mut self, eye: Vec3, at: Vec3, up: Vec3) {
        self.view = Mat4::look_at(eye, at, up);
    }

    pub fn get_matrix(&self) -> Mat4 {
        self.projection * self.view
    }
}

impl Component for CameraData {
    type Storage = DenseVecStorage<Self>;
}