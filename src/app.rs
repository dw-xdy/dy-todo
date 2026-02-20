// ==================== 标准库导入 ====================
use std::collections::HashSet; // 哈希集合，用于存储不重复的标签
use std::fs::File; // 文件操作
use std::io; // 输入输出
use std::io::BufReader; // 带缓冲的读取器，用于读取音频文件
use std::sync::{Arc, Mutex}; // 线程安全的共享所有权和互斥锁

// ==================== 第三方库导入 ====================
// 时间日期处理
use time::{Duration, OffsetDateTime}; // 使用 time 进行日期时间处理

// 终端事件处理
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind}; // 键盘事件

// TUI 渲染
use ratatui::{
    DefaultTerminal,         // 默认终端类型
    widgets::ListState,      // 列表状态管理
    widgets::ScrollbarState, // 滚动条状态管理
};

// 音频播放
use rodio::{Decoder, OutputStream, Sink}; // 音频解码和播放

// 目录遍历
use walkdir::WalkDir; // 递归遍历目录

// ==================== 项目内部模块导入 ====================
// UI 渲染模块
use crate::ui; // 界面渲染逻辑

// 数据模型模块 - 按功能分组
use crate::models::{
    // ----- 窗口相关模型 -----
    ActiveWindow, // 活动窗口
    // ----- 音乐相关模型 -----
    AudioFileInfo,    // 音频文件信息
    MusicPlayerState, // 音乐播放器状态
    PlaybackState,    // 播放状态（播放/暂停/停止）

    // ----- 标签相关模型 -----
    Tag,        // 标签
    TaskStatus, // 任务状态

    // ----- 任务相关模型 -----
    TodoTask,     // 待办任务
    WindowData,   // 窗口数据
    WindowLayout, // 窗口布局
    WindowType,   // 窗口类型
};

pub struct App {
    pub exit: bool,
    pub show_dashboard: bool, // 新增：控制是否显示启动界面
    pub tasks: Vec<TodoTask>,
    pub list_state: ListState,
    pub active_window: Option<ActiveWindow>,
    pub scroll_state: ScrollbarState,
    // 音乐
    pub music_scroll_state: ScrollbarState, // 新增：音乐列表滚动条
    pub music_files: Vec<AudioFileInfo>,
    pub music_list_state: ListState,
    pub music_player_state: MusicPlayerState, // 新增
    // 播放线程相关 - 适配新版 rodio
    #[allow(dead_code)]
    sink: Option<Arc<Mutex<Sink>>>,
    #[allow(dead_code)]
    stream_handle: Option<OutputStream>,
}

impl Default for App {
    fn default() -> Self {
        // 1. 准备一些初始数据（可选，方便你调试界面）
        let tasks = vec![
            // 测试用例1：未完成的代码任务
            TodoTask {
                title: "写代码".into(),
                description: "使用 Rust 和 Ratatui 编写 TUI 应用".into(),
                status: TaskStatus::Todo,
                tags: {
                    let mut tags = HashSet::new();
                    tags.insert(Tag::new("编程".to_string()));
                    tags.insert(Tag::new("学习".to_string()));
                    tags
                },
                created_at: OffsetDateTime::now_utc() - Duration::days(2), // 2天前创建
                due_date: Some(OffsetDateTime::now_utc() + Duration::days(5)), // 5天后截止
                finish_date: None,
            },
            // 测试用例2：已完成的任务
            TodoTask {
                title: "去运动".into(),
                description: "跑 5 公里，呼吸新鲜空气".into(),
                status: TaskStatus::Completed,
                tags: {
                    let mut tags = HashSet::new();
                    tags.insert(Tag::new("健康".to_string()));
                    tags.insert(Tag::new("运动".to_string()));
                    tags
                },
                created_at: OffsetDateTime::now_utc() - Duration::days(3), // 3天前创建
                due_date: Some(OffsetDateTime::now_utc() - Duration::days(1)), // 昨天截止（但已完成）
                finish_date: Some(OffsetDateTime::now_utc() - Duration::days(1)), // 昨天完成
            },
            // 测试用例3：另一个未完成的代码任务
            TodoTask {
                title: "调试程序".into(),
                description: "修复 TUI 应用中的渲染 bug".into(),
                status: TaskStatus::Todo,
                tags: {
                    let mut tags = HashSet::new();
                    tags.insert(Tag::new("编程".to_string()));
                    tags.insert(Tag::new("调试".to_string()));
                    tags
                },
                created_at: OffsetDateTime::now_utc() - Duration::hours(5), // 5小时前创建
                due_date: Some(OffsetDateTime::now_utc() + Duration::hours(3)), // 3小时后截止
                finish_date: None,
            },
            // 测试用例4：今日到期的任务
            TodoTask {
                title: "买牛奶".into(),
                description: "记得买低脂的".into(),
                status: TaskStatus::DueToday,
                tags: {
                    let mut tags = HashSet::new();
                    tags.insert(Tag::new("购物".to_string()));
                    tags
                },
                created_at: OffsetDateTime::now_utc() - Duration::days(1), // 1天前创建
                due_date: Some(OffsetDateTime::now_utc() + Duration::hours(5)), // 今天截止
                finish_date: None,
            },
            // 测试用例5：已逾期的任务
            TodoTask {
                title: "交水电费".into(),
                description: "否则会断水断电".into(),
                status: TaskStatus::Overdue,
                tags: {
                    let mut tags = HashSet::new();
                    tags.insert(Tag::new("生活".to_string()));
                    tags.insert(Tag::new("紧急".to_string()));
                    tags
                },
                created_at: OffsetDateTime::now_utc() - Duration::days(7), // 7天前创建
                due_date: Some(OffsetDateTime::now_utc() - Duration::days(2)), // 2天前截止
                finish_date: None,
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
            show_dashboard: true, // 初始显示 dashboard
            tasks,
            list_state,
            scroll_state: ScrollbarState::new(tasks_len),
            music_scroll_state: ScrollbarState::default(), // 初始化
            music_files: Vec::new(),
            music_list_state: ListState::default(),
            music_player_state: MusicPlayerState::default(), // 初始化
            active_window: None,
            sink: None,
            stream_handle: None,
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
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            self.handle_key_event(key);
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) {
        // 如果正在显示 dashboard，按任意键关闭
        if self.show_dashboard {
            self.show_dashboard = false;
            return;
        }
        // 1. 暂时取走窗口
        if let Some(mut window) = self.active_window.take() {
            // 2. 处理事件
            let handled = self.handle_window_key_event(&mut window, key);

            // 3. 根据窗口类型和按键决定是否关闭
            let should_close = match key.code {
                KeyCode::Esc => true, // Esc 总是取消并关闭
                KeyCode::Enter => {
                    // 只有特定窗口类型的 Enter 才关闭
                    match window.window_type {
                        WindowType::CreateTask => true,        // 创建任务窗口按 Enter 关闭
                        WindowType::PomodoroSettings => false, // 番茄钟设置窗口不关闭
                        WindowType::Settings => false,         // 设置窗口不关闭
                        WindowType::Search => true,            // 搜索窗口按 Enter 关闭
                    }
                }
                _ => false,
            };

            if !should_close {
                // 如果不需要关闭，把窗口放回去
                self.active_window = Some(window);
            } else {
                // 需要关闭窗口，局部变量 window 会在作用域结束时被销毁
                return;
            }

            if handled {
                return;
            }
        }

        // 4. 全局快捷键逻辑 (当没有窗口或窗口未拦截事件时触发)
        match key.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.active_window.is_none() {
                    self.next();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.active_window.is_none() {
                    self.previous();
                }
            }
            // 快捷键打开不同窗口
            KeyCode::Char('a') => self.open_window(WindowType::CreateTask),
            KeyCode::Char('p') => self.open_window(WindowType::PomodoroSettings),
            KeyCode::Char('o') => self.open_window(WindowType::Settings),
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
                cursor_position, // 新增
            } => match key.code {
                KeyCode::Tab => {
                    *current_field = (*current_field + 1) % 2;
                    // 切换字段时重置光标位置
                    *cursor_position = if *current_field == 0 {
                        title.len()
                    } else {
                        description.len()
                    };
                    true
                }
                KeyCode::Enter => {
                    self.create_task(title.clone(), description.clone());
                    self.close_window();
                    true
                }
                KeyCode::Esc => {
                    self.close_window();
                    true
                }
                KeyCode::Left => {
                    if *cursor_position > 0 {
                        *cursor_position -= 1;
                    }
                    true
                }
                KeyCode::Right => {
                    let max_len = if *current_field == 0 {
                        title.len()
                    } else {
                        description.len()
                    };
                    if *cursor_position < max_len {
                        *cursor_position += 1;
                    }
                    true
                }
                KeyCode::Home => {
                    *cursor_position = 0;
                    true
                }
                KeyCode::End => {
                    *cursor_position = if *current_field == 0 {
                        title.len()
                    } else {
                        description.len()
                    };
                    true
                }
                KeyCode::Char(c) => {
                    let text = if *current_field == 0 {
                        title
                    } else {
                        description
                    };
                    text.insert(*cursor_position, c);
                    *cursor_position += 1;
                    true
                }
                KeyCode::Backspace => {
                    if *cursor_position > 0 {
                        let text = if *current_field == 0 {
                            title
                        } else {
                            description
                        };
                        text.remove(*cursor_position - 1);
                        *cursor_position -= 1;
                    }
                    true
                }
                KeyCode::Delete => {
                    let text = if *current_field == 0 {
                        title
                    } else {
                        description
                    };
                    if *cursor_position < text.len() {
                        text.remove(*cursor_position);
                    }
                    true
                }
                _ => false,
            },

            WindowData::PomodoroSettings {
                selected_duration,
                custom_duration,
                current_focus,
            } => {
                match key.code {
                    KeyCode::Tab => {
                        *current_focus = (*current_focus + 1) % 3;
                        true
                    }
                    KeyCode::Enter => {
                        if *current_focus == 2 {
                            // 在音乐列表按Enter播放选中的音乐
                            self.play_selected_music();
                            true
                        } else {
                            // 保存设置
                            self.save_pomodoro_settings(
                                *selected_duration,
                                custom_duration.clone(),
                            );
                            true
                        }
                    }
                    KeyCode::Esc => {
                        self.close_window();
                        true
                    }
                    KeyCode::Char(' ') => {
                        if *current_focus == 2 {
                            // 在音乐列表按空格控制播放/暂停
                            self.toggle_playback();
                            true
                        } else {
                            true
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if *current_focus == 0 {
                            // 常用时间选择
                            if *selected_duration > 0 {
                                *selected_duration -= 1;
                            }
                        } else if *current_focus == 2 {
                            // 音乐列表向上移动
                            self.music_list_previous();
                        }
                        true
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if *current_focus == 0 {
                            // 常用时间选择
                            if *selected_duration < 4 {
                                *selected_duration += 1;
                            }
                        } else if *current_focus == 2 {
                            // 音乐列表向下移动
                            self.music_list_next();
                        }
                        true
                    }
                    _ => false,
                }
            }
            // 在 handle_window_key_event 方法中，为 Settings 窗口添加音乐列表的导航
            WindowData::Settings {
                play_during_pomodoro,
                play_on_finish,
                current_focus,
            } => {
                match key.code {
                    KeyCode::Tab => {
                        // 在三个选项之间切换（两个设置选项 + 音乐列表）
                        *current_focus = (*current_focus + 1) % 3;
                        true
                    }
                    KeyCode::Enter => {
                        if *current_focus == 2 {
                            // 在音乐列表按Enter播放选中的音乐
                            self.play_selected_music();
                            true
                        } else {
                            // TODO:
                            // 保存设置并关闭窗口
                            // self.save_settings(*play_during_pomodoro, *play_on_finish);
                            // self.close_window();
                            true
                        }
                    }
                    KeyCode::Esc => {
                        self.close_window();
                        true
                    }
                    KeyCode::Char(' ') => {
                        if *current_focus == 2 {
                            // 在音乐列表按空格控制播放/暂停
                            self.toggle_playback();
                            true
                        } else {
                            // 切换当前选中的设置选项
                            if *current_focus == 0 {
                                *play_during_pomodoro = !*play_during_pomodoro;
                            } else if *current_focus == 1 {
                                *play_on_finish = !*play_on_finish;
                            }
                            true
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if *current_focus == 2 {
                            // 音乐列表向上移动
                            self.music_list_previous();
                        } else if *current_focus > 0 {
                            *current_focus -= 1;
                        }
                        true
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if *current_focus == 2 {
                            // 音乐列表向下移动
                            self.music_list_next();
                        } else if *current_focus < 2 {
                            *current_focus += 1;
                        }
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    // 新增：音乐列表向上移动
    fn music_list_previous(&mut self) {
        let i = match self.music_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.music_files.is_empty() {
                        0
                    } else {
                        self.music_files.len() - 1
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.music_list_state.select(Some(i));
        self.music_scroll_state = self.music_scroll_state.position(i); // 更新滚动条
    }

    // 新增：音乐列表向下移动
    fn music_list_next(&mut self) {
        if self.music_files.is_empty() {
            return;
        }
        let i = match self.music_list_state.selected() {
            Some(i) => {
                if i >= self.music_files.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.music_list_state.select(Some(i));
        self.music_scroll_state = self.music_scroll_state.position(i); // 更新滚动条
    }

    // 播放选中的音乐
    pub fn play_selected_music(&mut self) {
        if let Some(selected) = self.music_list_state.selected()
            && selected < self.music_files.len()
        {
            let file_path = self.music_files[selected].path.clone();

            // 停止当前播放
            self.stop_music();

            // 更新状态
            self.music_player_state.current_playing_index = Some(selected);
            self.music_player_state.playback_state = PlaybackState::Playing;

            // 在新线程中播放音乐
            let path = file_path.clone();
            let volume = self.music_player_state.volume;

            // 新版 rodio API：使用 OutputStreamBuilder
            match rodio::OutputStreamBuilder::open_default_stream() {
                Ok(stream_handle) => {
                    // 创建 Sink，新版使用 connect_new 并传入 mixer
                    let sink = rodio::Sink::connect_new(stream_handle.mixer());
                    sink.set_volume(volume);

                    match File::open(&path) {
                        Ok(file) => {
                            match Decoder::new(BufReader::new(file)) {
                                Ok(source) => {
                                    sink.append(source);

                                    // 保存 sink 和 stream_handle 以便控制
                                    self.sink = Some(Arc::new(Mutex::new(sink)));
                                    self.stream_handle = Some(stream_handle);
                                }
                                Err(e) => {
                                    eprintln!("无法解码音频文件: {e}");
                                    self.music_player_state.playback_state = PlaybackState::Stopped;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("无法打开音频文件: {e}");
                            self.music_player_state.playback_state = PlaybackState::Stopped;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("无法打开音频输出: {e}");
                    self.music_player_state.playback_state = PlaybackState::Stopped;
                }
            }
        }
    }

    // 新增：切换播放/暂停
    pub fn toggle_playback(&mut self) {
        if let Some(sink_arc) = &self.sink {
            if let Ok(sink) = sink_arc.lock() {
                if sink.is_paused() {
                    sink.play();
                    self.music_player_state.playback_state = PlaybackState::Playing;
                } else {
                    sink.pause();
                    self.music_player_state.playback_state = PlaybackState::Paused;
                }
            }
        } else if self.music_player_state.current_playing_index.is_some() {
            // 如果有之前播放的索引但没有 sink，重新播放
            self.play_selected_music();
        }
    }

    // 新增：停止音乐
    pub fn stop_music(&mut self) {
        if let Some(sink) = self.sink.as_ref().and_then(|s| s.lock().ok()) {
            sink.stop();
        }

        // 重置状态.
        self.sink = None;
        self.stream_handle = None; // 释放 stream_handle
        self.music_player_state.playback_state = PlaybackState::Stopped;
    }

    /// 创建新任务
    fn create_task(&mut self, title: String, description: String) {
        // 使用 new() 构造函数创建任务
        let task = TodoTask::new(title, description);

        self.tasks.push(task);

        // 更新滚动条
        self.scroll_state = ScrollbarState::new(self.tasks.len());
    }

    /// 保存番茄钟设置
    // TODO:
    fn save_pomodoro_settings(&mut self, duration_index: usize, custom_duration: String) {
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
                cursor_position: 0, // 新增
            },
            WindowType::PomodoroSettings => WindowData::PomodoroSettings {
                selected_duration: 2, // 默认25分钟
                custom_duration: String::new(),
                current_focus: 0,
            },
            WindowType::Settings => WindowData::Settings {
                // 从配置文件或默认值加载设置
                play_during_pomodoro: false, // 可以从配置读取
                play_on_finish: false,       // 可以从配置读取
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

    /// 根据窗口类型获取动态布局
    fn get_window_layout(&self, window_type: &WindowType) -> WindowLayout {
        // 尝试获取当前终端大小
        let term_size = match terminal_size::terminal_size() {
            Some((w, h)) => (w.0, h.0),
            None => (120, 30), // 默认值
        };

        let (term_width, term_height) = term_size;

        // 可以根据窗口类型微调
        match window_type {
            WindowType::CreateTask => {
                // 创建任务窗口可以稍微窄一点
                let width = (term_width as f32 * 0.7) as u16;
                let height = (term_height as f32 * 0.8) as u16;
                let x = (term_width - width) / 2;
                let y = (term_height - height) / 3; // 偏上一点
                WindowLayout {
                    x,
                    y,
                    width,
                    height,
                }
            }

            WindowType::PomodoroSettings => {
                // 番茄钟设置窗口需要更宽
                let width = (term_width as f32 * 0.8) as u16;
                let height = (term_height as f32 * 0.85) as u16;
                let x = (term_width - width) / 2;
                let y = (term_height - height) / 2;
                WindowLayout {
                    x,
                    y,
                    width,
                    height,
                }
            }

            WindowType::Settings => {
                // 设置窗口
                let width = (term_width as f32 * 0.75) as u16;
                let height = (term_height as f32 * 0.9) as u16;
                let x = (term_width - width) / 2;
                let y = (term_height - height) / 2;
                WindowLayout {
                    x,
                    y,
                    width,
                    height,
                }
            }

            _ => {
                // 默认
                let width = (term_width as f32 * 0.7) as u16;
                let height = (term_height as f32 * 0.7) as u16;
                let x = (term_width - width) / 2;
                let y = (term_height - height) / 2;
                WindowLayout {
                    x,
                    y,
                    width,
                    height,
                }
            }
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
                && path.extension().is_some_and(|ext| {
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

        files.sort_by(|a, b| a.name.cmp(&b.name));
        self.music_files = files;

        // 更新音乐列表滚动条状态
        self.music_scroll_state = ScrollbarState::new(self.music_files.len());

        // 如果有文件，默认选中第一个
        if !self.music_files.is_empty() {
            self.music_list_state.select(Some(0));
            self.music_scroll_state = self.music_scroll_state.position(0);
        }
    }
}
