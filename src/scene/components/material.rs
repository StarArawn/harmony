/// A handle to a material.
#[derive(Default)]
pub struct Material {
    // We might have more than one material per mesh.
    pub index: u32,
}

impl Material {
    pub fn new(material_index: u32) -> Self {
        Self {
            index: material_index,
        }
    }
}
