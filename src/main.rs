use escpos_emulator::emulator::EmulatorState;
use escpos_emulator::gui::EscPosEmulatorApp;
use escpos_emulator::networking::server;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Starting ESC/POS Emulator...");

    let emulator_state = Arc::new(Mutex::new(EmulatorState::new()));

    let server_state = emulator_state.clone();
    tokio::spawn(async move {
        if let Err(e) = server::start_server(server_state).await {
            eprintln!("❌ Server error: {}", e);
        }
    });

    let tokio_handle = tokio::runtime::Handle::current();

    info!("✅ Emulator initialized successfully");

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "ESC/POS Virtual Printer Emulator",
        options,
        Box::new(move |_cc| Box::new(EscPosEmulatorApp::new(emulator_state, tokio_handle))),
    )
}
