use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum LogStreamEvent {
    Line(String),
    ErrorLine(String),
    Finished(bool, Option<i32>),
}

pub struct ProcessRunner;

impl ProcessRunner {
    /// Exécute une commande de lecture et retourne la chaîne JSON complète
    pub async fn run_json_command(binary_path: &str, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new(binary_path);
        cmd.args(args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd.output().await.with_context(|| {
            format!("Échec de l'exécution de {} avec les arguments {:?}", binary_path, args)
        })?;

        if !output.status.success() {
            let err_str = String::from_utf8_lossy(&output.stderr);
            log::warn!("La commande a retourné une erreur (code {:?}): {}", output.status.code(), err_str);
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Sortie stdout non-UTF8 lors de l'exécution de la commande")?;
        Ok(stdout)
    }

    /// Exécute une opération de mutation (install, remove, upgrade) en streamant chaque ligne
    pub fn spawn_streaming_operation(
        binary_path: String,
        args: Vec<String>,
        tx: mpsc::UnboundedSender<LogStreamEvent>,
    ) {
        tokio::spawn(async move {
            let mut cmd = Command::new(&binary_path);
            cmd.args(&args);
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    let _ = tx.send(LogStreamEvent::ErrorLine(format!("Erreur au lancement du processus : {}", e)));
                    let _ = tx.send(LogStreamEvent::Finished(false, None));
                    return;
                }
            };

            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            let tx_out = tx.clone();
            let stdout_task = tokio::spawn(async move {
                if let Some(stdout) = stdout {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        if tx_out.send(LogStreamEvent::Line(line)).is_err() {
                            break;
                        }
                    }
                }
            });

            let tx_err = tx.clone();
            let stderr_task = tokio::spawn(async move {
                if let Some(stderr) = stderr {
                    let mut reader = BufReader::new(stderr).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        if tx_err.send(LogStreamEvent::ErrorLine(line)).is_err() {
                            break;
                        }
                    }
                }
            });

            let _ = tokio::join!(stdout_task, stderr_task);

            match child.wait().await {
                Ok(status) => {
                    let success = status.success();
                    let code = status.code();
                    let _ = tx.send(LogStreamEvent::Finished(success, code));
                }
                Err(e) => {
                    let _ = tx.send(LogStreamEvent::ErrorLine(format!("Erreur lors de l'attente du processus : {}", e)));
                    let _ = tx.send(LogStreamEvent::Finished(false, None));
                }
            }
        });
    }
}
