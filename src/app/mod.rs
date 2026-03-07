use crossterm::event::KeyCode;

pub const RESIZE_SIGNAL: u8 = 0xFF;

#[derive(Debug, Clone)]
pub struct AppState {
    pub should_quit: bool,
    pub scroll_offset: u16,
    pub needs_redraw: bool,
    pub last_terminal_size: (u16, u16),
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            should_quit: false,
            scroll_offset: 0,
            needs_redraw: true,
            last_terminal_size: (0, 0),
        }
    }
}

impl AppState {
    pub fn apply_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                    self.needs_redraw = true;
                }
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
                self.needs_redraw = true;
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                self.needs_redraw = true;
            }
            KeyCode::PageDown => {
                self.scroll_offset = self.scroll_offset.saturating_add(10);
                self.needs_redraw = true;
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.scroll_offset = 0;
                self.needs_redraw = true;
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.scroll_offset = u16::MAX;
                self.needs_redraw = true;
            }
            _ => {}
        }
    }

    pub fn clamp_scroll(&mut self, total_lines: usize, viewport_height: u16) {
        let max_scroll = total_lines.saturating_sub(viewport_height as usize) as u16;
        self.scroll_offset = self.scroll_offset.min(max_scroll);
    }

    pub fn handle_resize(&mut self, size: (u16, u16)) {
        if size != self.last_terminal_size {
            self.last_terminal_size = size;
            self.needs_redraw = true;
        }
    }
}
