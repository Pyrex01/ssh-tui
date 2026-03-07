use std::collections::VecDeque;

use crossterm::event::{self, KeyCode, KeyModifiers};

// Parse terminal size response from ANSI escape sequence
// Format: \x1b[8;{height};{width}t or \x1b[18t response
pub fn parse_terminal_size_response(data: &[u8]) -> Option<(u16, u16)> {
    // Look for ANSI escape sequence: \x1b[8;{rows};{cols}t
    if data.len() < 8 {
        return None;
    }
    
    // Check for \x1b[8; pattern
    if data[0] == 0x1b && data[1] == b'[' && data[2] == b'8' && data[3] == b';' {
        let mut i = 4;
        let mut rows = 0u32;
        let mut cols = 0u32;
        
        // Parse rows
        while i < data.len() && data[i] >= b'0' && data[i] <= b'9' {
            rows = rows * 10 + (data[i] - b'0') as u32;
            i += 1;
        }
        
        // Check for semicolon
        if i >= data.len() || data[i] != b';' {
            return None;
        }
        i += 1;
        
        // Parse cols
        while i < data.len() && data[i] >= b'0' && data[i] <= b'9' {
            cols = cols * 10 + (data[i] - b'0') as u32;
            i += 1;
        }
        
        // Check for 't' terminator
        if i < data.len() && data[i] == b't' {
            return Some((cols as u16, rows as u16));
        }
    }
    
    None
}

// Parse SSH input from a buffer, extracting complete key events
pub fn parse_ssh_input_from_buffer(buffer: &mut VecDeque<u8>) -> Option<event::KeyEvent> {
    if buffer.is_empty() {
        return None;
    }
    
    // Check for escape sequences first
    if buffer[0] == 0x1b {
        // Escape sequence
        if buffer.len() < 2 {
            return None; // Need more data
        }
        
        match buffer[1] {
            b'[' => {
                // CSI sequence - need at least 3 bytes
                if buffer.len() < 3 {
                    return None;
                }
                
                // Check for arrow keys and other common sequences
                match buffer[2] {
                    b'A' => {
                        buffer.drain(0..3);
                        return Some(event::KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
                    }
                    b'B' => {
                        buffer.drain(0..3);
                        return Some(event::KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
                    }
                    b'C' => {
                        buffer.drain(0..3);
                        return Some(event::KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
                    }
                    b'D' => {
                        buffer.drain(0..3);
                        return Some(event::KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
                    }
                    _ => {
                        // Unknown CSI sequence - consume it up to the terminating character
                        for i in 3..buffer.len() {
                            if (buffer[i] >= b'@' && buffer[i] <= b'~') || buffer[i] == 0x1b {
                                buffer.drain(0..=i);
                                return None; // Unknown sequence, ignore it
                            }
                        }
                        return None; // Need more data
                    }
                }
            }
            b'O' => {
                // SS3 sequence (function keys) - consume it
                if buffer.len() >= 3 {
                    buffer.drain(0..3);
                }
                return None;
            }
            _ => {
                // Unknown escape sequence - consume the escape
                buffer.pop_front();
                return None;
            }
        }
    }
    
    // Check for control characters
    match buffer[0] {
        b'\r' | b'\n' => {
            buffer.pop_front();
            return Some(event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        }
        0x7f | 0x08 => {
            buffer.pop_front();
            return Some(event::KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        }
        0x03 => {
            // Ctrl+C
            buffer.pop_front();
            return Some(event::KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        }
        0x04 => {
            // Ctrl+D
            buffer.pop_front();
            return Some(event::KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL));
        }
        _ => {
            // Try to parse as UTF-8 character
            // We need to check if we have a complete UTF-8 sequence
            let mut utf8_bytes = Vec::new();
            let mut i = 0;
            
            while i < buffer.len() && i < 4 {
                utf8_bytes.push(buffer[i]);
                if let Ok(s) = std::str::from_utf8(&utf8_bytes) {
                    if let Some(ch) = s.chars().next() {
                        // Valid UTF-8 character
                        buffer.drain(0..=i);
                        // Skip control characters (except printable ones)
                        if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
                            return None;
                        }
                        return Some(event::KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
                    }
                }
                i += 1;
            }
            
            // If we can't parse it yet, we might need more data
            // But if it's clearly not valid UTF-8, consume it
            if buffer[0] < 0x80 {
                // ASCII character
                let ch = buffer.pop_front().unwrap() as char;
                if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
                    return None;
                }
                return Some(event::KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
            }
            
            // Need more data for multi-byte UTF-8
            return None;
        }
    }
}
