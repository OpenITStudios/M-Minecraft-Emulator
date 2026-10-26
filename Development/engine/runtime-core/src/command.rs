use crate::{
    inspect_base_preparation, read_operation_history, ClientId, DestructiveConfirmation,
    ErrorReport, SuccessReport, VirtualClients,
};
use serde::Serialize;
use serde_json::Value;
use std::io;

#[derive(Debug, Clone)]
pub struct PublicCommandResult {
    pub success: bool,
    pub json: String,
}

fn input_error(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn parse_client(value: &str) -> io::Result<ClientId> {
    match value {
        "Native" => Ok(ClientId::Native),
        "Virtual-01" => Ok(ClientId::Virtual01),
        "Virtual-02" => Ok(ClientId::Virtual02),
        "Virtual-03" => Ok(ClientId::Virtual03),
        _ => Err(input_error(format!("unknown client: {value}"))),
    }
}

fn require_no_args(command: &str, args: &[String]) -> io::Result<()> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(input_error(format!("{command} does not accept arguments")))
    }
}

fn one_arg<'a>(command: &str, args: &'a [String]) -> io::Result<&'a str> {
    match args {
        [value] => Ok(value),
        [] => Err(input_error(format!("{command} requires one argument"))),
        _ => Err(input_error(format!(
            "{command} accepts exactly one argument"
        ))),
    }
}

fn to_value<T: Serialize>(value: T) -> io::Result<Value> {
    serde_json::to_value(value).map_err(|error| io::Error::new(io::ErrorKind::Other, error))
}

// Single public-command boundary for pausing all legacy VM mutations.
// Direct native launches live in the desktop adapter, not this VM runtime.
const PAUSED_VM_COMMANDS: &[&str] = &[
    "actions",
    "stage-update",
    "register-base",
    "open-base-finalization",
    "provision",
    "reprovision",
    "verify-identities",
    "start",
    "start-setup",
    "start-client",
    "suspend",
    "stop",
    "restart",
    "set-ready",
    "reset",
    "open",
    "launch-minecraft",
];

fn legacy_operation_paused(command: &str, args: &[String]) -> bool {
    PAUSED_VM_COMMANDS.contains(&command)
        || (command == "app-sandbox-pilot" && args.first().map(String::as_str) != Some("inspect"))
}

fn execute_value(command: &str, args: &[String]) -> io::Result<Value> {
    // Fail closed before provider discovery, privilege escalation, VM
    // creation/deletion, state writes, snapshots, or guest power operations.
    if legacy_operation_paused(command, args) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Legacy VM controls are paused for native-first development; existing guest/account data is preserved",
        ));
    }

    let app = VirtualClients;

    match command {
        "product-status" => {
            require_no_args(command, args)?;
            to_value(serde_json::json!({
                "mode": "NATIVE_FEASIBILITY",
                "legacyVmMutationsEnabled": false,
                "fourClientGameplayVerified": false,
                "legacyAccountData": "PRESERVE"
            }))
        }
        "doctor" => {
            require_no_args(command, args)?;
            to_value(app.doctor())
        }
        "policy" => {
            require_no_args(command, args)?;
            to_value(app.policy())
        }
        "base-preflight" => {
            require_no_args(command, args)?;
            to_value(inspect_base_preparation()?)
        }
        "actions" => {
            require_no_args(command, args)?;
            to_value(app.lifecycle_actions()?)
        }
        "app-sandbox-pilot" => {
            // Explicit CLI only; not the production Virtual lifecycle.
            to_value(crate::provider::app_sandbox_pilot_command(args)?)
        }
        "diagnostics" => {
            require_no_args(command, args)?;
            // Explicit CLI diagnostics only. The frequent desktop snapshot and
            // support bundle never contact the privileged App Sandbox daemon.
            let mut value = to_value(app.diagnostics()?)?;
            if let Value::Object(ref mut fields) = value {
                fields.insert(
                    "appSandboxInspection".to_string(),
                    to_value(crate::provider::inspect_app_sandbox())?,
                );
            }
            Ok(value)
        }
        "snapshot" => {
            require_no_args(command, args)?;
            to_value(app.snapshot()?)
        }
        "support-bundle" => {
            require_no_args(command, args)?;
            to_value(app.support_bundle()?)
        }
        "history" => {
            require_no_args(command, args)?;
            to_value(read_operation_history()?)
        }
        "check-update" => {
            require_no_args(command, args)?;
            to_value(app.check_update()?)
        }
        "stage-update" => {
            require_no_args(command, args)?;
            to_value(app.stage_update()?)
        }
        "register-base" => {
            require_no_args(command, args)?;
            to_value(app.register_base()?)
        }
        "open-base-finalization" => {
            require_no_args(command, args)?;
            app.open_base_for_finalization()?;
            to_value(serde_json::json!({"opened": true}))
        }
        "provision" => {
            require_no_args(command, args)?;
            to_value(app.provision()?)
        }
        "status" => {
            require_no_args(command, args)?;
            to_value(app.status()?)
        }
        "resources" => {
            let value = one_arg(command, args)?;
            let count = value
                .parse::<usize>()
                .map_err(|_| input_error("virtual client count must be an integer"))?;
            to_value(app.resources(count)?)
        }
        "reprovision" => match args {
            [client, confirmation] if confirmation == "--destroy-account-state" => {
                to_value(app.reprovision(
                    parse_client(client)?,
                    DestructiveConfirmation::ReprovisionAccountState,
                )?)
            }
            [_, _] => Err(input_error(
                "reprovision requires --destroy-account-state after the client id",
            )),
            _ => Err(input_error(
                "reprovision requires <Virtual-01..03> --destroy-account-state",
            )),
        },
        "verify-identities" => {
            require_no_args(command, args)?;
            to_value(app.verify_identities()?)
        }
        "start" => {
            let value = one_arg(command, args)?;
            let count = value
                .parse::<usize>()
                .map_err(|_| input_error("virtual client count must be an integer"))?;
            to_value(app.start(count)?)
        }
        "start-setup" => to_value(app.start_setup(parse_client(one_arg(command, args)?)?)?),
        "start-client" => to_value(app.start_client(parse_client(one_arg(command, args)?)?)?),
        "suspend" => {
            let client = match args {
                [] => None,
                [client] => Some(parse_client(client)?),
                _ => return Err(input_error("suspend accepts zero or one client id")),
            };
            to_value(app.suspend(client)?)
        }
        "stop" => {
            let client = match args {
                [] => None,
                [client] => Some(parse_client(client)?),
                _ => return Err(input_error("stop accepts zero or one client id")),
            };
            to_value(app.stop(client)?)
        }
        "restart" => to_value(app.restart(parse_client(one_arg(command, args)?)?)?),
        "set-ready" => to_value(app.set_ready(parse_client(one_arg(command, args)?)?)?),
        "reset" => to_value(app.reset(parse_client(one_arg(command, args)?)?)?),
        "open" => to_value(app.open(parse_client(one_arg(command, args)?)?)?),
        "launch-minecraft" => {
            to_value(app.launch_minecraft(parse_client(one_arg(command, args)?)?)?)
        }
        _ => Err(input_error(format!("unknown command: {command}"))),
    }
}

fn serialize_error(error: &io::Error) -> String {
    let report = ErrorReport::from_io(error);
    serde_json::to_string(&report).unwrap_or_else(|_| {
        r#"{"schema":1,"code":"IO_FAILURE","message":"error serialization failed","retryable":false}"#
            .to_string()
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationPhase {
    Executing,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationProgress {
    pub schema: u32,
    pub operation: String,
    pub phase: OperationPhase,
}

// A per-invocation observation hook. It cannot authorize, cancel or replace an
// operation, and carries no arguments, paths, credentials or persisted state.
pub fn execute_public_command_with_progress(
    command: &str,
    args: &[String],
    mut report: impl FnMut(OperationProgress),
) -> PublicCommandResult {
    report(OperationProgress {
        schema: 2,
        operation: command.to_string(),
        phase: OperationPhase::Executing,
    });
    let result = execute_public_command_result(command, args);
    report(OperationProgress {
        schema: 2,
        operation: command.to_string(),
        phase: if result.success {
            OperationPhase::Succeeded
        } else {
            OperationPhase::Failed
        },
    });
    result
}

pub fn execute_public_command(command: &str, args: &[String]) -> PublicCommandResult {
    execute_public_command_with_progress(command, args, |_| {})
}

fn execute_public_command_result(command: &str, args: &[String]) -> PublicCommandResult {
    match execute_value(command, args) {
        Ok(value) => match serde_json::to_string_pretty(&SuccessReport::new(&value)) {
            Ok(json) => PublicCommandResult {
                success: true,
                json,
            },
            Err(error) => {
                let error = io::Error::new(io::ErrorKind::Other, error);
                PublicCommandResult {
                    success: false,
                    json: serialize_error(&error),
                }
            }
        },
        Err(error) => PublicCommandResult {
            success: false,
            json: serialize_error(&error),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::execute_public_command;

    #[test]
    fn native_first_product_mode_is_explicit_and_read_only() {
        let result = execute_public_command("product-status", &[]);
        assert!(result.success);
        let json: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(json["data"]["mode"], "NATIVE_FEASIBILITY");
        assert_eq!(json["data"]["legacyVmMutationsEnabled"], false);
        assert_eq!(json["data"]["fourClientGameplayVerified"], false);
        assert_eq!(json["data"]["legacyAccountData"], "PRESERVE");
    }

    #[test]
    fn all_legacy_vm_mutations_fail_closed_before_touching_provider() {
        let commands = [
            "actions",
            "stage-update",
            "register-base",
            "open-base-finalization",
            "provision",
            "reprovision",
            "verify-identities",
            "start",
            "start-setup",
            "start-client",
            "suspend",
            "stop",
            "restart",
            "set-ready",
            "reset",
            "open",
            "launch-minecraft",
        ];
        for command in commands {
            let result = execute_public_command(command, &[]);
            assert!(!result.success, "legacy {command} was accepted");
            let json: serde_json::Value = serde_json::from_str(&result.json).unwrap();
            assert_eq!(json["code"], "UNSUPPORTED", "{command}");
            assert_eq!(json["retryable"], false);
        }
        for operation in ["start", "shutdown", "display"] {
            let result = execute_public_command(
                "app-sandbox-pilot",
                &[operation.into(), "--confirm-disposable-pilot".into()],
            );
            assert!(!result.success);
            assert!(result.json.contains("UNSUPPORTED"));
        }
    }

    #[test]
    fn command_boundary_rejects_extra_arguments() {
        let result = execute_public_command("policy", &["extra".into()]);
        assert!(!result.success);
        assert!(result.json.contains("INVALID_INPUT"));
    }

    #[test]
    fn command_boundary_uses_public_contract_for_success() {
        let result = execute_public_command("policy", &[]);
        assert!(result.success);
        let json: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(json["schema"], 1);
        assert_eq!(json["data"]["maxVirtualClients"], 3);
    }
    #[test]
    fn progress_reports_real_dispatch_and_terminal_result() {
        let mut phases = Vec::new();
        let result = super::execute_public_command_with_progress("policy", &[], |event| {
            assert_eq!(event.schema, 2);
            assert_eq!(event.operation, "policy");
            phases.push(event.phase);
        });
        assert!(result.success);
        assert_eq!(
            phases,
            vec![
                super::OperationPhase::Executing,
                super::OperationPhase::Succeeded
            ]
        );
        assert_eq!(result.json, execute_public_command("policy", &[]).json);
    }

    #[test]
    fn progress_reports_failure_without_leaking_arguments() {
        let mut events = Vec::new();
        let result = super::execute_public_command_with_progress(
            "policy",
            &["private-input".into()],
            |event| events.push(event),
        );
        assert!(!result.success);
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].phase, super::OperationPhase::Failed);
        let json = serde_json::to_string(&events).unwrap();
        assert!(!json.contains("private-input"));
    }
}
