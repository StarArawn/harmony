use crate::assets::{mesh::Gltf, AssetHandle};
use std::sync::Arc;

/// A reference to the mesh.
pub struct Mesh {
    pub mesh_handle: Arc<AssetHandle<Gltf>>,
}

impl Mesh {
    pub fn new(mesh_handle: Arc<AssetHandle<Gltf>>) -> Self {
        Self { mesh_handle }
    }
}
