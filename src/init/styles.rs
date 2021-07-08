use ansi_term::{Color, Style};

pub fn path() -> Style {
    Style::new().fg(Color::White).italic().fg(Color::Cyan)
}
pub fn helpers() -> Style {
    Style::new().fg(Color::White).bold()
}
pub fn success() -> Style {
    Style::new().fg(Color::Green).bold()
}
pub fn already_done() -> Style {
    Style::new().fg(Color::Yellow)
}
pub fn failure() -> Style {
    Style::new().fg(Color::Red).bold()
}
