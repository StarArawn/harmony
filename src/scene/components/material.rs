#[derive(Default)]
pub struct Material {
    // We might have more than one material per mesh.
    pub index: u32,
}

impl Material {
    /// Mesh name is used to get the correct materials for the mesh.
    pub fn new(material_index: u32) -> Self {
        Self {
            index: material_index,
        }
    }
}