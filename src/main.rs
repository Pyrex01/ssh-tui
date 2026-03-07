mod app;
mod input;
mod render;
mod server;
mod session;
mod ui;

use std::sync::{atomic::AtomicU32, Arc};

use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    eprintln!("SSH TUI Server starting...");
    eprintln!("The TUI will be displayed to clients that connect via SSH.");
    eprintln!("Press Ctrl+C to stop the server.\n");

    let connection_count = Arc::new(AtomicU32::new(0));
    server::start_ssh_server(connection_count).await
}
