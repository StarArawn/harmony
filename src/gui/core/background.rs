use crate::gui::core::Color;

/// The background of some element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Background {
    /// A solid color
    Color(Color),
    // TODO: Add gradient and image variants
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Background::Color(color)
    }
}

impl Default for Background {
    fn default() -> Self {
        Background::from(Color::from_rgb(0.0, 0.0, 0.0))
    }
}
