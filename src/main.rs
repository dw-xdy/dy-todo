use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph},
};

// 主函数
fn main() -> io::Result<()> {
    // 运行应用
    ratatui::run(|terminal| App::default().run(terminal))
}

// 应用状态
#[derive(Debug, Default)]
struct App {
    exit: bool,
}

impl App {
    // 主循环
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            // 绘制界面
            terminal.draw(|frame| self.draw(frame))?;
            // 处理事件
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
    
        // --- 第一步：水平布局（切成三列） ---
        let main_layout = Layout::horizontal([
            Constraint::Percentage(20), // 左侧：标签框 (20%)
            Constraint::Percentage(50), // 中间：Todo 列表 (50%)
            Constraint::Percentage(30), // 右侧：复合功能区 (30%)
        ]);
        let cols = main_layout.split(area);
        
        // 为了代码清晰，我们给这三块区域起个名字
        let tags = cols[0];
        let todo_list = cols[1];
        let pomodoro_and_details  = cols[2];

        // --- 第二步：垂直布局（把右侧那一列切成上下两块） ---
        let right_layout = Layout::vertical([
            Constraint::Length(10),     // 右上：番茄钟（固定高度 10）
            Constraint::Min(0),         // 右下：详细信息（占据剩下所有空间）
        ]);

        let right_areas = right_layout.split(pomodoro_and_details);

        let pomodoro_timer_area = right_areas[0];
        let detail_area = right_areas[1];

        self.draw_tags(tags, frame);
        self.draw_todo_list(todo_list, frame);
        self.draw_pomodoro(pomodoro_timer_area, frame);
        self.draw_details(detail_area, frame);
    }

    fn draw_tags(&self, area: Rect, frame: &mut Frame) {
        let block = Block::default()
            .title("标签框")
            .borders(Borders::ALL);

        let paragraph = Paragraph::new ("标签区域")
            .block(block);

        frame.render_widget(paragraph, area);
    }
    
    fn draw_todo_list(&self, area: Rect, frame: &mut Frame) {
        
        let block = Block::default()
            .title("代办事项")
            .borders(Borders::ALL);


        let paragraph = Paragraph::new("TODO列表区域")
            .block(block);

        frame.render_widget(paragraph, area);
    }
    
    fn draw_pomodoro(&self, area: Rect, frame: &mut Frame) {
        let block = Block::default()
            .title("🍅 番茄钟")
            .borders(Borders::ALL);

        let paragraph = Paragraph::new("番茄钟区域\n\n(未来将显示计时器)")
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
    }
    
    fn draw_details(&self, area: Rect, frame: &mut Frame) {
        let block = Block::default()
            .title("任务详情")
            .borders(Borders::ALL);

        let paragraph = Paragraph::new("详情区域\n\n选择TODO项目查看详情")
            .block(block);

        frame.render_widget(paragraph, area);
    }

    // 处理事件
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    // 处理按键事件
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            _ => {}
        }
    }
}
