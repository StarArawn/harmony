use ultraviolet::{ Vec3, Rotor3, Mat4 };
use specs::{ Component, DenseVecStorage };

#[derive(Default)]
pub struct Transform {
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Rotor3,
    pub matrix: Mat4,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(0.0, 0.0, 0.0),
            rotation: Rotor3::from_euler_angles(0.0, 0.0, 0.0),
            matrix: Mat4::identity(),
        }
    }
}

impl Component for Transform {
    type Storage = DenseVecStorage<Self>;
}