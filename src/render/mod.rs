use ratatui::{prelude::*, style::Modifier};

pub fn render_buffer_to_ansi(buffer: &Buffer, width: u16, height: u16) -> String {
    use std::fmt::Write;

    let mut output = String::new();
    output.push_str("\x1b[2J\x1b[H");

    let mut last_fg = Color::Reset;
    let mut last_bg = Color::Reset;
    let mut last_modifier = Modifier::empty();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx >= buffer.content.len() {
                continue;
            }
            let cell = &buffer.content[idx];

            write!(output, "\x1b[{};{}H", y + 1, x + 1).unwrap();

            if last_fg != cell.fg {
                output.push_str(color_to_ansi_fg(cell.fg));
                last_fg = cell.fg;
            }
            if last_bg != cell.bg {
                output.push_str(color_to_ansi_bg(cell.bg));
                last_bg = cell.bg;
            }
            if last_modifier != cell.modifier {
                output.push_str(&modifier_to_ansi(cell.modifier));
                last_modifier = cell.modifier;
            }

            output.push_str(cell.symbol());
        }
    }

    output.push_str("\x1b[0m");
    output
}

fn color_to_ansi_fg(color: Color) -> &'static str {
    match color {
        Color::Black => "\x1b[30m",
        Color::Red => "\x1b[31m",
        Color::Green => "\x1b[32m",
        Color::Yellow => "\x1b[33m",
        Color::Blue => "\x1b[34m",
        Color::Magenta => "\x1b[35m",
        Color::Cyan => "\x1b[36m",
        Color::White => "\x1b[37m",
        Color::Gray => "\x1b[90m",
        Color::LightRed => "\x1b[91m",
        Color::LightGreen => "\x1b[92m",
        Color::LightYellow => "\x1b[93m",
        Color::LightBlue => "\x1b[94m",
        Color::LightMagenta => "\x1b[95m",
        Color::LightCyan => "\x1b[96m",
        _ => "\x1b[0m",
    }
}

fn color_to_ansi_bg(color: Color) -> &'static str {
    match color {
        Color::Black => "\x1b[40m",
        Color::Red => "\x1b[41m",
        Color::Green => "\x1b[42m",
        Color::Yellow => "\x1b[43m",
        Color::Blue => "\x1b[44m",
        Color::Magenta => "\x1b[45m",
        Color::Cyan => "\x1b[46m",
        Color::White => "\x1b[47m",
        _ => "\x1b[0m",
    }
}

fn modifier_to_ansi(modifier: Modifier) -> String {
    let mut codes = Vec::new();
    if modifier.contains(Modifier::BOLD) {
        codes.push("1");
    }
    if modifier.contains(Modifier::DIM) {
        codes.push("2");
    }
    if modifier.contains(Modifier::ITALIC) {
        codes.push("3");
    }
    if modifier.contains(Modifier::UNDERLINED) {
        codes.push("4");
    }

    if codes.is_empty() {
        "\x1b[0m".to_string()
    } else {
        format!("\x1b[{}m", codes.join(";"))
    }
}
