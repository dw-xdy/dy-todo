use ratatui::style::Color;
use std::collections::HashSet;
use ratatui::widgets::ListState; // 必须导入这个

pub struct TokyoNight;
impl TokyoNight {
    pub const CYAN: Color = Color::Rgb(125, 207, 255);
    pub const MAGENTA: Color = Color::Rgb(187, 154, 247);
    pub const ORANGE: Color = Color::Rgb(255, 158, 100);
    pub const RED: Color = Color::Rgb(247, 118, 142);
    pub const GRAY: Color = Color::Rgb(86, 95, 137);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    Work,
    Personal,
    Urgent,
    Custom(String),
}

pub struct TodoTask {
    pub title: String,
    pub description: String,
    pub is_completed: bool,
    pub tags: HashSet<Tag>,
}
