mod shader;
pub use shader::Shader;

mod image;
pub use self::image::Image;

mod hdr;
pub use self::hdr::HDRImage;

mod materials;
pub use self::materials::*;

#[derive(Debug)]
pub enum Material {
    Unlit(UnlitMaterial),
}
