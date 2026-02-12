use ratatui::style::Color;
use ratatui::widgets::ListState; // 必须导入这个
use std::collections::HashSet;

pub struct TokyoNight;
impl TokyoNight {
    pub const CYAN: Color = Color::Rgb(125, 207, 255);
    pub const MAGENTA: Color = Color::Rgb(187, 154, 247);
    pub const ORANGE: Color = Color::Rgb(255, 158, 100);
    pub const RED: Color = Color::Rgb(247, 118, 142);
    pub const GRAY: Color = Color::Rgb(86, 95, 137);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    Work,
    Personal,
    Urgent,
    Custom(String),
}

pub struct TodoTask {
    pub title: String,
    pub description: String,
    pub is_completed: bool,
    pub tags: HashSet<Tag>,
}

/// 窗口类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum WindowType {
    /// 创建新任务窗口
    CreateTask,
    /// 番茄钟设置窗口
    PomodoroSettings,
    /// 任务详情窗口
    TaskDetail,
}

/// 窗口位置和大小
#[derive(Debug, Clone)]
pub struct WindowLayout {
    pub x: u16,      // 左上角 x 坐标
    pub y: u16,      // 左上角 y 坐标
    pub width: u16,  // 窗口宽度
    pub height: u16, // 窗口高度
}

/// 当前活动窗口
#[derive(Debug, Clone)]
pub struct ActiveWindow {
    pub window_type: WindowType,
    pub layout: WindowLayout,
    pub data: WindowData,
    pub is_visible: bool,
}

// 音乐播放定义的结构
pub struct AudioFileInfo {
    pub name: String,
    pub path: std::path::PathBuf,
}

/// 窗口数据（根据不同类型存储不同数据）
#[derive(Debug, Clone)]
pub enum WindowData {
    CreateTask {
        title: String,
        description: String,
        current_field: usize, // 0: title, 1: description
    },
    PomodoroSettings {
        play_during_pomodoro: bool,
        play_on_finish: bool,
        selected_duration: usize, // 0: 15, 1: 20, 2: 25, 3: 30, 4: 45
        custom_duration: String,
        current_focus: usize, // 0-4 对应不同焦点区域
    },
    Search {
        query: String,
    },
    // 其他窗口类型的数据...
    Empty,
}
