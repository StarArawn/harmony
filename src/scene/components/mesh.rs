use specs::{ Component, DenseVecStorage };

#[derive(Default)]
pub struct Mesh {
    pub mesh_name: String,
}

impl Component for Mesh {
    type Storage = DenseVecStorage<Self>;
}