use crate::gui::components::Component;

/// A trait that allows you to create a UI scene.
pub trait Scene {
    /// Let the internals get a list of all of the components in your scene.
    /// Note: This gets wrapped by a single component with bounds that are the size of the frame buffer.
    fn get_components(&self) -> &Vec<Box<dyn Component>>;
}
