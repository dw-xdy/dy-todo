use crate::app::App;
use crate::models::{ActiveWindow, TokyoNight, WindowData, WindowType, PlaybackState};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Color,
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation},
};

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let main_layout = Layout::horizontal([
        Constraint::Percentage(15),
        Constraint::Percentage(55),
        Constraint::Percentage(30),
    ]);
    let cols = main_layout.split(area);

    let right_layout = Layout::vertical([Constraint::Percentage(30), Constraint::Percentage(70)]);
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
        .border_set(border::ROUNDED);
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
                .border_set(border::ROUNDED),
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
        .border_style(Style::default().fg(TokyoNight::RED))
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

    // 修改 draw_todo 函数显示输入框
    draw_todo(_app, left_areas[0], title, current_field == 0, frame);
    // 修改 draw_desc 函数显示输入框
    draw_desc(_app, left_areas[1], description, current_field == 1, frame);
    draw_must_tag(_app, right_areas[0], frame);
    draw_diy_tag(_app, right_areas[1], frame);
}

fn draw_todo(_app: &App, area: Rect, title: &str, is_active: bool, frame: &mut Frame) {
    let border_style = if is_active {
        Style::default().fg(TokyoNight::CYAN).bold()
    } else {
        Style::default().fg(TokyoNight::RED)
    };

    let block = Block::bordered()
        .title(Line::from(" 📝 新的todo ").centered())
        .border_set(border::ROUNDED)
        .border_style(border_style);

    // 显示当前输入的内容
    let display_text = if title.is_empty() {
        "输入任务标题..."
    } else {
        title
    };

    let paragraph = Paragraph::new(display_text)
        .block(block)
        .style(if is_active {
            Style::default().fg(Color::White).bg(TokyoNight::GRAY)
        } else {
            Style::default()
        });

    frame.render_widget(paragraph, area);
}

fn draw_desc(_app: &App, area: Rect, description: &str, is_active: bool, frame: &mut Frame) {
    let border_style = if is_active {
        Style::default().fg(TokyoNight::CYAN).bold()
    } else {
        Style::default().fg(TokyoNight::RED)
    };

    let block = Block::bordered()
        .title(Line::from(" 📋 todo的详细信息 ").centered())
        .border_set(border::ROUNDED)
        .border_style(border_style);

    let display_text = if description.is_empty() {
        "输入任务描述..."
    } else {
        description
    };

    let paragraph = Paragraph::new(display_text)
        .block(block)
        .style(if is_active {
            Style::default().fg(Color::White).bg(TokyoNight::GRAY)
        } else {
            Style::default()
        });

    frame.render_widget(paragraph, area);
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

    if let Some(window) = &_app.active_window
        && let WindowData::PomodoroSettings {
            play_during_pomodoro,
            play_on_finish,
            selected_duration,
            custom_duration,
            current_focus,
        } = &window.data
    {
        draw_up_left(
            _app,
            up_areas[0],
            *play_during_pomodoro,
            *current_focus == 0,
            frame,
        );
        draw_up_right(
            _app,
            up_areas[1],
            *play_on_finish,
            *current_focus == 1,
            frame,
        );
        draw_middle_left(
            _app,
            middle_areas[0],
            *selected_duration,
            *current_focus == 2,
            frame,
        );
        draw_middle_right(
            _app,
            middle_areas[1],
            custom_duration,
            *current_focus == 3,
            frame,
        );
        draw_down(_app, rows[2], *current_focus == 4, frame);
    }
}

fn draw_up_left(_app: &App, area: Rect, enabled: bool, is_active: bool, frame: &mut Frame) {
    let border_style = if is_active {
        Style::default().fg(TokyoNight::CYAN).bold()
    } else {
        Style::default().fg(TokyoNight::RED)
    };

    let block = Block::bordered()
        .title(Line::from(" 🎵 运行时播放音乐? ").centered())
        .border_set(border::ROUNDED)
        .border_style(border_style);

    let status = if enabled { "✅ 是" } else { "❌ 否" };
    let paragraph = Paragraph::new(status)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn draw_up_right(_app: &App, area: Rect, enabled: bool, is_active: bool, frame: &mut Frame) {
    let border_style = if is_active {
        Style::default().fg(TokyoNight::CYAN).bold()
    } else {
        Style::default().fg(TokyoNight::RED)
    };

    let block = Block::bordered()
        .title(Line::from(" ⏹️ 结束时播放音乐? ").centered())
        .border_set(border::ROUNDED)
        .border_style(border_style);

    let status = if enabled { "✅ 是" } else { "❌ 否" };
    let paragraph = Paragraph::new(status)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn draw_middle_left(_app: &App, area: Rect, selected: usize, is_active: bool, frame: &mut Frame) {
    let border_style = if is_active {
        Style::default().fg(TokyoNight::CYAN).bold()
    } else {
        Style::default().fg(TokyoNight::ORANGE)
    };

    let block = Block::bordered()
        .title(Line::from(" ⏱️ 常用番茄钟时间 ").centered())
        .border_set(border::ROUNDED)
        .border_style(border_style);

    let durations = ["15分钟", "20分钟", "25分钟", "30分钟", "45分钟"];
    let items: Vec<ListItem> = durations
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let prefix = if i == selected { "▶ " } else { "  " };
            ListItem::new(Line::from(vec![prefix.into(), (*d).into()]))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_middle_right(_app: &App, area: Rect, custom: &str, is_active: bool, frame: &mut Frame) {
    let border_style = if is_active {
        Style::default().fg(TokyoNight::CYAN).bold()
    } else {
        Style::default().fg(TokyoNight::ORANGE)
    };

    let block = Block::bordered()
        .title(Line::from(" ✏️ 自定义时间(分钟) ").centered())
        .border_set(border::ROUNDED)
        .border_style(border_style);

    let display_text = if custom.is_empty() {
        "输入数字..."
    } else {
        &custom[..]
    };

    let paragraph = Paragraph::new(display_text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center)
        .style(if is_active {
            Style::default().fg(Color::White).bg(TokyoNight::GRAY)
        } else {
            Style::default()
        });

    frame.render_widget(paragraph, area);
}

fn draw_down(app: &App, area: Rect, is_active: bool, frame: &mut Frame) {
    let border_style = if is_active {
        Style::default().fg(TokyoNight::CYAN).bold()
    } else {
        Style::default().fg(TokyoNight::CYAN)
    };

    let block = Block::bordered()
        .title(Line::from(" 🎵 音乐播放列表 ").centered())
        .border_set(border::ROUNDED)
        .border_style(border_style);

    // 显示播放状态提示
    let help_text = if is_active {
        Line::from(vec![
            " ↑/k ↓/j ".fg(TokyoNight::GRAY),
            " 选择 ".fg(Color::White),
            " Enter ".fg(TokyoNight::GRAY),
            " 播放 ".fg(Color::White),
            " Space ".fg(TokyoNight::GRAY),
            " 暂停/继续 ".fg(Color::White),
        ])
    } else {
        Line::from("")
    };

    // 构建列表项，显示播放状态
    let items: Vec<ListItem> = app
        .music_files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let is_playing = app.music_player_state.current_playing_index == Some(i)
                && app.music_player_state.playback_state == PlaybackState::Playing;
            let is_paused = app.music_player_state.current_playing_index == Some(i)
                && app.music_player_state.playback_state == PlaybackState::Paused;

            let icon = if is_playing {
                " ▶️ ".into()
            } else if is_paused {
                " ⏸️ ".into()
            } else {
                " 🎶 ".into()
            };

            ListItem::new(Line::from(vec![icon, file.name.clone().into()]))
        })
        .collect();

    // 如果音乐文件为空，显示提示信息
    let items = if items.is_empty() {
        vec![ListItem::new(Line::from(vec![
            " 📭 没有找到音乐文件".into(),
        ]))]
    } else {
        items
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(TokyoNight::GRAY)
                .fg(Color::White)
                .bold(),
        )
        .highlight_symbol("▶ ");

    // 步骤1：先渲染列表
    frame.render_stateful_widget(list, area, &mut app.music_list_state.clone());

    // 步骤2：再渲染滚动条（在列表上方）
    // 只有当音乐文件数量大于可见行数时才显示滚动条
    let visible_height = area.height.saturating_sub(2) as usize; // 减去边框
    if app.music_files.len() > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("░"))
            .thumb_symbol("█")
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        // 克隆滚动条状态
        let mut music_scroll_state = app.music_scroll_state;

        // 滚动条区域：在列表内部右侧
        // 注意：x坐标需要是 area.x + area.width - 2（右边留2列）
        let scrollbar_area = Rect {
            x: area.x + area.width - 2, // 从右边第2列开始
            y: area.y + 1,              // 顶部留1行给边框
            width: 1,                   // 宽度1列
            height: area.height - 2,    // 高度减去上下边框
        };

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut music_scroll_state);
    }

    // 步骤3：渲染帮助文本
    if is_active && !app.music_files.is_empty() {
        let help_area = Rect {
            x: area.x,
            y: area.y + area.height - 2,
            width: area.width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(help_text).alignment(ratatui::layout::Alignment::Center),
            help_area,
        );
    }
}
