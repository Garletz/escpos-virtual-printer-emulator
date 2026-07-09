use crate::emulator::EmulatorState;
use crate::gui::{CommandLog, ReceiptViewer, SettingsPanel};
use crate::networking::serial::SerialHandle;
use eframe::egui::{CentralPanel, TopBottomPanel};

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Receipt,
    Commands,
    Settings,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Receipt
    }
}

pub struct EscPosEmulatorApp {
    pub emulator_state: std::sync::Arc<tokio::sync::Mutex<EmulatorState>>,
    tokio_handle: tokio::runtime::Handle,
    selected_tab: Tab,
    receipt_viewer: ReceiptViewer,
    command_log: CommandLog,
    settings_panel: SettingsPanel,
    serial_handle: Option<SerialHandle>,
}

impl EscPosEmulatorApp {
    pub fn new(
        emulator_state: std::sync::Arc<tokio::sync::Mutex<EmulatorState>>,
        tokio_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            emulator_state,
            tokio_handle,
            selected_tab: Tab::Receipt,
            receipt_viewer: ReceiptViewer::new(),
            command_log: CommandLog::new(),
            settings_panel: SettingsPanel::default(),
            serial_handle: None,
        }
    }
}

impl eframe::App for EscPosEmulatorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.show(ctx);
    }
}

impl EscPosEmulatorApp {
    fn show(&mut self, ctx: &eframe::egui::Context) {
        TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::Receipt, "🖨️ Receipt");
                ui.selectable_value(&mut self.selected_tab, Tab::Commands, "📋 Commands");
                ui.selectable_value(&mut self.selected_tab, Tab::Settings, "⚙️ Settings");
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            match self.selected_tab {
                Tab::Receipt => {
                    self.receipt_viewer.show(ui, &self.emulator_state);
                }
                Tab::Commands => {
                    self.command_log.show(ui, &self.emulator_state);
                }
                Tab::Settings => {
                    self.settings_panel.show(
                        ui,
                        &mut self.serial_handle,
                        &self.emulator_state,
                        &self.tokio_handle,
                    );
                }
            }
        });
    }
}
