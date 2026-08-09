use crate::emulator::EmulatorState;
use crate::escpos::parser::EscPosParser;
use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

pub async fn start_server(emulator_state: Arc<Mutex<EmulatorState>>) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9100").await?;
    info!("ESC/POS Emulator server listening on 127.0.0.1:9100 (Raw TCP & Web HTTP/CORS)");

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
    let mut chunk = vec![0u8; 4096];

    let n = match socket.read(&mut chunk).await {
        Ok(0) => return Ok(()),
        Ok(n) => n,
        Err(e) => return Err(e.into()),
    };

    buffer.extend_from_slice(&chunk[..n]);

    // Check if client is a Web Browser sending HTTP OPTIONS (CORS preflight)
    if buffer.starts_with(b"OPTIONS ") {
        let response = "HTTP/1.1 204 No Content\r\n\
                        Access-Control-Allow-Origin: *\r\n\
                        Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
                        Access-Control-Allow-Headers: *\r\n\
                        Access-Control-Allow-Private-Network: true\r\n\
                        Access-Control-Max-Age: 86400\r\n\
                        Connection: close\r\n\r\n";
        socket.write_all(response.as_bytes()).await?;
        return Ok(());
    } 
    // Check if client is a Web Browser sending HTTP POST (fetch / axios)
    else if buffer.starts_with(b"POST ") {
        // Read full HTTP body if needed
        let mut full_request = buffer.clone();
        while !full_request.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = socket.read(&mut chunk).await?;
            if n == 0 { break; }
            full_request.extend_from_slice(&chunk[..n]);
        }

        if let Some(body_start) = find_subslice(&full_request, b"\r\n\r\n") {
            let body = &full_request[body_start + 4..];
            process_raw_bytes(body, &emulator_state).await;

            let response = "HTTP/1.1 200 OK\r\n\
                            Access-Control-Allow-Origin: *\r\n\
                            Access-Control-Allow-Private-Network: true\r\n\
                            Content-Type: text/plain\r\n\
                            Connection: close\r\n\r\n\
                            OK";
            socket.write_all(response.as_bytes()).await?;
            return Ok(());
        }
    }

    // Otherwise, handle as standard Raw TCP / ESC-POS stream
    let mut parser = EscPosParser::new();
    process_bytes_with_parser(&buffer, &mut parser, &emulator_state).await;

    loop {
        match socket.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => {
                process_bytes_with_parser(&chunk[..n], &mut parser, &emulator_state).await;
            }
            Err(e) => {
                warn!("Error reading from socket: {}", e);
                break;
            }
        }
    }

    let response = b"OK\n";
    let _ = socket.write_all(response).await;
    Ok(())
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

async fn process_raw_bytes(data: &[u8], emulator_state: &Arc<Mutex<EmulatorState>>) {
    let mut parser = EscPosParser::new();
    if let Ok(commands) = parser.parse_stream(data) {
        let mut state = emulator_state.lock().await;
        for command in commands {
            state.process_command(&command);
        }
    }
}

async fn process_bytes_with_parser(
    data: &[u8],
    parser: &mut EscPosParser,
    emulator_state: &Arc<Mutex<EmulatorState>>,
) {
    if let Ok(commands) = parser.parse_stream(data) {
        let mut state = emulator_state.lock().await;
        for command in commands {
            info!("Received command: {:?}", command);
            state.process_command(&command);
        }
    }
}
