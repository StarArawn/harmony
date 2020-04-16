use specs::{ Component, DenseVecStorage };

#[derive(Default)]
pub struct Material {
    // We might have more than one material per mesh.
    pub index: i32,
}

impl Material {
    /// Mesh name is used to get the correct materials for the mesh.
    pub fn new(material_index: i32) -> Self {
        Self {
           index: material_index,
        }
    }
}

impl Component for Material {
    type Storage = DenseVecStorage<Self>;
}