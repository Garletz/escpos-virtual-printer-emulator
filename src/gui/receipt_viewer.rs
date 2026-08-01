use crate::emulator::EmulatorState;
use crate::escpos::commands::{Font, Justification};
use crate::escpos::printer::{PaperWidth, PrinterState, ReceiptLine, TextLine};
use egui::{
    vec2, Color32, ColorImage, Frame, Layout, Margin, RichText, ScrollArea, Stroke, TextureHandle,
    TextureOptions, Ui,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptSubTab {
    RealPreview,
    RawText,
}

impl Default for ReceiptSubTab {
    fn default() -> Self {
        ReceiptSubTab::RealPreview
    }
}

pub struct ReceiptViewer {
    selected_sub_tab: ReceiptSubTab,
    show_paper_shadow: bool,
    zoom_factor: f32,
    bitmap_cache: HashMap<u64, TextureHandle>,
}

impl Default for ReceiptViewer {
    fn default() -> Self {
        Self {
            selected_sub_tab: ReceiptSubTab::RealPreview,
            show_paper_shadow: true,
            zoom_factor: 1.0,
            bitmap_cache: HashMap::new(),
        }
    }
}

fn hash_bytes(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

impl ReceiptViewer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut Ui, emulator_state: &Arc<Mutex<EmulatorState>>) {
        ui.vertical(|ui| {
            // Header & Sub-Tab Navigation
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.selected_sub_tab,
                    ReceiptSubTab::RealPreview,
                    "📄 Real Preview",
                );
                ui.selectable_value(
                    &mut self.selected_sub_tab,
                    ReceiptSubTab::RawText,
                    "📝 Raw Text",
                );

                ui.separator();

                // Paper width selector
                ui.label(RichText::new("📏 Paper Width:").strong());
                if let Ok(state) = emulator_state.try_lock() {
                    let current_pw = state.printer_state.paper_width.clone();
                    drop(state);

                    if ui.selectable_label(current_pw == PaperWidth::Width50mm, "50mm").clicked() {
                        if let Ok(mut state) = emulator_state.try_lock() {
                            state.set_paper_width(50);
                        }
                    }
                    if ui.selectable_label(current_pw == PaperWidth::Width78mm, "78mm").clicked() {
                        if let Ok(mut state) = emulator_state.try_lock() {
                            state.set_paper_width(78);
                        }
                    }
                    if ui.selectable_label(current_pw == PaperWidth::Width80mm, "80mm").clicked() {
                        if let Ok(mut state) = emulator_state.try_lock() {
                            state.set_paper_width(80);
                        }
                    }
                }

                ui.separator();

                if self.selected_sub_tab == ReceiptSubTab::RealPreview {
                    ui.checkbox(&mut self.show_paper_shadow, "Shadow");
                    ui.add(egui::Slider::new(&mut self.zoom_factor, 0.75..=1.40).text("Zoom"));
                }

                if ui.button("🗑️ Clear").on_hover_text("Clear current receipt buffer").clicked() {
                    if let Ok(mut state) = emulator_state.try_lock() {
                        state.clear_printer_buffer();
                    }
                    self.bitmap_cache.clear();
                }
            });

            ui.separator();

            // Render selected sub-tab
            match self.selected_sub_tab {
                ReceiptSubTab::RealPreview => {
                    Frame::none()
                        .fill(Color32::from_rgb(26, 28, 38)) // Dark workbench canvas
                        .inner_margin(Margin::same(16.0))
                        .show(ui, |ui| {
                            ScrollArea::both().show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    if let Ok(state) = emulator_state.try_lock() {
                                        self.render_dispenser_and_paper(ui, &state);
                                    } else {
                                        ui.colored_label(Color32::RED, "Loading printer state...");
                                    }
                                });
                            });
                        });
                }
                ReceiptSubTab::RawText => {
                    ScrollArea::both().show(ui, |ui| {
                        if let Ok(state) = emulator_state.try_lock() {
                            self.render_raw_text(ui, &state);
                        } else {
                            ui.colored_label(Color32::RED, "Loading printer state...");
                        }
                    });
                }
            }
        });
    }

    fn render_dispenser_and_paper(&mut self, ui: &mut Ui, state: &EmulatorState) {
        let printer_state = state.get_printer_state();
        let buffer = printer_state.get_buffer();

        // Calculate responsive canvas width based on thermal paper width & zoom
        let paper_mm_dots = printer_state.get_paper_width_dots();
        let base_paper_width_px = match printer_state.paper_width {
            PaperWidth::Width50mm => 340.0,
            PaperWidth::Width78mm => 440.0,
            PaperWidth::Width80mm => 490.0,
        };
        let paper_width = base_paper_width_px * self.zoom_factor;

        // 1. Thermal Printer Slot Dispenser Bar Header
        Frame::none()
            .fill(Color32::from_rgb(45, 48, 62))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(65, 70, 90)))
            .rounding(egui::Rounding {
                nw: 8.0,
                ne: 8.0,
                sw: 2.0,
                se: 2.0,
            })
            .inner_margin(Margin::symmetric(16.0, 8.0))
            .show(ui, |ui| {
                ui.set_width(paper_width);
                ui.horizontal(|ui| {
                    ui.colored_label(Color32::from_rgb(46, 204, 113), "●");
                    ui.label(
                        RichText::new(format!(
                            "PRINTER DISPENSER | {:?} ({} dots)",
                            printer_state.paper_width, paper_mm_dots
                        ))
                        .color(Color32::from_rgb(200, 205, 220))
                        .small()
                        .monospace(),
                    );
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{} Lines", buffer.len()))
                                .color(Color32::from_rgb(160, 165, 180))
                                .small(),
                        );
                    });
                });

                // Dispenser Slot Line
                ui.add_space(2.0);
                let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 4.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(15, 16, 22));
            });

        ui.add_space(2.0);

        // 2. Thermal Paper Canvas
        let paper_fill = Color32::from_rgb(250, 249, 245); // Thermal receipt off-white paper
        let paper_stroke = Stroke::new(1.0_f32, Color32::from_rgb(220, 218, 210));
        let shadow = if self.show_paper_shadow {
            egui::epaint::Shadow {
                extrusion: 10.0,
                color: Color32::from_black_alpha(80),
            }
        } else {
            egui::epaint::Shadow {
                extrusion: 0.0_f32,
                color: Color32::TRANSPARENT,
            }
        };

        Frame::none()
            .fill(paper_fill)
            .stroke(paper_stroke)
            .shadow(shadow)
            .rounding(egui::Rounding {
                nw: 0.0,
                ne: 0.0,
                sw: 4.0,
                se: 4.0,
            })
            .inner_margin(Margin::symmetric(20.0 * self.zoom_factor, 16.0 * self.zoom_factor))
            .show(ui, |ui| {
                ui.set_width(paper_width - 32.0);

                if buffer.is_empty() {
                    ui.add_space(30.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("📄 Thermal Paper Ready")
                                .color(Color32::from_rgb(120, 120, 130))
                                .size(16.0 * self.zoom_factor)
                                .strong(),
                        );
                        ui.label(
                            RichText::new("No print data received yet.\nSend raw ESC/POS commands via TCP port 9100 or virtual serial port.")
                                .color(Color32::from_rgb(150, 150, 160))
                                .small(),
                        );
                    });
                    ui.add_space(30.0);
                    return;
                }

                // Render buffer lines
                for (line_idx, line) in buffer.iter().enumerate() {
                    match line {
                        ReceiptLine::Text(text_line) => {
                            self.render_text_line(ui, text_line, printer_state);
                        }
                        ReceiptLine::Bitmap {
                            width_px,
                            height_px,
                            data,
                        } => {
                            self.render_bitmap_line(ui, *width_px, *height_px, data, paper_width);
                        }
                        ReceiptLine::Separator => {
                            self.render_cut_separator(ui, line_idx + 1);
                        }
                    }
                }

                // Bottom Paper Margin Feed
                ui.add_space(16.0 * self.zoom_factor);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("✂ ----------------- TEAR HERE ----------------- ✂")
                            .color(Color32::from_rgb(180, 180, 190))
                            .monospace()
                            .small(),
                    );
                });
            });
    }

    fn render_text_line(&mut self, ui: &mut Ui, text_line: &TextLine, _printer_state: &PrinterState) {
        if text_line.text.trim().is_empty() {
            ui.add_space(6.0 * self.zoom_factor);
            return;
        }

        let font_size = match text_line.font {
            Font::FontA => 13.5 * self.zoom_factor,
            Font::FontB => 11.5 * self.zoom_factor,
            Font::FontC => 10.0 * self.zoom_factor,
        };
        let font_size = if text_line.font_size > 12 {
            font_size * 1.3
        } else {
            font_size
        };

        let ink_color = Color32::from_rgb(18, 18, 22);

        let mut rt = RichText::new(&text_line.text)
            .monospace()
            .size(font_size)
            .color(ink_color);

        if text_line.emphasis {
            rt = rt.strong();
        }
        if text_line.underline {
            rt = rt.underline();
        }
        if text_line.italic {
            rt = rt.italics();
        }

        if text_line.text.contains("[ QR CODE:") {
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                Frame::none()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.5_f32, Color32::BLACK))
                    .inner_margin(Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(" 📱 QR CODE ")
                                .strong()
                                .monospace()
                                .color(Color32::BLACK),
                        );
                        ui.label(rt);
                    });
            });
            ui.add_space(6.0);
            return;
        }

        match text_line.justification {
            Justification::Left => {
                ui.horizontal(|ui| {
                    ui.label(rt);
                });
            }
            Justification::Center => {
                ui.vertical_centered(|ui| {
                    ui.label(rt);
                });
            }
            Justification::Right => {
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(rt);
                });
            }
        }
    }

    fn render_bitmap_line(
        &mut self,
        ui: &mut Ui,
        width_px: u32,
        height_px: u32,
        data: &[u8],
        paper_width: f32,
    ) {
        let cache_key = hash_bytes(data);

        let texture = self.bitmap_cache.entry(cache_key).or_insert_with(|| {
            let rgb_image = PrinterState::bitmap_to_rgb(width_px, height_px, data);
            let size = [rgb_image.width() as usize, rgb_image.height() as usize];
            let pixels: Vec<Color32> = rgb_image
                .pixels()
                .map(|p| Color32::from_rgb(p[0], p[1], p[2]))
                .collect();
            let color_image = ColorImage { size, pixels };
            ui.ctx().load_texture(
                format!("bitmap_{}", cache_key),
                color_image,
                TextureOptions::NEAREST,
            )
        });

        let max_display_w = paper_width - 60.0;
        let scale = (max_display_w / width_px as f32).min(1.5) * self.zoom_factor;
        let display_size = vec2(width_px as f32 * scale, height_px as f32 * scale);

        ui.vertical_centered(|ui| {
            ui.add_space(4.0);
            ui.image((texture.id(), display_size));
            ui.add_space(4.0);
        });
    }

    fn render_cut_separator(&mut self, ui: &mut Ui, _line_num: usize) {
        ui.add_space(8.0 * self.zoom_factor);
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 2.0), egui::Sense::hover());
            ui.painter().line_segment(
                [rect.left_top(), rect.right_top()],
                Stroke::new(1.0_f32, Color32::from_rgb(180, 180, 190)),
            );
        });
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("✂ --- CUT PAPER --- ✂")
                    .small()
                    .monospace()
                    .color(Color32::from_rgb(150, 150, 160)),
            );
        });
        ui.add_space(8.0 * self.zoom_factor);
    }

    fn render_raw_text(&mut self, ui: &mut Ui, state: &EmulatorState) {
        let printer_state = state.get_printer_state();
        let buffer = printer_state.get_buffer();

        if buffer.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No receipt data available");
            });
            return;
        }

        let max_chars = printer_state.paper_width.get_max_chars(printer_state.font_size);

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("📄 Paper: {:?}", printer_state.paper_width));
                ui.label(format!("🔤 Font: {:?}", printer_state.current_font));
                ui.label(format!("📐 Align: {:?}", printer_state.justification));
                if printer_state.codepage != 0 {
                    ui.label(format!("🌐 CP: {}", printer_state.codepage));
                }
            });

            ui.separator();

            for (line_num, line) in buffer.iter().enumerate() {
                match line {
                    ReceiptLine::Text(text_line) => {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{:03}", line_num + 1))
                                    .weak()
                                    .monospace(),
                            );
                            ui.label("│");
                            let mut rt = RichText::new(&text_line.text).monospace();
                            if text_line.emphasis {
                                rt = rt.strong();
                            }
                            ui.label(rt);
                        });
                    }
                    ReceiptLine::Bitmap { width_px, height_px, .. } => {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{:03}", line_num + 1))
                                    .weak()
                                    .monospace(),
                            );
                            ui.label("│");
                            ui.label(format!("[ RASTER BITMAP: {}x{} px ]", width_px, height_px));
                        });
                    }
                    ReceiptLine::Separator => {
                        let sep = "─".repeat(max_chars as usize);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{:03}", line_num + 1))
                                    .weak()
                                    .monospace(),
                            );
                            ui.label("│");
                            ui.label(&sep);
                        });
                    }
                }
            }

            ui.separator();
            ui.label("✂️ Cut line");
        });
    }
}
