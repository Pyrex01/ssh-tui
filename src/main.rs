use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use russh::*;
use russh::keys::ssh_key::rand_core::OsRng;
use russh::server::{Msg, Server as _, Session};
use std::collections::HashMap;
use std::net::SocketAddr;
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
        
        // Shared terminal size (default to 80x24)
        let terminal_size = Arc::new(Mutex::new((80u16, 24u16)));
        
        {
            let mut clients = self.clients.lock().await;
            clients.insert(self.id, (channel_id, handle.clone(), input_tx, terminal_size.clone()));
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

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Forward input to the TUI task
        let clients = self.clients.lock().await;
        if let Some((_, _, input_tx, _)) = clients.values().find(|(id, _, _, _)| *id == channel) {
            let _ = input_tx.send(data.to_vec());
        }
        Ok(())
    }
}

// Run TUI for an SSH client
async fn run_ssh_tui(
    channel_id: ChannelId,
    handle: russh::server::Handle,
    mut input_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    connection_count: Arc<AtomicU32>,
    terminal_size: Arc<Mutex<(u16, u16)>>,
) -> Result<()> {
    let mut should_quit = false;
    let mut input_buffer = VecDeque::<u8>::new();
    let mut needs_redraw = true;
    
    loop {
        // Check for input first (non-blocking)
        while let Ok(data) = input_rx.try_recv() {
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
                _ => {
                    // Handle other keys if needed
                }
            }
        }
        
        if should_quit {
            // Send goodbye message before quitting
            let goodbye = "\x1b[2J\x1b[HGoodbye! Connection closing...\r\n";
            let _ = handle.data(channel_id, CryptoVec::from(goodbye.as_bytes())).await;
            break;
        }
        
        // Only redraw if needed or periodically
        if needs_redraw {
            // Get current terminal size
            let (width, height) = *terminal_size.lock().await;
            
            // Get current connection count
            let connections = connection_count.load(Ordering::Relaxed);
            
            // Create the TUI content
            let message = format!(
                "Hello World!\n\nCombining:\n  • ratatui - Terminal UI framework\n  • russh - SSH client/server library\n  • tokio - Async runtime\n\nSSH Server Status:\n  • Active connections: {}\n  • Terminal size: {}x{}\n\nPress 'q' to quit",
                connections, width, height
            );
            
            // Create a buffer to render to
            let area = Rect::new(0, 0, width, height);
            let mut buffer = Buffer::empty(area);
            
            // Render the UI using ratatui
            {
                let paragraph = Paragraph::new(message.as_str())
                    .block(
                        Block::default()
                            .title("SSH TUI Hello World")
                            .borders(Borders::ALL)
                    )
                    .wrap(Wrap { trim: true })
                    .alignment(Alignment::Left);
                
                paragraph.render(area, &mut buffer);
            }
            
            // Convert buffer to ANSI escape sequences and send to SSH channel
            let output = render_buffer_to_ansi(&buffer, width, height);
            if let Err(e) = handle.data(channel_id, CryptoVec::from(output.as_bytes())).await {
                eprintln!("Error sending data to channel: {e:#?}");
                break;
            }
        }
        
        // Wait for input or timeout
        needs_redraw = tokio::select! {
            input = input_rx.recv() => {
                if let Some(data) = input {
                    input_buffer.extend(data);
                    true // Need redraw after input
                } else {
                    // Channel closed
                    break;
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // Periodic redraw to update connection count
                true
            }
        };
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
    // Generate server keys
    let key = russh::keys::PrivateKey::random(&mut OsRng, russh::keys::Algorithm::Ed25519)?;
    
    // Create server config
    let config = Arc::new(server::Config {
        inactivity_timeout: Some(Duration::from_secs(3600)),
        auth_rejection_time: Duration::from_secs(3),
        auth_rejection_time_initial: Some(Duration::from_secs(0)),
        keys: vec![key],
        ..Default::default()
    });

    // Start listening on port 2222
    let socket = TcpListener::bind("127.0.0.1:2222").await?;
    
    eprintln!("SSH server listening on 127.0.0.1:2222");
    eprintln!("Connect with: ssh -p 2222 user@127.0.0.1");
    eprintln!("(Any password will work for demo)");

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
