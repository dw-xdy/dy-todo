use ratatui::style::{Color, Style};

/// 应用程序的不同模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,     // 普通模式 - 导航和快捷键
    Insert,     // 插入模式 - 输入文本
    Visual,     // 可视模式 - 选择多个项目
    Command,    // 命令模式 - 输入命令
    Search,     // 搜索模式 - 搜索任务
}

impl AppMode {
    /// 获取模式的显示名称
    pub fn name(&self) -> &'static str {
        match self {
            AppMode::Normal => "NORMAL",
            AppMode::Insert => "INSERT",
            AppMode::Visual => "VISUAL",
            AppMode::Command => "COMMAND",
            AppMode::Search => "SEARCH",
        }
    }
    
    /// 获取模式的颜色
    pub fn color(&self) -> Color {
        match self {
            AppMode::Normal => Color::Rgb(86, 95, 137),      // TokyoNight 灰色
            AppMode::Insert => Color::Rgb(125, 207, 255),    // TokyoNight 青色
            AppMode::Visual => Color::Rgb(187, 154, 247),    // TokyoNight 紫色
            AppMode::Command => Color::Rgb(255, 158, 100),   // TokyoNight 橙色
            AppMode::Search => Color::Rgb(247, 118, 142),    // TokyoNight 红色
        }
    }
    
    /// 获取模式的背景色
    pub fn bg_color(&self) -> Color {
        match self {
            AppMode::Normal => Color::Rgb(30, 35, 50),
            AppMode::Insert => Color::Rgb(30, 40, 50),
            AppMode::Visual => Color::Rgb(40, 30, 50),
            AppMode::Command => Color::Rgb(50, 40, 30),
            AppMode::Search => Color::Rgb(50, 30, 30),
        }
    }
    
    /// 判断是否可以移动光标
    pub fn can_move_cursor(&self) -> bool {
        matches!(self, AppMode::Normal | AppMode::Visual)
    }
    
    /// 判断是否可以输入文本
    pub fn can_insert(&self) -> bool {
        matches!(self, AppMode::Insert | AppMode::Search | AppMode::Command)
    }
    
    /// 判断是否可以删除
    pub fn can_delete(&self) -> bool {
        matches!(self, AppMode::Insert | AppMode::Command | AppMode::Search)
    }
    
    /// 获取模式的键盘提示
    pub fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            AppMode::Normal => vec![
                ("j/k", "上下移动"),
                ("a", "新建任务"),
                ("p", "番茄钟"),
                ("o", "设置"),
                ("s", "搜索"),
                ("i", "插入模式"),
                ("v", "可视模式"),
                (":", "命令模式"),
                ("q", "退出"),
            ],
            AppMode::Insert => vec![
                ("Esc", "返回普通模式"),
                ("Enter", "确认"),
            ],
            AppMode::Visual => vec![
                ("j/k", "扩展选择"),
                ("x", "删除选中"),
                ("y", "复制选中"),
                ("Esc", "返回普通模式"),
            ],
            AppMode::Command => vec![
                ("Enter", "执行命令"),
                ("Esc", "取消"),
            ],
            AppMode::Search => vec![
                ("n", "下一个"),
                ("N", "上一个"),
                ("Enter", "确认"),
                ("Esc", "取消"),
            ],
        }
    }
}

/// 模式管理器
#[derive(Debug, Clone)]
pub struct ModeManager {
    current_mode: AppMode,
    previous_mode: AppMode,
    mode_stack: Vec<AppMode>,
}

impl Default for ModeManager {
    fn default() -> Self {
        Self {
            current_mode: AppMode::Normal,
            previous_mode: AppMode::Normal,
            mode_stack: Vec::new(),
        }
    }
}

impl ModeManager {
    /// 创建新的模式管理器
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 获取当前模式
    pub fn current(&self) -> AppMode {
        self.current_mode
    }
    
    /// 切换到新模式
    pub fn switch_to(&mut self, mode: AppMode) {
        self.previous_mode = self.current_mode;
        self.current_mode = mode;
    }
    
    /// 返回上一个模式
    pub fn back(&mut self) {
        std::mem::swap(&mut self.current_mode, &mut self.previous_mode);
    }
    
    /// 推入模式栈（进入子模式）
    pub fn push_mode(&mut self, mode: AppMode) {
        self.mode_stack.push(self.current_mode);
        self.current_mode = mode;
    }
    
    /// 弹出模式栈（返回父模式）
    pub fn pop_mode(&mut self) {
        if let Some(previous) = self.mode_stack.pop() {
            self.current_mode = previous;
        }
    }
    
    /// 判断是否在指定模式中
    pub fn is(&self, mode: AppMode) -> bool {
        self.current_mode == mode
    }
    
    /// 判断是否在多个模式中的任意一个
    pub fn is_any(&self, modes: &[AppMode]) -> bool {
        modes.contains(&self.current_mode)
    }
    
    /// 获取模式栏的渲染内容
    pub fn render_mode_line(&self) -> (String, Style) {
        let mode_name = self.current_mode.name();
        let style = Style::default()
            .fg(self.current_mode.color())
            .bg(self.current_mode.bg_color())
            .add_modifier(ratatui::style::Modifier::BOLD);
        
        (format!(" {} ", mode_name), style)
    }
    
    /// 获取提示信息
    pub fn get_hints(&self) -> Vec<String> {
        self.current_mode
            .key_hints()
            .iter()
            .map(|(key, desc)| format!("{}:{}", key, desc))
            .collect()
    }
}

/// 模式感知的键事件
#[derive(Debug, Clone)]
pub enum ModeAwareEvent {
    Key(crossterm::event::KeyEvent),
    ModeChange(AppMode),
    Command(String),
    Search(String),
}

/// 模式处理器 trait
pub trait ModeHandler {
    fn handle_mode_aware_event(&mut self, event: ModeAwareEvent) -> bool;
}
