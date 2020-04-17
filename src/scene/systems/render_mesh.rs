use crate::AssetManager;
use crate::{
    graphics::{material::Material, pipelines::UnlitUniforms, Pipeline},
    scene::components::{transform::LocalUniform, CameraData, Mesh, Transform},
};
use specs::{ReadStorage, System, WriteStorage};

pub struct RenderMesh<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) asset_manager: &'a AssetManager,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
    pub(crate) frame_view: &'a wgpu::TextureView,
    pub(crate) pipeline: &'a Pipeline,
    pub(crate) constants_buffer: &'a wgpu::Buffer,
    pub(crate) global_bind_group: &'a wgpu::BindGroup,
    pub(crate) depth: &'a wgpu::TextureView,
}

impl<'a> System<'a> for RenderMesh<'a> {
    type SystemData = (
        ReadStorage<'a, CameraData>,
        ReadStorage<'a, Mesh>,
        ReadStorage<'a, crate::scene::components::Material>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (camera_data, meshes, materials, mut transforms): Self::SystemData) {
        use specs::Join;
        if transforms.count() == 0 {
            return;
        }
        let filtered_camera_data: Vec<&CameraData> = camera_data
            .join()
            .filter(|data: &&CameraData| data.active)
            .collect();
        let camera_data = filtered_camera_data.first();

        if camera_data.is_none() {
            return;
        }

        let camera_data = camera_data.unwrap();
        let camera_matrix = camera_data.get_matrix();

        let uniforms = UnlitUniforms {
            view_projection: camera_matrix,
        };

        let constants_buffer = self
            .device
            .create_buffer_with_data(bytemuck::bytes_of(&uniforms), wgpu::BufferUsage::COPY_SRC);

        self.encoder.copy_buffer_to_buffer(
            &constants_buffer,
            0,
            &self.constants_buffer,
            0,
            std::mem::size_of::<UnlitUniforms>() as u64,
        );

        {
            let size = std::mem::size_of::<LocalUniform>();
            let temp_buf_data = self.device.create_buffer_mapped(&wgpu::BufferDescriptor {
                size: (transforms.count() * size) as u64,
                usage: wgpu::BufferUsage::COPY_SRC,
                label: None,
            });

            // FIXME: Align and use `LayoutVerified`
            for (transform, slot) in (&mut transforms)
                .join()
                .zip(temp_buf_data.data.chunks_exact_mut(size))
            {
                transform.update();
                slot.copy_from_slice(bytemuck::bytes_of(&LocalUniform {
                    world: transform.matrix,
                }));
            }

            let temp_buf = temp_buf_data.finish();

            let mut i = 0;
            for transform in transforms.join() {
                self.encoder.copy_buffer_to_buffer(
                    &temp_buf,
                    (i * size) as wgpu::BufferAddress,
                    &transform.local_buffer,
                    0,
                    size as wgpu::BufferAddress,
                );
                i += 1;
            }
        }

        // for (material, mesh, transform) in (&materials, &meshes, &mut transforms).join() {
        //     let mesh: &Mesh = mesh;
        //     let transform: &mut Transform = transform;
        //     transform.update();

        //     let asset_mesh = self.asset_manager.get_mesh(mesh.mesh_name.clone());
        //     for sub_mesh in asset_mesh.sub_meshes.iter() {
        //         let local_uniform = UnlitUniform {
        //             world: transform.matrix,
        //             color: unlit_material.color,
        //         };
        //     }
        // }

        let mut render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: self.frame_view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Load,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
            }],
            depth_stencil_attachment: None,
            // depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
            //     attachment: self.depth,
            //     depth_load_op: wgpu::LoadOp::Load,
            //     depth_store_op: wgpu::StoreOp::Store,
            //     stencil_load_op: wgpu::LoadOp::Load,
            //     stencil_store_op: wgpu::StoreOp::Store,
            //     clear_depth: 1.0,
            //     clear_stencil: 0,
            // }),
        });
        render_pass.set_pipeline(&self.pipeline.pipeline);
        render_pass.set_bind_group(0, self.global_bind_group, &[]);

        let asset_materials = self.asset_manager.get_materials();
        for asset_material in asset_materials {
            let joined_data = (&meshes, &materials, &transforms).join();
            match asset_material {
                Material::Unlit(unlit_material) => {
                    render_pass.set_bind_group(
                        2,
                        &unlit_material.bind_group_data.as_ref().unwrap().bind_group,
                        &[],
                    );
                    for (mesh, _, transform) in joined_data
                        .filter(|(_, material, _)| material.index == unlit_material.index)
                    {
                        render_pass.set_bind_group(1, &transform.bind_group, &[]);
                        let mesh: &Mesh = mesh;
                        let asset_mesh = self.asset_manager.get_mesh(mesh.mesh_name.clone());
                        for sub_mesh in asset_mesh.sub_meshes.iter() {
                            // render_pass.set_bind_group(1, &current_bind_group.bind_group, &[]);
                            render_pass.set_index_buffer(&sub_mesh.index_buffer, 0, 0);
                            render_pass.set_vertex_buffer(0, &sub_mesh.vertex_buffer, 0, 0);
                            render_pass.draw_indexed(0..sub_mesh.index_count as u32, 0, 0..1);
                        }
                    }
                }
            }
        }
    }
}
