use crate::gui::renderables::Renderable;

pub trait Component {
    fn node(&self, stretch: &mut stretch::Stretch, position_type: stretch::style::PositionType) -> stretch::node::Node;
    fn draw(&self, stretch: &mut stretch::Stretch, node: stretch::node::Node) -> Renderable;
}