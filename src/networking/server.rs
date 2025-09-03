use crate::emulator::EmulatorState;
use crate::escpos::parser::EscPosParser;
use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{info, warn, error};

pub async fn start_server(emulator_state: Arc<Mutex<EmulatorState>>) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9100").await?;
    info!("ESC/POS Emulator server listening on 127.0.0.1:9100");

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("New connection from: {}", addr);
                let state = emulator_state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket, state).await {
                        error!("Error handling connection from {}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    emulator_state: Arc<Mutex<EmulatorState>>,
) -> Result<()> {
    let mut buffer = Vec::new();
    let mut parser = EscPosParser::new();

    loop {
        let mut chunk = vec![0u8; 1024];
        match socket.read(&mut chunk).await {
            Ok(0) => {
                // Connection closed
                info!("Connection closed by client");
                break;
            }
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                
                // Process complete commands
                if let Ok(commands) = parser.parse_stream(&buffer[..]) {
                    for command in commands {
                        info!("Received command: {:?}", command);
                        
                        // Process command in emulator state
                        let mut state = emulator_state.lock().await;
                        state.process_command(&command);
                    }
                    
                    // Clear buffer after processing
                    buffer.clear();
                }
            }
            Err(e) => {
                warn!("Error reading from socket: {}", e);
                break;
            }
        }
    }

    // Send acknowledgment
    let response = b"OK\n";
    if let Err(e) = socket.write_all(response).await {
        warn!("Failed to send response: {}", e);
    }

    Ok(())
}
