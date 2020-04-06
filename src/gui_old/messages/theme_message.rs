use crate::gui::theme::style;

#[derive(Debug, Clone)]
pub enum ThemeMessage {
    ThemeChanged(style::Theme),
    InputChanged(String),
    ButtonPressed,
    SliderChanged(f32),
    CheckboxToggled(bool),
}
