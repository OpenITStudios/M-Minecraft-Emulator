//! Lazy Tester transport. The existing bootstrap owns games in one retained helper.
use super::owned_provider::OwnedProcess;
use serde::{Deserialize, Serialize};
use std::{
    io::{self, BufRead},
    sync::Mutex,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileSession {
    pub slot: u32,
    pub state: String,
    pub process: Option<OwnedProcess>,
    pub issue: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionSnapshot {
    pub schema: u32,
    pub profiles: Vec<ProfileSession>,
    pub storage: Option<Vec<ProfileStorage>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileStorage {
    pub slot: u32,
    pub bytes: u64,
    pub files: u32,
    pub partial: bool,
}
impl SessionSnapshot {
    pub fn stopped() -> Self {
        Self {
            schema: 1,
            storage: None,
            profiles: (1..=5)
                .map(|slot| ProfileSession {
                    slot,
                    state: "STOPPED".into(),
                    process: None,
                    issue: None,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "operation", deny_unknown_fields)]
pub enum SessionRequest {
    Inspect,
    InspectStorage,
    OpenProfileFolder { slot: u32 },
    Launch { slot: u32 },
}

type SessionReply = Result<SessionSnapshot, String>;
#[cfg(windows)]
struct Controller {
    child: std::process::Child,
    input: std::process::ChildStdin,
    replies: std::sync::mpsc::Receiver<io::Result<SessionReply>>,
    uncertain: bool,
}
#[cfg(windows)]
static CONTROLLER: Mutex<Option<Controller>> = Mutex::new(None);

#[cfg(windows)]
pub fn request(request: SessionRequest) -> io::Result<SessionSnapshot> {
    use std::{
        io::Write,
        os::windows::process::CommandExt,
        process::{Command, Stdio},
    };
    if matches!(request, SessionRequest::Launch { slot } | SessionRequest::OpenProfileFolder { slot } if !(1..=5).contains(&slot))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unknown Minecraft profile",
        ));
    }
    let mut owner = CONTROLLER
        .lock()
        .map_err(|_| io::Error::other("Session controller unavailable"))?;
    if owner.is_none() {
        // Opening the app and reading its state never starts a game or a controller.
        if matches!(request, SessionRequest::Inspect) {
            return Ok(SessionSnapshot::stopped());
        }
        let mut child = Command::new(std::env::current_exe()?)
            .arg("--lazy-tester-controller")
            .creation_flags(0x08000000)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let input = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("Missing controller input"))?;
        let output = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("Missing controller output"))?;
        let (sender, replies) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut reader = io::BufReader::new(output);
            loop {
                let result = read_message(&mut reader)
                    .and_then(|text| {
                        text.ok_or_else(|| io::Error::other("Session controller disconnected"))
                    })
                    .and_then(|text| serde_json::from_str(&text).map_err(io::Error::other));
                let failed = result.is_err();
                if sender.send(result).is_err() || failed {
                    break;
                }
            }
        });
        *owner = Some(Controller {
            child,
            input,
            replies,
            uncertain: false,
        });
    }
    let controller = owner.as_mut().unwrap();
    if controller.uncertain {
        return Err(io::Error::other(
            "Controller outcome is uncertain; existing games are preserved. Do not retry launch.",
        ));
    }
    if let Some(status) = controller.child.try_wait()? {
        return Err(io::Error::other(format!(
            "Session controller stopped ({status}); existing games were preserved"
        )));
    }
    let result = (|| {
        serde_json::to_writer(&mut controller.input, &request)?;
        controller.input.write_all(b"\n")?;
        controller.input.flush()?;
        controller
            .replies
            .recv_timeout(std::time::Duration::from_secs(45))
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Session request outcome is uncertain",
                )
            })?
    })();
    match result {
        Ok(reply) => reply.map_err(io::Error::other),
        Err(error) => {
            controller.uncertain = true;
            Err(error)
        }
    }
}

#[cfg(not(windows))]
pub fn request(_: SessionRequest) -> io::Result<SessionSnapshot> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Minecraft sessions require Windows",
    ))
}

pub(super) fn read_message(input: &mut impl io::BufRead) -> io::Result<Option<String>> {
    let mut bytes = Vec::new();
    let count = io::Read::take(input, 32769).read_until(b'\n', &mut bytes)?;
    if count == 0 {
        return Ok(None);
    }
    if count > 32768 || bytes.last() != Some(&b'\n') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid session message",
        ));
    }
    String::from_utf8(bytes).map(Some).map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protocol_rejects_paths_unknown_operations_and_unbounded_input() {
        for input in [
            r#"{"operation":"Launch","slot":1,"path":"other.exe"}"#,
            r#"{"operation":"Delete","slot":1}"#,
        ] {
            assert!(serde_json::from_str::<SessionRequest>(input).is_err());
        }
        assert!(read_message(&mut io::Cursor::new(vec![b'x'; 32769])).is_err());
        assert!(read_message(&mut io::Cursor::new(b"{}".to_vec())).is_err());
        assert_eq!(
            SessionSnapshot::stopped()
                .profiles
                .iter()
                .map(|p| p.slot)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
    }
}
