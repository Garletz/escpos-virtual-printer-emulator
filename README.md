# 🖨️ ESC/POS Virtual Printer Emulator

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-lightgrey.svg)](https://github.com/your-username/escpos-virtual-printer-emulator)

> **ESC/POS virtual printer emulator built in Rust**  
> Transform your computer into a virtual receipt printer for testing and development
<img width="1920" height="1080" alt="Capture d’écran (280)" src="https://github.com/user-attachments/assets/709335cd-79b9-40fd-ab51-7027f6ee0405" />
<img width="1920" height="1080" alt="Capture d’écran (281)" src="https://github.com/user-attachments/assets/c02db29b-53ca-49e1-b145-6b7cb31e4fc1" />




## Supported Paper Widths

| Width | Characters | Dots | Use Case |
|-------|------------|------|----------|
| **50mm** | 48 chars | 384 dots | Small receipts, tickets |
| **78mm** | 72 chars | 576 dots | Standard receipts |
| **80mm** | 80 chars | 640 dots | Large receipts, invoices |

##  Quick Start

### Prerequisites
- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Windows 10/11** or **Linux** with CUPS
- **Administrator privileges** (for printer installation)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-username/escpos-virtual-printer-emulator.git
   cd escpos-virtual-printer-emulator
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run the emulator**
   ```bash
   cargo run --release
   ```

4. **Install virtual printer**
   - Go to **Settings** tab
   - Click **"🖨️ Install Windows Printer"** (requires admin)
   - The printer will appear in Windows "Devices and Printers"


### Basic Usage

1. **Start the emulator** - The GUI will open with the server running on port 9100
2. **Install the printer** - Use the Settings tab to install the virtual printer
3. **Print from any application** - Select "ESC_POS_Virtual_Printer" as your printer
4. **View results** - Check the Receipt tab for live preview



### ESC/POS Commands Supported

| Command | Description | Example |
|---------|-------------|---------|
| `ESC @` | Initialize printer | `\x1B@` |
| `ESC M n` | Select font | `\x1BM0` (Font A) |
| `ESC a n` | Justification | `\x1Ba1` (Center) |
| `ESC E` | Emphasis (Bold) | `\x1BE` |
| `ESC - n` | Underline | `\x1B-1` |
| `ESC 4` | Italic | `\x1B4` |
| `ESC 3 n` | Line height | `\x1B324` |
| `ESC ! n` | Font size | `\x1B!16` |
| `ESC m` | Cut paper | `\x1Bm` |

##  Development

### Project Structure

```
escpos_emulator/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library exports
│   ├── escpos/              # ESC/POS command handling
│   │   ├── commands.rs      # Command definitions
│   │   ├── parser.rs        # Command parsing
│   │   └── printer.rs       # Printer state management
│   ├── emulator/            # Core emulator logic
│   │   └── mod.rs           # Emulator state
│   ├── networking/          # Network server
│   │   └── server.rs        # TCP server implementation
│   └── gui/                 # User interface
│       ├── app.rs           # Main application
│       ├── receipt_viewer.rs # Receipt display
│       ├── command_log.rs   # Command monitoring
│       └── settings_panel.rs # Settings and printer management
├── Cargo.toml               # Project configuration
└── README.md                # This file
```

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check code
cargo check
```

### Dependencies

- **eframe/egui** - Modern GUI framework
- **tokio** - Async runtime and networking
- **serde** - Serialization/deserialization
- **image** - Image processing
- **tracing** - Structured logging
- **anyhow/thiserror** - Error handling



This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.


---
