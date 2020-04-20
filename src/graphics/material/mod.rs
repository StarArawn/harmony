mod shader;
pub use shader::Shader;

mod image;
pub use self::image::Image;

mod skybox;
pub use self::skybox::Skybox;

mod unlit_material;
pub use self::unlit_material::*;

mod pbr_material;
pub use self::pbr_material::*;

#[derive(Debug)]
pub enum Material {
    Unlit(UnlitMaterial),
    PBR(PBRMaterial),
}
