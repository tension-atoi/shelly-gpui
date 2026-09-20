pub use crate::components::log_drawer::{LogEntry, OperationStatus};
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
    pub auto_open: bool,
    pub auto_scroll: bool,
    pub scroll_handle: ScrollHandle,
}

impl EventEmitter<ConsoleEvent> for ConsoleModel {}

impl ConsoleModel {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            status: OperationStatus::Idle,
            is_open: true,
            auto_open: true,
            auto_scroll: true,
            scroll_handle: ScrollHandle::new(),
        }
    }

    pub fn set_auto_open(&mut self, auto_open: bool) {
        self.auto_open = auto_open;
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status, OperationStatus::Running(_))
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

    /// Mutation d'état pure pour l'initialisation d'opération
    pub fn start_operation_state(&mut self, op_name: &str) -> bool {
        self.status = OperationStatus::Running(op_name.to_string());
        if self.auto_open {
            self.set_open_state(true)
        } else {
            false
        }
    }

    pub fn start_operation(&mut self, op_name: &str, cx: &mut Context<Self>) {
        let opened = self.start_operation_state(op_name);
        self.append_stdout(&format!(">>> Démarrage de l'opération : {}", op_name), cx);
        if opened {
            cx.emit(ConsoleEvent::Toggled(true));
        }
        cx.emit(ConsoleEvent::OperationStarted(op_name.to_string()));
        cx.notify();
    }

    /// Mutation d'état pure pour la terminaison d'opération
    pub fn finish_operation_state(
        &mut self,
        success: bool,
        message: &str,
    ) -> (OperationStatus, bool) {
        let status = if success {
            OperationStatus::Success(message.to_string())
        } else {
            OperationStatus::Error(message.to_string())
        };
        self.status = status.clone();
        let opened = if !success {
            self.set_open_state(true)
        } else {
            false
        };
        (status, opened)
    }

    pub fn finish_operation(&mut self, success: bool, message: &str, cx: &mut Context<Self>) {
        let (status, opened) = self.finish_operation_state(success, message);
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
        if opened {
            cx.emit(ConsoleEvent::Toggled(true));
        }
        cx.emit(ConsoleEvent::OperationFinished(status));
        cx.notify();
    }

    pub fn set_open_state(&mut self, open: bool) -> bool {
        if self.is_open != open {
            self.is_open = open;
            true
        } else {
            false
        }
    }

    pub fn toggle_drawer(&mut self, cx: &mut Context<Self>) {
        let new_state = !self.is_open;
        self.set_open(new_state, cx);
    }

    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.set_open_state(open) {
            cx.emit(ConsoleEvent::Toggled(open));
            cx.notify();
        }
    }

    pub fn toggle_auto_scroll(&mut self, cx: &mut Context<Self>) {
        self.auto_scroll = !self.auto_scroll;
        cx.emit(ConsoleEvent::AutoScrollToggled(self.auto_scroll));
        cx.notify();
    }

    /// Efface l'historique des lignes de log affichées sans altérer le statut du cycle de vie
    pub fn clear_history(&mut self) {
        self.logs.clear();
    }

    pub fn clear_logs(&mut self, cx: &mut Context<Self>) {
        self.clear_history();
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
        assert!(console.auto_open);
        assert!(console.auto_scroll);
    }

    #[test]
    fn test_clear_logs_preserves_lifecycle_status() {
        let mut console = ConsoleModel::new();
        console.status = OperationStatus::Running("upgrade".to_string());
        console.logs.push(LogEntry::stdout("Building package..."));

        assert_eq!(console.logs.len(), 1);
        assert_eq!(
            console.status,
            OperationStatus::Running("upgrade".to_string())
        );

        // Effacer les logs ne doit vider que l'historique sans falsifier le statut opérationnel
        console.clear_history();
        assert!(console.logs.is_empty());
        assert_eq!(
            console.status,
            OperationStatus::Running("upgrade".to_string())
        );

        // Vérification identique pour les états terminaux Success et Error
        console.status = OperationStatus::Success("Done".to_string());
        console.logs.push(LogEntry::stdout("Finished successfully"));
        console.clear_history();
        assert!(console.logs.is_empty());
        assert_eq!(console.status, OperationStatus::Success("Done".to_string()));
    }

    #[test]
    fn test_start_operation_honors_auto_open_setting() {
        let mut console = ConsoleModel::new();
        console.is_open = false;

        // 1. Avec auto_open = false, le démarrage d'une opération ne doit PAS ouvrir le tiroir
        console.auto_open = false;
        let opened = console.start_operation_state("pkg-build");
        assert!(!opened);
        assert!(
            !console.is_open,
            "Drawer must remain closed when auto_open is disabled"
        );
        assert_eq!(
            console.status,
            OperationStatus::Running("pkg-build".to_string())
        );

        // 2. Avec auto_open = true, le démarrage d'une opération doit ouvrir le tiroir
        console.is_open = false;
        console.auto_open = true;
        let opened = console.start_operation_state("pkg-install");
        assert!(opened);
        assert!(
            console.is_open,
            "Drawer must be logically open when auto_open is enabled"
        );
    }

    #[test]
    fn test_finish_operation_failure_auto_discloses_logical_open() {
        let mut console = ConsoleModel::new();
        // L'utilisateur ferme manuellement la console pendant l'opération
        console.is_open = false;
        console.status = OperationStatus::Running("download".to_string());

        // Si l'opération échoue, le tiroir DOIT être ouvert logiquement (ConsoleModel.is_open = true)
        let (status, opened) = console.finish_operation_state(false, "Network failure");
        assert!(opened, "Failure must trigger logical disclosure");
        assert!(
            console.is_open,
            "ConsoleModel.is_open must be true on failure"
        );
        assert_eq!(
            status,
            OperationStatus::Error("Network failure".to_string())
        );

        // Si l'opération réussit alors qu'elle était fermée, elle reste fermée
        console.is_open = false;
        let (status, opened) = console.finish_operation_state(true, "All packages up to date");
        assert!(!opened);
        assert!(!console.is_open);
        assert_eq!(
            status,
            OperationStatus::Success("All packages up to date".to_string())
        );
    }

    #[test]
    fn test_console_is_running_authority() {
        let mut console = ConsoleModel::new();
        assert!(!console.is_running());

        console.start_operation_state("upgrade");
        assert!(console.is_running());

        console.finish_operation_state(true, "Upgrade complete");
        assert!(!console.is_running());

        console.start_operation_state("install");
        assert!(console.is_running());

        console.finish_operation_state(false, "Failed");
        assert!(!console.is_running());
    }
}
