use crate::AssetManager;
use crate::{
    graphics::{material::Material, pipelines::{DirectionalLight, GlobalUniforms, PointLight, LightingUniform, MAX_LIGHTS}, Pipeline},
    scene::components::{transform::LocalUniform, CameraData, Mesh, Transform, DirectionalLightData, PointLightData},
};
use specs::{ReadStorage, System, WriteStorage};
use std::convert::TryInto;

pub struct RenderPBR<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) asset_manager: &'a AssetManager,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
    pub(crate) frame_view: &'a wgpu::TextureView,
    pub(crate) pipeline: &'a Pipeline,
    pub(crate) constants_buffer: &'a wgpu::Buffer,
    pub(crate) lighting_buffer: &'a wgpu::Buffer,
    pub(crate) global_bind_group: &'a wgpu::BindGroup,
    pub(crate) depth: &'a wgpu::TextureView,
}

impl<'a> System<'a> for RenderPBR<'a> {
    type SystemData = (
        ReadStorage<'a, CameraData>,
        ReadStorage<'a, Mesh>,
        ReadStorage<'a, crate::scene::components::Material>,
        ReadStorage<'a, DirectionalLightData>,
        ReadStorage<'a, PointLightData>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (camera_data, meshes, materials, directional_lights, point_lights, mut transforms): Self::SystemData) {
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

        let uniforms = GlobalUniforms {
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
            std::mem::size_of::<GlobalUniforms>() as u64,
        );

        // Get lighting data.
        let mut directional_light_data_vec: Vec<DirectionalLight> = directional_lights.join().map(|data| DirectionalLight {
            direction: data.direction,
            color: data.color,
        }).collect();

        let mut point_light_data_vec: Vec<PointLight> = (&point_lights, &transforms).join().map(|(data, transform)| PointLight {
            attenuation: data.attenuation,
            color: data.color,
            position: transform.position,
        }).collect();

        let total_dir_lights = directional_light_data_vec.len() as u32;
        let total_point_lights = point_light_data_vec.len() as u32; 

        // Fill in missing data if we don't have it.
        point_light_data_vec.resize_with(MAX_LIGHTS / 2, || PointLight::default());
        directional_light_data_vec.resize_with(MAX_LIGHTS / 2, || DirectionalLight::default());

        let light_uniform = LightingUniform {
            TOTAL_DIRECTIONAL_LIGHTS: total_dir_lights,
            TOTAL_POINT_LIGHTS: total_point_lights,
            directional_lights: directional_light_data_vec.as_slice().try_into().unwrap(),
            point_lights: point_light_data_vec.as_slice().try_into().unwrap(),
        };

        let lighting_buffer = self
            .device
            .create_buffer_with_data(bytemuck::bytes_of(&light_uniform), wgpu::BufferUsage::COPY_SRC);

        dbg!(std::mem::size_of::<LightingUniform>());
        self.encoder.copy_buffer_to_buffer(
            &lighting_buffer,
            0,
            &self.lighting_buffer,
            0,
            std::mem::size_of::<LightingUniform>() as u64,
        );

        {
            let size = std::mem::size_of::<LocalUniform>();
            let mut temp_buf_data = self.device.create_buffer_mapped(&wgpu::BufferDescriptor {
                size: (transforms.count() * size) as u64,
                usage: wgpu::BufferUsage::COPY_SRC,
                label: None,
            });

            // FIXME: Align and use `LayoutVerified`
            for (transform, slot) in (&mut transforms)
                .join()
                .zip(temp_buf_data.data().chunks_exact_mut(size))
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
        render_pass.set_bind_group(1, self.global_bind_group, &[]);

        let asset_materials = self.asset_manager.get_materials();
        /* 
            TODO: It's not very efficient to loop through each entity that has a material. Fix that.
            Look into using: https://docs.rs/specs/0.16.1/specs/struct.FlaggedStorage.html
        */
        for asset_material in asset_materials {
            let joined_data = (&meshes, &materials, &transforms).join();
            match asset_material {
                Material::PBR(pbr_material) => {
                    render_pass.set_bind_group(
                        2,
                        &pbr_material.bind_group_data.as_ref().unwrap().bind_group,
                        &[],
                    );
                    for (mesh, _, transform) in joined_data
                        .filter(|(_, material, _)| material.index == pbr_material.index)
                    {
                        render_pass.set_bind_group(0, &transform.bind_group, &[]);
                        let mesh: &Mesh = mesh;
                        let asset_mesh = self.asset_manager.get_mesh(mesh.mesh_name.clone());
                        for sub_mesh in asset_mesh.sub_meshes.iter() {
                            // render_pass.set_bind_group(1, &current_bind_group.bind_group, &[]);
                            render_pass.set_index_buffer(&sub_mesh.index_buffer, 0, 0);
                            render_pass.set_vertex_buffer(0, &sub_mesh.vertex_buffer, 0, 0);
                            render_pass.draw_indexed(0..sub_mesh.index_count as u32, 0, 0..1);
                        }
                    }
                },
                _ => (),
            }
        }
    }
}
