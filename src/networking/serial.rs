use crate::emulator::EmulatorState;
use crate::escpos::parser::EscPosParser;
use anyhow::{Context, Result};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, warn};

pub struct SerialHandle {
    running: Arc<AtomicBool>,
}

impl SerialHandle {
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

pub fn list_com_ports() -> Vec<String> {
    match serialport::available_ports() {
        Ok(ports) => ports.into_iter().map(|p| p.port_name).collect(),
        Err(e) => {
            warn!("Failed to list serial ports: {}", e);
            Vec::new()
        }
    }
}

pub fn start_serial_listener(
    port_name: String,
    baud_rate: u32,
    emulator_state: Arc<Mutex<EmulatorState>>,
    tokio_handle: tokio::runtime::Handle,
) -> Result<SerialHandle> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    // Async task: receive bytes from channel, parse ESC/POS, process commands
    tokio_handle.spawn(async move {
        let mut parser = EscPosParser::new();
        while let Some(data) = rx.recv().await {
            match parser.parse_stream(&data) {
                Ok(commands) => {
                    let mut state = emulator_state.lock().await;
                    for command in &commands {
                        state.process_command(command);
                    }
                }
                Err(e) => warn!("Serial parse error: {}", e),
            }
        }
        info!("Serial data processor stopped");
    });

    // Open port synchronously so we can return an error immediately if it fails
    let port = serialport::new(&port_name, baud_rate)
        .timeout(Duration::from_millis(100))
        .open()
        .with_context(|| format!("Failed to open serial port {}", port_name))?;

    // Sync thread: read bytes from serial port and forward to async channel
    std::thread::spawn(move || {
        let mut port = port;
        let mut buf = vec![0u8; 1024];

        while running_clone.load(Ordering::SeqCst) {
            match port.read(&mut buf) {
                Ok(0) => {}
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break; // receiver dropped
                    }
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::TimedOut
                        || e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    warn!("Serial read error: {}", e);
                    break;
                }
            }
        }

        running_clone.store(false, Ordering::SeqCst);
        info!("Serial listener stopped");
    });

    info!("Serial listener started on {} @ {} baud", port_name, baud_rate);
    Ok(SerialHandle { running })
}
