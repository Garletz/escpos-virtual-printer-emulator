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
        // Calculer le nombre max de caractères selon la taille de police
        match font_size {
            8..=12 => dots / 8,   // Police normale
            13..=16 => dots / 10, // Police moyenne
            17..=24 => dots / 12, // Police grande
            _ => dots / 8,        // Par défaut
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterState {
    pub paper_width: PaperWidth,
    pub current_font: Font,
    pub justification: Justification,
    pub emphasis: bool,
    pub underline: bool,
    pub italic: bool,
    pub buffer: Vec<String>,
    pub line_height: u32,
    pub font_size: u32,
    pub dpi: u32,
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
                // Simuler la coupe du papier
                self.add_separator();
            }
            EscPosCommand::PrintImage(_image_data) => {
                // TODO: Implémenter l'affichage d'image
                self.add_text("[IMAGE]");
            }
            EscPosCommand::SetLineHeight(height) => {
                self.line_height = *height;
            }
            EscPosCommand::SetFontSize(size) => {
                self.font_size = *size;
            }
            EscPosCommand::Unknown(_) => {
                // Ignorer les commandes inconnues
            }
            _ => {
                // Ignorer les autres commandes
            }
        }
    }

    fn add_text(&mut self, text: &str) {
        if let Some(last_line) = self.buffer.last_mut() {
            // Vérifier si le texte dépasse la largeur du papier
            let max_chars = self.paper_width.get_max_chars(self.font_size);
            let current_length = last_line.chars().count();
            
            if current_length + text.chars().count() > max_chars as usize {
                // Le texte dépasse, créer une nouvelle ligne
                self.add_new_line();
                self.buffer.last_mut().unwrap().push_str(text);
            } else {
                last_line.push_str(text);
            }
        } else {
            self.buffer.push(text.to_string());
        }
    }

    fn add_new_line(&mut self) {
        self.buffer.push(String::new());
    }

    fn add_separator(&mut self) {
        let max_chars = self.paper_width.get_max_chars(self.font_size);
        let separator = "-".repeat(max_chars as usize);
        self.buffer.push(separator);
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    pub fn get_buffer(&self) -> &[String] {
        &self.buffer
    }

    pub fn get_paper_width_dots(&self) -> u32 {
        self.paper_width.get_width_dots()
    }

    pub fn get_printing_width_dots(&self) -> u32 {
        // Largeur d'impression = largeur du papier - marges
        let dots = self.paper_width.get_width_dots();
        dots.saturating_sub(30) // 8mm = ~30 dots de marges
    }

    pub fn render_receipt(&self) -> RgbImage {
        let width = self.get_paper_width_dots() as u32;
        let height = self.calculate_total_height();
        
        let mut image = ImageBuffer::new(width, height);
        
        // Remplir avec du blanc
        for pixel in image.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }
        
        // Rendu du texte (simplifié)
        let mut y_offset = 0;
        for line in &self.buffer {
            if !line.is_empty() {
                self.render_text_line(&mut image, line, y_offset);
            }
            y_offset += self.line_height;
        }
        
        image
    }

    fn render_text_line(&self, _image: &mut RgbImage, _text: &str, _y_offset: u32) {
        // TODO: Implémenter le rendu du texte
        // Pour l'instant, c'est juste un placeholder
    }

    pub fn calculate_total_height(&self) -> u32 {
        self.buffer.len() as u32 * self.line_height
    }

    // Nouvelles méthodes pour les paramètres
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
