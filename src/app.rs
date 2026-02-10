use crate::models::{ActiveWindow, AudioFileInfo, TodoTask, WindowData, WindowLayout, WindowType};
use crate::ui; // 引入 UI 渲染
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::ScrollbarState; // 确保导入
use ratatui::{DefaultTerminal, widgets::ListState};
use std::io;
use walkdir::WalkDir; // 确保 Cargo.toml 有 walkdir 依赖

pub struct App {
    pub exit: bool,
    pub tasks: Vec<TodoTask>,
    pub list_state: ListState,
    pub active_window: Option<ActiveWindow>, // 当前活动窗口（None 表示无窗口）
    pub scroll_state: ScrollbarState,
    // 音乐
    pub music_files: Vec<AudioFileInfo>,
    pub music_list_state: ListState, // 音乐列表的选中状态
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
            TodoTask {
                title: "写代码".into(),
                description: "使用 Rust 和 Ratatui 编写 TUI 应用".into(),
                is_completed: false,
                tags: std::collections::HashSet::new(),
            },
        ];

        // 2. 初始化 ListState
        let mut list_state = ListState::default();

        // 3. 如果列表不为空，默认选中第一项
        if !tasks.is_empty() {
            list_state.select(Some(0));
        }

        let tasks_len = tasks.len();
        let mut app = Self {
            exit: false,
            tasks,
            list_state,
            scroll_state: ScrollbarState::new(tasks_len), // <--- 初始化
            music_files: Vec::new(),
            music_list_state: ListState::default(),
            active_window: None,
        };

        // 2. 在这里调用加载目录的代码
        // 建议：由于 "F:\\..." 是 Windows 路径，确保你的开发环境路径正确
        app.load_music_from_dir("F:\\D\\音乐\\音乐文件");

        // 3. 返回配置好的 app
        app
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

            // 快捷键打开不同窗口
            KeyCode::Char('n') => self.open_window(WindowType::CreateTask),
            KeyCode::Char('p') => self.open_window(WindowType::PomodoroSettings),

            // ESC 关闭当前窗口
            KeyCode::Char('w') => self.close_window(),
            _ => {}
        }
    }
    // 修改 next 方法
    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.tasks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i); // <--- 同步位置
    }

    // 修改 previous 方法
    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tasks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i); // <--- 同步位置
    }
    /// 打开一个新窗口
    fn open_window(&mut self, window_type: WindowType) {
        let layout = self.get_window_layout(&window_type);
        let data = self.get_window_data(&window_type);

        self.active_window = Some(ActiveWindow {
            window_type,
            layout,
            data,
            is_visible: true,
        });
    }

    /// 关闭当前窗口
    fn close_window(&mut self) {
        self.active_window = None;
    }

    /// 根据窗口类型获取默认布局
    fn get_window_layout(&self, window_type: &WindowType) -> WindowLayout {
        // 这里可以根据屏幕大小动态计算，先使用固定值
        match window_type {
            WindowType::CreateTask => WindowLayout {
                x: 15,
                y: 2,
                width: 80,
                height: 21,
            },
            // ... 其他窗口的默认布局
            _ => WindowLayout {
                x: 15,
                y: 2,
                width: 80,
                height: 21,
            },
        }
    }

    /// 根据窗口类型获取默认数据p
    fn get_window_data(&self, window_type: &WindowType) -> WindowData {
        match window_type {
            WindowType::CreateTask => WindowData::CreateTask {
                title: String::new(),
                description: String::new(),
                current_field: 0,
            },
            _ => WindowData::Empty,
        }
    }

    /// 扫描指定目录并将音频文件载入应用
    pub fn load_music_from_dir(&mut self, directory: &str) {
        let mut files = Vec::new();

        // 遍历目录寻找 mp3/wav 文件
        for entry in WalkDir::new(directory)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file()
                && path.extension().map_or(false, |ext| {
                    let ext_str = ext.to_ascii_lowercase();
                    ext_str == "mp3" || ext_str == "wav"
                })
            {
                files.push(AudioFileInfo {
                    name: path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    path: path.to_path_buf(),
                });
            }
        }

        files.sort_by(|a, b| a.name.cmp(&b.name)); // 排序
        self.music_files = files;

        // 如果有文件，默认选中第一个
        if !self.music_files.is_empty() {
            self.music_list_state.select(Some(0));
        }
    }
}
