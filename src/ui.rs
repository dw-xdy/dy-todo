use crate::app::App;
use crate::models::{ActiveWindow, TokyoNight, WindowData, WindowType};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Color,
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation},
};
use std::net::IpAddr;

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

    // 如果有活动窗口，渲染在顶层
    if let Some(window) = &app.active_window {
        draw_window(app, window, frame);
    }
}

fn draw_search(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 🔍 Search ").centered())
        .border_style(Style::default().fg(TokyoNight::MAGENTA))
        .border_set(border::THICK);
    frame.render_widget(Paragraph::new("输入关键词搜索...").block(block), area);
}

fn draw_todo_list(app: &App, area: Rect, frame: &mut Frame) {
    // 1. 原有的渲染列表逻辑 ( 保持不变 )
    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .map(|task| {
            let status = if task.is_completed { " ✅ " } else { " ❌ " };
            ListItem::new(Line::from(vec![status.into(), task.title.clone().into()]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(Line::from(" 📝 Todo List ").centered())
                .border_set(border::DOUBLE),
        )
        .highlight_style(
            Style::default()
                .bg(TokyoNight::GRAY)
                .fg(Color::White)
                .bold(),
        )
        .highlight_symbol(">> ");

    // 注意：这里需要传入可变引用的拷贝
    frame.render_stateful_widget(list, area, &mut app.list_state.clone());

    // 2. 渲染滚动条
    // 我们创建一个垂直滚动条，放在区域的右侧
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .track_symbol(Some("░"))
        .thumb_symbol("█");

    // 渲染滚动条需要它的状态
    // 我们通常在 block 内部渲染它，所以可以用 area
    frame.render_stateful_widget(
        scrollbar,
        area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 0,
        }), // 稍微内缩，避免压住边框
        &mut app.scroll_state.clone(),
    );
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

/// 渲染窗口（覆盖在现有界面上）
fn draw_window(_app: &App, window: &ActiveWindow, frame: &mut Frame) {
    if !window.is_visible {
        return;
    }

    // 创建窗口区域
    let area = Rect::new(
        window.layout.x,
        window.layout.y,
        window.layout.width,
        window.layout.height,
    );

    // 根据窗口类型渲染不同内容
    match (&window.window_type, &window.data) {
        (
            WindowType::CreateTask,
            WindowData::CreateTask {
                title,
                description,
                current_field,
            },
        ) => {
            draw_create_task_window(_app, area, title, description, *current_field, frame);
        }
        (WindowType::PomodoroSettings, _) => {
            draw_pomodoro_settings_window(_app, area, frame);
        }
        _ => {}
    }
}

/// 创建任务窗口
fn draw_create_task_window(
    _app: &App,
    area: Rect,
    title: &str,
    description: &str,
    current_field: usize,
    frame: &mut Frame,
) {
    // 先清除区域（创建半透明遮罩效果）
    let clear_block = Block::default();
    frame.render_widget(Clear, area);
    frame.render_widget(clear_block, area);

    let block = Block::bordered()
        .title(Line::from(" 🆕 创建一个新的todo ").centered())
        .border_style(Style::default().fg(TokyoNight::CYAN))
        .border_set(border::DOUBLE)
        .bg(Color::Rgb(20, 20, 40)); // 深色背景

    let inner_area = block.inner(area);
    frame.render_widget(block.clone(), area);

    // 分割窗口内部区域
    let layout = Layout::horizontal([
        Constraint::Percentage(70), // 分隔
        Constraint::Percentage(30),
    ]);
    let chunks = layout.split(inner_area);

    let left_layout = Layout::vertical([Constraint::Percentage(30), Constraint::Percentage(70)]);
    let left_areas = left_layout.split(chunks[0]);

    let right_layout = Layout::vertical([Constraint::Percentage(40), Constraint::Percentage(60)]);
    let right_areas = right_layout.split(chunks[1]);

    draw_todo(_app, left_areas[0], frame);
    draw_desc(_app, left_areas[1], frame);
    draw_must_tag(_app, right_areas[0], frame);
    draw_diy_tag(_app, right_areas[1], frame);
}

fn draw_todo(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 新的todo ").centered())
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(TokyoNight::RED));

    frame.render_widget(block, area);
}

fn draw_desc(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" todo的详细信息 ").centered())
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(TokyoNight::RED));

    frame.render_widget(block, area);
}

fn draw_must_tag(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 必选的标签 ").centered())
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(TokyoNight::ORANGE));

    frame.render_widget(block, area);
}

fn draw_diy_tag(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 自定义标签 ").centered())
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(TokyoNight::ORANGE));

    frame.render_widget(block, area);
}

fn draw_pomodoro_settings_window(_app: &App, area: Rect, frame: &mut Frame) {
    frame.render_widget(Clear, area);

    let block = Block::bordered()
        .title(Line::from(" 🍅 Pomodoro设置 ").centered())
        .border_style(Style::default().fg(TokyoNight::GRAY))
        .border_set(border::THICK)
        .bg(Color::Rgb(20, 20, 40)); // 深色背景

    let inner_area = block.inner(area);

    frame.render_widget(block.clone(), area);

    let main_layout = Layout::vertical([
        Constraint::Percentage(15),
        Constraint::Percentage(25),
        Constraint::Percentage(60),
    ]);

    let rows = main_layout.split(inner_area);

    // 上面切割出界面是否在番茄钟进行中和结束时播放音乐.
    let up_layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let up_areas = up_layout.split(rows[0]);

    // 中间切割出常用时间和自定义的时间
    let middle_layout =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]);
    let middle_areas = middle_layout.split(rows[1]);

    // 下面就不切割了, 因为是音乐播放列表

    draw_up_left(_app, up_areas[0], frame);
    draw_up_right(_app, up_areas[1], frame);
    draw_middle_left(_app, middle_areas[0], frame);
    draw_middle_right(_app, middle_areas[1], frame);
    draw_down(_app, rows[2], frame);
}

fn draw_up_left(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 是否在番茄钟运行时播放音乐? ").centered())
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(TokyoNight::RED));

    frame.render_widget(block, area);
}

fn draw_up_right(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 是否在番茄钟结束时播放音乐? ").centered())
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(TokyoNight::RED));

    frame.render_widget(block, area);
}

fn draw_middle_left(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 常用番茄钟时间 ").centered())
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(TokyoNight::ORANGE));

    frame.render_widget(block, area);
}

fn draw_middle_right(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 自定义番茄钟时间 ").centered())
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(TokyoNight::ORANGE));

    frame.render_widget(block, area);
}

// 在 ui.rs 中
fn draw_down(app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 🎵 音乐播放列表 ").centered())
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(TokyoNight::CYAN));

    // 将音频文件转换为 ListItem
    let items: Vec<ListItem> = app
        .music_files
        .iter()
        .map(|file| ListItem::new(Line::from(vec![" 🎶 ".into(), file.name.clone().into()])))
        .collect();

    // 创建列表组件
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(TokyoNight::GRAY)
                .fg(Color::White)
                .bold(),
        )
        .highlight_symbol("▶ ");

    // 使用 music_list_state 进行有状态渲染
    frame.render_stateful_widget(list, area, &mut app.music_list_state.clone());
}
