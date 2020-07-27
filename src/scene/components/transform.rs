use crate::{
    graphics::resources::{BindGroup, GPUResourceManager},
    Application, TransformCount,
};
use bytemuck::{Pod, Zeroable};
use nalgebra_glm::{Mat4, Quat, Vec3};
use std::{borrow::Cow, sync::Arc};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LocalUniform {
    pub world: Mat4,
}
unsafe impl Zeroable for LocalUniform {}
unsafe impl Pod for LocalUniform {}

impl Default for LocalUniform {
    fn default() -> Self {
        Self {
            world: Mat4::identity(),
        }
    }
}

/// A transform used to move entities around in the world. Consists of:
/// index: Used internally
/// position: A Vector3 representing it's world coordinates.
/// scale: A Vector3 representing it's world scale.
/// rotation: A quaternion representing it's world rotation.
/// matrix: A world matrix.
#[derive(Debug, PartialEq, Clone)]
pub struct Transform {
    /// Index of the transform used internally.
    pub(crate) index: u32,
    /// Position of the transform.
    pub position: Vec3,
    /// Scale of the transform.
    pub scale: Vec3,
    /// Rotation quaternion.
    pub rotation: Quat,
    /// Transformation matrix.
    pub matrix: Mat4,
     /* 
        Represents if this entity is culled or not.
        Automatically set by an internal system.
     */
    pub cull: bool,
}

impl Transform {
    /// Creates a new transform with default values.
    pub fn new(app: &mut Application) -> Self {
        let mut index = app.resources.get_mut::<TransformCount>().unwrap();
        index.0 += 1;
        Self::create_bindings(app, index.0);

        Self {
            index: index.0,
            position: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: Quat::identity(),
            matrix: Mat4::identity(),
            cull: false,
        }
    }

    /// Lets you rotate the transform along a specific axis.
    pub fn rotate_on_axis<'a>(&'a mut self, axis: Vec3, angle: f32) -> &'a mut Self {
        self.rotation = self.rotation * nalgebra_glm::quat_angle_axis(angle, &axis);
        self
    }

    /// Rotate the transform along the X axis.
    pub fn rotate_on_x<'a>(&'a mut self, angle: f32) -> &'a mut Self {
        self.rotation =
            self.rotation * nalgebra_glm::quat_angle_axis(angle, &Vec3::new(1.0, 0.0, 0.0));
        self
    }

    /// Rotate the transform along the Y axis.
    pub fn rotate_on_y<'a>(&'a mut self, angle: f32) -> &'a mut Self {
        self.rotation =
            self.rotation * nalgebra_glm::quat_angle_axis(angle, &Vec3::new(0.0, 1.0, 0.0));
        self
    }

    /// Rotate the transform along the Z axis.
    pub fn rotate_on_z<'a>(&'a mut self, angle: f32) -> &'a mut Self {
        self.rotation =
            self.rotation * nalgebra_glm::quat_angle_axis(angle, &Vec3::new(0.0, 0.0, 1.0));
        self
    }

    // pub fn update_euler(&mut self, rotation: Vec3) {
    //     self.rotation = *nalgebra::UnitQuaternion::from_euler_angles(rotation.x, rotation.y, rotation.z).quaternion();
    // }

    // pub fn get_euler(&mut self) -> Vec3 {
    //     let weird_rotation = nalgebra_glm::quat_euler_angles(&self.rotation);
    //     Vec3::new(weird_rotation.z, weird_rotation.y, weird_rotation.x)
    // }

    /// Used internally to recalculate the world matrix.
    /// Can also be used if an updated world matrix is needed.
    pub fn update(&mut self) {
        let scale = nalgebra_glm::scaling(&self.scale);
        let rotation = nalgebra_glm::quat_to_mat4(&self.rotation);
        let translation = nalgebra_glm::translation(&self.position);
        self.matrix = translation * rotation * scale;
    }

    pub(crate) fn create_bindings(app: &Application, index: u32) {
        let resource_manager = app.resources.get::<Arc<GPUResourceManager>>().unwrap();
        let bind_group_layout = resource_manager.get_bind_group_layout("locals").unwrap();
        // This data needs to be saved and passed onto the pipeline.
        let device = app.resources.get_mut::<Arc<wgpu::Device>>().unwrap();
        let local_buffer = device.create_buffer_with_data(
            bytemuck::bytes_of(&LocalUniform::default()),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let local_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: Cow::Borrowed(&[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(local_buffer.slice(..)),
            }]),
            label: None,
        });

        resource_manager.add_multi_bind_group(
            "transform",
            BindGroup::new(0, local_bind_group),
            index,
        );
        resource_manager.add_multi_buffer("transform", local_buffer, index);
    }
}
