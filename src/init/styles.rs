use ansi_term::{Color, Style};

pub fn path() -> Style {
    Style::new().fg(Color::White).italic().fg(Color::Cyan)
}
pub fn helpers() -> Style {
    Style::new().fg(Color::White).bold()
}
pub fn failure() -> Style {
    Style::new().fg(Color::Red).bold()
}
