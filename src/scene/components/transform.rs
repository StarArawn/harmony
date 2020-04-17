use nalgebra_glm::{ Vec3, Vec4, Mat4, Quat };
use specs::{ Component, DenseVecStorage };
use bytemuck::{Pod, Zeroable};
use crate::Application;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LocalUniform {
    pub world: Mat4,
}
unsafe impl Zeroable for LocalUniform { }
unsafe impl Pod for LocalUniform { }

impl Default for LocalUniform {
    fn default() -> Self {
        Self {
            world: Mat4::identity(),
        }
    }
}

#[derive(Debug)]
pub struct Transform {
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
    pub matrix: Mat4,

    pub(crate) local_buffer: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl Transform {
    pub fn new(app: &mut Application) -> Self {
        let (local_buffer, bind_group) = Self::create_bindings(app);

        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: Quat::identity(),
            matrix: Mat4::identity(),
            local_buffer,
            bind_group,
        }
    }

    pub fn update_euler(&mut self, rotation: Vec3) {
        self.rotation = *nalgebra::UnitQuaternion::from_euler_angles(rotation.x, rotation.y, rotation.z).quaternion();
    }

    pub fn get_euler(&mut self) -> Vec3 {
        let weird_rotation = nalgebra_glm::quat_euler_angles(&self.rotation);
        Vec3::new(weird_rotation.z,  weird_rotation.x, weird_rotation.y)
    }

    pub fn update(&mut self) {
        let scale =  nalgebra_glm::scaling(&self.scale);
        let rotation = nalgebra_glm::quat_to_mat4(&self.rotation);
        let translation = nalgebra_glm::translation(&self.position);
        self.matrix = translation * rotation * scale;
    }

    pub(crate) fn create_bindings(app: &Application) -> (wgpu::Buffer, wgpu::BindGroup) {
        let bind_group_layout = &app.render_graph.as_ref().unwrap().get("unlit").pipeline.bind_group_layouts[1];
        // This data needs to be saved and passed onto the pipeline.
        let local_buffer = app.renderer.device
            .create_buffer_with_data(bytemuck::bytes_of(&LocalUniform::default()), wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST);

        let global_bind_group = app.renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &local_buffer,
                        range: 0..std::mem::size_of::<LocalUniform>() as u64,
                    },
                },
            ],
            label: None,
        });

        (local_buffer, global_bind_group)
    }
}

impl Component for Transform {
    type Storage = DenseVecStorage<Self>;
}