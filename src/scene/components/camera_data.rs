use nalgebra_glm::{Mat4, Vec3};

pub struct CameraData {
    pub active: bool,
    pub position: Vec3,
    pub projection: Mat4,
    pub view: Mat4,

    pub yaw: f32,
    pub pitch: f32,

    fov: f32,
    z_near: f32,
    z_far: f32,
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            active: false,
            position: Vec3::zeros(),
            projection: Mat4::identity(),
            view: Mat4::identity(),
            yaw: 0.0,
            pitch: 0.0,
            fov: 70.0,
            z_near: 0.1,
            z_far: 100.0,
        }
    }
}

impl CameraData {
    pub fn new_perspective(fov: f32, width: f32, height: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            position: Vec3::zeros(),
            projection: nalgebra_glm::perspective_fov_rh_zo(fov, width, height, z_near, z_far),
            view: Mat4::identity(),
            active: true,
            yaw: 0.0,
            pitch: 0.0,
            fov: fov * std::f32::consts::PI / 180.0, // Convert fov to radians.
            z_near,
            z_far,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.projection =
            nalgebra_glm::perspective_fov_rh_zo(self.fov, width, height, self.z_near, self.z_far);
    }

    pub fn update_view(&mut self, eye: Vec3, at: Vec3, up: Vec3) {
        self.view = nalgebra_glm::look_at_rh(&eye, &at, &up);
    }

    pub fn get_matrix(&self) -> Mat4 {
        self.projection * self.view
    }
}
