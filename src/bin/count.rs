// ANCHOR: imports
use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
    style::{Color, Style}, // 确保这里有 Color 和 Style
};
// ANCHOR_END: imports

// 这里我们创建了APP, 延迟传播了App::run() 返回的结果, 确保在终端恢复之后才程序退出之后将所有的 error 结果返回
fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

// ANCHOR: app
// 这里创建一个APP, 然后对应的, 确定一个 u8 用来计数, 并且还有一个退出的标志: bool
// 还有这里的: Default trait 使用来设置这个 struct 的默认值的, Rust 并不会像 Java 一样自动设置默认值.
#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
}
// ANCHOR_END: app


// ANCHOR: impl App
impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    // ANCHOR: handle_key_event fn
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            KeyCode::Char('r') => self.reset(),
            _ => {}
        }
    }
    // ANCHOR_END: handle_key_event fn

    fn exit(&mut self) {
        self.exit = true;
    }

    // 添加了两个防护机制, 只有在 > 0 的情况下才会减 1, 只有在 < 255 的情况下才会加 1.
    fn increment_counter(&mut self) {
        if self.counter < 255 {
            self.counter += 1;
        }    
    }

    fn decrement_counter(&mut self) {
        if self.counter > 0 {
            self.counter -= 1;
        }
    }

    fn reset(&mut self) {
        self.counter = 0;
    }
}
// ANCHOR_END: impl App

// ANCHOR: impl Widget
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" 我的第一个计数器应用 ".red());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
            // 添加一个复位按键提示
            " Reset ".into(),
            "<R> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title)
            .title_bottom(instructions.centered())
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Cyan));
        // 将原来的 border::THICK 修改为: border::ROUNDED 即: 变为圆角

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
// ANCHOR_END: impl Widget
