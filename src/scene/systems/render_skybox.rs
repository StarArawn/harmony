use specs::{ReadStorage, System, WriteStorage};
use crate::AssetManager;
use crate::{
    graphics::{
        pipelines::SkyboxUniforms,
        Pipeline,
    },
    scene::components::{ CameraData, SkyboxData }
};

pub struct RenderSkybox<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) asset_manager: &'a AssetManager,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
    pub(crate) frame_view: &'a wgpu::TextureView,
    pub(crate) pipeline: &'a Pipeline,
    pub(crate) constants_buffer: &'a wgpu::Buffer,
    pub(crate) global_bind_group: &'a wgpu::BindGroup,
}

impl<'a> System<'a> for RenderSkybox<'a> {
    type SystemData = (
        ReadStorage<'a, CameraData>,
        ReadStorage<'a, SkyboxData>,
    );

    fn run(&mut self, (camera_data, skyboxes): Self::SystemData) {
        use specs::Join;
        let filtered_camera_data: Vec<&CameraData> = camera_data.join().filter(|data: &&CameraData| data.active).collect();
        let camera_data= filtered_camera_data.first();

        if camera_data.is_none() {
            return;
        }

        let camera_data = camera_data.unwrap();
        let camera_matrix = camera_data.get_matrix();

        let uniforms = SkyboxUniforms {
            view_projection: camera_matrix,
        };

        let constants_buffer = self.device.create_buffer_with_data(bytemuck::bytes_of(&uniforms), wgpu::BufferUsage::COPY_SRC);
        
        self.encoder.copy_buffer_to_buffer(
            &constants_buffer,
            0,
            &self.constants_buffer,
            0,
            std::mem::size_of::<SkyboxUniforms>() as u64,
        );

        let mut render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[
                wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: self.frame_view,
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
        render_pass.set_bind_group(0, self.global_bind_group, &[]);

        for skybox in skyboxes.join() {
            let hdr_image = self.asset_manager.get_hdr_image(&skybox.name);
            render_pass.set_bind_group(1, hdr_image.cubemap_bind_group.as_ref().unwrap(), &[]);
            render_pass.draw(0 .. 3 as u32, 0 .. 1);
        }

    }
}