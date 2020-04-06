/// Is something that can  be drawn to the screen.
pub trait Drawable {
    /// Draws `self` to the screen (or a canvas, if one is enabled), using the specified parameters.
    ///
    /// Any type that implements `Into<DrawParams>` can be passed into this method. For example, since the majority
    /// of the time, you only care about changing the position, a `Vec2` can be passed to set the position and leave
    /// everything else as the defaults.
    fn draw(&self);

    fn id(&self) -> usize;
}