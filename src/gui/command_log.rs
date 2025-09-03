use crate::emulator::EmulatorState;
use egui::{ScrollArea, Ui};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CommandLog {
    show_timestamps: bool,
    show_raw_data: bool,
    max_display_lines: usize,
    filter_text: String,
}

impl Default for CommandLog {
    fn default() -> Self {
        Self {
            show_timestamps: true,
            show_raw_data: false,
            max_display_lines: 1000,
            filter_text: String::new(),
        }
    }
}

impl CommandLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut Ui, emulator_state: &Arc<Mutex<EmulatorState>>) {
        ui.heading("📋 Command Log");
        ui.separator();

        // Controls
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_timestamps, "Timestamps");
            ui.checkbox(&mut self.show_raw_data, "Raw data");
            
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter_text);
            
            if ui.button("🗑️ Clear").clicked() {
                if let Ok(mut state) = emulator_state.try_lock() {
                    state.clear_history();
                }
            }
        });

        ui.separator();

        // Log area
        ScrollArea::vertical().show(ui, |ui| {
            if let Ok(state) = emulator_state.try_lock() {
                self.render_command_list(ui, &state);
            } else {
                ui.label("Cannot load emulator state");
            }
        });
    }

    fn render_command_list(&self, ui: &mut Ui, state: &EmulatorState) {
        let history = state.get_command_history();
        
        if history.is_empty() {
            ui.label("No commands received");
            return;
        }

        // Apply filter
        let filtered_commands: Vec<_> = history.iter()
            .filter(|entry| {
                if self.filter_text.is_empty() {
                    return true;
                }
                
                match &entry.command {
                    crate::escpos::commands::EscPosCommand::Text(text) => {
                        text.to_lowercase().contains(&self.filter_text.to_lowercase())
                    }
                    _ => {
                        format!("{:?}", entry.command).to_lowercase().contains(&self.filter_text.to_lowercase())
                    }
                }
            })
            .collect();

        // Limit displayed lines
        let display_commands: Vec<_> = filtered_commands.iter()
            .rev() // Most recent first
            .take(self.max_display_lines)
            .collect();

        let display_count = display_commands.len();
        for entry in &display_commands {
            self.render_command_entry(ui, entry);
        }

        // Statistics
        ui.separator();
        ui.label(format!("Total: {} commands | Displayed: {} | Filtered: {}", 
            history.len(), 
            display_count,
            filtered_commands.len()
        ));
    }

    fn render_command_entry(&self, ui: &mut Ui, entry: &crate::emulator::CommandEntry) {
        ui.group(|ui| {
            // Timestamp
            if self.show_timestamps {
                if let Ok(duration) = entry.timestamp.duration_since(std::time::UNIX_EPOCH) {
                    let time_str = if duration.as_secs() < 60 {
                        format!("{}s", duration.as_secs())
                    } else if duration.as_secs() < 3600 {
                        format!("{}m {}s", duration.as_secs() / 60, duration.as_secs() % 60)
                    } else {
                        format!("{}h {}m", duration.as_secs() / 3600, (duration.as_secs() % 3600) / 60)
                    };
                    
                    ui.label(format!("⏰ {}", time_str));
                }
            }

            // Command
            let command_text = match &entry.command {
                crate::escpos::commands::EscPosCommand::Text(text) => {
                    format!("📝 {}", text)
                }
                crate::escpos::commands::EscPosCommand::NewLine => {
                    "↵ New line".to_string()
                }
                crate::escpos::commands::EscPosCommand::SetFont(font) => {
                    format!("🔤 Font: {:?}", font)
                }
                crate::escpos::commands::EscPosCommand::SetJustification(just) => {
                    format!("📐 Justification: {:?}", just)
                }
                crate::escpos::commands::EscPosCommand::SetEmphasis(enabled) => {
                    format!("💪 Emphasis: {}", if *enabled { "ON" } else { "OFF" })
                }
                crate::escpos::commands::EscPosCommand::SetUnderline(enabled) => {
                    format!("➖ Underline: {}", if *enabled { "ON" } else { "OFF" })
                }
                crate::escpos::commands::EscPosCommand::SetItalic(enabled) => {
                    format!("📝 Italic: {}", if *enabled { "ON" } else { "OFF" })
                }
                crate::escpos::commands::EscPosCommand::CutPaper => {
                    "✂️ Paper cut".to_string()
                }
                crate::escpos::commands::EscPosCommand::PrintImage(_) => {
                    "🖼️ Image".to_string()
                }
                crate::escpos::commands::EscPosCommand::SetLineHeight(height) => {
                    format!("📏 Line height: {}", height)
                }
                crate::escpos::commands::EscPosCommand::SetFontSize(size) => {
                    format!("🔤 Font size: {}", size)
                }
                crate::escpos::commands::EscPosCommand::Unknown(_) => {
                    "❓ Unknown command".to_string()
                }
                _ => {
                    format!("⚙️ {:?}", entry.command)
                }
            };

            ui.label(command_text);

            // Raw data if enabled
            if self.show_raw_data && !entry.raw_data.is_empty() {
                let hex_data: String = entry.raw_data.iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                
                ui.label(format!("🔢 Data: {}", hex_data));
            }
        });
    }
}
