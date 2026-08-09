use crate::escpos::commands::{EscPosCommand, Font, Justification};
use image::{ImageBuffer, Rgb, RgbImage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaperWidth {
    Width50mm,  // 384 dots (48 chars normal font)
    Width78mm,  // 576 dots (72 chars normal font)
    Width80mm,  // 640 dots (80 chars normal font)
}

impl PaperWidth {
    pub fn get_width_dots(&self) -> u32 {
        match self {
            PaperWidth::Width50mm => 384,
            PaperWidth::Width78mm => 576,
            PaperWidth::Width80mm => 640,
        }
    }

    pub fn get_max_chars(&self, font_size: u32) -> u32 {
        let dots = self.get_width_dots();
        match font_size {
            8..=12 => dots / 8,
            13..=16 => dots / 10,
            17..=24 => dots / 12,
            _ => dots / 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLine {
    pub text: String,
    pub font: Font,
    pub justification: Justification,
    pub emphasis: bool,
    pub underline: bool,
    pub italic: bool,
    pub font_size: u32,
}

/// A single line element in the receipt buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReceiptLine {
    Text(TextLine),
    /// Monochrome bitmap: width in pixels, height in pixels, 1-bit-per-pixel packed data
    Bitmap { width_px: u32, height_px: u32, data: Vec<u8> },
    Separator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterState {
    pub paper_width: PaperWidth,
    pub current_font: Font,
    pub justification: Justification,
    pub emphasis: bool,
    pub underline: bool,
    pub italic: bool,
    pub buffer: Vec<ReceiptLine>,
    pub line_height: u32,
    pub font_size: u32,
    pub dpi: u32,
    pub codepage: u8,
}

impl PrinterState {
    pub fn new() -> Self {
        Self {
            paper_width: PaperWidth::Width80mm,
            current_font: Font::FontA,
            justification: Justification::Left,
            emphasis: false,
            underline: false,
            italic: false,
            buffer: Vec::new(),
            line_height: 24,
            font_size: 12,
            dpi: 180,
            codepage: 0,
        }
    }

    pub fn process_command(&mut self, command: &EscPosCommand) {
        match command {
            EscPosCommand::Text(text) => {
                self.add_text(text);
            }
            EscPosCommand::NewLine => {
                self.add_new_line();
            }
            EscPosCommand::SetFont(font) => {
                self.current_font = font.clone();
            }
            EscPosCommand::SetJustification(justification) => {
                self.justification = justification.clone();
            }
            EscPosCommand::SetEmphasis(enabled) => {
                self.emphasis = *enabled;
            }
            EscPosCommand::SetUnderline(enabled) => {
                self.underline = *enabled;
            }
            EscPosCommand::SetItalic(enabled) => {
                self.italic = *enabled;
            }
            EscPosCommand::CutPaper => {
                self.add_separator();
            }
            EscPosCommand::PrintImage(_image_data) => {
                // ESC * bit image — store as text placeholder for now
                self.add_text("[BIT IMAGE]");
            }
            EscPosCommand::PrintRasterImage { width_bytes, height, data } => {
                // GS v 0 raster image — width_bytes is bytes per row, each byte = 8 pixels
                let width_px = *width_bytes as u32 * 8;
                let height_px = *height as u32;
                self.buffer.push(ReceiptLine::Bitmap {
                    width_px,
                    height_px,
                    data: data.clone(),
                });
            }
            EscPosCommand::SetCodepage(cp) => {
                self.codepage = *cp;
            }
            EscPosCommand::SetLineHeight(height) => {
                self.line_height = *height;
            }
            EscPosCommand::SetFontSize(size) => {
                self.font_size = *size;
            }
            EscPosCommand::Unknown(_) => {}
            _ => {}
        }
    }

    fn add_text(&mut self, text: &str) {
        let current_line_style = TextLine {
            text: String::new(),
            font: self.current_font.clone(),
            justification: self.justification.clone(),
            emphasis: self.emphasis,
            underline: self.underline,
            italic: self.italic,
            font_size: self.font_size,
        };

        if let Some(ReceiptLine::Text(last_line)) = self.buffer.last_mut() {
            let matches_style = last_line.font == current_line_style.font
                && last_line.justification == current_line_style.justification
                && last_line.emphasis == current_line_style.emphasis
                && last_line.underline == current_line_style.underline
                && last_line.italic == current_line_style.italic
                && last_line.font_size == current_line_style.font_size;

            if matches_style {
                let max_chars = self.paper_width.get_max_chars(self.font_size);
                let current_length = last_line.text.chars().count();

                if current_length + text.chars().count() > max_chars as usize {
                    let mut new_line = current_line_style;
                    new_line.text.push_str(text);
                    self.buffer.push(ReceiptLine::Text(new_line));
                } else {
                    last_line.text.push_str(text);
                }
                return;
            }
        }

        let mut new_line = current_line_style;
        new_line.text.push_str(text);
        self.buffer.push(ReceiptLine::Text(new_line));
    }

    fn add_new_line(&mut self) {
        let line = TextLine {
            text: String::new(),
            font: self.current_font.clone(),
            justification: self.justification.clone(),
            emphasis: self.emphasis,
            underline: self.underline,
            italic: self.italic,
            font_size: self.font_size,
        };
        self.buffer.push(ReceiptLine::Text(line));
    }

    fn add_separator(&mut self) {
        self.buffer.push(ReceiptLine::Separator);
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    pub fn get_buffer(&self) -> &[ReceiptLine] {
        &self.buffer
    }

    pub fn get_paper_width_dots(&self) -> u32 {
        self.paper_width.get_width_dots()
    }

    pub fn get_printing_width_dots(&self) -> u32 {
        let dots = self.paper_width.get_width_dots();
        dots.saturating_sub(30)
    }

    /// Convert a monochrome 1bpp bitmap to an RGB image for display
    pub fn bitmap_to_rgb(width_px: u32, height_px: u32, data: &[u8]) -> RgbImage {
        let mut image = ImageBuffer::new(width_px, height_px);
        // Fill white
        for pixel in image.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }

        let bytes_per_row = (width_px + 7) / 8;
        for y in 0..height_px {
            for x in 0..width_px {
                let byte_idx = (y * bytes_per_row + x / 8) as usize;
                let bit_idx = 7 - (x % 8);
                if byte_idx < data.len() {
                    if (data[byte_idx] >> bit_idx) & 1 == 1 {
                        image.put_pixel(x, y, Rgb([0, 0, 0])); // Black pixel
                    }
                }
            }
        }
        image
    }

    pub fn render_receipt(&self) -> RgbImage {
        let width = self.get_paper_width_dots();
        let height = self.calculate_total_height();

        let mut image = ImageBuffer::new(width, height);
        for pixel in image.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }

        image
    }

    pub fn calculate_total_height(&self) -> u32 {
        let mut h = 0u32;
        for line in &self.buffer {
            match line {
                ReceiptLine::Text(_) => h += self.line_height,
                ReceiptLine::Bitmap { height_px, .. } => h += height_px,
                ReceiptLine::Separator => h += self.line_height,
            }
        }
        h.max(1)
    }

    pub fn set_paper_width(&mut self, width: PaperWidth) {
        self.paper_width = width;
    }

    pub fn set_line_height(&mut self, height: u32) {
        self.line_height = height;
    }

    pub fn set_font_size(&mut self, size: u32) {
        self.font_size = size;
    }
}
