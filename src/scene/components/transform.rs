use ultraviolet::{ Vec3, Vec4, Mat4 };
use specs::{ Component, DenseVecStorage };
use bytemuck::{Pod, Zeroable};
use crate::Application;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct LocalUniform {
    pub world: Mat4,
}
unsafe impl Zeroable for LocalUniform { }
unsafe impl Pod for LocalUniform { }

#[derive(Debug)]
pub struct Transform {
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Vec3,
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
            rotation: Vec3::new(0.0, 0.0, 0.0),
            matrix: Mat4::identity(),
            local_buffer,
            bind_group,
        }
    }

    pub fn update(&mut self) {
        let scale =  Mat4::from_nonuniform_scale(Vec4::new(self.scale.x, self.scale.y, self.scale.z, 1.0));
        let rotation = Mat4::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z);
        let translation = Mat4::from_translation(self.position);
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