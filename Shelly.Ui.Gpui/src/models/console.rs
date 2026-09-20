use crate::components::log_drawer::{LogEntry, OperationStatus};
use gpui::{Context, EventEmitter};

#[derive(Debug, Clone)]
pub enum ConsoleEvent {
    LogAppended(LogEntry),
    OperationStarted(String),
    OperationFinished { success: bool, message: String },
    Toggled(bool),
    LogsCleared,
    AutoScrollToggled(bool),
}

pub struct ConsoleModel {
    pub logs: Vec<LogEntry>,
    pub status: OperationStatus,
    pub is_open: bool,
    pub auto_scroll: bool,
    pub height: f32,
}

impl EventEmitter<ConsoleEvent> for ConsoleModel {}

impl Default for ConsoleModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleModel {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            status: OperationStatus::Idle,
            is_open: false,
            auto_scroll: true,
            height: 200.0,
        }
    }

    pub fn append_stdout(&mut self, line: impl Into<String>, cx: &mut Context<Self>) {
        let entry = LogEntry::stdout(line);
        self.logs.push(entry.clone());
        cx.emit(ConsoleEvent::LogAppended(entry));
        cx.notify();
    }

    pub fn append_stderr(&mut self, line: impl Into<String>, cx: &mut Context<Self>) {
        let entry = LogEntry::stderr(line);
        self.logs.push(entry.clone());
        cx.emit(ConsoleEvent::LogAppended(entry));
        cx.notify();
    }

    pub fn start_operation(&mut self, title: impl Into<String>, cx: &mut Context<Self>) {
        let t = title.into();
        self.status = OperationStatus::Running(t.clone());
        self.is_open = true; // Auto-open when operation starts
        cx.emit(ConsoleEvent::OperationStarted(t));
        cx.notify();
    }

    pub fn finish_operation(
        &mut self,
        success: bool,
        message: impl Into<String>,
        cx: &mut Context<Self>,
    ) {
        let msg = message.into();
        if success {
            self.status = OperationStatus::Success(msg.clone());
        } else {
            self.status = OperationStatus::Error(msg.clone());
            self.is_open = true; // Auto-open on failure
        }
        cx.emit(ConsoleEvent::OperationFinished {
            success,
            message: msg,
        });
        cx.notify();
    }

    pub fn clear_logs(&mut self, cx: &mut Context<Self>) {
        self.logs.clear();
        cx.emit(ConsoleEvent::LogsCleared);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_model_default() {
        let model = ConsoleModel::new();
        assert!(model.logs.is_empty());
        assert_eq!(model.status, OperationStatus::Idle);
        assert!(!model.is_open);
        assert!(model.auto_scroll);
        assert_eq!(model.height, 200.0);
    }
}
