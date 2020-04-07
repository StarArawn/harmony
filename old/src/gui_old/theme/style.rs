use crate::gui::theme::dark;
use crate::gui::theme::light;

use iced_wgpu::{
    button, checkbox, container, progress_bar, radio, scrollable, slider,
    text_input,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub const ALL: [Theme; 2] = [Theme::Light, Theme::Dark];
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::Light
    }
}

impl From<Theme> for Box<dyn container::StyleSheet> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => Default::default(),
            Theme::Dark => dark::Container.into(),
        }
    }
}

impl From<Theme> for Box<dyn radio::StyleSheet> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => Default::default(),
            Theme::Dark => dark::Radio.into(),
        }
    }
}

impl From<Theme> for Box<dyn text_input::StyleSheet> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => Default::default(),
            Theme::Dark => dark::TextInput.into(),
        }
    }
}

impl From<Theme> for Box<dyn button::StyleSheet> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => light::Button.into(),
            Theme::Dark => dark::Button.into(),
        }
    }
}

impl From<Theme> for Box<dyn scrollable::StyleSheet> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => Default::default(),
            Theme::Dark => dark::Scrollable.into(),
        }
    }
}

impl From<Theme> for Box<dyn slider::StyleSheet> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => Default::default(),
            Theme::Dark => dark::Slider.into(),
        }
    }
}

impl From<Theme> for Box<dyn progress_bar::StyleSheet> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => Default::default(),
            Theme::Dark => dark::ProgressBar.into(),
        }
    }
}

impl From<Theme> for Box<dyn checkbox::StyleSheet> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => Default::default(),
            Theme::Dark => dark::Checkbox.into(),
        }
    }
}