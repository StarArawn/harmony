
mod theme_message;
pub use theme_message::ThemeMessage;

#[derive(Debug, Clone)]
pub enum Message {
    ThemeMessage(ThemeMessage),
}