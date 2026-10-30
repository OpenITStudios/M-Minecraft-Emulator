//! One-shot, same-image native adapter for the local MCP stdio bridge.
//! The retained authenticated controller remains the only Minecraft game owner.
//! Does not open a Tauri window or own Minecraft. An explicitly opted-in
//! MC02-only background Win32 message experiment is NOT gameplay control.
use std::ffi::OsStr;
use std::io::{self, Write};
use serde_json::{json, Value};

fn parse_operation(args: &[String]) -> Result<(&'static str, Option<u32>), &'static str> {
    match args {
        [one] if one == "inspect" => Ok(("inspect", None)),
        [one, slot] if one == "launch" => {
            let slot = slot.parse::<u32>().map_err(|_| "INVALID_ARGUMENTS")?;
            if !(1..=5).contains(&slot) {
                return Err("INVALID_ARGUMENTS");
            }
            Ok(("launch", Some(slot)))
        }
        [operation, slot] if operation == "probe-background" && slot == "2" =>
            Ok(("probe-background", Some(2))),
        _ => Err("INVALID_ARGUMENTS"),
    }
}

fn native_error(error: io::Error) -> &'static str {
    match error.kind() {
        io::ErrorKind::WouldBlock => "NATIVE_BUSY",
        io::ErrorKind::TimedOut | io::ErrorKind::Interrupted => "OUTCOME_UNCERTAIN",
        io::ErrorKind::PermissionDenied => "NATIVE_REFUSED",
        io::ErrorKind::Unsupported => "UNSUPPORTED",
        _ => "NATIVE_UNAVAILABLE",
    }
}

fn execute(args: &[String]) -> Result<Value, &'static str> {
    let (operation, slot) = parse_operation(args)?;
    #[cfg(not(all(windows, target_arch = "x86_64", feature = "lazy-tester-package")))]
    {
        let _ = (operation, slot);
        Err("UNSUPPORTED")
    }
    #[cfg(all(windows, target_arch = "x86_64", feature = "lazy-tester-package"))]
    {
        use super::native_clients_session::{self, SessionRequest};
        // An Inspect without retained session ownership cannot truthfully label
        // five potentially unadopted games as STOPPED.
        if operation == "inspect" {
            let owner = native_clients_session::existing_controller_gate(
                "Local\\MivubiEducationStartupObservation",
            )
            .map_err(native_error)?;
            if !owner {
                return Err("CONTROLLER_UNAVAILABLE");
            }
        }
        if operation == "probe-background" {
            // Manual opt-in only; never an automatically advertised MCP tool.
            if std::env::var("LAZY_TESTER_MC02_INPUT_PROBE").ok().as_deref()
                != Some("I_ACCEPT_ONE_W_PULSE")
            {
                return Err("EXPERIMENT_NOT_AUTHORIZED");
            }
            if !native_clients_session::existing_controller_gate(
                "Local\\MivubiEducationStartupObservation",
            ).map_err(native_error)? {
                return Err("CONTROLLER_UNAVAILABLE");
            }
            let snapshot = native_clients_session::request(SessionRequest::Inspect)
                .map_err(native_error)?;
            if snapshot.schema != 1 || snapshot.profiles.len() != 5 ||
                snapshot.profiles.iter().enumerate().any(|(i,p)|p.slot as usize != i+1) {
                return Err("INVALID_NATIVE_SNAPSHOT");
            }
            let human = &snapshot.profiles[0];
            let ai = &snapshot.profiles[1];
            if human.state != "RUNNING" || ai.state != "RUNNING" {
                return Err("PROBE_REQUIRES_TWO_RUNNING_GAMES");
            }
            let human_process = human.process.as_ref().filter(|p|p.responding)
                .ok_or("HUMAN_GAME_NOT_READY")?;
            let ai_process = ai.process.as_ref().filter(|p|p.responding)
                .ok_or("AI_GAME_NOT_READY")?;
            super::background_input_probe::one_forward_pulse(human_process,ai_process)
                .map_err(native_error)?;
            return Ok(json!({
                "schema":1, "ok":true, "operation":"probe-background", "slot":2,
                "delivery":"WINDOW_MESSAGES_POSTED_ONLY",
                "humanForegroundPreserved":true, "gameplayObserved":false,
                "evidence":"NOT_VERIFIED_IN_MINECRAFT"
            }));
        }
        let request = match slot {
            Some(slot) => SessionRequest::Launch { slot },
            None => SessionRequest::Inspect,
        };
        let snapshot = native_clients_session::request(request).map_err(native_error)?;
        if snapshot.schema != 1 || snapshot.profiles.len() != 5 {
            return Err("INVALID_NATIVE_SNAPSHOT");
        }
        let mut profiles = Vec::with_capacity(5);
        for (index, profile) in snapshot.profiles.into_iter().enumerate() {
            if profile.slot as usize != index + 1 {
                return Err("INVALID_NATIVE_SNAPSHOT");
            }
            profiles.push(json!({
                "slot": profile.slot,
                "state": profile.state,
                "hasWindow": profile.process.is_some(),
                "responding": profile.process.as_ref().is_some_and(|p|p.responding),
                "needsAttention": profile.issue.is_some(),
            }));
        }
        Ok(json!({
            "schema": 1,
            "ok": true,
            "operation": operation,
            "profiles": profiles,
            "evidence": "NATIVE_SESSION_ONLY_NOT_GAMEPLAY_OR_ACCOUNT_SIGNIN",
        }))
    }
}

pub fn maybe_run() -> bool {
    let mut argv = std::env::args_os().skip(1);
    if argv.next().as_deref() != Some(OsStr::new("--lazy-tester-agent-cli")) {
        return false;
    }
    // Malformed arguments cannot fall through to launching the ordinary manager.
    let args: Option<Vec<String>> = argv.map(|a| a.to_str().map(str::to_owned)).collect();
    let result = match args {
        Some(values) => execute(&values),
        None => Err("INVALID_ARGUMENTS"),
    };
    let ok = result.is_ok();
    let response = result.unwrap_or_else(|code| json!({
        "schema": 1, "ok": false, "code": code,
    }));
    let mut out = io::stdout().lock();
    let _ = writeln!(out, "{response}");
    let _ = out.flush();
    if !ok {
        std::process::exit(2);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_shot_command_accepts_only_inspect_or_five_slots() {
        assert_eq!(parse_operation(&["inspect".into()]).unwrap(), ("inspect", None));
        assert_eq!(parse_operation(&["launch".into(), "5".into()]).unwrap(), ("launch", Some(5)));
        assert_eq!(parse_operation(&["probe-background".into(), "2".into()]).unwrap(),
            ("probe-background", Some(2)));
        for args in [vec!["close"], vec!["launch"], vec!["launch", "0"],
            vec!["launch", "6"], vec!["inspect", "1"], vec!["launch", "1", "extra"],
            vec!["probe-background", "1"], vec!["probe-background", "3"],
            vec!["probe-background"], vec!["probe-background", "2", "extra"]] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            assert!(parse_operation(&args).is_err());
        }
    }
}
