use crate::app::App;
use crate::dashboard::Dashboard;
use crate::models::{ActiveWindow, PlaybackState, TaskStatus, TokyoNight, WindowData, WindowType};
use chrono::Utc;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation},
};

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // 如果显示 dashboard，只渲染 dashboard
    if app.show_dashboard {
        Dashboard::render(area, frame, "0.1.5");
        return;
    }
    // 竖着进行分割, 分割成三份
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
    frame.render_widget(Paragraph::new("输入标签搜索...").block(block), area);
}

fn draw_todo(app: &App, area: Rect, title: &str, is_active: bool, frame: &mut Frame) {
    let border_style = if is_active {
        Style::default().fg(TokyoNight::CYAN).bold()
    } else {
        Style::default().fg(TokyoNight::RED)
    };

    let block = Block::bordered()
        .title(Line::from(" 📝 新的todo ").centered())
        .border_set(border::ROUNDED)
        .border_style(border_style);

    // 获取光标位置
    let cursor_pos = if is_active {
        if let Some(window) = &app.active_window {
            if let WindowData::CreateTask {
                cursor_position,
                current_field,
                ..
            } = &window.data
            {
                if *current_field == 0 {
                    Some(*cursor_position)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // 构建显示文本
    let display_text = if title.is_empty() {
        if is_active {
            " ".to_string() // 空文本时显示一个空格让光标可见
        } else {
            "输入任务标题...".to_string()
        }
    } else {
        title.to_string()
    };

    // 先设置光标位置（使用 display_text 的引用）
    if let Some(pos) = cursor_pos {
        // 计算光标的屏幕位置
        let visible_start = if pos > area.width as usize - 3 {
            pos.saturating_sub(area.width as usize - 3)
        } else {
            0
        };

        let cursor_x = area.x + 1 + (pos - visible_start) as u16;
        let cursor_y = area.y + 1;

        if cursor_x < area.x + area.width - 1 {
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }

    // 先渲染 Paragraph
    let paragraph = Paragraph::new(display_text)
        .block(block)
        .style(if is_active {
            Style::default().fg(Color::White).bg(TokyoNight::GRAY)
        } else {
            Style::default()
        });

    frame.render_widget(paragraph, area);

    // 再设置光标位置（渲染之后）
    if let Some(pos) = cursor_pos {
        let visible_start = if pos > area.width as usize - 3 {
            pos.saturating_sub(area.width as usize - 3)
        } else {
            0
        };

        let cursor_x = area.x + 1 + (pos - visible_start) as u16;
        let cursor_y = area.y + 1;

        if cursor_x < area.x + area.width - 1 {
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }
}

fn draw_desc(app: &App, area: Rect, description: &str, is_active: bool, frame: &mut Frame) {
    let border_style = if is_active {
        Style::default().fg(TokyoNight::CYAN).bold()
    } else {
        Style::default().fg(TokyoNight::RED)
    };

    let block = Block::bordered()
        .title(Line::from(" 📋 todo的详细信息 ").centered())
        .border_set(border::ROUNDED)
        .border_style(border_style);

    // 获取光标位置
    let cursor_pos = if is_active {
        if let Some(window) = &app.active_window {
            if let WindowData::CreateTask {
                cursor_position,
                current_field,
                ..
            } = &window.data
            {
                if *current_field == 1 {
                    Some(*cursor_position)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // 构建显示文本
    let display_text = if description.is_empty() {
        if is_active {
            " ".to_string()
        } else {
            "输入任务描述...".to_string()
        }
    } else {
        description.to_string()
    };

    // 先计算光标位置（使用 display_text 的引用）
    if let Some(pos) = cursor_pos {
        let visible_start = if pos > area.width as usize - 3 {
            pos.saturating_sub(area.width as usize - 3)
        } else {
            0
        };

        let cursor_x = area.x + 1 + (pos - visible_start) as u16;
        let cursor_y = area.y + 1;

        if cursor_x < area.x + area.width - 1 {
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }

    // 先渲染 Paragraph
    let paragraph = Paragraph::new(display_text)
        .block(block)
        .style(if is_active {
            Style::default().fg(Color::White).bg(TokyoNight::GRAY)
        } else {
            Style::default()
        });

    frame.render_widget(paragraph, area);

    // 再设置光标位置（渲染之后）
    if let Some(pos) = cursor_pos {
        let visible_start = if pos > area.width as usize - 3 {
            pos.saturating_sub(area.width as usize - 3)
        } else {
            0
        };

        let cursor_x = area.x + 1 + (pos - visible_start) as u16;
        let cursor_y = area.y + 1;

        if cursor_x < area.x + area.width - 1 {
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }
}

fn draw_todo_list(app: &App, area: Rect, frame: &mut Frame) {
    // 1. 使用 status 枚举获取图标
    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .map(|task| {
            // 使用 status.icon() 获取对应的图标
            let status_icon = task.status.icon();

            // 可以根据状态设置不同颜色（可选）
            let icon_color = match task.status {
                TaskStatus::Completed => Color::Green,
                TaskStatus::Todo => Color::White,
                TaskStatus::Overdue => Color::Red,
                TaskStatus::DueToday => Color::Yellow,
            };

            // 创建带颜色的图标和标题
            let icon_span =
                Span::styled(format!(" {status_icon} "), Style::default().fg(icon_color));

            let title_span = Span::raw(task.title.clone());

            // 如果有截止日期，添加额外信息（可选）
            let due_info = if let Some(due) = task.due_date {
                let now = Utc::now();
                let days = (due - now).num_days();
                if days > 0 && task.status != TaskStatus::Completed {
                    format!(" ({days}d)")
                } else if days == 0 && task.status != TaskStatus::Completed {
                    " (今天)".to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let due_span = Span::raw(due_info);

            ListItem::new(Line::from(vec![icon_span, title_span, due_span]))
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
    let visible_height = area.height.saturating_sub(2) as usize;
    if app.tasks.len() > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("░"))
            .thumb_symbol("█");

        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut app.scroll_state.clone(),
        );
    }
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
        .border_set(border::ROUNDED);

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
                cursor_position,
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
    frame.render_widget(block, area);

    // 分割窗口内部区域
    let layout = Layout::horizontal([
        Constraint::Percentage(70), // 分隔
        Constraint::Percentage(30),
    ]);
    let chunks = layout.split(inner_area);

    let left_layout = Layout::vertical([Constraint::Percentage(30), Constraint::Percentage(70)]);
    let left_areas = left_layout.split(chunks[0]);

    draw_todo(_app, left_areas[0], title, current_field == 0, frame);
    draw_desc(_app, left_areas[1], description, current_field == 1, frame);
    draw_tag(_app, chunks[1], frame);
}

fn draw_tag(_app: &App, area: Rect, frame: &mut Frame) {
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

    frame.render_widget(block, area);

    let main_layout = Layout::vertical([Constraint::Percentage(40), Constraint::Percentage(60)]);

    let rows = main_layout.split(inner_area);

    // 上面切割出常用时间和自定义的时间
    let up_layout = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]);
    let up_areas = up_layout.split(rows[0]);

    // 下面就不切割了, 因为是音乐播放列表

    if let Some(window) = &_app.active_window
        && let WindowData::PomodoroSettings {
            selected_duration,
            custom_duration,
            current_focus,
        } = &window.data
    {
        draw_selected_duration(
            _app,
            up_areas[0],
            *selected_duration,
            *current_focus == 2,
            frame,
        );
        draw_middle_right(
            _app,
            up_areas[1],
            custom_duration,
            *current_focus == 3,
            frame,
        );
        draw_down(_app, rows[1], *current_focus == 4, frame);
    }
}

fn draw_selected_duration(
    _app: &App,
    area: Rect,
    selected: usize,
    is_active: bool,
    frame: &mut Frame,
) {
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
            let prefix = if i == selected { " ▶ " } else { "  " };
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
        custom
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

    // 显示播放状态提示 - 彩虹色
    let help_text = if is_active {
        Line::from(vec![
            " ↑/k ↓/j ".fg(Color::Rgb(255, 200, 100)), // 暖黄
            "选择 ".fg(Color::White),
            "Enter ".fg(Color::Rgb(100, 255, 100)), // 亮绿
            "播放 ".fg(Color::White),
            "Space ".fg(Color::Rgb(100, 200, 255)), // 天蓝
            "暂停/继续 ".fg(Color::White),
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
            .thumb_symbol("█");

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
            y: area.y + area.height - 1, // 放在列表的最后一行
            width: area.width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(help_text).alignment(ratatui::layout::Alignment::Center),
            help_area,
        );
    }
}
