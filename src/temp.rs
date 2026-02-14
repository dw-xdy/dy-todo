// 上面切割出界面是否在番茄钟进行中和结束时播放音乐.
// let up_layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
// let up_areas = up_layout.split(rows[0]);

// draw_play_during_pomodoro(
//     _app,
//     up_areas[0],
//     *play_during_pomodoro,
//     *current_focus == 0,
//     frame,
// );
// draw_play_on_finish(
//     _app,
//     up_areas[1],
//     *play_on_finish,
//     *current_focus == 1,
//     frame,
// );

fn draw_play_during_pomodoro(
    _app: &App,
    area: Rect,
    enabled: bool,
    is_active: bool,
    frame: &mut Frame,
) {
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

fn draw_play_on_finish(_app: &App, area: Rect, enabled: bool, is_active: bool, frame: &mut Frame) {
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
