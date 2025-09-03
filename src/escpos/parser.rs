use crate::escpos::commands::{EscPosCommand, Font, Justification};
use anyhow::Result;

pub struct EscPosParser {
    buffer: Vec<u8>,
}

impl EscPosParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    pub fn parse_stream(&mut self, data: &[u8]) -> Result<Vec<EscPosCommand>> {
        self.buffer.extend_from_slice(data);
        let mut commands = Vec::new();
        let mut i = 0;

        while i < self.buffer.len() {
            match self.buffer[i] {
                // Commandes de base
                b'\n' => {
                    commands.push(EscPosCommand::NewLine);
                    i += 1;
                }
                b'\r' => {
                    commands.push(EscPosCommand::CarriageReturn);
                    i += 1;
                }
                b'\x1B' => {
                    // Séquence ESC
                    if i + 1 < self.buffer.len() {
                        let command = self.parse_esc_command(&self.buffer[i..])?;
                        if let Some(cmd) = command {
                            commands.push(cmd);
                        }
                        i += 2; // ESC + commande
                    } else {
                        break; // Attendre plus de données
                    }
                }
                _ => {
                    // Texte normal
                    let text_start = i;
                    while i < self.buffer.len() && self.buffer[i] != b'\x1B' && self.buffer[i] != b'\n' && self.buffer[i] != b'\r' {
                        i += 1;
                    }
                    if i > text_start {
                        let text = String::from_utf8_lossy(&self.buffer[text_start..i]).to_string();
                        if !text.is_empty() {
                            commands.push(EscPosCommand::Text(text));
                        }
                    }
                }
            }
        }

        // Nettoyer le buffer des données traitées
        if i > 0 {
            self.buffer.drain(0..i);
        }

        Ok(commands)
    }

    fn parse_esc_command(&self, data: &[u8]) -> Result<Option<EscPosCommand>> {
        if data.len() < 2 {
            return Ok(None);
        }

        match data[1] {
            // Initialisation de l'imprimante
            b'@' => Ok(Some(EscPosCommand::InitializePrinter)),

            // Sélection de la police
            b'M' => {
                if data.len() >= 3 {
                    let font = match data[2] {
                        0 => Font::FontA,
                        1 => Font::FontB,
                        2 => Font::FontC,
                        _ => Font::FontA,
                    };
                    Ok(Some(EscPosCommand::SetFont(font)))
                } else {
                    Ok(Some(EscPosCommand::SetFont(Font::FontA)))
                }
            }

            // Justification
            b'a' => {
                if data.len() >= 3 {
                    let justification = match data[2] {
                        0 => Justification::Left,
                        1 => Justification::Center,
                        2 => Justification::Right,
                        _ => Justification::Left,
                    };
                    Ok(Some(EscPosCommand::SetJustification(justification)))
                } else {
                    Ok(Some(EscPosCommand::SetJustification(Justification::Left)))
                }
            }

            // Emphase
            b'E' => Ok(Some(EscPosCommand::SetEmphasis(true))),
            b'F' => Ok(Some(EscPosCommand::SetEmphasis(false))),

            // Soulignement
            b'-' => {
                if data.len() >= 3 {
                    let underline = data[2];
                    Ok(Some(EscPosCommand::SetUnderline(underline != 0)))
                } else {
                    Ok(Some(EscPosCommand::SetUnderline(false)))
                }
            }

            // Italique
            b'4' => Ok(Some(EscPosCommand::SetItalic(true))),
            b'5' => Ok(Some(EscPosCommand::SetItalic(false))),

            // Hauteur de ligne
            b'3' => {
                if data.len() >= 3 {
                    let height = data[2] as u32;
                    Ok(Some(EscPosCommand::SetLineHeight(height)))
                } else {
                    Ok(Some(EscPosCommand::SetLineHeight(24)))
                }
            }

            // Taille de police
            b'!' => {
                if data.len() >= 3 {
                    let size = data[2] as u32;
                    Ok(Some(EscPosCommand::SetFontSize(size)))
                } else {
                    Ok(Some(EscPosCommand::SetFontSize(12)))
                }
            }

            // Coupe du papier
            b'm' => Ok(Some(EscPosCommand::CutPaper)),
            b'i' => Ok(Some(EscPosCommand::CutPaper)),

            // Alimentation du papier
            b'J' => {
                if data.len() >= 3 {
                    let _units = data[2] as u32;
                    Ok(Some(EscPosCommand::LineFeed))
                } else {
                    Ok(Some(EscPosCommand::LineFeed))
                }
            }

            // Image raster (simplifié)
            b'*' => {
                if data.len() >= 6 {
                    let width = ((data[2] as u16) << 8) | (data[3] as u16);
                    let height = ((data[4] as u16) << 8) | (data[5] as u16);
                    let image_data = if data.len() >= 6 + (width * height / 8) as usize {
                        data[6..6 + (width * height / 8) as usize].to_vec()
                    } else {
                        vec![]
                    };
                    Ok(Some(EscPosCommand::PrintImage(image_data)))
                } else {
                    Ok(Some(EscPosCommand::PrintImage(vec![])))
                }
            }

            // Commande inconnue
            _ => {
                let unknown_data = data.to_vec();
                Ok(Some(EscPosCommand::Unknown(unknown_data)))
            }
        }
    }
}

impl Default for EscPosParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EscPosParser {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
        }
    }
}
