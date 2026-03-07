use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::Paragraph,
    text::{Line, Span},
};
use russh::*;
use russh::keys::ssh_key::rand_core::OsRng;
use russh::server::{Msg, Server as _, Session};
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, mpsc};
use std::collections::VecDeque;

// SSH Server handler
#[derive(Clone)]
struct ServerHandler {
    connection_count: Arc<AtomicU32>,
    clients: Arc<Mutex<HashMap<usize, (ChannelId, russh::server::Handle, mpsc::UnboundedSender<Vec<u8>>, Arc<Mutex<(u16, u16)>>)>>>,
    id: usize,
}

impl server::Server for ServerHandler {
    type Handler = Self;
    
    fn new_client(&mut self, _: Option<SocketAddr>) -> Self {
        let mut s = self.clone();
        s.id = self.id;
        self.id += 1;
        self.connection_count.fetch_add(1, Ordering::Relaxed);
        s
    }
    
    fn handle_session_error(&mut self, error: <Self::Handler as russh::server::Handler>::Error) {
        eprintln!("Session error: {error:#?}");
        self.connection_count.fetch_sub(1, Ordering::Relaxed);
    }
}

impl server::Handler for ServerHandler {
    type Error = russh::Error;

    async fn auth_none(&mut self, _user: &str) -> Result<server::Auth, Self::Error> {
        Ok(server::Auth::Accept)
    }

    async fn auth_password(
        &mut self,
        _user: &str,
        _password: &str,
    ) -> Result<server::Auth, Self::Error> {
        // Accept any password for demo purposes
        Ok(server::Auth::Accept)
    }

    async fn auth_publickey(
        &mut self,
        _user: &str,
        _public_key: &russh::keys::PublicKey,
    ) -> Result<server::Auth, Self::Error> {
        // Accept any public key for demo purposes
        Ok(server::Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut Session,
    ) -> Result<bool, Self::Error> {
        let channel_id = channel.id();
        let handle = session.handle();
        
        // Create a channel for sending input to the TUI task
        let (input_tx, input_rx) = mpsc::unbounded_channel();
        
        // Shared terminal size (default to reasonable size, will be updated dynamically)
        // Use larger defaults to encourage full-screen usage
        let terminal_size = Arc::new(Mutex::new((120u16, 40u16)));
        
        {
            let mut clients = self.clients.lock().await;
            clients.insert(self.id, (channel_id, handle.clone(), input_tx, terminal_size.clone()));
        }
        
        // Request terminal size from client using ANSI escape sequence
        // This will cause the client to respond with terminal dimensions
        let size_request = "\x1b[18t"; // Request terminal size (DSR - Device Status Report)
        let _ = handle.data(channel_id, CryptoVec::from(size_request.as_bytes())).await;
        
        // Also try to get terminal size via environment variables if available
        // Some SSH clients send this information
        if let (Ok(cols_str), Ok(rows_str)) = (std::env::var("COLUMNS"), std::env::var("LINES")) {
            if let (Ok(cols), Ok(rows)) = (cols_str.parse::<u16>(), rows_str.parse::<u16>()) {
                let mut size = terminal_size.lock().await;
                *size = (cols, rows);
            }
        }
        
        // Spawn TUI task for this client
        let client_id = self.id;
        let connection_count = self.connection_count.clone();
        tokio::spawn(async move {
            if let Err(e) = run_ssh_tui(channel_id, handle, input_rx, connection_count, terminal_size).await {
                eprintln!("TUI error for client {}: {e:#?}", client_id);
            }
        });
        
        Ok(true)
    }

    async fn channel_open_direct_tcpip(
        &mut self,
        _channel: Channel<Msg>,
        _address: &str,
        _port: u32,
        _originator_address: &str,
        _originator_port: u32,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        Ok(false)
    }


    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Check if this is a terminal size response
        let clients = self.clients.lock().await;
        if let Some((_, _, input_tx, terminal_size)) = clients.values().find(|(id, _, _, _)| *id == channel) {
            // Try to parse terminal size from ANSI escape sequence response
            if let Some((width, height)) = parse_terminal_size_response(data) {
                let mut size = terminal_size.lock().await;
                *size = (width, height);
                // Trigger a redraw by sending a resize signal
                let _ = input_tx.send(vec![0xFF]); // Special marker for resize
            } else {
                // Forward input to the TUI task
                let _ = input_tx.send(data.to_vec());
            }
        }
        Ok(())
    }

    async fn channel_close(
        &mut self,
        channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Clean up client when channel closes
        let mut clients = self.clients.lock().await;
        clients.retain(|_, (id, _, _, _)| *id != channel);
        self.connection_count.fetch_sub(1, Ordering::Relaxed);
        Ok(())
    }
}

// Run TUI for an SSH client
async fn run_ssh_tui(
    channel_id: ChannelId,
    handle: russh::server::Handle,
    mut input_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    _connection_count: Arc<AtomicU32>,
    terminal_size: Arc<Mutex<(u16, u16)>>,
) -> Result<()> {
    let mut should_quit = false;
    let mut input_buffer = VecDeque::<u8>::new();
    let mut needs_redraw = true;
    let mut scroll_offset = 0u16;
    let mut last_terminal_size = (0u16, 0u16);
    let mut size_request_interval = tokio::time::interval(Duration::from_secs(2));
    size_request_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    
    loop {
        // Check for input first (non-blocking)
        while let Ok(data) = input_rx.try_recv() {
            // Check for resize signal (0xFF marker)
            if data.len() == 1 && data[0] == 0xFF {
                needs_redraw = true;
                continue;
            }
            input_buffer.extend(data);
            needs_redraw = true;
        }
        
        // Process buffered input to extract complete key events
        while let Some(key_event) = parse_ssh_input_from_buffer(&mut input_buffer) {
            match key_event.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    should_quit = true;
                    break;
                }
                KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                    if scroll_offset > 0 {
                        scroll_offset -= 1;
                        needs_redraw = true;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                    scroll_offset = scroll_offset.saturating_add(1);
                    needs_redraw = true;
                }
                KeyCode::PageUp => {
                    scroll_offset = scroll_offset.saturating_sub(10);
                    needs_redraw = true;
                }
                KeyCode::PageDown => {
                    scroll_offset = scroll_offset.saturating_add(10);
                    needs_redraw = true;
                }
                KeyCode::Home | KeyCode::Char('g') => {
                    scroll_offset = 0;
                    needs_redraw = true;
                }
                KeyCode::End | KeyCode::Char('G') => {
                    // Will be clamped in render based on total lines
                    scroll_offset = u16::MAX;
                    needs_redraw = true;
                }
                _ => {
                    // Handle other keys if needed
                }
            }
        }
        
        if should_quit {
            // Send goodbye message before quitting
            let goodbye = "\x1b[2J\x1b[H\x1b[36m[SYSTEM] Connection terminated. Goodbye!\x1b[0m\r\n";
            let _ = handle.data(channel_id, CryptoVec::from(goodbye.as_bytes())).await;
            // Give a small delay to ensure the message is sent
            tokio::time::sleep(Duration::from_millis(100)).await;
            // Close the channel properly
            let _ = handle.eof(channel_id).await;
            let _ = handle.close(channel_id).await;
            break;
        }
        
        // Only redraw if needed or periodically
        if needs_redraw {
            // Get current terminal size
            let (width, height) = *terminal_size.lock().await;
            
            // Ensure minimum size
            let width = width.max(40);
            let height = height.max(10);
            
            // Create a buffer to render to - use full terminal size
            let area = Rect::new(0, 0, width, height);
            
            // Calculate total lines and clamp scroll_offset before rendering
            let lines = build_resume_lines(area);
            let total_lines = lines.len();
            let max_scroll = total_lines.saturating_sub(height as usize).max(0);
            scroll_offset = scroll_offset.min(max_scroll as u16);
            
            let mut buffer = Buffer::empty(area);
            
            // Render the resume UI to full terminal area
            render_resume_ui(&mut buffer, area, scroll_offset);
            
            // Convert buffer to ANSI escape sequences and send to SSH channel
            let output = render_buffer_to_ansi(&buffer, width, height);
            if let Err(e) = handle.data(channel_id, CryptoVec::from(output.as_bytes())).await {
                eprintln!("Error sending data to channel: {e:#?}");
                break;
            }
            
            needs_redraw = false;
        }
        
        // Check if terminal size changed
        let current_size = *terminal_size.lock().await;
        if current_size != last_terminal_size {
            last_terminal_size = current_size;
            needs_redraw = true;
        }
        
        // Wait for input, timeout, or periodic size check
        let input_received = tokio::select! {
            input = input_rx.recv() => {
                if let Some(data) = input {
                    // Check for resize signal
                    if data.len() == 1 && data[0] == 0xFF {
                        needs_redraw = true;
                        false
                    } else {
                        input_buffer.extend(data);
                        true // Need redraw after input
                    }
                } else {
                    // Channel closed
                    break;
                }
            }
            _ = size_request_interval.tick() => {
                // Periodically request terminal size to detect resizes
                let _ = handle.data(channel_id, CryptoVec::from("\x1b[18t".as_bytes())).await;
                false // No input, but might need periodic update
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                false // No input, but might need periodic update
            }
        };
        
        if input_received {
            needs_redraw = true;
        }
    }
    
    Ok(())
}

// Render buffer to ANSI escape sequences
fn render_buffer_to_ansi(buffer: &Buffer, width: u16, height: u16) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    
    // Clear screen and move cursor to top-left
    output.push_str("\x1b[2J\x1b[H");
    
    let mut last_fg = Color::Reset;
    let mut last_bg = Color::Reset;
    let mut last_modifier = ratatui::style::Modifier::empty();
    
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx >= buffer.content.len() {
                continue;
            }
            let cell = &buffer.content[idx];
            
            // Set position
            write!(output, "\x1b[{};{}H", y + 1, x + 1).unwrap();
            
            // Set colors if changed
            if last_fg != cell.fg {
                output.push_str(&color_to_ansi_fg(cell.fg));
                last_fg = cell.fg;
            }
            
            if last_bg != cell.bg {
                output.push_str(&color_to_ansi_bg(cell.bg));
                last_bg = cell.bg;
            }
            
            if last_modifier != cell.modifier {
                output.push_str(&modifier_to_ansi(cell.modifier));
                last_modifier = cell.modifier;
            }
            
            // Write the character (use the string representation)
            let symbol = cell.symbol();
            output.push_str(symbol);
        }
    }
    
    // Reset colors
    output.push_str("\x1b[0m");
    
    output
}

fn color_to_ansi_fg(color: Color) -> String {
    match color {
        Color::Black => "\x1b[30m".to_string(),
        Color::Red => "\x1b[31m".to_string(),
        Color::Green => "\x1b[32m".to_string(),
        Color::Yellow => "\x1b[33m".to_string(),
        Color::Blue => "\x1b[34m".to_string(),
        Color::Magenta => "\x1b[35m".to_string(),
        Color::Cyan => "\x1b[36m".to_string(),
        Color::White => "\x1b[37m".to_string(),
        Color::Gray => "\x1b[90m".to_string(),
        Color::LightRed => "\x1b[91m".to_string(),
        Color::LightGreen => "\x1b[92m".to_string(),
        Color::LightYellow => "\x1b[93m".to_string(),
        Color::LightBlue => "\x1b[94m".to_string(),
        Color::LightMagenta => "\x1b[95m".to_string(),
        Color::LightCyan => "\x1b[96m".to_string(),
        Color::Reset => "\x1b[0m".to_string(),
        _ => "\x1b[0m".to_string(),
    }
}

fn color_to_ansi_bg(color: Color) -> String {
    match color {
        Color::Black => "\x1b[40m".to_string(),
        Color::Red => "\x1b[41m".to_string(),
        Color::Green => "\x1b[42m".to_string(),
        Color::Yellow => "\x1b[43m".to_string(),
        Color::Blue => "\x1b[44m".to_string(),
        Color::Magenta => "\x1b[45m".to_string(),
        Color::Cyan => "\x1b[46m".to_string(),
        Color::White => "\x1b[47m".to_string(),
        Color::Reset => "\x1b[0m".to_string(),
        _ => "\x1b[0m".to_string(),
    }
}

fn modifier_to_ansi(modifier: ratatui::style::Modifier) -> String {
    let mut codes = Vec::new();
    if modifier.contains(ratatui::style::Modifier::BOLD) {
        codes.push("1");
    }
    if modifier.contains(ratatui::style::Modifier::DIM) {
        codes.push("2");
    }
    if modifier.contains(ratatui::style::Modifier::ITALIC) {
        codes.push("3");
    }
    if modifier.contains(ratatui::style::Modifier::UNDERLINED) {
        codes.push("4");
    }
    if codes.is_empty() {
        "\x1b[0m".to_string()
    } else {
        format!("\x1b[{}m", codes.join(";"))
    }
}

// Helper function to create progress bar visual
fn create_progress_bar(filled: u8, total: u8, width: usize) -> String {
    let filled_chars = ((filled as f32 / total as f32) * width as f32) as usize;
    let empty_chars = width.saturating_sub(filled_chars);
    format!(
        "{}{}",
        "█".repeat(filled_chars.min(width)),
        "░".repeat(empty_chars)
    )
}

// Build all resume lines and return them (used for both counting and rendering)
fn build_resume_lines(area: Rect) -> Vec<Line<'static>> {
    // Enhanced btop-inspired color scheme with vibrant accents
    let accent_cyan = Color::LightCyan;
    let accent_green = Color::LightGreen;
    let accent_yellow = Color::LightYellow;
    let accent_magenta = Color::LightMagenta;
    let accent_blue = Color::LightBlue;
    let text_color = Color::White;
    let dim_color = Color::Gray;
    
    // Enhanced styles with better visual hierarchy
    let border_style = Style::default()
        .fg(accent_cyan)
        .add_modifier(ratatui::style::Modifier::BOLD);
    let title_style = Style::default()
        .fg(accent_cyan)
        .add_modifier(ratatui::style::Modifier::BOLD | ratatui::style::Modifier::UNDERLINED);
    let section_style = Style::default()
        .fg(accent_green)
        .add_modifier(ratatui::style::Modifier::BOLD);
    let highlight_style = Style::default()
        .fg(accent_yellow)
        .add_modifier(ratatui::style::Modifier::BOLD);
    let normal_style = Style::default().fg(text_color);
    let dim_style = Style::default().fg(dim_color);
    let project_style = Style::default()
        .fg(accent_magenta)
        .add_modifier(ratatui::style::Modifier::BOLD);
    let link_style = Style::default()
        .fg(accent_blue)
        .add_modifier(ratatui::style::Modifier::UNDERLINED);
    
    // Build all lines with proper styling
    let mut lines = Vec::new();
    
    // Calculate border width based on terminal width
    let border_width = area.width.saturating_sub(2);
    let border_line = "═".repeat(border_width as usize);
    let border_start = "╔";
    let border_end = "╗";
    let border_mid = "╠";
    let border_side = "║";
    let border_bottom_start = "╚";
    let border_bottom_end = "╝";
    
    // Helper macro to create styled lines
    macro_rules! add_line {
        ($($span:expr),*) => {
            lines.push(Line::from(vec![$($span),*]));
        };
    }
    
    macro_rules! styled_str {
        ($fmt:expr, $style:expr) => {
            Span::styled($fmt.to_string(), $style)
        };
    }
    
    // Enhanced header with visual effects
    let top_border = format!("{}{}{}", border_start, border_line, border_end);
    add_line!(Span::styled("", normal_style));
    add_line!(styled_str!(top_border, border_style));
    
    // Header with gradient-like effect
    let name_padding = (border_width as usize).saturating_sub(30) / 2;
    lines.push(Line::from(vec![
        Span::styled(border_side, border_style),
        Span::styled("  ", normal_style),
        Span::styled(" ".repeat(name_padding), normal_style),
        Span::styled("RIYAN KHAN", title_style),
        Span::styled("  ", normal_style),
        Span::styled("◆", accent_cyan),
        Span::styled("  ", normal_style),
        Span::styled("Software Engineer", highlight_style),
    ]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Navigation help text
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2050_17 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_17, normal_style),
        Span::styled("⌨  Navigation: ", dim_style),
        Span::styled("[↑/↓]", accent_cyan),
        Span::styled(" or ", dim_style),
        Span::styled("[j/k]", accent_cyan),
        Span::styled(" Scroll  ", dim_style),
        Span::styled("[Home/End]", accent_cyan),
        Span::styled(" or ", dim_style),
        Span::styled("[g/G]", accent_cyan),
        Span::styled(" Jump  ", dim_style),
        Span::styled("[Q]", accent_yellow),
        Span::styled(" Quit", dim_style),
    ]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Contact information with icons
    let _s3389 = format!("{}  📧 CONTACT INFORMATION", border_side);
    lines.push(Line::from(vec![Span::styled(_s3389, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    
    let _s7089 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s7089, normal_style),
        Span::styled("✉ ", accent_blue),
        Span::styled("Email: ", highlight_style),
        Span::styled("riyankhanpyrex01@gmail.com", link_style),
    ]));
    
    let _s872 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s872, normal_style),
        Span::styled("📱 ", accent_blue),
        Span::styled("Phone: ", highlight_style),
        Span::styled("+91-7304100368", normal_style),
    ]));
    
    let _s3327 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s3327, normal_style),
        Span::styled("💼 ", accent_blue),
        Span::styled("LinkedIn: ", highlight_style),
        Span::styled("riyan--khan", link_style),
    ]));
    
    let _s1384 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s1384, normal_style),
        Span::styled("🌐 ", accent_blue),
        Span::styled("Website: ", highlight_style),
        Span::styled("pyrex01.github.io/Pyrex01/", link_style),
    ]));
    
    let _s7200 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s7200, normal_style),
        Span::styled("🐙 ", accent_blue),
        Span::styled("GitHub: ", highlight_style),
        Span::styled("Pyrex01", link_style),
    ]));
    
    let _s2380 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2380, normal_style),
        Span::styled("📍 ", accent_blue),
        Span::styled("Location: ", highlight_style),
        Span::styled("Mumbai, India", normal_style),
    ]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Summary with icon
    let _s9799 = format!("{}  📋 SUMMARY", border_side);
    lines.push(Line::from(vec![Span::styled(_s9799, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s8815 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s8815, normal_style),
        Span::styled("Java and Node.js Developer with ", normal_style),
        Span::styled("3+ years", highlight_style),
        Span::styled(" of experience", normal_style),
    ]));
    let _s9348 = format!("{}  designing and deploying scalable microservices and", border_side);
    lines.push(Line::from(vec![Span::styled(_s9348, normal_style)]));
    let _s9509 = format!("{}  cloud-native applications. Proficient in ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s9509, normal_style),
        Span::styled("Java, Spring Boot, RESTful APIs", highlight_style),
        Span::styled(", and cloud platforms like AWS.", normal_style),
    ]));
    let _s9026 = format!("{}  Experienced in CI/CD, containerization, and agile", border_side);
    lines.push(Line::from(vec![Span::styled(_s9026, normal_style)]));
    let _s6881 = format!("{}  workflows. Strong in data structures, algorithms, and", border_side);
    lines.push(Line::from(vec![Span::styled(_s6881, normal_style)]));
    let _s8766 = format!("{}  system design. Passionate about building efficient and", border_side);
    lines.push(Line::from(vec![Span::styled(_s8766, normal_style)]));
    let _s5206 = format!("{}  reliable backend systems.", border_side);
    lines.push(Line::from(vec![Span::styled(_s5206, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Skills with progress bars (btop-style)
    let _s4406 = format!("{}  ⚡ SKILLS", border_side);
    lines.push(Line::from(vec![Span::styled(_s4406, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    
    // Languages with progress bars
    let _s2050 = format!("{}  ", border_side);
    let lang_bar_width = (border_width as usize).saturating_sub(25).min(30);
    lines.push(Line::from(vec![
        Span::styled(_s2050, normal_style),
        Span::styled("🔷 Languages: ", highlight_style),
        Span::styled("Java, Node.js, Rust, SQL, Bash", normal_style),
    ]));
    let _s_lang_bar = format!("{}  ", border_side);
    let lang_progress = create_progress_bar(9, 10, lang_bar_width);
    lines.push(Line::from(vec![
        Span::styled(_s_lang_bar, normal_style),
        Span::styled("    ", normal_style),
        Span::styled(lang_progress, Style::default().fg(accent_green)),
        Span::styled(" 95%", Style::default().fg(accent_green).add_modifier(ratatui::style::Modifier::BOLD)),
    ]));
    
    // Frameworks
    let _s2050_2 = format!("{}  ", border_side);
    let framework_bar_width = (border_width as usize).saturating_sub(25).min(30);
    lines.push(Line::from(vec![
        Span::styled(_s2050_2, normal_style),
        Span::styled("🔷 Frameworks: ", highlight_style),
        Span::styled("Spring Boot, Spring WebFlux, NestJS", normal_style),
    ]));
    let _s_framework_bar = format!("{}  ", border_side);
    let framework_progress = create_progress_bar(9, 10, framework_bar_width);
    lines.push(Line::from(vec![
        Span::styled(_s_framework_bar, normal_style),
        Span::styled("    ", normal_style),
        Span::styled(framework_progress, Style::default().fg(accent_cyan)),
        Span::styled(" 92%", Style::default().fg(accent_cyan).add_modifier(ratatui::style::Modifier::BOLD)),
    ]));
    
    // Databases
    let _s2050_3 = format!("{}  ", border_side);
    let db_bar_width = (border_width as usize).saturating_sub(25).min(30);
    lines.push(Line::from(vec![
        Span::styled(_s2050_3, normal_style),
        Span::styled("🔷 Databases: ", highlight_style),
        Span::styled("MySQL, PostgreSQL", normal_style),
    ]));
    let _s_db_bar = format!("{}  ", border_side);
    let db_progress = create_progress_bar(8, 10, db_bar_width);
    lines.push(Line::from(vec![
        Span::styled(_s_db_bar, normal_style),
        Span::styled("    ", normal_style),
        Span::styled(db_progress, Style::default().fg(accent_yellow)),
        Span::styled(" 88%", Style::default().fg(accent_yellow).add_modifier(ratatui::style::Modifier::BOLD)),
    ]));
    
    // DevOps
    let _s2050_7 = format!("{}  ", border_side);
    let devops_bar_width = (border_width as usize).saturating_sub(25).min(30);
    lines.push(Line::from(vec![
        Span::styled(_s2050_7, normal_style),
        Span::styled("🔷 Dev-Ops: ", highlight_style),
        Span::styled("Git, Docker, Kubernetes, CI/CD, AWS", normal_style),
    ]));
    let _s_devops_bar = format!("{}  ", border_side);
    let devops_progress = create_progress_bar(9, 10, devops_bar_width);
    lines.push(Line::from(vec![
        Span::styled(_s_devops_bar, normal_style),
        Span::styled("    ", normal_style),
        Span::styled(devops_progress, Style::default().fg(accent_magenta)),
        Span::styled(" 94%", Style::default().fg(accent_magenta).add_modifier(ratatui::style::Modifier::BOLD)),
    ]));
    let _s_devops = format!("{}              (S3, EC2, RDS, CloudWatch, Lambda)", border_side);
    lines.push(Line::from(vec![Span::styled(_s_devops, dim_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Projects with enhanced visuals
    let _s255 = format!("{}  🚀 PROJECTS", border_side);
    lines.push(Line::from(vec![Span::styled(_s255, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2050_8 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_8, normal_style),
        Span::styled("▶ ", accent_cyan),
        Span::styled("Kryptoria", project_style),
        Span::styled(" - Blockchain-based app", normal_style),
    ]));
    let _s6702 = format!("{}    • Built and optimized backend infrastructure using", border_side);
    lines.push(Line::from(vec![Span::styled(_s6702, normal_style)]));
    let _s762 = format!("{}      Node.js, Express.js, and Spring Boot, supporting", border_side);
    lines.push(Line::from(vec![Span::styled(_s762, normal_style)]));
    let _s4539 = format!("{}      dynamic in-game NFT asset transactions.", border_side);
    lines.push(Line::from(vec![Span::styled(_s4539, normal_style)]));
    let _s2080 = format!("{}    • Designed and developed the wallet connection module,", border_side);
    lines.push(Line::from(vec![Span::styled(_s2080, normal_style)]));
    let _s6298 = format!("{}      integrating with major cryptocurrency wallets for", border_side);
    lines.push(Line::from(vec![Span::styled(_s6298, normal_style)]));
    let _s6153 = format!("{}      secure user login and asset management.", border_side);
    lines.push(Line::from(vec![Span::styled(_s6153, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    
    let _s2050_9 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_9, normal_style),
        Span::styled("▶ ", accent_cyan),
        Span::styled("Wrktalk", project_style),
        Span::styled(" - Real-time chat application", normal_style),
    ]));
    let _s8189 = format!("{}    • Developed cross-platform frontend and backend features", border_side);
    lines.push(Line::from(vec![Span::styled(_s8189, normal_style)]));
    let _s5344 = format!("{}      enabling secure retrieval and synchronization of", border_side);
    lines.push(Line::from(vec![Span::styled(_s5344, normal_style)]));
    let _s1612 = format!("{}      historical messages after new installs.", border_side);
    lines.push(Line::from(vec![Span::styled(_s1612, normal_style)]));
    let _s4278 = format!("{}    • Architected and implemented real-time messaging with", border_side);
    lines.push(Line::from(vec![Span::styled(_s4278, normal_style)]));
    let _s3876 = format!("{}      Socket.IO for reliable, low-latency communication.", border_side);
    lines.push(Line::from(vec![Span::styled(_s3876, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    
    let _s2050_10 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_10, normal_style),
        Span::styled("▶ ", accent_cyan),
        Span::styled("BBPS Integration", project_style),
        Span::styled(" - Payment gateway", normal_style),
    ]));
    let _s5735 = format!("{}    • Implemented secure payment workflows with instant", border_side);
    lines.push(Line::from(vec![Span::styled(_s5735, normal_style)]));
    let _s3634 = format!("{}      digital receipt generation and real-time transaction", border_side);
    lines.push(Line::from(vec![Span::styled(_s3634, normal_style)]));
    let _s6958 = format!("{}      confirmations.", border_side);
    lines.push(Line::from(vec![Span::styled(_s6958, normal_style)]));
    let _s1627 = format!("{}    • Designed backend modules for bill fetching, validation,", border_side);
    lines.push(Line::from(vec![Span::styled(_s1627, normal_style)]));
    let _s202 = format!("{}      and payment processing.", border_side);
    lines.push(Line::from(vec![Span::styled(_s202, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    
    let _s2050_11 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_11, normal_style),
        Span::styled("▶ ", accent_cyan),
        Span::styled("Abra DeFi", project_style),
        Span::styled(" - Bridge, Trading & Payments", normal_style),
    ]));
    let _s4808 = format!("{}    • Engineered reliable USD to USDC on-ramping and", border_side);
    lines.push(Line::from(vec![Span::styled(_s4808, normal_style)]));
    let _s9780 = format!("{}      off-ramping functionality for seamless fiat-to-crypto", border_side);
    lines.push(Line::from(vec![Span::styled(_s9780, normal_style)]));
    let _s1211 = format!("{}      transactions.", border_side);
    lines.push(Line::from(vec![Span::styled(_s1211, normal_style)]));
    let _s3530 = format!("{}    • Integrated Rails.io for high-throughput crypto payments.", border_side);
    lines.push(Line::from(vec![Span::styled(_s3530, normal_style)]));
    let _s0 = format!("{}    • Contributed to Talos trading integration.", border_side);
    lines.push(Line::from(vec![Span::styled(_s0, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    
    let _s2050_4 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_4, normal_style),
        Span::styled("▶ ", accent_cyan),
        Span::styled("Abra-Fi", project_style),
        Span::styled(" - Solana + Spring WebFlux", normal_style),
    ]));
    let _s702 = format!("{}    • Spearheaded end-to-end backend development, integrating", border_side);
    lines.push(Line::from(vec![Span::styled(_s702, normal_style)]));
    let _s9277 = format!("{}      Solana blockchain capabilities with reactive microservices.", border_side);
    lines.push(Line::from(vec![Span::styled(_s9277, normal_style)]));
    let _s787 = format!("{}    • Integrated Solana SDK/RPC with Spring WebFlux for", border_side);
    lines.push(Line::from(vec![Span::styled(_s787, normal_style)]));
    let _s3806 = format!("{}      non-blocking smart contract interactions.", border_side);
    lines.push(Line::from(vec![Span::styled(_s3806, normal_style)]));
    let _s7625 = format!("{}    • Built blockchain crawlers to continuously fetch and", border_side);
    lines.push(Line::from(vec![Span::styled(_s7625, normal_style)]));
    let _s1524 = format!("{}      process on-chain program data.", border_side);
    lines.push(Line::from(vec![Span::styled(_s1524, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Personal Project
    let _s7097 = format!("{}  💻 PERSONAL PROJECT", border_side);
    lines.push(Line::from(vec![Span::styled(_s7097, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2050_12 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_12, normal_style),
        Span::styled("▶ ", accent_cyan),
        Span::styled("Walkie-Talkie App", project_style),
        Span::styled(" - Rust + Android", normal_style),
    ]));
    let _s2057 = format!("{}    • Developed a custom Rust library for low-level,", border_side);
    lines.push(Line::from(vec![Span::styled(_s2057, normal_style)]));
    let _s7641 = format!("{}      CPU-efficient audio processing, integrated with", border_side);
    lines.push(Line::from(vec![Span::styled(_s7641, normal_style)]));
    let _s2680 = format!("{}      native Android.", border_side);
    lines.push(Line::from(vec![Span::styled(_s2680, normal_style)]));
    let _s4607 = format!("{}    • Designed and implemented a system to capture audio,", border_side);
    lines.push(Line::from(vec![Span::styled(_s4607, normal_style)]));
    let _s646 = format!("{}      encode it, and transmit through UDP over Ethernet", border_side);
    lines.push(Line::from(vec![Span::styled(_s646, normal_style)]));
    let _s726 = format!("{}      in real time.", border_side);
    lines.push(Line::from(vec![Span::styled(_s726, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Professional Experience
    let _s6459 = format!("{}  💼 PROFESSIONAL EXPERIENCE", border_side);
    lines.push(Line::from(vec![Span::styled(_s6459, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2050_5 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_5, normal_style),
        Span::styled("Software Engineer", highlight_style),
        Span::styled("  ", normal_style),
        Span::styled("◆ ", accent_cyan),
        Span::styled("Rejolut Solutions Pvt Ltd", accent_cyan),
    ]));
    let _s2050_6 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_6, normal_style),
        Span::styled("May 2022 – Present | India", dim_style),
    ]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s7345 = format!("{}  • Designed and developed RESTful APIs using Spring Boot", border_side);
    lines.push(Line::from(vec![Span::styled(_s7345, normal_style)]));
    let _s5052 = format!("{}    and Node.js for enterprise applications.", border_side);
    lines.push(Line::from(vec![Span::styled(_s5052, normal_style)]));
    let _s5218 = format!("{}  • Integrated containerization workflows with Docker and", border_side);
    lines.push(Line::from(vec![Span::styled(_s5218, normal_style)]));
    let _s6084 = format!("{}    managed orchestration using Kubernetes.", border_side);
    lines.push(Line::from(vec![Span::styled(_s6084, normal_style)]));
    let _s8108 = format!("{}  • Automated deployment pipelines with GitHub Actions.", border_side);
    lines.push(Line::from(vec![Span::styled(_s8108, normal_style)]));
    let _s3550 = format!("{}  • Regularly participated in code reviews and Agile", border_side);
    lines.push(Line::from(vec![Span::styled(_s3550, normal_style)]));
    let _s1462 = format!("{}    sprint planning.", border_side);
    lines.push(Line::from(vec![Span::styled(_s1462, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Education
    let _s505 = format!("{}  🎓 EDUCATION", border_side);
    lines.push(Line::from(vec![Span::styled(_s505, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2050_13 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_13, normal_style),
        Span::styled("Bachelor's of Science in Information and Technology", highlight_style),
    ]));
    let _s2050_14 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_14, normal_style),
        Span::styled("Kalsekar Degree College", accent_cyan),
        Span::styled("  ", normal_style),
        Span::styled("2019 – 2022", dim_style),
    ]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2050_15 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_15, normal_style),
        Span::styled("Masters in Computer Application", highlight_style),
    ]));
    let _s2050_16 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_16, normal_style),
        Span::styled("Lovely Professional University", accent_cyan),
        Span::styled("  ", normal_style),
        Span::styled("2022 – 2026", dim_style),
    ]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Hobbies
    let _s5688 = format!("{}  🎯 HOBBIES", border_side);
    lines.push(Line::from(vec![Span::styled(_s5688, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s169 = format!("{}  • Exploring ARM and IoT devices", border_side);
    lines.push(Line::from(vec![Span::styled(_s169, normal_style)]));
    let _s_languages = format!("{}  • Interested in low-level languages (Rust, C, Go)", border_side);
    lines.push(Line::from(vec![Span::styled(_s_languages, normal_style)]));
    let _s7209 = format!("{}  • Built custom network routing systems for Kubernetes", border_side);
    lines.push(Line::from(vec![Span::styled(_s7209, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_bottom_start, border_line, border_bottom_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    lines
}

// Render the resume UI with retro-futuristic styling
// Returns the total number of lines
fn render_resume_ui(buffer: &mut Buffer, area: Rect, scroll_offset: u16) -> usize {
    
    // Build all lines
    let lines = build_resume_lines(area);
    let total_lines = lines.len();
    
    // Render lines with scroll offset
    let start_y = scroll_offset as usize;
    let max_lines = area.height as usize;
    
    for (idx, line) in lines.iter().enumerate() {
        let y_pos = idx.saturating_sub(start_y);
        if y_pos < max_lines && idx >= start_y {
            let paragraph = Paragraph::new(line.clone());
            paragraph.render(Rect::new(0, y_pos as u16, area.width, 1), buffer);
        }
    }
    
    total_lines
}

// Parse terminal size response from ANSI escape sequence
// Format: \x1b[8;{height};{width}t or \x1b[18t response
fn parse_terminal_size_response(data: &[u8]) -> Option<(u16, u16)> {
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
fn parse_ssh_input_from_buffer(buffer: &mut VecDeque<u8>) -> Option<event::KeyEvent> {
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

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize color-eyre for better error reporting
    color_eyre::install()?;

    // Just run the SSH server - TUI will be shown to SSH clients, not locally
    eprintln!("SSH TUI Server starting...");
    eprintln!("The TUI will be displayed to clients that connect via SSH.");
    eprintln!("Press Ctrl+C to stop the server.\n");
    
    let connection_count = Arc::new(AtomicU32::new(0));
    start_ssh_server(connection_count).await
}

async fn start_ssh_server(connection_count: Arc<AtomicU32>) -> Result<()> {
    let key_path = temp_server_key_path();
    let key = load_or_create_server_key(&key_path)?;
    
    // Create server config
    let config = Arc::new(server::Config {
        inactivity_timeout: Some(Duration::from_secs(3600)),
        auth_rejection_time: Duration::from_secs(3),
        auth_rejection_time_initial: Some(Duration::from_secs(0)),
        methods: MethodSet::from(&[MethodKind::None][..]),
        keys: vec![key],
        ..Default::default()
    });

    // Start listening on port 2222
    let socket = TcpListener::bind("127.0.0.1:2222").await?;
    
    eprintln!("SSH server listening on 127.0.0.1:2222");
    eprintln!("Connect with: ssh -p 2222 localhost");
    eprintln!("Host key path: {}", key_path.display());
    eprintln!("Authentication: none (no user/password prompt)");

    let mut server = ServerHandler {
        connection_count,
        clients: Arc::new(Mutex::new(HashMap::new())),
        id: 0,
    };

    // Run the server (this will block until the server shuts down)
    let server_task = server.run_on_socket(config, &socket);
    server_task.await?;

    Ok(())
}

fn temp_server_key_path() -> PathBuf {
    std::env::temp_dir().join("ssh-tui").join("server_ed25519")
}

fn load_or_create_server_key(path: &Path) -> Result<russh::keys::PrivateKey> {
    if path.exists() {
        if let Ok(key) = russh::keys::PrivateKey::read_openssh_file(path) {
            return Ok(key);
        }
        eprintln!(
            "Warning: failed to read SSH host key at {}, generating a new one.",
            path.display()
        );
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let key = russh::keys::PrivateKey::random(&mut OsRng, russh::keys::Algorithm::Ed25519)?;
    key.write_openssh_file(path, russh::keys::ssh_key::LineEnding::LF)?;
    Ok(key)
}
