use crate::gui::messages::Message;
use crate::gui::messages::ThemeMessage;
use crate::gui::Scene;
use crate::gui::theme::style;

use iced_wgpu::Renderer;
use iced_winit::{
    button, scrollable, slider, text_input, Align, Button, Checkbox, Column,
    Container, Element, Length, ProgressBar, Radio, Row, Scrollable,
    Slider, Space, Text, TextInput,
};

#[derive(Default)]
pub struct ThemeScene {
    theme: style::Theme,
    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
    button: button::State,
    slider: slider::State,
    slider_value: f32,
    toggle_value: bool,
}

impl ThemeScene {
    pub fn new() -> Self {
        ThemeScene::default()
    }

    fn convertInto(element: Element<'_, ThemeMessage, Renderer>) -> Element<'_, Message, Renderer> {
        element.map(Message::ThemeMessage)
    }
}

impl Scene for ThemeScene {
    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeMessage(theme_message) => {
                match theme_message {
                    ThemeMessage::ThemeChanged(theme) => self.theme = theme,
                    ThemeMessage::InputChanged(value) => self.input_value = value,
                    ThemeMessage::ButtonPressed => (),
                    ThemeMessage::SliderChanged(value) => self.slider_value = value,
                    ThemeMessage::CheckboxToggled(value) => self.toggle_value = value,
                }
            }
        }
    }

    fn view(&mut self) -> Element<'_, Message, Renderer> {
        let choose_theme = style::Theme::ALL.iter().fold(
            Column::new().spacing(10).push(Text::new("Choose a theme:")),
            |column, theme| {
                column.push(
                    Radio::new(
                        *theme,
                        &format!("{:?}", theme),
                        Some(self.theme),
                        ThemeMessage::ThemeChanged,
                    )
                    .style(self.theme),
                )
            },
        );

        let text_input = TextInput::<ThemeMessage, Renderer>::new(
            &mut self.input,
            "Type something...",
            &self.input_value,
            ThemeMessage::InputChanged,
        )
        .padding(10)
        .size(20)
        .style(self.theme);

        let button = Button::<ThemeMessage, Renderer>::new(&mut self.button, Text::new("Submit"))
            .padding(10)
            .on_press(ThemeMessage::ButtonPressed)
            .style(self.theme);

        let slider = Slider::<ThemeMessage, Renderer>::new(
            &mut self.slider,
            0.0..=100.0,
            self.slider_value,
            ThemeMessage::SliderChanged,
        )
        .style(self.theme);

        let progress_bar =
            ProgressBar::<Renderer>::new(0.0..=100.0, self.slider_value).style(self.theme);

        let scrollable = Scrollable::<ThemeMessage, Renderer>::new(&mut self.scroll)
            .width(Length::Fill)
            .height(Length::Units(100))
            .style(self.theme)
            .push(Text::new("Scroll me!"))
            .push(Space::with_height(Length::Units(800)))
            .push(Text::new("You did it!"));

        let checkbox = Checkbox::<ThemeMessage, Renderer>::new(
            self.toggle_value,
            "Toggle me!",
            ThemeMessage::CheckboxToggled,
        )
        .width(Length::Fill)
        .style(self.theme);

        let content = Column::<ThemeMessage, Renderer>::new()
            .spacing(20)
            .padding(20)
            .max_width(600)
            .push(choose_theme)
            .push(Row::new().spacing(10).push(text_input).push(button))
            .push(slider)
            .push(progress_bar)
            .push(
                Row::new()
                    .spacing(10)
                    .align_items(Align::Center)
                    .push(scrollable)
                    .push(checkbox),
            );

        Self::convertInto(
            Container::new(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(self.theme)
                .into()
        )
            
    }
}
