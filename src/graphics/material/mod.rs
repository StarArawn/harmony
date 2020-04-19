mod shader;
pub use shader::Shader;

mod image;
pub use self::image::Image;

mod hdr;
pub use self::hdr::HDRImage;

mod unlit_material;
pub use self::unlit_material::*;

mod pbr_material;
pub use self::pbr_material::*;

#[derive(Debug)]
pub enum Material {
    Unlit(UnlitMaterial),
    PBR(PBRMaterial),
}
