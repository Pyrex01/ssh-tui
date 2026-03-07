use std::{collections::VecDeque, sync::{atomic::AtomicU32, Arc}, time::Duration};

use color_eyre::Result;
use ratatui::prelude::*;
use russh::{server::Handle, ChannelId, CryptoVec};
use tokio::sync::{mpsc, Mutex};

use crate::{
    app::{AppState, RESIZE_SIGNAL},
    input::parse_ssh_input_from_buffer,
    render::render_buffer_to_ansi,
    ui::{build_resume_lines, render_resume_ui},
};

pub async fn run_ssh_tui(
    channel_id: ChannelId,
    handle: Handle,
    mut input_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    _connection_count: Arc<AtomicU32>,
    terminal_size: Arc<Mutex<(u16, u16)>>,
) -> Result<()> {
    let mut app = AppState::default();
    let mut input_buffer = VecDeque::<u8>::new();
    let mut size_request_interval = tokio::time::interval(Duration::from_secs(2));
    size_request_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        while let Ok(data) = input_rx.try_recv() {
            if data.len() == 1 && data[0] == RESIZE_SIGNAL {
                app.needs_redraw = true;
                continue;
            }
            input_buffer.extend(data);
            app.needs_redraw = true;
        }

        while let Some(key_event) = parse_ssh_input_from_buffer(&mut input_buffer) {
            app.apply_key(key_event.code);
            if app.should_quit {
                break;
            }
        }

        if app.should_quit {
            let goodbye = "\x1b[2J\x1b[H\x1b[36m[SYSTEM] Connection terminated. Goodbye!\x1b[0m\r\n";
            let _ = handle.data(channel_id, CryptoVec::from(goodbye.as_bytes())).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
            let _ = handle.eof(channel_id).await;
            let _ = handle.close(channel_id).await;
            break;
        }

        if app.needs_redraw {
            let (width, height) = *terminal_size.lock().await;
            let width = width.max(40);
            let height = height.max(10);
            let area = Rect::new(0, 0, width, height);

            let lines = build_resume_lines(area);
            app.clamp_scroll(lines.len(), height);

            let mut buffer = Buffer::empty(area);
            render_resume_ui(&mut buffer, area, app.scroll_offset);

            let output = render_buffer_to_ansi(&buffer, width, height);
            if let Err(e) = handle.data(channel_id, CryptoVec::from(output.as_bytes())).await {
                eprintln!("Error sending data to channel: {e:#?}");
                break;
            }

            app.needs_redraw = false;
        }

        let current_size = *terminal_size.lock().await;
        app.handle_resize(current_size);

        let input_received = tokio::select! {
            input = input_rx.recv() => {
                if let Some(data) = input {
                    if data.len() == 1 && data[0] == RESIZE_SIGNAL {
                        app.needs_redraw = true;
                        false
                    } else {
                        input_buffer.extend(data);
                        true
                    }
                } else {
                    break;
                }
            }
            _ = size_request_interval.tick() => {
                let _ = handle.data(channel_id, CryptoVec::from("\x1b[18t".as_bytes())).await;
                false
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                false
            }
        };

        if input_received {
            app.needs_redraw = true;
        }
    }

    Ok(())
}
