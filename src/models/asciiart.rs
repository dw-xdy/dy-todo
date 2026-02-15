pub struct AsciiArt;

impl AsciiArt {
    /// 动态居中（根据终端宽度）
    pub fn centered_header(version: &str, terminal_width: u16) -> Vec<String> {
        let raw_lines = vec![
            "██████╗ ██╗    ██╗██╗  ██╗██████╗ ██╗   ██╗",
            "██╔══██╗██║    ██║╚██╗██╔╝██╔══██╗╚██╗ ██╔╝",
            "██║  ██║██║ █╗ ██║ ╚███╔╝ ██║  ██║ ╚████╔╝ ",
            "██║  ██║██║███╗██║ ██╔██╗ ██║  ██║  ╚██╔╝  ",
            "██████╔╝╚███╔███╔╝██╔╝ ██╗██████╔╝   ██║   ",
            "╚═════╝  ╚══╝╚══╝ ╚═╝  ╚═╝╚═════╝    ╚═╝   ",
        ];

        let max_line_length = raw_lines.iter().map(|line| line.len()).max().unwrap_or(60);
        let padding = if terminal_width as usize > max_line_length {
            (terminal_width as usize - max_line_length) / 2
        } else {
            0
        };

        let mut centered = Vec::new();
        centered.push(" ".to_string());

        for line in raw_lines {
            centered.push(format!("{:padding$}{}", "", line, padding = padding));
        }

        centered.push(" ".to_string());
        centered.push(format!("{:padding$}{}", "", version, padding = padding));
        centered.push(" ".to_string());

        centered
    }
}
