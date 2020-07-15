mod asset_manager;
pub use asset_manager::AssetManager;

pub mod image;
pub use self::image::Image;

pub mod material;
mod material_manager;

pub mod texture;
mod texture_manager;

mod file_manager;
pub use file_manager::{AssetCache, AssetError, AssetHandle, FileManager};

pub mod shader;
mod shader_manager;

pub mod mesh;
mod mesh_manager;
