use crate::gui::core::{ Background, Color, Rectangle };

mod quad;
pub use quad::Quad;

pub enum Renderable {
    /// An empty primitive
    None,
    /// A group of primitives
    Group {
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
    }
}