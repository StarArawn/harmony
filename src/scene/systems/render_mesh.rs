use specs::{ReadStorage, System};
use crate::AssetManager;
use crate::scene::components::Mesh;

pub struct RenderMesh<'a> {
    pub(crate) asset_manager: &'a AssetManager,
    pub(crate) render_pass: wgpu::RenderPass<'a>,
}

impl<'a> System<'a> for RenderMesh<'a> {
    type SystemData = ReadStorage<'a, Mesh>;

    fn run(&mut self, mesh: Self::SystemData) {
        use specs::Join;

        for mesh in mesh.join() {  
            let mesh: &Mesh = mesh;
            let asset_mesh = self.asset_manager.get_mesh(mesh.mesh_name.clone());
            for sub_mesh in asset_mesh.sub_meshes.iter() {
                self.render_pass.set_index_buffer(&sub_mesh.index_buffer, 0, 0);
                self.render_pass.set_vertex_buffer(0, &sub_mesh.vertex_buffer, 0, 0);
                self.render_pass.draw_indexed(0..sub_mesh.index_count as u32, 0, 0..1);
            }
        }
    }
}