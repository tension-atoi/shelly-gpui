use crate::config::ConfigManager;
use crate::control::protocol::{
    ControlCommand, ControlResponse, ControlStatus, CONTROL_PROTOCOL_VERSION,
};
use crate::control::socket::ControlSocket;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub struct CliInvocation {
    pub json: bool,
    pub command: Option<CliCommand>,
    pub internal_gui: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CliCommand {
    Help,
    Version,
    Open,
    Focus,
    Status,
    Quit,
    Navigate { destination: String },
    Search { query: String },
    View { mode: String },
    Inspect { package: String },
    Inspector { tab: String },
    Logs { operation: String },
    Settings(CliSettingsCommand),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CliSettingsCommand {
    List,
    Get { key: String },
    Set { key: String, value: String },
    Reset { key: Option<String> },
}

pub enum CliOutcome {
    LaunchGui,
    Exit(i32),
}

impl CliInvocation {
    pub fn parse_from_args(args: &[String]) -> Result<Self, String> {
        if args.iter().any(|a| a == "--internal-gui") {
            return Ok(Self {
                json: false,
                command: None,
                internal_gui: true,
            });
        }

        let mut json = false;
        let mut positional = Vec::new();

        for arg in args.iter().skip(1) {
            match arg.as_str() {
                "-j" | "--json" => json = true,
                "-h" | "--help" => {
                    return Ok(Self {
                        json,
                        command: Some(CliCommand::Help),
                        internal_gui: false,
                    })
                }
                "-V" | "--version" => {
                    return Ok(Self {
                        json,
                        command: Some(CliCommand::Version),
                        internal_gui: false,
                    })
                }
                other => positional.push(other.to_string()),
            }
        }

        if positional.is_empty() {
            return Ok(Self {
                json,
                command: None,
                internal_gui: false,
            });
        }

        let cmd_str = positional[0].to_ascii_lowercase();
        let cmd = match cmd_str.as_str() {
            "help" => CliCommand::Help,
            "version" => CliCommand::Version,
            "open" => CliCommand::Open,
            "focus" => CliCommand::Focus,
            "status" => CliCommand::Status,
            "quit" => CliCommand::Quit,
            "navigate" => {
                if positional.len() < 2 {
                    return Err("Missing destination argument for 'navigate' (e.g. updates, installed, browse)".into());
                }
                CliCommand::Navigate {
                    destination: positional[1].clone(),
                }
            }
            "search" => {
                if positional.len() < 2 {
                    return Err("Missing query argument for 'search'".into());
                }
                let query = positional[1..].join(" ");
                CliCommand::Search { query }
            }
            "view" => {
                if positional.len() < 2 {
                    return Err("Missing view mode argument for 'view' ('table' or 'cards')".into());
                }
                let mode = positional[1].to_ascii_lowercase();
                if mode != "table" && mode != "cards" {
                    return Err(format!(
                        "Invalid view mode '{mode}', expected 'table' or 'cards'"
                    ));
                }
                CliCommand::View { mode }
            }
            "inspect" => {
                if positional.len() < 2 {
                    return Err("Missing package argument for 'inspect'".into());
                }
                CliCommand::Inspect {
                    package: positional[1].clone(),
                }
            }
            "inspector" => {
                if positional.len() < 2 {
                    return Err("Missing tab argument for 'inspector' ('overview', 'dependencies', 'files')".into());
                }
                CliCommand::Inspector {
                    tab: positional[1].clone(),
                }
            }
            "logs" => {
                if positional.len() < 2 {
                    return Err(
                        "Missing operation argument for 'logs' ('show', 'hide', 'clear')".into(),
                    );
                }
                let op = positional[1].to_ascii_lowercase();
                if op != "show" && op != "hide" && op != "clear" {
                    return Err(format!(
                        "Invalid logs operation '{op}', expected 'show', 'hide', or 'clear'"
                    ));
                }
                CliCommand::Logs { operation: op }
            }
            "settings" => {
                if positional.len() < 2 {
                    return Err(
                        "Missing settings subcommand ('list', 'get', 'set', 'reset')".into(),
                    );
                }
                let sub = positional[1].to_ascii_lowercase();
                match sub.as_str() {
                    "list" => CliCommand::Settings(CliSettingsCommand::List),
                    "get" => {
                        if positional.len() < 3 {
                            return Err("Missing setting key for 'settings get'".into());
                        }
                        CliCommand::Settings(CliSettingsCommand::Get { key: positional[2].clone() })
                    }
                    "set" => {
                        if positional.len() < 3 {
                            return Err("Missing setting key for 'settings set'".into());
                        }
                        if positional.len() < 4 {
                            return Err("Missing setting value for 'settings set'".into());
                        }
                        CliCommand::Settings(CliSettingsCommand::Set {
                            key: positional[2].clone(),
                            value: positional[3].clone(),
                        })
                    }
                    "reset" => {
                        let key = if positional.len() >= 3 {
                            let k = positional[2].clone();
                            if k.eq_ignore_ascii_case("all") {
                                None
                            } else {
                                Some(k)
                            }
                        } else {
                            None
                        };
                        CliCommand::Settings(CliSettingsCommand::Reset { key })
                    }
                    other => return Err(format!("Unknown settings subcommand '{other}'. Expected 'list', 'get', 'set', or 'reset'")),
                }
            }
            unknown => {
                return Err(format!(
                    "Unknown command '{unknown}'. Run 'shelly-gpui --help' for usage."
                ))
            }
        };

        Ok(Self {
            json,
            command: Some(cmd),
            internal_gui: false,
        })
    }

    pub fn print_help() {
        println!(
            r#"Shelly GPUI — GPU-Accelerated Universal Package Manager
Usage: shelly-gpui [OPTIONS] [SUBCOMMAND]

Options:
  -j, --json     Output machine-readable JSON
  -h, --help     Print help and exit
  -V, --version  Print version and exit

Runtime Control Commands:
  open                       Focus existing Shelly window, or launch GUI if offline
  focus                      Focus running Shelly window
  status                     Print runtime status and health
  quit                       Cleanly terminate running Shelly instance
  navigate <dest>            Navigate to 'updates', 'installed', 'browse', 'news', 'settings'
  search <query>             Search for packages in the browse workstation
  view <table|cards>         Switch package workstation view mode
  inspect <package>          Select and inspect a package
  inspector <tab>            Switch inspector tab ('overview', 'dependencies', 'files')
  logs <show|hide|clear>     Show, hide, or clear the operation console log drawer

Settings Authority Commands:
  settings list              List all supported settings, defaults, and authorities
  settings get <key>         Get current value of a setting
  settings set <key> <val>   Set and validate a setting value atomically
  settings reset [key|all]   Reset a specific setting or all settings to defaults
"#
        );
    }

    pub fn print_version() {
        println!(
            "shelly-gpui 0.1.0 (control protocol v{})",
            CONTROL_PROTOCOL_VERSION
        );
    }
}

/// Ensures that a background Shelly GPUI instance is running and listening on the socket.
/// Spawns the GUI in detached mode with `--internal-gui` and waits bounded time for socket readiness.
pub async fn ensure_gui_running() -> Result<()> {
    if ControlSocket::is_instance_running().await {
        return Ok(());
    }

    use std::os::unix::process::CommandExt;
    let exe = std::env::current_exe()?;
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("--internal-gui")
        .process_group(0)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    if std::env::var("SHELLY_BIN").is_err()
        && std::path::Path::new("/usr/lib/shelly/shelly").exists()
    {
        cmd.env("SHELLY_BIN", "/usr/lib/shelly/shelly");
    }

    let _child = cmd.spawn()?;

    // Wait bounded readiness for the socket (up to 3 seconds, polling every 50ms)
    for _ in 0..60 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        if ControlSocket::is_instance_running().await {
            return Ok(());
        }
    }

    anyhow::bail!("Timed out waiting for Shelly GPUI instance to initialize socket")
}

pub async fn run_cli_invocation(invocation: CliInvocation) -> Result<CliOutcome> {
    if invocation.internal_gui {
        return Ok(CliOutcome::LaunchGui);
    }

    let json = invocation.json;

    let cmd = match invocation.command {
        Some(CliCommand::Help) => {
            CliInvocation::print_help();
            return Ok(CliOutcome::Exit(0));
        }
        Some(CliCommand::Version) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "version": "0.1.0",
                        "control_protocol": CONTROL_PROTOCOL_VERSION,
                    })
                );
            } else {
                CliInvocation::print_version();
            }
            return Ok(CliOutcome::Exit(0));
        }
        None => {
            // Check if instance is running
            if ControlSocket::is_instance_running().await {
                // Focus existing window
                let resp = ControlSocket::send_command(ControlCommand::Open).await?;
                return outcome_from_response(&resp, json);
            } else {
                return Ok(CliOutcome::LaunchGui);
            }
        }
        Some(c) => c,
    };

    let is_running = ControlSocket::is_instance_running().await;

    match cmd {
        CliCommand::Help | CliCommand::Version => unreachable!(),
        CliCommand::Open => {
            if !is_running {
                ensure_gui_running().await?;
            }
            let resp = ControlSocket::send_command(ControlCommand::Open).await?;
            outcome_from_response(&resp, json)
        }
        CliCommand::Focus => {
            if is_running {
                let resp = ControlSocket::send_command(ControlCommand::Focus).await?;
                outcome_from_response(&resp, json)
            } else {
                let resp = ControlResponse::error("Shelly GUI is not running");
                outcome_from_response(&resp, json)
            }
        }
        CliCommand::Status => {
            if is_running {
                let resp = ControlSocket::send_command(ControlCommand::Status).await?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&resp).unwrap_or_default()
                    );
                } else if let Some(ref data) = resp.data {
                    if let Ok(st) = serde_json::from_value::<ControlStatus>(data.clone()) {
                        println!("Shelly GPUI Status:");
                        println!("  Running:        yes (PID {})", st.pid);
                        println!("  Executable:     {}", st.executable);
                        println!("  Protocol:       v{}", st.protocol_version);
                        println!("  Destination:    {}", st.destination);
                        println!("  Search Query:   \"{}\"", st.query);
                        println!("  View Mode:      {}", st.view_mode);
                        println!("  Inspector Tab:  {}", st.inspector_tab);
                        println!(
                            "  Selected Pkg:   {}",
                            st.selected_package.as_deref().unwrap_or("<none>")
                        );
                        println!("  Active Task:    {}", st.operation_running);
                    } else {
                        println!("Shelly GUI is running (status data available via --json)");
                    }
                } else {
                    println!("Shelly GUI is running");
                }
                Ok(CliOutcome::Exit(if resp.ok { 0 } else { 1 }))
            } else {
                let offline_status = ControlStatus::not_running();
                if json {
                    let resp = ControlResponse::ok_with_data(
                        "Offline",
                        serde_json::to_value(&offline_status).unwrap_or_default(),
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&resp).unwrap_or_default()
                    );
                } else {
                    println!("Shelly GPUI Status:");
                    println!("  Running:        no (offline)");
                    println!("  Executable:     {}", offline_status.executable);
                    println!("  Protocol:       v{}", offline_status.protocol_version);
                }
                Ok(CliOutcome::Exit(0))
            }
        }
        CliCommand::Quit => {
            if is_running {
                let resp = ControlSocket::send_command(ControlCommand::Quit).await?;
                outcome_from_response(&resp, json)
            } else {
                let resp = ControlResponse::error("Shelly GUI is not running");
                outcome_from_response(&resp, json)
            }
        }
        CliCommand::Navigate { destination } => {
            if !is_running {
                ensure_gui_running().await?;
            }
            let resp =
                ControlSocket::send_command(ControlCommand::Navigate { destination }).await?;
            outcome_from_response(&resp, json)
        }
        CliCommand::Search { query } => {
            if !is_running {
                ensure_gui_running().await?;
            }
            let resp = ControlSocket::send_command(ControlCommand::Search { query }).await?;
            outcome_from_response(&resp, json)
        }
        CliCommand::View { mode } => {
            if is_running {
                let resp = ControlSocket::send_command(ControlCommand::View { mode }).await?;
                outcome_from_response(&resp, json)
            } else {
                match ConfigManager::set_setting("view-mode", &mode) {
                    Ok(()) => {
                        let resp =
                            ControlResponse::ok(format!("View mode set to '{mode}' (offline)"));
                        outcome_from_response(&resp, json)
                    }
                    Err(e) => {
                        let resp = ControlResponse::error(e.to_string());
                        outcome_from_response(&resp, json)
                    }
                }
            }
        }
        CliCommand::Inspect { package } => {
            if !is_running {
                ensure_gui_running().await?;
            }
            let resp = ControlSocket::send_command(ControlCommand::Inspect { package }).await?;
            outcome_from_response(&resp, json)
        }
        CliCommand::Inspector { tab } => {
            if !is_running {
                ensure_gui_running().await?;
            }
            let resp = ControlSocket::send_command(ControlCommand::Inspector { tab }).await?;
            outcome_from_response(&resp, json)
        }
        CliCommand::Logs { operation } => {
            if is_running {
                let resp = ControlSocket::send_command(ControlCommand::Logs { operation }).await?;
                outcome_from_response(&resp, json)
            } else if operation == "show" || operation == "hide" {
                let open_val = if operation == "show" { "true" } else { "false" };
                let _ = ConfigManager::set_setting("log-drawer-open", open_val);
                let resp = ControlResponse::ok(format!(
                    "Log drawer default set to '{operation}' (offline)"
                ));
                outcome_from_response(&resp, json)
            } else {
                let resp = ControlResponse::error(
                    "Shelly GUI is not running (cannot clear in-memory logs)",
                );
                outcome_from_response(&resp, json)
            }
        }
        CliCommand::Settings(settings_cmd) => match settings_cmd {
            CliSettingsCommand::List => {
                let entries = ConfigManager::list_settings();
                if json {
                    let resp = ControlResponse::ok_with_data(
                        "Settings listed",
                        serde_json::to_value(&entries).unwrap_or_default(),
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&resp).unwrap_or_default()
                    );
                } else {
                    println!(
                        "{:<22} {:<12} {:<10} {:<10} DESCRIPTION",
                        "KEY", "VALUE", "DEFAULT", "AUTHORITY"
                    );
                    println!("{}", "-".repeat(88));
                    for e in entries {
                        println!(
                            "{:<22} {:<12} {:<10} {:<10} {}",
                            e.key, e.value, e.default, e.authority, e.description
                        );
                    }
                }
                Ok(CliOutcome::Exit(0))
            }
            CliSettingsCommand::Get { key } => match ConfigManager::get_setting(&key) {
                Ok(val) => {
                    if json {
                        let resp = ControlResponse::ok_with_data(
                            format!("{key}: {val}"),
                            serde_json::json!({ "key": key, "value": val }),
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&resp).unwrap_or_default()
                        );
                    } else {
                        println!("{key} = {val}");
                    }
                    Ok(CliOutcome::Exit(0))
                }
                Err(e) => {
                    let resp = ControlResponse::error(e.to_string());
                    outcome_from_response(&resp, json)
                }
            },
            CliSettingsCommand::Set { key, value } => {
                if is_running {
                    let resp =
                        ControlSocket::send_command(ControlCommand::SettingsSet { key, value })
                            .await?;
                    outcome_from_response(&resp, json)
                } else {
                    match ConfigManager::set_setting(&key, &value) {
                        Ok(()) => {
                            let resp = ControlResponse::ok(format!(
                                "Setting '{key}' set to '{value}' (offline)"
                            ));
                            outcome_from_response(&resp, json)
                        }
                        Err(e) => {
                            let resp = ControlResponse::error(e.to_string());
                            outcome_from_response(&resp, json)
                        }
                    }
                }
            }
            CliSettingsCommand::Reset { key } => {
                if is_running {
                    let resp =
                        ControlSocket::send_command(ControlCommand::SettingsReset { key }).await?;
                    outcome_from_response(&resp, json)
                } else {
                    let res = match key.as_deref() {
                        Some(k) => ConfigManager::reset_setting(k),
                        None => ConfigManager::reset_all(),
                    };
                    match res {
                        Ok(()) => {
                            let target = key.as_deref().unwrap_or("all settings");
                            let resp = ControlResponse::ok(format!(
                                "Reset '{target}' to default (offline)"
                            ));
                            outcome_from_response(&resp, json)
                        }
                        Err(e) => {
                            let resp = ControlResponse::error(e.to_string());
                            outcome_from_response(&resp, json)
                        }
                    }
                }
            }
        },
    }
}

pub(crate) fn outcome_from_response(resp: &ControlResponse, json: bool) -> Result<CliOutcome> {
    print_response(resp, json);
    if resp.ok {
        Ok(CliOutcome::Exit(0))
    } else {
        Ok(CliOutcome::Exit(1))
    }
}

fn print_response(resp: &ControlResponse, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(resp).unwrap_or_default());
    } else if resp.ok {
        if let Some(ref msg) = resp.message {
            println!("{msg}");
        }
    } else if let Some(ref err) = resp.error {
        eprintln!("Error: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_parse_empty_args_launches_gui() {
        let args = vec!["shelly-gpui".to_string()];
        let parsed = CliInvocation::parse_from_args(&args).expect("Should parse empty args");
        assert!(!parsed.json);
        assert_eq!(parsed.command, None);
    }

    #[test]
    fn test_parse_json_flag_anywhere() {
        let args1 = vec![
            "shelly-gpui".to_string(),
            "--json".to_string(),
            "status".to_string(),
        ];
        let parsed1 = CliInvocation::parse_from_args(&args1).expect("Should parse");
        assert!(parsed1.json);
        assert_eq!(parsed1.command, Some(CliCommand::Status));

        let args2 = vec![
            "shelly-gpui".to_string(),
            "status".to_string(),
            "-j".to_string(),
        ];
        let parsed2 = CliInvocation::parse_from_args(&args2).expect("Should parse");
        assert!(parsed2.json);
        assert_eq!(parsed2.command, Some(CliCommand::Status));
    }

    #[test]
    fn test_parse_subcommands() {
        let cases = vec![
            (vec!["shelly-gpui", "open"], Some(CliCommand::Open)),
            (vec!["shelly-gpui", "focus"], Some(CliCommand::Focus)),
            (vec!["shelly-gpui", "status"], Some(CliCommand::Status)),
            (vec!["shelly-gpui", "quit"], Some(CliCommand::Quit)),
            (
                vec!["shelly-gpui", "navigate", "installed"],
                Some(CliCommand::Navigate {
                    destination: "installed".to_string(),
                }),
            ),
            (
                vec!["shelly-gpui", "search", "ripgrep"],
                Some(CliCommand::Search {
                    query: "ripgrep".to_string(),
                }),
            ),
            (
                vec!["shelly-gpui", "view", "table"],
                Some(CliCommand::View {
                    mode: "table".to_string(),
                }),
            ),
            (
                vec!["shelly-gpui", "inspect", "aalib"],
                Some(CliCommand::Inspect {
                    package: "aalib".to_string(),
                }),
            ),
            (
                vec!["shelly-gpui", "inspector", "overview"],
                Some(CliCommand::Inspector {
                    tab: "overview".to_string(),
                }),
            ),
            (
                vec!["shelly-gpui", "logs", "clear"],
                Some(CliCommand::Logs {
                    operation: "clear".to_string(),
                }),
            ),
            (
                vec!["shelly-gpui", "settings", "list"],
                Some(CliCommand::Settings(CliSettingsCommand::List)),
            ),
            (
                vec!["shelly-gpui", "settings", "get", "theme"],
                Some(CliCommand::Settings(CliSettingsCommand::Get {
                    key: "theme".to_string(),
                })),
            ),
            (
                vec!["shelly-gpui", "settings", "set", "theme", "dark"],
                Some(CliCommand::Settings(CliSettingsCommand::Set {
                    key: "theme".to_string(),
                    value: "dark".to_string(),
                })),
            ),
            (
                vec!["shelly-gpui", "settings", "reset", "theme"],
                Some(CliCommand::Settings(CliSettingsCommand::Reset {
                    key: Some("theme".to_string()),
                })),
            ),
            (
                vec!["shelly-gpui", "settings", "reset", "all"],
                Some(CliCommand::Settings(CliSettingsCommand::Reset {
                    key: None,
                })),
            ),
        ];

        for (input, expected) in cases {
            let str_vec: Vec<String> = input.into_iter().map(|s| s.to_string()).collect();
            let parsed = CliInvocation::parse_from_args(&str_vec).expect("Parse error");
            assert_eq!(parsed.command, expected);
        }
    }

    #[test]
    fn test_parse_validation_failures() {
        let bad_view = vec![
            "shelly-gpui".to_string(),
            "view".to_string(),
            "banana".to_string(),
        ];
        assert!(CliInvocation::parse_from_args(&bad_view).is_err());

        let missing_nav = vec!["shelly-gpui".to_string(), "navigate".to_string()];
        assert!(CliInvocation::parse_from_args(&missing_nav).is_err());

        let missing_search = vec!["shelly-gpui".to_string(), "search".to_string()];
        assert!(CliInvocation::parse_from_args(&missing_search).is_err());
    }

    #[test]
    fn test_parse_internal_gui_flag() {
        let args = vec!["shelly-gpui".to_string(), "--internal-gui".to_string()];
        let parsed = CliInvocation::parse_from_args(&args).expect("Should parse --internal-gui");
        assert!(parsed.internal_gui);
        assert_eq!(parsed.command, None);
        assert!(!parsed.json);

        let args_with_extra = vec![
            "shelly-gpui".to_string(),
            "--internal-gui".to_string(),
            "something".to_string(),
        ];
        let parsed_extra =
            CliInvocation::parse_from_args(&args_with_extra).expect("Should parse --internal-gui");
        assert!(parsed_extra.internal_gui);
    }

    #[test]
    fn test_outcome_from_response_exit_codes() {
        let ok_resp = ControlResponse::ok("operation succeeded");
        let ok_outcome = outcome_from_response(&ok_resp, false).expect("outcome ok");
        match ok_outcome {
            CliOutcome::Exit(code) => assert_eq!(code, 0),
            _ => panic!("Expected CliOutcome::Exit(0)"),
        }

        let err_resp = ControlResponse::error("operation failed");
        let err_outcome = outcome_from_response(&err_resp, false).expect("outcome err");
        match err_outcome {
            CliOutcome::Exit(code) => assert_eq!(code, 1),
            _ => panic!("Expected CliOutcome::Exit(1)"),
        }
    }
}
