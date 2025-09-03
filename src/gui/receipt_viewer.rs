use crate::emulator::EmulatorState;
use egui::{ScrollArea, Ui};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ReceiptViewer {
    show_paper_edges: bool,
    show_grid: bool,
}

impl Default for ReceiptViewer {
    fn default() -> Self {
        Self {
            show_paper_edges: true,
            show_grid: false,
        }
    }
}

impl ReceiptViewer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut Ui, emulator_state: &Arc<Mutex<EmulatorState>>) {
        ui.heading("🖨️ Receipt Viewer");
        ui.separator();

        // Controls
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_paper_edges, "Show paper edges");
            ui.checkbox(&mut self.show_grid, "Show grid");
            
            if ui.button("🗑️ Clear").clicked() {
                if let Ok(mut state) = emulator_state.try_lock() {
                    state.clear_printer_buffer();
                }
            }
        });

        ui.separator();

        // Receipt display area
        ScrollArea::both().show(ui, |ui| {
            if let Ok(state) = emulator_state.try_lock() {
                self.render_receipt(ui, &state);
            } else {
                ui.label("Cannot load emulator state");
            }
        });
    }

    fn render_receipt(&self, ui: &mut Ui, state: &EmulatorState) {
        let printer_state = state.get_printer_state();
        let buffer = printer_state.get_buffer();
        
        if buffer.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No receipt data available");
                ui.label("Send ESC/POS commands to see the receipt here");
            });
            return;
        }

        // Paper simulation
        let paper_width = printer_state.get_paper_width_dots();
        let _max_width = (paper_width as f32 * 0.5) as f32; // Scale down for display
        
        ui.group(|ui| {
            // Paper header
            ui.horizontal(|ui| {
                ui.label(format!("📄 Paper: {:?}", printer_state.paper_width));
                ui.label(format!("🔤 Font: {:?}", printer_state.current_font));
                ui.label(format!("📐 Align: {:?}", printer_state.justification));
            });

            ui.separator();

            // Receipt content
            for (line_num, line) in buffer.iter().enumerate() {
                if !line.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{:03}", line_num + 1));
                        ui.label("│");
                        ui.label(line);
                    });
                } else {
                    ui.label(""); // Empty line
                }
            }

            // Paper footer
            ui.separator();
            ui.label("✂️ Cut line");
        });
    }
}
