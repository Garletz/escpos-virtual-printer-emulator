use crate::emulator::EmulatorState;
use crate::networking::serial::{list_com_ports, start_serial_listener, SerialHandle};
use egui::Ui;
use std::net::{SocketAddr, TcpStream};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
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

            ui.label("Note: Requires administrator / root privileges");

            ui.horizontal(|ui| {
                if ui.button("🔍 Check Printer Status").clicked() {
                    self.check_printer_status();
                }
            });

            if !self.status_message.is_empty() {
                ui.separator();
                ui.label(&self.status_message);
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

                let baud_text = self
                    .baud_rates
                    .get(self.selected_baud_idx)
                    .map(|b| b.to_string())
                    .unwrap_or_else(|| "9600".to_string());

                egui::ComboBox::from_id_source("baud_rate_select")
                    .selected_text(&baud_text)
                    .show_ui(ui, |ui| {
                        for (i, baud) in self.baud_rates.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_baud_idx, i, baud.to_string());
                        }
                    });
            });

            // Start / Stop controls
            ui.horizontal(|ui| {
                if is_running {
                    if ui.button("⏹ Stop Serial Listener").clicked() {
                        if let Some(handle) = serial_handle.take() {
                            handle.stop();
                            self.status_message = "Serial listener stopped.".to_string();
                        }
                    }
                    ui.colored_label(egui::Color32::GREEN, "● Running");
                } else {
                    let port_name = self.available_ports.get(self.selected_port_idx).cloned();
                    let baud_rate = self.baud_rates.get(self.selected_baud_idx).copied().unwrap_or(9600);

                    let can_start = port_name.is_some() && !self.available_ports.is_empty();

                    if ui
                        .add_enabled(can_start, egui::Button::new("▶ Start Serial Listener"))
                        .clicked()
                    {
                        if let Some(port) = port_name {
                            match start_serial_listener(
                                port.clone(),
                                baud_rate,
                                Arc::clone(emulator_state),
                                tokio_handle.clone(),
                            ) {
                                Ok(handle) => {
                                    *serial_handle = Some(handle);
                                    self.status_message = format!("Listening on {} @ {} baud", port, baud_rate);
                                }
                                Err(e) => {
                                    self.status_message = format!("Error: {}", e);
                                }
                            }
                        }
                    }
                    ui.colored_label(egui::Color32::RED, "● Stopped");
                }
            });

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
            ui.label("ℹ️ Automatic Operation");
            ui.label("• The emulator automatically respects ESC/POS standards");
            ui.label("• Paper width: 50mm, 78mm, 80mm (auto-detection)");
            ui.label("• Font, justification, emphasis: ESC/POS commands");
            ui.label("• No manual configuration needed!");
        });
    }

    fn install_windows_printer(&mut self) {
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Add-PrinterPort -Name '127.0.0.1:9100' -PrinterHostAddress '127.0.0.1' -PortNumber 9100; \
                 $driver = (Get-PrinterDriver | Where-Object { $_.Name -like '*Microsoft*' } | Select-Object -First 1).Name; \
                 Add-Printer -Name 'ESC_POS_Virtual_Printer' -DriverName $driver -PortName '127.0.0.1:9100'; \
                 Write-Host 'Windows printer installed successfully!'",
            ])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    self.status_message = format!("✅ {}", String::from_utf8_lossy(&output.stdout).trim());
                } else {
                    self.status_message = format!("❌ Error: {}", String::from_utf8_lossy(&output.stderr).trim());
                }
            }
            Err(e) => self.status_message = format!("❌ Cannot execute PowerShell: {}", e),
        }
    }

    fn install_linux_printer(&mut self) {
        let cmd = "if command -v lpstat &> /dev/null; then \
            if command -v pkexec &> /dev/null; then \
                pkexec lpadmin -p ESC_POS_Linux_Printer -E -v socket://127.0.0.1:9100 -m raw && \
                pkexec lpadmin -d ESC_POS_Linux_Printer; \
            else \
                sudo lpadmin -p ESC_POS_Linux_Printer -E -v socket://127.0.0.1:9100 -m raw && \
                sudo lpadmin -d ESC_POS_Linux_Printer; \
            fi; \
            echo 'Linux printer (ESC_POS_Linux_Printer) installed successfully!'; \
        else \
            echo 'CUPS not found. Please install CUPS first.'; \
        fi";

        let output = Command::new("bash").args(["-c", cmd]).output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                if output.status.success() && !stdout.is_empty() {
                    self.status_message = format!("✅ {}", stdout);
                } else if !stderr.is_empty() {
                    self.status_message = format!("ℹ️ {}", stderr);
                } else {
                    self.status_message = "✅ Installation command sent.".to_string();
                }
            }
            Err(e) => self.status_message = format!("❌ Cannot execute bash: {}", e),
        }
    }

    fn uninstall_printer(&mut self) {
        if cfg!(target_os = "windows") {
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
                        self.status_message = format!("✅ {}", String::from_utf8_lossy(&output.stdout).trim());
                    } else {
                        self.status_message = format!("❌ Error: {}", String::from_utf8_lossy(&output.stderr).trim());
                    }
                }
                Err(e) => self.status_message = format!("❌ Cannot execute PowerShell: {}", e),
            }
        } else {
            let cmd = "if command -v pkexec &> /dev/null; then \
                pkexec lpadmin -x ESC_POS_Linux_Printer; \
            else \
                sudo lpadmin -x ESC_POS_Linux_Printer; \
            fi";
            let output = Command::new("bash").args(["-c", cmd]).output();
            match output {
                Ok(output) => {
                    if output.status.success() {
                        self.status_message = "✅ Linux printer (ESC_POS_Linux_Printer) uninstalled successfully.".to_string();
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                        self.status_message = format!("❌ Uninstall failed: {}", stderr);
                    }
                }
                Err(e) => self.status_message = format!("❌ Cannot execute bash: {}", e),
            }
        }
    }

    fn check_printer_status(&mut self) {
        if cfg!(target_os = "windows") {
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
                        self.status_message = "ℹ️ Virtual printer not installed on Windows".to_string();
                    } else {
                        self.status_message = format!("✅ Virtual printer installed:\n{}", stdout);
                    }
                }
                Ok(_) => self.status_message = "❌ Could not check printer status".to_string(),
                Err(e) => self.status_message = format!("❌ Cannot check status: {}", e),
            }
        } else {
            let output = Command::new("bash")
                .args(["-c", "lpstat -p ESC_POS_Linux_Printer 2>&1"])
                .output();

            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if stdout.contains("printer ESC_POS_Linux_Printer") {
                        self.status_message = format!("✅ Printer Status:\n{}", stdout);
                    } else {
                        self.status_message = format!("ℹ️ Linux virtual printer not installed (or not found). Output:\n{}", stdout);
                    }
                }
                Err(e) => self.status_message = format!("❌ Cannot execute lpstat: {}", e),
            }
        }
    }

    fn test_network_connection(&mut self) {
        let addr_res = "127.0.0.1:9100".parse::<SocketAddr>();
        if let Ok(addr) = addr_res {
            match TcpStream::connect_timeout(&addr, Duration::from_secs(2)) {
                Ok(_) => {
                    self.status_message = "✅ Connection to TCP port 9100 successful (Emulator is listening)".to_string();
                }
                Err(e) => {
                    self.status_message = format!("❌ Connection to TCP port 9100 failed: {}", e);
                }
            }
        }
    }
}
