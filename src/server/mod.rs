use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use color_eyre::Result;
use russh::keys::ssh_key::rand_core::OsRng;
use russh::server::{Msg, Server as _, Session};
use russh::*;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};

use crate::{
    app::RESIZE_SIGNAL,
    input::parse_terminal_size_response,
    session::run_ssh_tui,
};

#[derive(Clone)]
struct ClientConnection {
    channel_id: ChannelId,
    input_tx: mpsc::UnboundedSender<Vec<u8>>,
    terminal_size: Arc<Mutex<(u16, u16)>>,
}

type ClientRegistry = Arc<Mutex<HashMap<usize, ClientConnection>>>;

#[derive(Clone)]
struct ServerHandler {
    connection_count: Arc<AtomicU32>,
    clients: ClientRegistry,
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

    async fn auth_none(&mut self, _user: &str) -> std::result::Result<server::Auth, Self::Error> {
        Ok(server::Auth::Accept)
    }

    async fn auth_password(
        &mut self,
        _user: &str,
        _password: &str,
    ) -> std::result::Result<server::Auth, Self::Error> {
        Ok(server::Auth::Accept)
    }

    async fn auth_publickey(
        &mut self,
        _user: &str,
        _public_key: &russh::keys::PublicKey,
    ) -> std::result::Result<server::Auth, Self::Error> {
        Ok(server::Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut Session,
    ) -> std::result::Result<bool, Self::Error> {
        let channel_id = channel.id();
        let handle = session.handle();
        let (input_tx, input_rx) = mpsc::unbounded_channel();
        let terminal_size = Arc::new(Mutex::new((120u16, 40u16)));

        let client = ClientConnection {
            channel_id,
            input_tx,
            terminal_size: terminal_size.clone(),
        };

        {
            let mut clients = self.clients.lock().await;
            clients.insert(self.id, client);
        }

        let _ = handle
            .data(channel_id, CryptoVec::from("\x1b[18t".as_bytes()))
            .await;

        if let (Ok(cols_str), Ok(rows_str)) = (std::env::var("COLUMNS"), std::env::var("LINES")) {
            if let (Ok(cols), Ok(rows)) = (cols_str.parse::<u16>(), rows_str.parse::<u16>()) {
                let mut size = terminal_size.lock().await;
                *size = (cols, rows);
            }
        }

        let client_id = self.id;
        let connection_count = self.connection_count.clone();
        tokio::spawn(async move {
            if let Err(e) = run_ssh_tui(channel_id, handle, input_rx, connection_count, terminal_size).await
            {
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
    ) -> std::result::Result<bool, Self::Error> {
        Ok(false)
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> std::result::Result<(), Self::Error> {
        let target = {
            let clients = self.clients.lock().await;
            clients
                .values()
                .find(|client| client.channel_id == channel)
                .map(|client| (client.input_tx.clone(), client.terminal_size.clone()))
        };

        if let Some((input_tx, terminal_size)) = target {
            if let Some((width, height)) = parse_terminal_size_response(data) {
                let mut size = terminal_size.lock().await;
                *size = (width, height);
                let _ = input_tx.send(vec![RESIZE_SIGNAL]);
            } else {
                let _ = input_tx.send(data.to_vec());
            }
        }

        Ok(())
    }

    async fn channel_close(
        &mut self,
        channel: ChannelId,
        _session: &mut Session,
    ) -> std::result::Result<(), Self::Error> {
        let mut clients = self.clients.lock().await;
        clients.retain(|_, client| client.channel_id != channel);
        self.connection_count.fetch_sub(1, Ordering::Relaxed);
        Ok(())
    }
}

pub async fn start_ssh_server(connection_count: Arc<AtomicU32>) -> Result<()> {
    let key_path = temp_server_key_path();
    let key = load_or_create_server_key(&key_path)?;

    let config = Arc::new(server::Config {
        inactivity_timeout: Some(Duration::from_secs(3600)),
        auth_rejection_time: Duration::from_secs(3),
        auth_rejection_time_initial: Some(Duration::from_secs(0)),
        methods: MethodSet::from(&[MethodKind::None][..]),
        keys: vec![key],
        ..Default::default()
    });

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

    server.run_on_socket(config, &socket).await?;
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
