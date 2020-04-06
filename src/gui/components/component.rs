use crate::gui::renderables::Renderable;

trait Component {
    fn draw() -> Renderable;
}