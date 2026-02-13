use crate::models::{
    // 标签相关
    // Tag,

    // 窗口相关
    ActiveWindow,
    // 音乐相关
    AudioFileInfo,
    MusicPlayerState,
    PlaybackState,
    // 任务相关
    TodoTask,
    WindowData,
    WindowLayout,
    WindowType,
};
use crate::ui; // 引入 UI 渲染
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
// 滚动条
use ratatui::widgets::ScrollbarState; // 确保导入
use ratatui::{DefaultTerminal, widgets::ListState};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir; // 确保 Cargo.toml 有 walkdir 依赖

pub struct App {
    pub exit: bool,
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
        // 1. 暂时取走窗口
        if let Some(mut window) = self.active_window.take() {
            // 2. 处理事件
            let handled = self.handle_window_key_event(&mut window, key);

            // 3. 根据窗口类型和按键决定是否关闭
            let should_close = match key.code {
                KeyCode::Esc => true, // Esc 总是取消并关闭
                KeyCode::Enter => {
                    // 只有非番茄钟设置窗口的 Enter 才关闭
                    !matches!(window.window_type, WindowType::PomodoroSettings)
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
                KeyCode::Char(' ') => {
                    let text = if *current_field == 0 {
                        title
                    } else {
                        description
                    };
                    text.insert(*cursor_position, ' ');
                    *cursor_position += 1;
                    true
                }
                _ => false,
            },

            WindowData::PomodoroSettings {
                play_during_pomodoro,
                play_on_finish,
                selected_duration,
                custom_duration,
                current_focus,
            } => {
                match key.code {
                    KeyCode::Tab => {
                        *current_focus = (*current_focus + 1) % 5;
                        true
                    }
                    KeyCode::Enter => {
                        if *current_focus == 4 {
                            // 在音乐列表按Enter播放选中的音乐
                            self.play_selected_music();
                            true
                        } else {
                            // 保存设置
                            self.save_pomodoro_settings(
                                *play_during_pomodoro,
                                *play_on_finish,
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
                        if *current_focus == 4 {
                            // 在音乐列表按空格控制播放/暂停
                            self.toggle_playback();
                            true
                        } else {
                            match *current_focus {
                                0 => *play_during_pomodoro = !*play_during_pomodoro,
                                1 => *play_on_finish = !*play_on_finish,
                                _ => {}
                            }
                            true
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if *current_focus == 2 {
                            // 常用时间选择
                            if *selected_duration > 0 {
                                *selected_duration -= 1;
                            }
                        } else if *current_focus == 4 {
                            // 音乐列表向上移动
                            self.music_list_previous();
                        }
                        true
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if *current_focus == 2 {
                            // 常用时间选择
                            if *selected_duration < 4 {
                                *selected_duration += 1;
                            }
                        } else if *current_focus == 4 {
                            // 音乐列表向下移动
                            self.music_list_next();
                        }
                        true
                    }
                    KeyCode::Char(c) => {
                        if *current_focus == 3 && c.is_ascii_digit() {
                            custom_duration.push(c);
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

    // 新增：播放选中的音乐
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
    // TODO:
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
                cursor_position: 0, // 新增
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
