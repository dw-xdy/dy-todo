use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{Write, stdout};
use std::time::{Duration, Instant};

// ==================== ASCII 数字定义（保持不变）====================
const DIGITS: [&[&str; 5]; 10] = [
    // 0
    &[
        "██████",
        "██  ██",
        "██  ██",
        "██  ██",
        "██████",
    ],
    // 1
    &[
        "  ██  ",
        "████  ",
        "  ██  ",
        "  ██  ",
        "██████",
    ],
    // 2
    &[
        "██████",
        "    ██",
        "██████",
        "██    ",
        "██████",
    ],
    // 3
    &[
        "██████",
        "    ██",
        "██████",
        "    ██",
        "██████",
    ],
    // 4
    &[
        "██  ██",
        "██  ██",
        "██████",
        "    ██",
        "    ██",
    ],
    // 5
    &[
        "██████",
        "██    ",
        "██████",
        "    ██",
        "██████",
    ],
    // 6
    &[
        "██████",
        "██    ",
        "██████",
        "██  ██",
        "██████",
    ],
    // 7
    &[
        "██████",
        "    ██",
        "    ██",
        "    ██",
        "    ██",
    ],
    // 8
    &[
        "██████",
        "██  ██",
        "██████",
        "██  ██",
        "██████",
    ],
    // 9
    &[
        "██████",
        "██  ██",
        "██████",
        "    ██",
        "██████",
    ],
];

// 数字中间的冒号 :
const COLON: &[&str; 5] = &[
    "    ",
    " ██ ",
    "    ",
    " ██ ",
    "    ",
];

// ==================== 辅助函数（保持不变）====================
fn render_time(time_str: &str) -> Vec<String> {
    let mut lines = vec![String::new(); 5];
    for ch in time_str.chars() {
        let digit_lines = if ch == ':' {
            COLON
        } else if let Some(d) = ch.to_digit(10) {
            DIGITS[d as usize]
        } else {
            continue;
        };
        for (i, line) in digit_lines.iter().enumerate() {
            lines[i].push_str(line);
            if ch != ':' {
                lines[i].push(' ');
            }
        }
    }
    lines
}

fn format_time(d: Duration) -> String {
    let total_secs = d.as_secs();
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}", mins, secs)
}

// ==================== 状态定义 ====================
enum TimerState {
    Running(Instant),
    Paused(Duration),
    Stopped,
}

// 更新剩余时间，并自动处理倒计时结束
fn update_timer(state: &mut TimerState, total: Duration) -> Duration {
    match state {
        TimerState::Running(start) => {
            let elapsed = start.elapsed();
            if elapsed >= total {
                *state = TimerState::Stopped;
                total
            } else {
                total - elapsed
            }
        }
        TimerState::Paused(elapsed) => {
            if *elapsed >= total {
                *state = TimerState::Stopped;
                total
            } else {
                total - *elapsed
            }
        }
        TimerState::Stopped => total,
    }
}

// ==================== 主程序 ====================
fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    let mut timer_state = TimerState::Stopped;
    let total_duration = Duration::from_secs(25 * 60); // 25分钟

    // 记录上一次绘制的剩余秒数，用于判断是否需要重绘
    let mut last_remaining_secs = total_duration.as_secs();

    loop {
        // 更新剩余时间
        let remaining = update_timer(&mut timer_state, total_duration);
        let remaining_secs = remaining.as_secs();

        // 只有当剩余秒数发生变化，或者状态切换时，才重绘画面
        if remaining_secs != last_remaining_secs {
            let time_str = format_time(remaining);
            let big_time = render_time(&time_str);

            // 覆盖绘制（不清屏）
            execute!(stdout, cursor::MoveTo(0, 0))?;
            for line in big_time {
                print!("{}\x1b[K\r\n", line);
            }
            // 显示状态和帮助
            let state_icon = match timer_state {
                TimerState::Running(_) => "▶️ 运行中",
                TimerState::Paused(_) => "⏸️ 暂停",
                TimerState::Stopped => "⏹️ 停止",
            };
            println!("\x1b[K{}", state_icon);
            println!("\x1b[K[Space] 开始/暂停  [r] 重置  [q] 退出");
            print!("\x1b[J"); // 清除可能的多余行
            stdout.flush()?;

            last_remaining_secs = remaining_secs;
        }

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == event::KeyEventKind::Press
        {
            // 处理按键
            match key.code {
                KeyCode::Char(' ') => {
                    timer_state = match timer_state {
                        TimerState::Stopped => TimerState::Running(Instant::now()),
                        TimerState::Running(start) => TimerState::Paused(start.elapsed()),
                        TimerState::Paused(elapsed) => {
                            TimerState::Running(Instant::now() - elapsed)
                        }
                    };
                    last_remaining_secs = u64::MAX; // 强制重绘
                }
                KeyCode::Char('r') => {
                    timer_state = TimerState::Stopped;
                    last_remaining_secs = u64::MAX;
                }
                KeyCode::Char('q') => break,
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}
