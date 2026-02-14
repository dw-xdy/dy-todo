mod app;
mod models;
mod ui;
mod dashboard;

use app::App;
use std::io;

fn main() -> io::Result<()> {
    // 可以在这里初始化 color_eyre 等错误处理库
    ratatui::run(|terminal| App::default().run(terminal))
}
