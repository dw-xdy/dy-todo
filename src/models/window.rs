#[derive(Debug, Clone, PartialEq)]
pub enum WindowType {
    CreateTask,
    PomodoroSettings,
    TaskDetail,
}

#[derive(Debug, Clone)]
pub struct WindowLayout {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone)]
pub struct ActiveWindow {
    pub window_type: WindowType,
    pub layout: WindowLayout,
    pub data: WindowData,
    pub is_visible: bool,
}

#[derive(Debug, Clone)]
pub enum WindowData {
    CreateTask {
        title: String,
        description: String,
        current_field: usize,
        cursor_position: usize,
    },
    PomodoroSettings {
        // play_during_pomodoro: bool,
        // play_on_finish: bool,
        selected_duration: usize,
        custom_duration: String,
        current_focus: usize,
    },
    Search {
        query: String,
    },
    Empty,
}
