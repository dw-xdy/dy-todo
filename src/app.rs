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
        // 1. 暂时取走窗口
        if let Some(mut window) = self.active_window.take() {
            // 2. 处理事件
            let handled = self.handle_window_key_event(&mut window, key);

            // 3. 检查处理后的状态
            // 如果内部调用了 close_window()，那么此时 self.active_window 依然是 None。
            // 我们增加一个逻辑：只有当按键不是 Enter 且不是 Esc 时，才归还 window。
            // 或者更严谨地：检查 handle_window_key_event 的意图。

            let should_close = match key.code {
                KeyCode::Enter => true, // 假设 Enter 总是提交并关闭
                KeyCode::Esc => true,   // Esc 总是取消并关闭
                _ => false,
            };

            if !should_close {
                // 如果不需要关闭，把窗口放回去
                self.active_window = Some(window);
            } else {
                // 如果是 Enter 或 Esc，我们就不执行赋值操作，
                // 局部变量 window 会在作用域结束时被销毁，窗口也就彻底关闭了。
                return;
            }

            if handled {
                return;
            }
        }
        // 4. 全局快捷键逻辑 (当没有窗口或窗口未拦截事件时触发)
        match key.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('j') | KeyCode::Down => self.next(),
            KeyCode::Char('k') | KeyCode::Up => self.previous(),

            // 快捷键打开不同窗口
            KeyCode::Char('n') => self.open_window(WindowType::CreateTask),
            KeyCode::Char('p') => self.open_window(WindowType::PomodoroSettings),
            _ => {}
        }
    }

    /// 处理窗口内的键盘事件
    fn handle_window_key_event(&mut self, window: &mut ActiveWindow, key: KeyEvent) -> bool {
        match &mut window.data {
            WindowData::CreateTask {
                title,
                description,
                current_field,
            } => {
                match key.code {
                    // Tab 切换输入字段
                    KeyCode::Tab => {
                        *current_field = (*current_field + 1) % 2;
                        true
                    }
                    // Enter 提交表单
                    KeyCode::Enter => {
                        self.create_task(title.clone(), description.clone());
                        self.close_window();
                        true
                    }
                    // Esc 关闭窗口
                    KeyCode::Esc => {
                        self.close_window();
                        true
                    }
                    // 字符输入
                    KeyCode::Char(c) => {
                        if *current_field == 0 {
                            title.push(c);
                        } else {
                            description.push(c);
                        }
                        true
                    }
                    // 退格删除
                    KeyCode::Backspace => {
                        if *current_field == 0 {
                            title.pop();
                        } else {
                            description.pop();
                        }
                        true
                    }
                    // 空格键
                    KeyCode::Char(' ') => {
                        if *current_field == 0 {
                            title.push(' ');
                        } else {
                            description.push(' ');
                        }
                        true
                    }
                    _ => false, // 未处理的事件
                }
            }
            WindowData::PomodoroSettings {
                play_during_pomodoro,
                play_on_finish,
                selected_duration,
                custom_duration,
                current_focus,
            } => {
                match key.code {
                    KeyCode::Tab => {
                        *current_focus = (*current_focus + 1) % 5; // 循环5个焦点区域
                        true
                    }
                    KeyCode::Enter => {
                        // 保存设置
                        self.save_pomodoro_settings(
                            *play_during_pomodoro,
                            *play_on_finish,
                            *selected_duration,
                            custom_duration.clone(),
                        );
                        self.close_window();
                        true
                    }
                    KeyCode::Esc => {
                        self.close_window();
                        true
                    }
                    KeyCode::Char(' ') => {
                        match *current_focus {
                            0 => *play_during_pomodoro = !*play_during_pomodoro,
                            1 => *play_on_finish = !*play_on_finish,
                            _ => {}
                        }
                        true
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if *current_focus == 2 {
                            // 常用时间选择
                            if *selected_duration > 0 {
                                *selected_duration -= 1;
                            }
                        }
                        true
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if *current_focus == 2 {
                            // 常用时间选择
                            if *selected_duration < 4 {
                                *selected_duration += 1;
                            }
                        }
                        true
                    }
                    KeyCode::Char(c) => {
                        if *current_focus == 3 {
                            // 自定义时间输入
                            if c.is_ascii_digit() {
                                custom_duration.push(c);
                            }
                        }
                        true
                    }
                    KeyCode::Backspace => {
                        if *current_focus == 3 {
                            custom_duration.pop();
                        }
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// 创建新任务
    fn create_task(&mut self, title: String, description: String) {
        let task = TodoTask {
            title,
            description,
            is_completed: false,
            tags: std::collections::HashSet::new(),
        };
        self.tasks.push(task);

        // 更新滚动条
        self.scroll_state = ScrollbarState::new(self.tasks.len());
    }

    /// 保存番茄钟设置
    fn save_pomodoro_settings(
        &mut self,
        play_during: bool,
        play_finish: bool,
        duration_index: usize,
        custom_duration: String,
    ) {
        // 这里实现保存番茄钟设置的逻辑
    }

    /// 打开新窗口
    fn open_window(&mut self, window_type: WindowType) {
        let layout = self.get_window_layout(&window_type);
        let data = match window_type {
            WindowType::CreateTask => WindowData::CreateTask {
                title: String::new(),
                description: String::new(),
                current_field: 0,
            },
            WindowType::PomodoroSettings => WindowData::PomodoroSettings {
                play_during_pomodoro: false,
                play_on_finish: false,
                selected_duration: 2, // 默认25分钟
                custom_duration: String::new(),
                current_focus: 0,
            },
            _ => WindowData::Empty,
        };

        self.active_window = Some(ActiveWindow {
            window_type,
            layout,
            data,
            is_visible: true,
        });
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
