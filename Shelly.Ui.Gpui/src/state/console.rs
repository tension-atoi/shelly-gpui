use crate::components::log_drawer::{LogEntry, OperationStatus};
use gpui::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleEvent {
    LogAppended(LogEntry),
    OperationStarted(String),
    OperationFinished(OperationStatus),
    Toggled(bool),
    LogsCleared,
    AutoScrollToggled(bool),
}

/// Modèle d'état découplé de la console d'opérations avec contrôle de défilement
pub struct ConsoleModel {
    pub logs: Vec<LogEntry>,
    pub status: OperationStatus,
    pub is_open: bool,
    pub auto_scroll: bool,
    pub height: f32,
    pub scroll_handle: ScrollHandle,
}

impl EventEmitter<ConsoleEvent> for ConsoleModel {}

impl ConsoleModel {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            status: OperationStatus::Idle,
            is_open: true,
            auto_scroll: true,
            height: 180.0,
            scroll_handle: ScrollHandle::new(),
        }
    }

    pub fn append_stdout(&mut self, line: &str, cx: &mut Context<Self>) {
        let entry = LogEntry::stdout(line);
        self.logs.push(entry.clone());
        if self.logs.len() > 2000 {
            self.logs.remove(0);
        }
        cx.emit(ConsoleEvent::LogAppended(entry));
        cx.notify();
    }

    pub fn append_stderr(&mut self, line: &str, cx: &mut Context<Self>) {
        let entry = LogEntry::stderr(line);
        self.logs.push(entry.clone());
        if self.logs.len() > 2000 {
            self.logs.remove(0);
        }
        cx.emit(ConsoleEvent::LogAppended(entry));
        cx.notify();
    }

    pub fn start_operation(&mut self, op_name: &str, cx: &mut Context<Self>) {
        self.status = OperationStatus::Running(op_name.to_string());
        self.is_open = true;
        self.append_stdout(&format!(">>> Démarrage de l'opération : {}", op_name), cx);
        cx.emit(ConsoleEvent::OperationStarted(op_name.to_string()));
        cx.notify();
    }

    pub fn finish_operation(&mut self, success: bool, message: &str, cx: &mut Context<Self>) {
        let status = if success {
            OperationStatus::Success(message.to_string())
        } else {
            OperationStatus::Error(message.to_string())
        };
        self.status = status.clone();
        let log_line = if success {
            format!(">>> Succès : {}", message)
        } else {
            format!(">>> Échec : {}", message)
        };
        if success {
            self.append_stdout(&log_line, cx);
        } else {
            self.append_stderr(&log_line, cx);
        }
        cx.emit(ConsoleEvent::OperationFinished(status));
        cx.notify();
    }

    pub fn toggle_drawer(&mut self, cx: &mut Context<Self>) {
        self.is_open = !self.is_open;
        cx.emit(ConsoleEvent::Toggled(self.is_open));
        cx.notify();
    }

    pub fn toggle_auto_scroll(&mut self, cx: &mut Context<Self>) {
        self.auto_scroll = !self.auto_scroll;
        cx.emit(ConsoleEvent::AutoScrollToggled(self.auto_scroll));
        cx.notify();
    }

    pub fn clear_logs(&mut self, cx: &mut Context<Self>) {
        self.logs.clear();
        self.status = OperationStatus::Idle;
        cx.emit(ConsoleEvent::LogsCleared);
        cx.notify();
    }
}

impl Default for ConsoleModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_console_model_defaults() {
        let console = ConsoleModel::new();
        assert!(console.logs.is_empty());
        assert_eq!(console.status, OperationStatus::Idle);
        assert!(console.is_open);
        assert!(console.auto_scroll);
        assert_eq!(console.height, 180.0);
    }
}
