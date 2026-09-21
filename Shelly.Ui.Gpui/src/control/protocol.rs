use serde::{Deserialize, Serialize};

pub const CONTROL_PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlRequest {
    pub version: u32,
    pub command: ControlCommand,
}

impl ControlRequest {
    pub fn new(command: ControlCommand) -> Self {
        Self {
            version: CONTROL_PROTOCOL_VERSION,
            command,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", content = "payload", rename_all = "kebab-case")]
pub enum ControlCommand {
    Open,
    Focus,
    Quit,
    Status,
    Navigate { destination: String },
    Search { query: String },
    View { mode: String },
    Inspect { package: String },
    Inspector { tab: String },
    Logs { operation: String },
    SettingsList,
    SettingsGet { key: String },
    SettingsSet { key: String, value: String },
    SettingsReset { key: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlResponse {
    pub version: u32,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl ControlResponse {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            version: CONTROL_PROTOCOL_VERSION,
            ok: true,
            error: None,
            message: Some(message.into()),
            data: None,
        }
    }

    pub fn ok_with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            version: CONTROL_PROTOCOL_VERSION,
            ok: true,
            error: None,
            message: Some(message.into()),
            data: Some(data),
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            version: CONTROL_PROTOCOL_VERSION,
            ok: false,
            error: Some(error.into()),
            message: None,
            data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlStatus {
    pub protocol_version: u32,
    pub gui_running: bool,
    pub pid: u32,
    pub executable: String,
    pub destination: String,
    pub query: String,
    pub view_mode: String,
    pub inspector_tab: String,
    pub selected_package: Option<String>,
    pub operation_running: bool,
}

impl ControlStatus {
    pub fn not_running() -> Self {
        let exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/usr/lib/shelly/shelly-gpui-bin".to_string());
        Self {
            protocol_version: CONTROL_PROTOCOL_VERSION,
            gui_running: false,
            pid: 0,
            executable: exe,
            destination: "unknown".to_string(),
            query: String::new(),
            view_mode: "unknown".to_string(),
            inspector_tab: "unknown".to_string(),
            selected_package: None,
            operation_running: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_control_protocol_round_trip() {
        let req = ControlRequest::new(ControlCommand::Navigate {
            destination: "installed".to_string(),
        });
        let json = serde_json::to_string(&req).expect("Failed to serialize request");
        let parsed: ControlRequest =
            serde_json::from_str(&json).expect("Failed to deserialize request");
        assert_eq!(req, parsed);
        assert_eq!(parsed.version, CONTROL_PROTOCOL_VERSION);

        let resp =
            ControlResponse::ok_with_data("Navigated", serde_json::json!({"dest": "installed"}));
        let resp_json = serde_json::to_string(&resp).expect("Failed to serialize response");
        let parsed_resp: ControlResponse =
            serde_json::from_str(&resp_json).expect("Failed to deserialize response");
        assert_eq!(resp, parsed_resp);
        assert!(parsed_resp.ok);
    }

    #[test]
    fn test_control_status_serialization() {
        let status = ControlStatus {
            protocol_version: CONTROL_PROTOCOL_VERSION,
            gui_running: true,
            pid: 12345,
            executable: "/usr/lib/shelly/shelly-gpui-bin".to_string(),
            destination: "browse".to_string(),
            query: "ripgrep".to_string(),
            view_mode: "table".to_string(),
            inspector_tab: "overview".to_string(),
            selected_package: Some("ripgrep".to_string()),
            operation_running: false,
        };
        let json = serde_json::to_string(&status).expect("Serialization failed");
        let parsed: ControlStatus = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(status, parsed);
        assert!(parsed.gui_running);
        assert_eq!(parsed.selected_package.as_deref(), Some("ripgrep"));
    }
}
