//! ONE global virtual Xbox controller for an explicitly authorized lab ONLY.
//! No Minecraft slot routing! Every Windows game may see this device.
//! Not compiled, installed, connected to Codex/MCP, or executed by ChatGPT.
//! The production independent-input gate remains BLOCKED.
#[cfg(not(windows))]
fn main() {
    eprintln!("WINDOWS_ONLY_VIRTUAL_PAD_LAB");
    std::process::exit(2);
}

#[cfg(windows)]
mod lab {
    use serde::Deserialize;
    use std::{
        collections::HashSet,
        error::Error,
        io::{self, Read},
        sync::mpsc::{self, RecvTimeoutError},
        thread,
        time::{Duration, Instant},
    };
    use vigem_rust::{Client, X360Button, X360Report};

    const LAB_DURATION: Duration = Duration::from_secs(10);
    const WATCHDOG: Duration = Duration::from_millis(350);
    const MAX_LINE: usize = 512;
    const MAX_SEQUENCE: u32 = 10000;

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Frame {
        seq: u32,
        #[serde(rename = "atMs")]
        at_ms: u32,
        lx: i16,
        ly: i16,
        rx: i16,
        ry: i16,
        lt: u8,
        rt: u8,
        buttons: Vec<String>,
    }

    fn to_report(frame: &Frame) -> Result<X360Report, &'static str> {
        if frame.seq == 0 || frame.seq > MAX_SEQUENCE ||
            frame.at_ms > LAB_DURATION.as_millis() as u32 ||
            frame.buttons.len() > 6 {
            return Err("INVALID_FRAME");
        }
        let mut buttons = X360Button::empty();
        let mut seen = HashSet::new();
        for button in &frame.buttons {
            if !seen.insert(button.as_str()) { return Err("DUPLICATE_BUTTON"); }
            buttons |= match button.as_str() {
                "a" => X360Button::A,
                "b" => X360Button::B,
                "x" => X360Button::X,
                "y" => X360Button::Y,
                "left_shoulder" => X360Button::LEFT_SHOULDER,
                "right_shoulder" => X360Button::RIGHT_SHOULDER,
                _ => return Err("UNKNOWN_BUTTON"),
            };
        }
        Ok(X360Report {
            buttons, left_trigger:frame.lt, right_trigger:frame.rt,
            thumb_lx:frame.lx, thumb_ly:frame.ly,
            thumb_rx:frame.rx, thumb_ry:frame.ry,
        })
    }

    // Stdin reader operates in a separate thread so a missing newline or
    // hung producer can NEVER extend the ten-second gamepad lifetime.
    // No filenames, network listeners or process/window access.
    fn read_lines(sender: mpsc::Sender<Result<Vec<u8>, &'static str>>) {
        let stdin = io::stdin();
        let mut input = stdin.lock();
        let mut chunk = [0u8; 128];
        let mut line = Vec::with_capacity(128);
        loop {
            let count = match input.read(&mut chunk) {
                Ok(n) => n,
                Err(_) => { let _ = sender.send(Err("STDIN_READ_FAILED")); return; }
            };
            if count == 0 { return; }
            for byte in &chunk[..count] {
                if *byte == b'\n' {
                    if line.is_empty() {
                        let _ = sender.send(Err("EMPTY_FRAME"));
                        return;
                    }
                    if sender.send(Ok(std::mem::take(&mut line))).is_err() { return; }
                    continue;
                }
                if line.len() >= MAX_LINE {
                    let _ = sender.send(Err("FRAME_TOO_LARGE"));
                    return;
                }
                line.push(*byte);
            }
        }
    }

    pub fn run() -> Result<(), Box<dyn Error>> {
        // This is a deliberately unsafe-to-share GLOBAL device experiment.
        // An operator must explicitly approve local experiments separately
        // BEFORE setting this flag. It is not integrated into MCP or CI.
        let args = std::env::args().skip(1).collect::<Vec<_>>();
        if args.len() != 1 || args[0] != "--unassigned-global-pad-lab" ||
            std::env::var("LAZY_TESTER_GLOBAL_GAMEPAD_LAB").ok().as_deref()
                != Some("I_ACKNOWLEDGE_NOT_PER_MINECRAFT") {
            return Err("EXPERIMENT_NOT_AUTHORIZED".into());
        }

        let client = Client::connect()?; // Requires ALREADY installed ViGEmBus.
        let pad = client.new_x360_target().plug()?.wait_for_ready()?;
        pad.update(&X360Report::default())?; // Neutral from the first report.
        let (sender,receiver) = mpsc::channel();
        thread::spawn(move || read_lines(sender));

        let started = Instant::now();
        let mut previous_sequence = 0u32;
        let mut previous_at_ms = 0u32;
        let mut last_input = started;
        let mut non_neutral = false;
        let mut failure: Option<&'static str> = None;
        while started.elapsed() < LAB_DURATION {
            let remaining = LAB_DURATION.saturating_sub(started.elapsed());
            let interval = remaining.min(Duration::from_millis(50));
            match receiver.recv_timeout(interval) {
                Ok(Ok(bytes)) => {
                    let parsed: Frame = match serde_json::from_slice(&bytes) {
                        Ok(value) => value,
                        Err(_) => { failure=Some("MALFORMED_FRAME"); break; }
                    };
                    if parsed.seq <= previous_sequence ||
                        parsed.at_ms < previous_at_ms {
                        failure=Some("NON_MONOTONIC_REPORT");
                        break;
                    }
                    // Reports can be queued in advance; honor their relative
                    // time while respecting the hard deadline and watchdog.
                    while started.elapsed().as_millis() < u128::from(parsed.at_ms) {
                        if started.elapsed() >= LAB_DURATION { break; }
                        if non_neutral && last_input.elapsed() >= WATCHDOG {
                            if pad.update(&X360Report::default()).is_err() {
                                failure=Some("WATCHDOG_NEUTRAL_FAILED");
                                break;
                            }
                            non_neutral=false;
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    if failure.is_some() || started.elapsed() >= LAB_DURATION {
                        break;
                    }
                    let report=match to_report(&parsed) {
                        Ok(value) => value,
                        Err(code) => { failure=Some(code); break; }
                    };
                    if pad.update(&report).is_err() {
                        failure=Some("VIRTUAL_DEVICE_UPDATE_FAILED");
                        break;
                    }
                    previous_sequence=parsed.seq;
                    previous_at_ms=parsed.at_ms;
                    last_input=Instant::now();
                    non_neutral=report!=X360Report::default();
                }
                Ok(Err(code)) => { failure=Some(code); break; }
                Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {
                    if non_neutral && last_input.elapsed()>=WATCHDOG {
                        if pad.update(&X360Report::default()).is_err() {
                            failure=Some("WATCHDOG_NEUTRAL_FAILED");
                            break;
                        }
                        non_neutral=false;
                    }
                }
            }
        }
        // Neutralize first, THEN let the virtual controller disconnect.
        // This is still not a guarantee for game-level input release on crash.
        let reset=pad.update(&X360Report::default());
        if reset.is_err() { return Err("FINAL_NEUTRAL_FAILED".into()); }
        if let Some(code)=failure { return Err(code.into()); }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn default_is_neutral_and_gameplay_buttons_are_bounded() {
            let f=Frame {seq:1,at_ms:0,lx:0,ly:0,rx:0,ry:0,lt:0,rt:0,buttons:vec![]};
            assert_eq!(to_report(&f).unwrap(),X360Report::default());
            let a=Frame {buttons:vec!["a".into(),"a".into()],..f};
            assert_eq!(to_report(&a).unwrap_err(),"DUPLICATE_BUTTON");
        }
        #[test]
        fn arbitrary_system_or_guide_buttons_are_not_allowed() {
            let f=Frame {seq:1,at_ms:0,lx:0,ly:0,rx:0,ry:0,lt:0,rt:0,
                buttons:vec!["guide".into()]};
            assert_eq!(to_report(&f).unwrap_err(),"UNKNOWN_BUTTON");
        }
    }
}
#[cfg(windows)]
fn main() {
    if let Err(_error) = lab::run() {
        // No game/account/process metadata or raw input printed to stderr.
        eprintln!("GLOBAL_GAMEPAD_LAB_REFUSED_OR_UNAVAILABLE");
        std::process::exit(2);
    }
}
