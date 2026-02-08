use crate::app::App;
use ratatui::widgets::{List, ListItem};
use crate::models::TokyoNight;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    style::Color,
    widgets::{Block, Paragraph},
};

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let main_layout = Layout::horizontal([
        Constraint::Percentage(15),
        Constraint::Percentage(55),
        Constraint::Percentage(30),
    ]);
    let cols = main_layout.split(area);

    let right_layout = Layout::vertical([Constraint::Length(10), Constraint::Min(0)]);
    let right_areas = right_layout.split(cols[2]);

    draw_search(app, cols[0], frame);
    draw_todo_list(app, cols[1], frame);
    draw_pomodoro(app, right_areas[0], frame);
    draw_details(app, right_areas[1], frame);
}

fn draw_search(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 🔍 Search ").centered())
        .border_style(Style::default().fg(TokyoNight::MAGENTA))
        .border_set(border::THICK);
    frame.render_widget(Paragraph::new("输入关键词搜索...").block(block), area);
}

fn draw_todo_list(app: &App, area: Rect, frame: &mut Frame) {
    // 1. 将任务转换为 ListItem
    let items: Vec<ListItem> = app.tasks.iter().map(|task| {
        let status = if task.is_completed { " ✅ " } else { " ❌ " };
        ListItem::new(Line::from(vec![
            status.into(),
            task.title.clone().into(),
        ]))
    }).collect();

    // 2. 创建 List 组件并设置样式
    let list = List::new(items)
        .block(Block::bordered()
            .title(Line::from(" 📝 Todo List ").centered())
            .border_style(Style::default().fg(TokyoNight::CYAN))
            .border_set(border::DOUBLE))
        // 设置选中行的高亮样式
        .highlight_style(Style::default().bg(TokyoNight::GRAY).fg(Color::White).bold())
        .highlight_symbol(">> ");

    // 3. 使用 state 进行渲染（关键：必须用 render_stateful_widget）
    frame.render_stateful_widget(list, area, &mut app.list_state.clone());
}

fn draw_pomodoro(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 🍅 Pomodoro ").centered())
        .border_style(Style::default().fg(TokyoNight::ORANGE))
        .border_set(border::ROUNDED);

    let paragraph = Paragraph::new("番茄钟")
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn draw_details(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" ℹ️ Info ").centered())
        .border_style(Style::default().fg(TokyoNight::GRAY))
        .border_set(border::THICK);

    let paragraph = Paragraph::new("这里是任务的详细描述...").block(block);
    frame.render_widget(paragraph, area);
}




