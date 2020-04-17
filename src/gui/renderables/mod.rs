use crate::gui::core::{Background, Color, Rectangle};
use nalgebra_glm::Vec2;

mod quad;
pub use quad::Quad;

#[derive(Debug, Clone)]
pub struct Text {
    pub text: String,
    pub size: f32,
    pub bounds: Rectangle,
    pub color: Color,
    pub font: String,
}

#[derive(Debug, Clone)]
pub enum Renderable {
    /// An empty primitive
    None,
    /// A group of primitives
    Group {
        bounds: Rectangle,
        /// The primitives of the group
        renderables: Vec<Renderable>,
    },
    /// A quad primitive
    Quad {
        /// The bounds of the quad
        bounds: Rectangle,
        /// The background of the quad
        background: Background,
        /// The border radius of the quad
        border_radius: u16,
        /// The border width of the quad
        border_width: u16,
        /// The border color of the quad
        border_color: Color,
    },
    Text(Text),
    Clip {
        offset: Vec2,
        bounds: Rectangle,
        content: Box<Renderable>,
    },
}
