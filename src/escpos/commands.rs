use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscPosCommand {
    // Commandes de base
    Text(String),
    NewLine,
    LineFeed,
    CarriageReturn,
    
    // Commandes de police
    SetFont(Font),
    SetFontSize(u32),
    
    // Commandes de formatage
    SetJustification(Justification),
    SetEmphasis(bool),
    SetUnderline(bool),
    SetItalic(bool),
    SetLineHeight(u32),
    
    // Commandes d'impression
    CutPaper,
    PrintImage(Vec<u8>),
    
    // Commandes de contrôle
    InitializePrinter,
    
    // Commandes inconnues
    Unknown(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Font {
    FontA,
    FontB,
    FontC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Justification {
    Left,
    Center,
    Right,
}
