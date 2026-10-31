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
        #[cfg(feature="mc02-native-input")]
        [one,slot] if one=="launch-mc02-input" && slot=="2" =>
            Ok(("launch-mc02-input",Some(2))),
        [one, slot] if one == "launch" => {
            let slot = slot.parse::<u32>().map_err(|_| "INVALID_ARGUMENTS")?;
            if !(1..=5).contains(&slot) {
                return Err("INVALID_ARGUMENTS");
            }
            Ok(("launch", Some(slot)))
        }
        [one, slot] if one == "target" => {
            let slot = slot.parse::<u32>().map_err(|_| "INVALID_ARGUMENTS")?;
            if !(1..=5).contains(&slot) {
                return Err("INVALID_ARGUMENTS");
            }
            Ok(("target", Some(slot)))
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
    #[cfg(all(windows,target_arch="x86_64",
              feature="lazy-tester-package",feature="mc02-native-input"))]
    if args.first().map(String::as_str)==Some("mc02-input") {
        // Explicit one-shot CLI adapter, NOT an automatically exposed MCP
        // tool. The authenticated native controller remains sole writer.
        use super::native_clients_session::{self,SessionRequest};
        if std::env::var("LAZY_TESTER_MC02_VIRTUAL_INPUT").ok().as_deref()
            !=Some("I_ACCEPT_EXPERIMENTAL_MC02_INPUT") {
            return Err("EXPERIMENT_NOT_AUTHORIZED");
        }
        if args.len()!=7 || args[1]!="2" {return Err("INVALID_ARGUMENTS");}
        let keys=args[2].parse::<u32>().map_err(|_|"INVALID_ARGUMENTS")?;
        let dx=args[3].parse::<i32>().map_err(|_|"INVALID_ARGUMENTS")?;
        let dy=args[4].parse::<i32>().map_err(|_|"INVALID_ARGUMENTS")?;
        let ttl_ms=args[5].parse::<u32>().map_err(|_|"INVALID_ARGUMENTS")?;
        let requested_at_ms=args[6].parse::<u64>().map_err(|_|"INVALID_ARGUMENTS")?;
        // Enforce the same per-slot keyboard/camera admission before
        // asking the retained native controller to inspect any process.
        native_clients_session::admit_native_input_frame(2,keys,dx,dy,ttl_ms)
            .map_err(|_|"INVALID_ARGUMENTS")?;
        // A delayed Node request must not receive a new execution window
        // just because this one-shot executable finally started. UTC is
        // only an admission check; the native pipe then uses shared uptime.
        let wall_now=std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_|"STALE_MC02_INPUT")?
            .as_millis();
        if requested_at_ms==0 || wall_now<u128::from(requested_at_ms) ||
            wall_now-u128::from(requested_at_ms)>500 {
            return Err("STALE_MC02_INPUT");
        }
        let issued_at_ms=unsafe{
            windows_sys::Win32::System::SystemInformation::GetTickCount64()
        };
        let snapshot=native_clients_session::request(
            SessionRequest::Mc02Input{slot:2,keys,dx,dy,ttl_ms,issued_at_ms}
        ).map_err(native_error)?;
        if snapshot.schema!=1||snapshot.profiles.len()!=5 {
            return Err("INVALID_NATIVE_SNAPSHOT");
        }
        return Ok(json!({"schema":1,"ok":true,
            "operation":"mc02-input","slot":2,
            "delivery":"PER_CLIENT_SHARED_STATE_WRITTEN_NOT_GAMEPLAY_PROOF",
            "gameplayObserved":false}));
    }
    let (operation, slot) = parse_operation(args)?;
    #[cfg(all(windows, target_arch="x86_64",feature="lazy-tester-package",
              feature="mc02-native-input"))]
    if operation=="launch-mc02-input" &&
       std::env::var("LAZY_TESTER_MC02_VIRTUAL_INPUT").ok().as_deref()
         !=Some("I_ACCEPT_EXPERIMENTAL_MC02_INPUT") {
        return Err("EXPERIMENT_NOT_AUTHORIZED");
    }
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
        if operation == "inspect" || operation == "target" {
            let owner = native_clients_session::existing_controller_gate(
                "Local\\MivubiEducationStartupObservation",
            )
            .map_err(native_error)?;
            if !owner {
                return Err("CONTROLLER_UNAVAILABLE");
            }
        }
        if operation == "target" {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                GetForegroundWindow, IsHungAppWindow, IsIconic, IsWindowVisible,
            };
            // Same authenticated native controller as the UI. No PID scanning
            // or adoption of an unowned Minecraft client.
            let slot = slot.ok_or("INVALID_ARGUMENTS")?;
            let snapshot = native_clients_session::request(SessionRequest::Inspect)
                .map_err(native_error)?;
            if snapshot.schema != 1 || snapshot.profiles.len() != 5 ||
                snapshot.profiles.iter().enumerate()
                    .any(|(index,profile)| profile.slot as usize != index+1) {
                return Err("INVALID_NATIVE_SNAPSHOT");
            }
            let profile = &snapshot.profiles[(slot - 1) as usize];
            if profile.state != "RUNNING" || profile.issue.is_some() {
                return Err("TARGET_NOT_RUNNING");
            }
            let process = profile.process.as_ref().filter(|p|p.responding)
                .ok_or("TARGET_NOT_READY")?;
            // Reuse the existing process HANDLE / birth FILETIME / Windows
            // session / root HWND verifier. It pins the source process while
            // the window is checked, rather than trusting a numeric PID.
            let (window_id,foreground) =
                super::workspace_surface::with_verified_native_window(
                    process, |verified_hwnd| {
                        let window = verified_hwnd as _;
                        if verified_hwnd == 0 ||
                            verified_hwnd as u64 > ((1u64 << 53) - 1) {
                            return Err(io::Error::new(io::ErrorKind::InvalidData,
                                "HWND cannot be represented as a safe MCP number"));
                        }
                        if unsafe { IsWindowVisible(window) } == 0 ||
                            unsafe { IsIconic(window) } != 0 ||
                            unsafe { IsHungAppWindow(window) } != 0 {
                            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                                "Minecraft window is unavailable for inspection"));
                        }
                        Ok((verified_hwnd as u64,
                            unsafe { GetForegroundWindow() } == window))
                    }).map_err(native_error)?;
            return Ok(json!({
                "schema": 1, "ok": true, "operation": "target",
                "slot": slot, "pid": process.pid, "windowId": window_id,
                "createdFileTime": process.created_file_time,
                "sessionId": process.session_id, "foreground": foreground,
                "visible": true, "minimized": false,
                "verification": "SAME_SESSION_PID_BIRTH_ROOT_HWND",
                "gameplayInput": "NOT_VERIFIED"
            }));
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
        #[cfg(feature="mc02-native-input")]
        let request=if operation=="launch-mc02-input" {
            SessionRequest::LaunchMc02Experiment {
                slot:slot.ok_or("INVALID_ARGUMENTS")?
            }
        } else {
            match slot {
                Some(slot)=>SessionRequest::Launch {slot},
                None=>SessionRequest::Inspect,
            }
        };
        #[cfg(not(feature="mc02-native-input"))]
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
        #[cfg(feature="mc02-native-input")]
        assert_eq!(parse_operation(&["launch-mc02-input".into(), "2".into()]).unwrap(),
            ("launch-mc02-input",Some(2)));
        assert_eq!(parse_operation(&["target".into(), "2".into()]).unwrap(),
            ("target", Some(2)));
        assert_eq!(parse_operation(&["probe-background".into(), "2".into()]).unwrap(),
            ("probe-background", Some(2)));
        for args in [vec!["close"], vec!["launch"], vec!["launch", "0"],
            vec!["launch", "6"], vec!["inspect", "1"], vec!["launch", "1", "extra"],
            vec!["launch-mc02-input","1"],vec!["launch-mc02-input","3"],
            vec!["probe-background", "1"], vec!["probe-background", "3"],
            vec!["probe-background"], vec!["probe-background", "2", "extra"],
            vec!["target"], vec!["target", "0"], vec!["target", "6"],
            vec!["target", "2", "more"]] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            assert!(parse_operation(&args).is_err());
        }
    }
}
