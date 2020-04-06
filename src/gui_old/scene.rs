use crate::gui::messages::Message;

use iced_wgpu::Renderer;
use iced_winit::Element;

pub trait Scene {
    fn update(&mut self, message: Message);
    fn view(&mut self) -> Element<'_, Message, Renderer>;
}