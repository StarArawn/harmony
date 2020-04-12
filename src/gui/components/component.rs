use crate::gui::renderables::Renderable;
use std::any::Any;

pub trait Component {
    fn update(&mut self, delta_time: f32);
    fn draw(&self, parent_bounds: crate::gui::core::Rectangle) -> Renderable;
    fn as_any(&mut self) -> &mut dyn Any;
}