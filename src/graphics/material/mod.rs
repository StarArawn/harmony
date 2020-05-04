pub(crate) mod shader;
pub use shader::Shader;

pub(crate) mod image;
pub use self::image::Image;

pub(crate) mod skybox;
pub use self::skybox::Skybox;

pub(crate) mod unlit_material;
pub use self::unlit_material::*;

pub(crate) mod pbr_material;
pub use self::pbr_material::*;

pub(crate) mod new_material;
pub use self::new_material::*;

pub enum Material {
    Unlit(UnlitMaterial),
    PBR(PBRMaterial),
}
