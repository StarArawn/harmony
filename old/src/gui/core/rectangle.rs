use ultraviolet::vec::Vec2;

/// A rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rectangle<T = f32> {
    /// X coordinate of the top-left corner.
    pub x: T,

    /// Y coordinate of the top-left corner.
    pub y: T,

    /// Width of the rectangle.
    pub width: T,

    /// Height of the rectangle.
    pub height: T,
}

impl Rectangle<f32> {
    /// Returns true if the given [`Point`] is contained in the [`Rectangle`].
    ///
    /// [`Point`]: struct.Point.html
    /// [`Rectangle`]: struct.Rectangle.html
    pub fn contains(&self, point: Vec2) -> bool {
        self.x <= point.x
            && point.x <= self.x + self.width
            && self.y <= point.y
            && point.y <= self.y + self.height
    }

    /// Computes the intersection with the given [`Rectangle`].
    ///
    /// [`Rectangle`]: struct.Rectangle.html
    pub fn intersection(
        &self,
        other: &Rectangle<f32>,
    ) -> Option<Rectangle<f32>> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);

        let lower_right_x = (self.x + self.width).min(other.x + other.width);
        let lower_right_y = (self.y + self.height).min(other.y + other.height);

        let width = lower_right_x - x;
        let height = lower_right_y - y;

        if width > 0.0 && height > 0.0 {
            Some(Rectangle {
                x,
                y,
                width,
                height,
            })
        } else {
            None
        }
    }
}

impl std::ops::Mul<f32> for Rectangle<u32> {
    type Output = Self;

    fn mul(self, scale: f32) -> Self {
        Self {
            x: (self.x as f32 * scale).round() as u32,
            y: (self.y as f32 * scale).round() as u32,
            width: (self.width as f32 * scale).round() as u32,
            height: (self.height as f32 * scale).round() as u32,
        }
    }
}

impl From<Rectangle<u32>> for Rectangle<f32> {
    fn from(rectangle: Rectangle<u32>) -> Rectangle<f32> {
        Rectangle {
            x: rectangle.x as f32,
            y: rectangle.y as f32,
            width: rectangle.width as f32,
            height: rectangle.height as f32,
        }
    }
}

impl From<Rectangle<f32>> for Rectangle<u32> {
    fn from(rectangle: Rectangle<f32>) -> Rectangle<u32> {
        Rectangle {
            x: rectangle.x as u32,
            y: rectangle.y as u32,
            width: rectangle.width.ceil() as u32,
            height: rectangle.height.ceil() as u32,
        }
    }
}
