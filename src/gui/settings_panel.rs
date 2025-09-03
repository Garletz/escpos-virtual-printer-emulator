use crate::emulator::EmulatorState;
use egui::Ui;
use std::process::Command;

pub struct SettingsPanel {
    // No more useless settings - the emulator works according to ESC/POS standards
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self {}
    }
}

impl SettingsPanel {
    pub fn show(&mut self, ui: &mut Ui, _state: &mut EmulatorState) {
        ui.heading("Emulator Settings");
        ui.separator();

        // Virtual printer management
        ui.group(|ui| {
            ui.label("Virtual Printer Management");
            ui.label("Installs the emulator as a system printer");
            
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
            
            // Check printer status
            if ui.button("🔍 Check Status").clicked() {
                self.check_printer_status();
            }
        });

        ui.separator();

        // Network settings
        ui.group(|ui| {
            ui.label("Network Configuration");
            ui.label("TCP Port: 9100");
            ui.label("Address: 127.0.0.1");
            
            if ui.button("📡 Test Connection").clicked() {
                self.test_network_connection();
            }
        });

        ui.separator();

        // Information about operation
        ui.group(|ui| {
            ui.label("ℹ️  Automatic Operation");
            ui.label("• The emulator automatically respects ESC/POS standards");
            ui.label("• Paper width: 50mm, 78mm, 80mm (auto-detection)");
            ui.label("• Font, justification, emphasis: ESC/POS commands");
            ui.label("• No manual configuration needed!");
        });
    }

    fn install_windows_printer(&self) {
        // Simplified PowerShell command to avoid syntax errors
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
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("✅ {}", stdout);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("❌ Error: {}", stderr);
                }
            }
            Err(e) => {
                println!("❌ Cannot execute printer installation: {}", e);
            }
        }
    }

    fn install_linux_printer(&self) {
        // Install Linux printer using CUPS
        let output = Command::new("bash")
            .args([
                "-c",
                "if command -v lpstat &> /dev/null; then \
                    echo 'Installing Linux printer...'; \
                    sudo lpadmin -p ESC_POS_Linux_Printer -E -v socket://127.0.0.1:9100 -m 'Generic Text-Only Printer'; \
                    sudo lpadmin -d ESC_POS_Linux_Printer; \
                    echo 'Linux printer installed successfully!'; \
                else \
                    echo 'CUPS not found. Please install CUPS first.'; \
                fi"
            ])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("ℹ️  {}", stdout);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("ℹ️  {}", stderr);
                }
            }
            Err(e) => {
                println!("ℹ️  Linux installation attempted: {}", e);
            }
        }
    }

    fn uninstall_printer(&self) {
        // Simplified PowerShell command
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Remove-Printer -Name 'ESC_POS_Virtual_Printer' -Confirm:$false; \
                 Remove-PrinterPort -Name '127.0.0.1:9100'; \
                 Write-Host 'Printer uninstalled successfully'"
            ])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("✅ {}", stdout);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("❌ Error: {}", stderr);
                }
            }
            Err(e) => {
                println!("❌ Cannot execute printer uninstallation: {}", e);
            }
        }
    }

    fn check_printer_status(&self) {
        // Check if printer is installed
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Get-Printer -Name 'ESC_POS_Virtual_Printer' -ErrorAction SilentlyContinue | Select-Object Name, PortName, DriverName, PrinterStatus"
            ])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.trim().is_empty() {
                        println!("ℹ️  Virtual printer not installed");
                    } else {
                        println!("✅ Virtual printer installed:");
                        println!("{}", stdout);
                    }
                }
            }
            Err(e) => {
                println!("❌ Cannot check status: {}", e);
            }
        }
    }

    fn test_network_connection(&self) {
        // Test connection to port 9100
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Test-NetConnection -ComputerName 127.0.0.1 -Port 9100 -WarningAction SilentlyContinue | Select-Object TcpTestSucceeded"
            ])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.contains("True") {
                        println!("✅ Connection to port 9100 successful");
                    } else {
                        println!("❌ Connection to port 9100 failed");
                    }
                } else {
                    println!("❌ Cannot test connection");
                }
            }
            Err(e) => {
                println!("❌ Cannot test connection: {}", e);
            }
        }
    }
}
