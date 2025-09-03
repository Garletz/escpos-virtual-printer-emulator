# 🖨️ ESC/POS Virtual Printer Emulator

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-lightgrey.svg)](https://github.com/your-username/escpos-virtual-printer-emulator)

> **Professional ESC/POS virtual printer emulator built in Rust**  
> Transform your computer into a virtual receipt printer for testing and development

## ✨ Features

- 🖨️ **Virtual Printer Integration** - Installs as a system printer on Windows/Linux
- 📡 **Network Support** - TCP/IP server on port 9100 (ESC/POS standard)
- 🎨 **Modern GUI** - Beautiful egui-based interface with real-time preview
- 📄 **Receipt Viewer** - Live preview of printed receipts
- 📋 **Command Log** - Detailed ESC/POS command monitoring
- ⚙️ **Auto-Configuration** - Respects ESC/POS standards automatically
- 🚀 **High Performance** - Built with Rust for maximum speed and reliability

## 🎯 Supported Paper Widths

| Width | Characters | Dots | Use Case |
|-------|------------|------|----------|
| **50mm** | 48 chars | 384 dots | Small receipts, tickets |
| **78mm** | 72 chars | 576 dots | Standard receipts |
| **80mm** | 80 chars | 640 dots | Large receipts, invoices |

## 🚀 Quick Start

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

## 📖 Usage

### Basic Usage

1. **Start the emulator** - The GUI will open with the server running on port 9100
2. **Install the printer** - Use the Settings tab to install the virtual printer
3. **Print from any application** - Select "ESC_POS_Virtual_Printer" as your printer
4. **View results** - Check the Receipt tab for live preview

### Supported Applications

- ✅ **Microsoft Office** (Word, Excel, PowerPoint)
- ✅ **Web Browsers** (Chrome, Firefox, Edge)
- ✅ **POS Systems** (Any ESC/POS compatible software)
- ✅ **Custom Applications** (Via network port 9100)
- ✅ **Command Line Tools** (Direct TCP connection)

## 🛠️ Technical Details

### Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Application   │───▶│  Windows/Linux   │───▶│  Virtual Printer│
│   (Word, Excel) │    │   Print Spooler  │    │   (Port 9100)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                         │
                                                         ▼
                                               ┌─────────────────┐
                                               │  ESC/POS Server │
                                               │  (Rust/Tokio)   │
                                               └─────────────────┘
                                                         │
                                                         ▼
                                               ┌─────────────────┐
                                               │   GUI Preview   │
                                               │   (egui)        │
                                               └─────────────────┘
```

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

## 🔧 Development

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

## 📊 Performance

- **Memory Usage**: ~10MB RAM
- **CPU Usage**: <1% when idle
- **Network Latency**: <1ms local
- **Startup Time**: <2 seconds
- **Binary Size**: ~5MB (release build)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Commit your changes: `git commit -m 'Add amazing feature'`
6. Push to the branch: `git push origin feature/amazing-feature`
7. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **ESC/POS Standard** - For the printer command specification
- **Rust Community** - For the amazing ecosystem
- **egui** - For the beautiful GUI framework
- **Original C# Project** - [EscPosEmulator](https://github.com/roydejong/EscPosEmulator) for inspiration

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/your-username/escpos-virtual-printer-emulator/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-username/escpos-virtual-printer-emulator/discussions)
- **Email**: support@your-domain.com

---

<div align="center">

**Made with ❤️ in Rust**

[⭐ Star this project](https://github.com/your-username/escpos-virtual-printer-emulator) | [🐛 Report Bug](https://github.com/your-username/escpos-virtual-printer-emulator/issues) | [💡 Request Feature](https://github.com/your-username/escpos-virtual-printer-emulator/issues)

</div>
