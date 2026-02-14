pub struct AsciiArt;

impl AsciiArt {
    /// 生成带有固定前导空格的 EWXDY ASCII Art（已预居中）
    /// 注意：当前未被使用，保留作为固定宽度的备选方案
    pub fn header() -> Vec<&'static str> {
        vec![
            " ",
            "                 ██████╗ ██╗    ██╗██╗  ██╗██████╗ ██╗   ██╗",
            "                 ██╔══██╗██║    ██║╚██╗██╔╝██╔══██╗╚██╗ ██╔╝",
            "                 ██║  ██║██║ █╗ ██║ ╚███╔╝ ██║  ██║ ╚████╔╝ ",
            "                 ██║  ██║██║███╗██║ ██╔██╗ ██║  ██║  ╚██╔╝  ",
            "                 ██████╔╝╚███╔███╔╝██╔╝ ██╗██████╔╝   ██║   ",
            "                 ╚═════╝  ╚══╝╚══╝ ╚═╝  ╚═╝╚═════╝    ╚═╝   ",
            " ",
        ]
    }

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

    /// 生成带版本号的完整标题（使用固定宽度居中）
    /// 注意：当前未被使用，保留作为静态版本的备选
    pub fn full_header(version: &str) -> Vec<String> {
        let mut header: Vec<String> = Self::header().iter().map(|&s| s.to_string()).collect();

        // 计算版本号需要的填充
        let max_line_length = header.iter().map(|line| line.len()).max().unwrap_or(60);

        let version_str = format!("版本 {}", version);
        let padding = if version_str.len() < max_line_length {
            (max_line_length - version_str.len()) / 2
        } else {
            0
        };

        header.push(format!("{:padding$}{}", "", version_str, padding = padding));
        header.push(" ".to_string());
        header
    }
}
