use ansi_term::{Color, Style};

pub fn path() -> Style {
    Style::new().fg(Color::White).italic().fg(Color::Cyan)
}
pub fn failure() -> Style {
    Style::new().fg(Color::Red).bold()
}
pub fn number() -> Style {
    Style::new().fg(Color::Yellow).bold()
}
pub fn error() -> Style {
    Style::new().fg(Color::Red).bold()
}
