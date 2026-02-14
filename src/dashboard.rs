use crate::models::AsciiArt;
use crate::models::TokyoNight;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

pub struct Dashboard;

impl Dashboard {
    pub fn render(area: Rect, frame: &mut Frame, version: &str) {
        // 创建垂直布局，将屏幕分成三部分
        let layout = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Percentage(30), // 上部留白
            ratatui::layout::Constraint::Percentage(40), // 中间放 ASCII Art
            ratatui::layout::Constraint::Percentage(30), // 下部放版本和提示
        ]);
        let chunks = layout.split(area);

        // 获取终端宽度用于动态居中
        let terminal_width = area.width;

        // 生成居中的 ASCII Art
        let header_lines = AsciiArt::centered_header(version, terminal_width);

        // 转换为带颜色的文本
        let header_text: Vec<Line> = header_lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let color = match i % 6 {
                    0 => TokyoNight::CYAN,
                    1 => TokyoNight::MAGENTA,
                    2 => TokyoNight::ORANGE,
                    3 => TokyoNight::RED,
                    4 => TokyoNight::GRAY,
                    5 => TokyoNight::CYAN,
                    _ => TokyoNight::GRAY,
                };
                Line::from(vec![Span::styled(line, Style::default().fg(color))])
            })
            .collect();

        // 渲染 ASCII Art
        let header_paragraph = Paragraph::new(header_text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(header_paragraph, chunks[1]);

        // 渲染版本信息
        let version_text = format!("版本: {}", version);
        let version_paragraph = Paragraph::new(version_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(TokyoNight::GRAY));
        frame.render_widget(version_paragraph, chunks[2]);

        // 渲染底部提示
        let footer_area = Rect {
            x: area.x,
            y: area.y + area.height - 3,
            width: area.width,
            height: 3,
        };

        let footer_text = vec![
            Line::from(vec![
                "按 ".fg(Color::White),
                "任意键".fg(Color::Rgb(255, 200, 100)).bold(),
                " 继续".fg(Color::White),
            ]),
            Line::from(vec![
                "使用 ".fg(Color::White),
                "a".fg(Color::Rgb(100, 255, 100)).bold(),
                " 创建任务 | ".fg(Color::White),
                "p".fg(Color::Rgb(100, 255, 100)).bold(),
                " 番茄钟 | ".fg(Color::White),
                "j/k".fg(Color::Rgb(100, 255, 100)).bold(),
                " 导航 | ".fg(Color::White),
                "q".fg(Color::Rgb(255, 100, 100)).bold(),
                " 退出".fg(Color::White),
            ]),
        ];

        let footer_paragraph = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .block(Block::default());
        frame.render_widget(footer_paragraph, footer_area);
    }
}
