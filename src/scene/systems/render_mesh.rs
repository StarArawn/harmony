use specs::{ReadStorage, System};
use crate::AssetManager;
use crate::{
    graphics::{
        pipelines::UnlitUniforms,
        Pipeline
    },
    scene::components::{ CameraData, Mesh }
};

pub struct RenderMesh<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) asset_manager: &'a AssetManager,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
    pub(crate) frame: &'a wgpu::SwapChainOutput,
    pub(crate) pipeline: &'a Pipeline,
    pub(crate) bind_group: &'a wgpu::BindGroup,
    pub(crate) constants_buffer: &'a wgpu::Buffer,
}

impl<'a> System<'a> for RenderMesh<'a> {
    type SystemData = (
        ReadStorage<'a, CameraData>,
        ReadStorage<'a, Mesh>,
    );

    fn run(&mut self, (camera_data, mesh): Self::SystemData) {
        use specs::Join;
        let filtered_camera_data: Vec<&CameraData> = camera_data.join().filter(|data: &&CameraData| data.active).collect();
        let camera_data= filtered_camera_data.first();

        if camera_data.is_none() {
            return;
        }

        let camera_data = camera_data.unwrap();
        let camera_matrix = camera_data.get_matrix();

        let uniforms = UnlitUniforms {
            view_projection: camera_matrix,
        };

        let constants_buffer = self.device.create_buffer_with_data(bytemuck::bytes_of(&uniforms), wgpu::BufferUsage::COPY_SRC);

        self.encoder.copy_buffer_to_buffer(
            &constants_buffer,
            0,
            &self.constants_buffer,
            0,
            std::mem::size_of::<UnlitUniforms>() as u64,
        );

        let mut render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[
                wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &self.frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                },
            ],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.pipeline.pipeline);
        render_pass.set_bind_group(0, self.bind_group, &[]);

        for mesh in mesh.join() {  
            let mesh: &Mesh = mesh;
            let asset_mesh = self.asset_manager.get_mesh(mesh.mesh_name.clone());
            for sub_mesh in asset_mesh.sub_meshes.iter() {
                render_pass.set_index_buffer(&sub_mesh.index_buffer, 0, 0);
                render_pass.set_vertex_buffer(0, &sub_mesh.vertex_buffer, 0, 0);
                render_pass.draw_indexed(0..sub_mesh.index_count as u32, 0, 0..1);
            }
        }
    }
}