use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscPosCommand {
    // Basic commands
    Text(String),
    NewLine,
    LineFeed,
    CarriageReturn,

    // Font commands
    SetFont(Font),
    SetFontSize(u32),

    // Formatting commands
    SetJustification(Justification),
    SetEmphasis(bool),
    SetUnderline(bool),
    SetItalic(bool),
    SetLineHeight(u32),

    // Print commands
    CutPaper,
    PrintImage(Vec<u8>),
    /// Raster bitmap with width (bytes per row) and height (rows)
    PrintRasterImage { width_bytes: u16, height: u16, data: Vec<u8> },

    // Codepage selection (ESC t n)
    SetCodepage(u8),

    // Control commands
    InitializePrinter,

    // Unknown commands
    Unknown(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Font {
    FontA,
    FontB,
    FontC,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Justification {
    Left,
    Center,
    Right,
}
