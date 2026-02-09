use crate::app::App;
use crate::models::{ActiveWindow, TokyoNight, WindowData, WindowType};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Color,
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Clear, List, ListItem, Paragraph},
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
    // 1. 将任务转换为 ListItem
    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .map(|task| {
            let status = if task.is_completed { " ✅ " } else { " ❌ " };
            ListItem::new(Line::from(vec![status.into(), task.title.clone().into()]))
        })
        .collect();

    // 2. 创建 List 组件并设置样式
    let list = List::new(items)
        .block(
            Block::bordered()
                .title(Line::from(" 📝 Todo List ").centered())
                .border_style(Style::default().fg(TokyoNight::CYAN))
                .border_set(border::DOUBLE),
        )
        // 设置选中行的高亮样式
        .highlight_style(
            Style::default()
                .bg(TokyoNight::GRAY)
                .fg(Color::White)
                .bold(),
        )
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
        _ => {
            draw_default_window(_app, area, &window.window_type, frame);
        }
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
        .title(Line::from(" 🆕 Create New Task ").centered())
        .border_style(Style::default().fg(TokyoNight::CYAN))
        .border_set(border::DOUBLE)
        .bg(Color::Rgb(20, 20, 40)); // 深色背景

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // 分割窗口内部区域
    let layout = Layout::horizontal([
        Constraint::Percentage(70), // 分隔
        Constraint::Percentage(30),
    ]);
    let chunks = layout.split(inner_area);

    let left_layout = Layout::vertical([Constraint::Length(20), Constraint::Length(80)]);
    let left_areas = left_layout.split(chunks[0]);

    let right_layout = Layout::vertical([Constraint::Length(20), Constraint::Length(80)]);
    let right_areas = right_layout.split(chunks[1]);

    // 标题输入框
    let title_style = if current_field == 0 {
        Style::default().fg(TokyoNight::CYAN).bold()
    } else {
        Style::default().fg(Color::Gray)
    };

    let title_block = Block::default().title(" Title ").title_style(title_style);

    let title_text = if title.is_empty() {
        "Enter task title...".to_string()
    } else {
        title.clone().parse().unwrap()
    };

    frame.render_widget(
        Paragraph::new(title_text)
            .block(title_block)
            .style(title_style),
        chunks[0],
    );

    // 分隔线
    frame.render_widget(
        Paragraph::new("─".repeat(chunks[1].width as usize)),
        chunks[1],
    );

    // 描述输入框
    let desc_style = if current_field == 1 {
        Style::default().fg(TokyoNight::CYAN).bold()
    } else {
        Style::default().fg(Color::Gray)
    };

    let desc_block = Block::default()
        .title(" Description ")
        .title_style(desc_style);

    let desc_text = if description.is_empty() {
        "Enter task description...".to_string()
    } else {
        description.clone().parse().unwrap()
    };

    frame.render_widget(
        Paragraph::new(desc_text)
            .block(desc_block)
            .style(desc_style),
        chunks[2],
    );

    // 底部提示
    let help_text = "Press Tab to switch field • Enter to save • Esc to cancel";
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(help_paragraph, chunks[3]);
}

/// 默认窗口（用于测试）
fn draw_default_window(_app: &App, area: Rect, window_type: &WindowType, frame: &mut Frame) {
    let title = format!(" {:?} Window ", window_type);
    let block = Block::bordered()
        .title(Line::from(title).centered())
        .border_style(Style::default().fg(TokyoNight::ORANGE))
        .border_set(border::ROUNDED)
        .bg(Color::Rgb(20, 20, 40));

    frame.render_widget(Clear, area);

    // 先渲染区块
    frame.render_widget(block.clone(), area); // 使用 clone

    // 然后获取内部区域（从原始 block）
    let inner_area = block.inner(area);

    let content = format!(
        "This is a {:?} window.\n\nPress 'Esc' to close.",
        window_type
    );
    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, inner_area);
}

fn draw_pomodoro_settings_window(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 🍅 Pomodoro设置 ").centered())
        .border_style(Style::default().fg(TokyoNight::CYAN))
        .border_set(border::DOUBLE)
        .bg(Color::Rgb(20, 20, 40)); // 深色背景

    let main_layout = Layout::vertical([
        Constraint::Percentage(15),
        Constraint::Percentage(25),
        Constraint::Percentage(60),
    ]);

    let rows = main_layout.split(area);

    // 上面切割出界面什么时候播放音乐.
    let up_layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let up_areas = up_layout.split(rows[0]);

    // 中间切割出常用时间和自定义的时间
    let middle_layout =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
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
        .border_style(Style::default().fg(TokyoNight::MAGENTA));

    frame.render_widget(block, area);
}

fn draw_up_right(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 是否在番茄钟结束时播放音乐? ").centered())
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(TokyoNight::MAGENTA));

    frame.render_widget(block, area);
}

fn draw_middle_left(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 常用番茄钟时间 ").centered())
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(TokyoNight::MAGENTA));

    frame.render_widget(Paragraph::new(" 选择番茄钟时间 ").block(block), area);
}

fn draw_middle_right(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 自定义番茄钟时间 ").centered())
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(TokyoNight::MAGENTA));

    frame.render_widget(Paragraph::new(" 请自定义番茄钟时间 ").block(block), area);
}

fn draw_down(_app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .title(Line::from(" 音乐播放列表 ").centered())
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(TokyoNight::MAGENTA));

    frame.render_widget(
        Paragraph::new(" 请选择你想要播放的音乐 ").block(block),
        area,
    );
}
