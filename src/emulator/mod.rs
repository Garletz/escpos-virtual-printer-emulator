use crate::escpos::commands::EscPosCommand;
use crate::escpos::printer::{PrinterState, PaperWidth};
use std::collections::VecDeque;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorState {
    pub printer_state: PrinterState,
    pub command_history: VecDeque<CommandEntry>,
    pub max_history_size: usize,
    pub start_time: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    pub timestamp: SystemTime,
    pub command: EscPosCommand,
    pub raw_data: Vec<u8>,
}

impl EmulatorState {
    pub fn new() -> Self {
        Self {
            printer_state: PrinterState::new(),
            command_history: VecDeque::new(),
            max_history_size: 1000,
            start_time: SystemTime::now(),
        }
    }

    pub fn process_command(&mut self, command: &EscPosCommand) {
        let entry = CommandEntry {
            timestamp: SystemTime::now(),
            command: command.clone(),
            raw_data: vec![], // TODO: Stocker les données brutes
        };

        self.command_history.push_back(entry);

        while self.command_history.len() > self.max_history_size {
            self.command_history.pop_front();
        }

        self.printer_state.process_command(command);
    }

    pub fn get_command_history(&self) -> &VecDeque<CommandEntry> {
        &self.command_history
    }

    pub fn clear_history(&mut self) {
        self.command_history.clear();
    }

    pub fn clear_printer_buffer(&mut self) {
        self.printer_state.clear_buffer();
    }

    pub fn get_printer_state(&self) -> &PrinterState {
        &self.printer_state
    }

    pub fn get_status_summary(&self) -> StatusSummary {
        StatusSummary {
            paper_width: format!("{:?}", self.printer_state.paper_width),
            current_font: format!("{:?}", self.printer_state.current_font),
            justification: format!("{:?}", self.printer_state.justification),
            emphasis: self.printer_state.emphasis,
            underline: self.printer_state.underline,
            italic: self.printer_state.italic,
            buffer_lines: self.printer_state.buffer.len(),
            command_count: self.command_history.len(),
            dpi: self.printer_state.dpi,
        }
    }

    pub fn set_paper_width(&mut self, width_mm: u32) {
        let paper_width = match width_mm {
            50 => PaperWidth::Width50mm,
            78 => PaperWidth::Width78mm,
            80 => PaperWidth::Width80mm,
            _ => PaperWidth::Width80mm,
        };
        self.printer_state.set_paper_width(paper_width);
    }

    pub fn set_line_height(&mut self, height: u32) {
        self.printer_state.set_line_height(height);
    }

    pub fn set_font_size(&mut self, size: u32) {
        self.printer_state.set_font_size(size);
    }
}

#[derive(Debug)]
pub struct StatusSummary {
    pub paper_width: String,
    pub current_font: String,
    pub justification: String,
    pub emphasis: bool,
    pub underline: bool,
    pub italic: bool,
    pub buffer_lines: usize,
    pub command_count: usize,
    pub dpi: u32,
}
