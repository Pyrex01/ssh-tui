use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
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
use tokio::sync::Mutex;

// SSH Server handler
#[derive(Clone)]
struct ServerHandler {
    connection_count: Arc<AtomicU32>,
    clients: Arc<Mutex<HashMap<usize, (ChannelId, russh::server::Handle)>>>,
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
        {
            let mut clients = self.clients.lock().await;
            clients.insert(self.id, (channel.id(), session.handle()));
        }
        Ok(true)
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Echo back the data
        let response = format!("Echo: {}\r\n", String::from_utf8_lossy(data));
        session.data(channel, CryptoVec::from(response.as_bytes()))?;
        Ok(())
    }
}

// Simple SSH client handler to demonstrate russh
#[derive(Clone)]
struct Client;

impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize color-eyre for better error reporting
    color_eyre::install()?;

    // Check if we have a TTY (interactive terminal)
    let has_tty = atty::is(atty::Stream::Stdout);
    
    if has_tty {
        // Initialize the terminal for TUI mode
        let mut terminal = ratatui::init();
        
        // Run the app with TUI
        let result = run(&mut terminal).await;
        
        // Restore the terminal
        ratatui::restore();
        
        result
    } else {
        // No TTY, just run the server without TUI
        eprintln!("Running in server-only mode (no TUI available)");
        let connection_count = Arc::new(AtomicU32::new(0));
        start_ssh_server(connection_count).await
    }
}

async fn run(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    let mut should_quit = false;
    
    // Connection counter for the SSH server
    let connection_count = Arc::new(AtomicU32::new(0));
    
    // Start SSH server in background
    let server_connection_count = connection_count.clone();
    let server_handle = tokio::spawn(async move {
        start_ssh_server(server_connection_count).await
    });

    // Demonstrate russh: Create an SSH config and handler
    let _config = Arc::new(client::Config::default());
    let _client_handler = Client {};

    loop {
        // Get current connection count
        let connections = connection_count.load(Ordering::Relaxed);
        
        // Create a message combining both libraries
        let message = format!(
            "Hello World!\n\nCombining:\n  • ratatui - Terminal UI framework\n  • russh - SSH client/server library\n  • tokio - Async runtime\n\nSSH Server Status:\n  • Listening on port 2222\n  • Active connections: {}\n  • Accepts any password/key (demo mode)\n\nConnect with:\n  ssh -p 2222 user@127.0.0.1\n\nPress 'q' to quit",
            connections
        );

        // Draw the UI
        terminal.draw(|frame| {
            let area = frame.area();
            
            // Create a paragraph widget with the message
            let paragraph = Paragraph::new(message.as_str())
                .block(
                    Block::default()
                        .title("SSH TUI Hello World - Server Running")
                        .borders(Borders::ALL)
                )
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Left);
            
            frame.render_widget(paragraph, area);
        })?;

        // Handle events
        if crossterm::event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            should_quit = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        if should_quit {
            break;
        }
    }

    // Cancel the server task
    server_handle.abort();
    let _ = server_handle.await;

    Ok(())
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
