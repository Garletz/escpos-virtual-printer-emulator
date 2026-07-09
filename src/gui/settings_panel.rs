use crate::emulator::EmulatorState;
use crate::networking::serial::{list_com_ports, start_serial_listener, SerialHandle};
use egui::Ui;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SettingsPanel {
    available_ports: Vec<String>,
    selected_port_idx: usize,
    baud_rates: Vec<u32>,
    selected_baud_idx: usize,
    status_message: String,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self {
            available_ports: list_com_ports(),
            selected_port_idx: 0,
            baud_rates: vec![9600, 19200, 38400, 57600, 115200],
            selected_baud_idx: 0,
            status_message: String::new(),
        }
    }
}

impl SettingsPanel {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        serial_handle: &mut Option<SerialHandle>,
        emulator_state: &Arc<Mutex<EmulatorState>>,
        tokio_handle: &tokio::runtime::Handle,
    ) {
        ui.heading("Emulator Settings");
        ui.separator();

        // Virtual printer management
        ui.group(|ui| {
            ui.label("Virtual Printer Management");
            ui.label("Installs the emulator as a system printer (TCP port 9100)");

            ui.horizontal(|ui| {
                if ui.button("🖨️ Install Windows Printer").clicked() {
                    self.install_windows_printer();
                }

                if ui.button("🐧 Install Linux Printer").clicked() {
                    self.install_linux_printer();
                }

                if ui.button("🗑️ Uninstall Printer").clicked() {
                    self.uninstall_printer();
                }
            });

            ui.label("Note: Requires administrator privileges");

            if ui.button("🔍 Check Status").clicked() {
                self.check_printer_status();
            }
        });

        ui.separator();

        // Serial / COM port section
        ui.group(|ui| {
            ui.label("Serial / COM Port (USB Virtual)");
            ui.label("Receives ESC/POS data via a virtual COM port pair (com0com)");

            let is_running = serial_handle.as_ref().map(|h| h.is_running()).unwrap_or(false);

            // Port selection row
            ui.horizontal(|ui| {
                ui.label("Port:");

                let selected_text = self
                    .available_ports
                    .get(self.selected_port_idx)
                    .cloned()
                    .unwrap_or_else(|| "No ports found".to_string());

                egui::ComboBox::from_id_source("com_port_select")
                    .selected_text(&selected_text)
                    .show_ui(ui, |ui| {
                        for (i, port) in self.available_ports.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_port_idx, i, port);
                        }
                    });

                if ui.button("🔄").on_hover_text("Refresh available ports").clicked() {
                    self.available_ports = list_com_ports();
                    self.selected_port_idx = 0;
                }
            });

            // Baud rate row
            ui.horizontal(|ui| {
                ui.label("Baud:");
                let selected_baud = self.baud_rates[self.selected_baud_idx];
                egui::ComboBox::from_id_source("baud_rate_select")
                    .selected_text(selected_baud.to_string())
                    .show_ui(ui, |ui| {
                        for (i, baud) in self.baud_rates.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_baud_idx, i, baud.to_string());
                        }
                    });
            });

            // Start / Stop row
            ui.horizontal(|ui| {
                if is_running {
                    if ui.button("⏹ Stop").clicked() {
                        if let Some(h) = serial_handle.take() {
                            h.stop();
                        }
                        self.status_message = "Serial listener stopped.".to_string();
                    }
                    ui.colored_label(egui::Color32::GREEN, "● Running");
                } else {
                    let can_start = !self.available_ports.is_empty();
                    if ui
                        .add_enabled(can_start, egui::Button::new("▶ Start COM Listener"))
                        .clicked()
                    {
                        let port = self.available_ports[self.selected_port_idx].clone();
                        let baud = self.baud_rates[self.selected_baud_idx];
                        match start_serial_listener(
                            port.clone(),
                            baud,
                            emulator_state.clone(),
                            tokio_handle.clone(),
                        ) {
                            Ok(handle) => {
                                *serial_handle = Some(handle);
                                self.status_message =
                                    format!("Listening on {} @ {} baud", port, baud);
                            }
                            Err(e) => {
                                self.status_message = format!("Error: {}", e);
                            }
                        }
                    }
                    ui.colored_label(egui::Color32::RED, "● Stopped");
                }
            });

            if !self.status_message.is_empty() {
                ui.label(&self.status_message);
            }

            ui.separator();

            ui.collapsing("com0com Setup Guide", |ui| {
                ui.label("com0com creates a virtual COM port pair on Windows.");
                ui.label("One port is used by your PDV, the other by this emulator.");
                ui.label("");
                ui.label("1. Download: sourceforge.net/projects/com0com");
                ui.label("2. Install com0com (run as administrator)");
                ui.label("3. Open com0com Setup and create a pair (e.g. COM3 <-> COM4)");
                ui.label("4. Configure your PDV/POS to send to COM3");
                ui.label("5. Select COM4 in this emulator and click Start");
                ui.label("");
                ui.label("All data sent to COM3 will appear as a receipt here in real time.");
            });
        });

        ui.separator();

        // Network settings
        ui.group(|ui| {
            ui.label("Network Configuration");
            ui.label("TCP Port: 9100  |  Address: 127.0.0.1");

            if ui.button("📡 Test Connection").clicked() {
                self.test_network_connection();
            }
        });

        ui.separator();

        ui.group(|ui| {
            ui.label("ℹ️  Automatic Operation");
            ui.label("• The emulator automatically respects ESC/POS standards");
            ui.label("• Paper width: 50mm, 78mm, 80mm (auto-detection)");
            ui.label("• Font, justification, emphasis: ESC/POS commands");
            ui.label("• No manual configuration needed!");
        });
    }

    fn install_windows_printer(&self) {
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Add-PrinterPort -Name '127.0.0.1:9100' -PrinterHostAddress '127.0.0.1' -PortNumber 9100; \
                 $driver = (Get-PrinterDriver | Where-Object { $_.Name -like '*Microsoft*' } | Select-Object -First 1).Name; \
                 Add-Printer -Name 'ESC_POS_Virtual_Printer' -DriverName $driver -PortName '127.0.0.1:9100'; \
                 Write-Host 'Printer installed successfully'"
            ])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("✅ {}", String::from_utf8_lossy(&output.stdout));
                } else {
                    println!("❌ Error: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => println!("❌ Cannot execute printer installation: {}", e),
        }
    }

    fn install_linux_printer(&self) {
        let output = Command::new("bash")
            .args([
                "-c",
                "if command -v lpstat &> /dev/null; then \
                    sudo lpadmin -p ESC_POS_Linux_Printer -E -v socket://127.0.0.1:9100 -m 'Generic Text-Only Printer'; \
                    sudo lpadmin -d ESC_POS_Linux_Printer; \
                    echo 'Linux printer installed successfully!'; \
                else \
                    echo 'CUPS not found. Please install CUPS first.'; \
                fi",
            ])
            .output();

        match output {
            Ok(output) => println!("ℹ️  {}", String::from_utf8_lossy(&output.stdout)),
            Err(e) => println!("ℹ️  Linux installation attempted: {}", e),
        }
    }

    fn uninstall_printer(&self) {
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Remove-Printer -Name 'ESC_POS_Virtual_Printer' -Confirm:$false; \
                 Remove-PrinterPort -Name '127.0.0.1:9100'; \
                 Write-Host 'Printer uninstalled successfully'",
            ])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("✅ {}", String::from_utf8_lossy(&output.stdout));
                } else {
                    println!("❌ Error: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => println!("❌ Cannot execute printer uninstallation: {}", e),
        }
    }

    fn check_printer_status(&self) {
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Get-Printer -Name 'ESC_POS_Virtual_Printer' -ErrorAction SilentlyContinue | Select-Object Name, PortName, DriverName, PrinterStatus",
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim().is_empty() {
                    println!("ℹ️  Virtual printer not installed");
                } else {
                    println!("✅ Virtual printer installed:\n{}", stdout);
                }
            }
            Ok(_) => println!("❌ Could not check printer status"),
            Err(e) => println!("❌ Cannot check status: {}", e),
        }
    }

    fn test_network_connection(&self) {
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Test-NetConnection -ComputerName 127.0.0.1 -Port 9100 -WarningAction SilentlyContinue | Select-Object TcpTestSucceeded",
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("True") {
                    println!("✅ Connection to port 9100 successful");
                } else {
                    println!("❌ Connection to port 9100 failed");
                }
            }
            Ok(_) => println!("❌ Cannot test connection"),
            Err(e) => println!("❌ Cannot test connection: {}", e),
        }
    }
}
