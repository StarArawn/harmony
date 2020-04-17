use specs::{Component, DenseVecStorage};

#[derive(Default)]
pub struct SkyboxData {
    // We might have more than one material per mesh.
    pub name: String,
}

impl SkyboxData {
    /// Mesh name is used to get the correct materials for the mesh.
    pub fn new<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        Self { name: name.into() }
    }
}

impl Component for SkyboxData {
    type Storage = DenseVecStorage<Self>;
}
