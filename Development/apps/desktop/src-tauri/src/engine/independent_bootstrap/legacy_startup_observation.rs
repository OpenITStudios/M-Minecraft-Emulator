//! Historical opt-in startup observation and replay preparation CLI.
//! Not a mounted product session owner. Shared bootstrap contracts remain in the parent.
use super::*;

pub(super) fn maybe_run() -> bool {
    let mut args = std::env::args_os().skip(1);
    let mode = args.next();
    let count = match mode.as_deref() {
        Some(v) if v == OsStr::new("--education-startup-observation-once") => 2,
        Some(v) if v == OsStr::new("--education-five-startup-observation-once") => 5,
        Some(v) if v == OsStr::new("--education-prepare-five-profiles-once") => 0,
        _ => return false,
    };
    let result = if args.next().is_some() {
        Err(invalid(
            "Startup observation accepts no additional arguments",
        ))
    } else {
        if count == 0 {
            prepare_five_profiles()
        } else {
            run_startup_observation(count)
        }
    };
    let code = match result {
        Ok(code) => code,
        Err(error) => {
            use std::io::Write;
            let _ = writeln!(
                io::stderr(),
                "{{\"event\":\"startup-refused\",\"kind\":\"{:?}\",\"osCode\":{}}}",
                error.kind(),
                error
                    .raw_os_error()
                    .map_or_else(|| "null".to_owned(), |code| code.to_string())
            );
            2
        }
    };
    std::process::exit(code);
}

fn run_startup_observation(count: usize) -> io::Result<i32> {
    // One experiment process per interactive session; no persisted readiness/account state.
    let _gate = acquire_startup_gate(
        crate::engine::native_clients_session::CONTROLLER_GATE_NAME,
        io::Error::new(io::ErrorKind::AlreadyExists, "Startup experiment already active"),
    )?;
    let mut launch_guard = super::acquire_native_launch()?;
    // Preserve existing games; no termination, sign-out or activation fallback.
    if !super::inspect_native_workspace()?.processes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Close the existing game normally first",
        ));
    }
    if count == 5 {
        let started =
            prepare_registered_enrolled_for::<5>()?.resume_for_authorized_observation()?;
        *launch_guard = Some(std::time::Instant::now());
        observe_until_exit(&started)
    } else if count == 2 {
        let started = prepare_registered_enrolled()?.resume_for_authorized_observation()?;
        *launch_guard = Some(std::time::Instant::now());
        observe_until_exit(&started)
    } else {
        Err(invalid("Unsupported Education slot count"))
    }
}


fn label_started_windows<const N: usize>(started: &StartedEducationObservation<N>) {
    use windows_sys::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId},
    };
    struct Selection {
        pid: u32,
        window: HWND,
        ambiguous: bool,
    }
    unsafe extern "system" fn select(hwnd: HWND, data: isize) -> i32 {
        let state = &mut *(data as *mut Selection);
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == state.pid && super::verify_live_client_02_window(hwnd, pid).is_ok() {
            if state.window.is_null() {
                state.window = hwnd;
            } else {
                state.ambiguous = true;
            }
        }
        1
    }
    for child in &started.children {
        if !child.resume_attempted || child.identity().is_err() {
            continue;
        }
        let Some(store) = &child.storage else {
            continue;
        };
        let mut selection = Selection {
            pid: child.pid,
            window: ptr::null_mut(),
            ambiguous: false,
        };
        if unsafe { EnumWindows(Some(select), &mut selection as *mut Selection as isize) } == 0
            || selection.ambiguous
            || selection.window.is_null()
            || child.identity().is_err()
        {
            continue;
        }
        // The immutable store slot, never enumeration order or a matching title, assigns the number.
        let _ = label_slot_window(selection.window, child.pid, store.slot);
    }
}

pub(super) fn observe_until_exit<const N: usize>(started: &StartedEducationObservation<N>) -> io::Result<i32> {
    use std::io::Write;
    let mut completed = [false; N];
    let mut failed = false;
    let mut caption_check = std::time::Instant::now();
    for (index, attempt) in started.attempts.iter().enumerate() {
        let state = match attempt {
            StartupAttempt::NotAttempted => {
                completed[index] = true;
                failed = true;
                "not-attempted"
            }
            StartupAttempt::Resumed => "resume-call-succeeded",
            StartupAttempt::Unconfirmed(_) => {
                failed = true;
                "resume-unconfirmed"
            }
        };
        writeln!(io::stdout(), "{{\"event\":\"startup-attempt\",\"slot\":{},\"state\":\"{}\",\"providerReady\":false}}",
            index + 1, state)?;
    }
    // Recording ends at its existing bound. Keep original handles until operator closure;
    // dropping this runner at any time still preserves every attempted child.
    while completed.iter().any(|done| !done) {
        if caption_check.elapsed() >= std::time::Duration::from_secs(1) {
            label_started_windows(started);
            caption_check = std::time::Instant::now();
        }
        for (index, done) in completed.iter_mut().enumerate() {
            if *done {
                continue;
            }
            match started.save_after_exit(index as u32 + 1) {
                Ok(saved) => {
                    *done = true;
                    failed |= saved.exit_code != 0;
                    writeln!(io::stdout(),
                        "{{\"event\":\"journal-saved\",\"slot\":{},\"exitCode\":{},\"providerReady\":false}}",
                        index + 1, saved.exit_code)?;
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => {
                    *done = true;
                    failed = true;
                    // A rejected/incomplete journal must not erase the original child's exit outcome.
                    let code = started.children[index]
                        .exit_code_after_exit()
                        .ok()
                        .map_or_else(|| "null".to_owned(), |code| code.to_string());
                    writeln!(io::stdout(),
                        "{{\"event\":\"journal-failed\",\"slot\":{},\"kind\":\"{:?}\",\"exitCode\":{},\"providerReady\":false}}",
                        index + 1, error.kind(), code)?;
                }
            }
        }
        if completed.iter().any(|done| !done) {
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }
    Ok(i32::from(failed))
}

