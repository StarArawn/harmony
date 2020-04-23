use crate::AssetManager;
use crate::{
    graphics::{
        material::{Material, Skybox},
        Pipeline,
    },
    scene::components::{Mesh, Transform},
};
use specs::{Read, ReadStorage, System};

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
        ReadStorage<'a, Mesh>,
        ReadStorage<'a, crate::scene::components::Material>,
        ReadStorage<'a, Transform>,
        Option<Read<'a, Skybox>>,
    );

    fn run(&mut self, (meshes, materials, transforms, skybox): Self::SystemData) {
        use specs::Join;

        if skybox.is_none() {
            return;
        }
        let skybox = skybox.unwrap();

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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: self.depth,
                depth_load_op: wgpu::LoadOp::Load,
                depth_store_op: wgpu::StoreOp::Store,
                stencil_load_op: wgpu::LoadOp::Load,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                clear_stencil: 0,
            }),
        });
        render_pass.set_pipeline(&self.pipeline.pipeline);
        render_pass.set_bind_group(1, self.global_bind_group, &[]);
        render_pass.set_bind_group(3, skybox.pbr_bind_group.as_ref().unwrap(), &[]);

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
                    for (mesh, _, transform) in
                        joined_data.filter(|(_, material, _)| material.index == pbr_material.index)
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
                }
                _ => (),
            }
        }
    }
}
