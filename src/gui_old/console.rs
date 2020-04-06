use crate::gui::messages::Message;

use crate::gui::Scene;
use iced_wgpu::Renderer;
use iced_winit::{
    Align, Color, Column, Element, Length, Row, Text,
};

pub struct Console;

impl Console {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scene for Console {
    fn update(&mut self, _message: Message) {

    }

    fn view(&mut self) -> Element<'_, Message, Renderer> {
        // Row::new()
        //     .width(Length::Fill)
        //     .height(Length::Fill)
        //     .align_items(Align::Start)
        //     .push(
        //         Column::new()
        //             .width(Length::Fill)
        //             .align_items(Align::Start)
        //             .push(
        //                 Column::new()
        //                     .padding(10)
        //                     .spacing(10)
        //                     .push(
        //                         Text::new("Console")
        //                             .color(Color::WHITE),
        //                     ),
        //             ),
        //     )
        Text::new("Hello World!")
            .color(Color::WHITE)    
            .into()
    }
}
