use crate::models::TodoTask;
use crate::ui; // 引入 UI 渲染
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, widgets::ListState};
use std::io;

pub struct App {
    pub exit: bool,
    pub tasks: Vec<TodoTask>,
    pub list_state: ListState,
}

impl Default for App {
    fn default() -> Self {
        // 1. 准备一些初始数据（可选，方便你调试界面）
        let tasks = vec![
            TodoTask {
                title: "写代码".into(),
                description: "使用 Rust 和 Ratatui 编写 TUI 应用".into(),
                is_completed: false,
                tags: std::collections::HashSet::new(),
            },
            TodoTask {
                title: "去运动".into(),
                description: "跑 5 公里，呼吸新鲜空气".into(),
                is_completed: true,
                tags: std::collections::HashSet::new(),
            },
        ];

        // 2. 初始化 ListState
        let mut list_state = ListState::default();
        
        // 3. 如果列表不为空，默认选中第一项
        if !tasks.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            exit: false,
            tasks,
            list_state,
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| ui::render(self, frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                self.handle_key_event(key);
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('j') | KeyCode::Down => self.next(),
            KeyCode::Char('k') | KeyCode::Up => self.previous(),
            _ => {}
        }
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.tasks.len() - 1 { 0 } else { i + 1 }
            }

            None => 0,
        };
        self.list_state.select(Some(i));
    }
    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 { self.tasks.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}
