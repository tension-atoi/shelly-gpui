use crate::control::protocol::{ControlCommand, ControlRequest, ControlResponse};
use anyhow::{Context, Result};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

pub struct ControlSocket;

impl ControlSocket {
    pub fn socket_dir() -> PathBuf {
        dirs::runtime_dir()
            .unwrap_or_else(|| {
                let uid = std::fs::metadata("/proc/self")
                    .map(|m| std::os::unix::fs::MetadataExt::uid(&m))
                    .unwrap_or(1000);
                PathBuf::from(format!("/run/user/{}", uid))
            })
            .join("shelly-gpui")
    }

    pub fn socket_path() -> PathBuf {
        Self::socket_dir().join("control.sock")
    }

    /// Checks whether an existing GUI instance is running and responding on the control socket
    pub async fn is_instance_running() -> bool {
        let path = Self::socket_path();
        if !path.exists() {
            return false;
        }

        match tokio::time::timeout(
            std::time::Duration::from_millis(150),
            UnixStream::connect(&path),
        )
        .await
        {
            Ok(Ok(_stream)) => true,
            _ => {
                // Stale socket or timeout; clean up stale file
                let _ = std::fs::remove_file(&path);
                false
            }
        }
    }

    /// Sends a typed command to the running GUI instance over the control socket
    pub async fn send_command(command: ControlCommand) -> Result<ControlResponse> {
        let path = Self::socket_path();
        let stream = UnixStream::connect(&path).await.with_context(|| {
            format!("Failed to connect to control socket at {}", path.display())
        })?;

        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);

        let request = ControlRequest::new(command);
        let mut req_str = serde_json::to_string(&request)?;
        req_str.push('\n');

        writer.write_all(req_str.as_bytes()).await?;
        writer.flush().await?;

        let mut line = String::new();
        buf_reader.read_line(&mut line).await?;

        let response: ControlResponse = serde_json::from_str(&line)
            .with_context(|| format!("Malformed control response: {line}"))?;

        Ok(response)
    }

    /// Binds the Unix domain socket for the GPUI server listener
    pub fn bind_listener() -> Result<UnixListener> {
        let dir = Self::socket_dir();
        std::fs::create_dir_all(&dir)?;

        // Enforce user-only directory permissions 0700
        let mut dir_perms = std::fs::metadata(&dir)?.permissions();
        dir_perms.set_mode(0o700);
        let _ = std::fs::set_permissions(&dir, dir_perms);

        let path = Self::socket_path();
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }

        let listener = UnixListener::bind(&path)?;

        // Enforce user-only socket permissions 0600
        let mut sock_perms = std::fs::metadata(&path)?.permissions();
        sock_perms.set_mode(0o600);
        let _ = std::fs::set_permissions(&path, sock_perms);

        Ok(listener)
    }

    /// Cleans up the socket file upon clean exit
    pub fn cleanup() {
        let path = Self::socket_path();
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_socket_path_resolution() {
        let path = ControlSocket::socket_path();
        assert!(path.ends_with("shelly-gpui/control.sock"));
    }
}
