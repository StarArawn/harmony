use rendy::hal::format::Format;
use rendy::mesh::AsAttribute;

/// Type for position attribute of vertex.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position2D(pub [f32; 2]);
impl<T> From<T> for Position2D
where
    T: Into<[f32; 2]>,
{
    fn from(from: T) -> Self {
        Position2D(from.into())
    }
}
impl AsAttribute for Position2D {
    const NAME: &'static str = "position2d";
    const FORMAT: Format = Format::Rg32Sfloat;
}

/// Type for position attribute of vertex.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Float(pub f32);
impl<T> From<T> for Float
where
    T: Into<f32>,
{
    fn from(from: T) -> Self {
        Float(from.into())
    }
}
impl AsAttribute for Float {
    const NAME: &'static str = "float";
    const FORMAT: Format = Format::R32Sfloat;
}
