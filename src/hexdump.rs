use chrono::Local;
use colored::*;

pub struct HexdumpConfig {
    pub prefix: String,
    pub color: Option<Color>,
    pub show_prefix: bool,
    pub timestamp: bool,
    pub show_end_offset: bool,
}

#[derive(Clone, Copy)]
pub enum Color {
    Blue,
    Green,
    Red,
    Yellow,
    Cyan,
    Magenta,
    White,
}

impl Color {
    pub fn apply<'a>(&self, input: &'a str) -> colored::ColoredString {
        match self {
            Color::Blue => input.blue().bold(),
            Color::Green => input.green().bold(),
            Color::Red => input.red().bold(),
            Color::Yellow => input.yellow().bold(),
            Color::Cyan => input.cyan().bold(),
            Color::Magenta => input.magenta().bold(),
            Color::White => input.white().bold(),
        }
    }
}

pub fn format_hex_dump(data: &[u8], offset_start: usize, config: &HexdumpConfig) -> String {
    let mut result = String::new();
    let mut offset = 0;

    while offset < data.len() {
        let chunk_size = std::cmp::min(16, data.len() - offset);
        let chunk = &data[offset..offset + chunk_size];
        let full_offset = offset_start + offset;

        // Timestamp
        if config.timestamp {
            let now = Local::now();
            result.push_str(&format!("[{}] ", now.format("%H:%M:%S%.3f")));
        }

        // Colored prefix
        if config.show_prefix && !config.prefix.is_empty() {
            if let Some(color) = config.color {
                result.push_str(&format!("{}", color.apply(&config.prefix)));
            } else {
                result.push_str(&config.prefix);
            }
        }

        // Offset at start
        result.push_str(&format!("{:04x}  ", full_offset));

        // Hex bytes
        for i in 0..16 {
            if i < chunk_size {
                result.push_str(&format!("{:02x} ", chunk[i]));
            } else {
                result.push_str("   ");
            }

            // Single extra gap after 8th byte IF chunk has more than 8 bytes
            if i == 7 {
                result.push(' ');
            }
        }

        // ASCII
        result.push_str(" |");
        for &byte in chunk {
            if byte >= 32 && byte <= 126 {
                result.push(byte as char);
            } else {
                result.push('.');
            }
        }
        // Fill ASCII print to always 16
        for _ in chunk_size..16 {
            result.push(' ');
        }
        result.push('|');

        // End offset, always shown
        if config.show_end_offset {
            result.push_str(&format!("  [{:04x}]", full_offset + chunk_size));
        }

        result.push('\n');
        offset += chunk_size;
    }

    result
}
