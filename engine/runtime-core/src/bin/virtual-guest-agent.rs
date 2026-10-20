#![forbid(unsafe_code)]

use m_bedrock_virtual_clients_core::{
    bounded_command_output, guest_agent_minecraft_profile, GuestStatus, GUEST_AGENT_PORT,
    GUEST_AGENT_PROTOCOL_VERSION, GUEST_STATUS_SCHEMA, MINECRAFT_LAUNCH_SCHEMA,
};

#[cfg(target_os = "windows")]
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

// SSH command execution takes place in sshd's non-interactive session.
// Reuse the existing signed-in-user launcher for gameplay; no second SYSTEM
// service, account sign-in automation or listener is created here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    VmwareService,
    RegisterLauncher,
    InteractiveLauncher,
    SshStatus,
    SshLaunch,
}

fn run_mode(args: &[String]) -> io::Result<RunMode> {
    match args {
        [] => Ok(RunMode::VmwareService),
        [flag] if flag == "--register-interactive-launcher" => Ok(RunMode::RegisterLauncher),
        [flag] if flag == "--interactive-launcher" => Ok(RunMode::InteractiveLauncher),
        [flag] if flag == "--ssh-status" => Ok(RunMode::SshStatus),
        [flag] if flag == "--ssh-launch-minecraft" => Ok(RunMode::SshLaunch),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unsupported guest utility command",
        )),
    }
}

fn current_guest_status() -> GuestStatus {
    GuestStatus {
        schema: GUEST_STATUS_SCHEMA,
        protocol_version: GUEST_AGENT_PROTOCOL_VERSION,
        agent_version: env!("CARGO_PKG_VERSION").to_string(),
        minecraft: guest_agent_minecraft_profile(),
        minecraft_running: Some(minecraft_process_running()),
        interactive_launcher_ready: Some(interactive_launcher_ready()),
        machine_identity: guest_machine_identity(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run_mode(&args)? {
        RunMode::RegisterLauncher => return register_interactive_launcher(),
        RunMode::InteractiveLauncher => return run_interactive_launcher(),
        RunMode::SshStatus => {
            // stdout is a single structured response for the authenticated SSH
            // client, never a VM process/graphics playability certificate.
            println!("{}", serde_json::to_string(&current_guest_status())?);
            return Ok(());
        }
        RunMode::SshLaunch => {
            // The existing localhost per-user launcher checks interactive
            // session availability. SSH itself must not spawn the game.
            let result = launch_minecraft()?;
            println!("{}", serde_json::to_string(&result)?);
            return Ok(());
        }
        RunMode::VmwareService => {}
    }
    let token = Arc::new(guest_agent_token()?);
    let active = Arc::new(AtomicUsize::new(0));
    let launch_gate = Arc::new(Mutex::new(()));
    let listener = TcpListener::bind(("0.0.0.0", GUEST_AGENT_PORT))?;

    for incoming in listener.incoming() {
        match incoming {
            Ok(mut stream) => {
                // Bound worker count: a slow connection must not block all status calls.
                if active
                    .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                        (count < MAX_ACTIVE_REQUESTS).then_some(count + 1)
                    })
                    .is_err()
                {
                    let _ = stream.set_write_timeout(Some(Duration::from_millis(250)));
                    let _ = stream.write_all(SERVICE_UNAVAILABLE);
                    continue;
                }
                let token = Arc::clone(&token);
                let active = Arc::clone(&active);
                let launch_gate = Arc::clone(&launch_gate);
                thread::spawn(move || {
                    let _permit = ActiveRequest(active);
                    if let Err(error) = handle(stream, token.as_str(), launch_gate.as_ref()) {
                        eprintln!("guest agent request failed: {error}");
                    }
                });
            }
            Err(error) => eprintln!("guest agent accept error: {error}"),
        }
    }

    Ok(())
}

const INTERACTIVE_LAUNCHER_PORT: u16 = 47832;
const MAX_GUEST_REQUEST_BYTES: usize = 4096;
const MAX_ACTIVE_REQUESTS: usize = 8;
const REQUEST_HEADER_TIMEOUT: Duration = Duration::from_secs(2);
const SERVICE_UNAVAILABLE: &[u8] =
    b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
const MAX_INTERACTIVE_REQUEST_BYTES: u64 = 512;

// Release capacity even when a handler returns early or panics.
struct ActiveRequest(Arc<AtomicUsize>);
impl Drop for ActiveRequest {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}
const REQUEST_ID_HEADER: &str = "X-Virtual-Clients-Request-Id";
const MINECRAFT_PROCESS_START_TIMEOUT: Duration = Duration::from_secs(30);
const INTERACTIVE_LAUNCH_TIMEOUT: Duration = Duration::from_secs(35);

fn interactive_launcher_address() -> String {
    format!("127.0.0.1:{INTERACTIVE_LAUNCHER_PORT}")
}

#[cfg(target_os = "windows")]
fn interactive_launcher_ready() -> bool {
    let Ok(address) = interactive_launcher_address().parse() else {
        return false;
    };
    let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_millis(150)) else {
        return false;
    };
    let request_id = "0000000000000000";
    if stream
        .set_read_timeout(Some(Duration::from_millis(250)))
        .is_err()
        || stream
            .set_write_timeout(Some(Duration::from_millis(150)))
            .is_err()
        || stream
            .write_all(format!("PING {request_id}\n").as_bytes())
            .is_err()
        || stream.shutdown(Shutdown::Write).is_err()
    {
        return false;
    }
    let mut response = String::new();
    stream.read_to_string(&mut response).is_ok() && response.trim() == format!("READY:{request_id}")
}

#[cfg(target_os = "windows")]
fn request_interactive_minecraft_launch(
) -> Result<m_bedrock_virtual_clients_core::MinecraftLaunchResult, Box<dyn std::error::Error>> {
    use m_bedrock_virtual_clients_core::{MinecraftLaunchResult, MinecraftLaunchState};
    if minecraft_process_running() {
        return Ok(MinecraftLaunchResult {
            schema: MINECRAFT_LAUNCH_SCHEMA,
            state: MinecraftLaunchState::AlreadyRunning,
        });
    }

    let address = interactive_launcher_address().parse()?;
    let mut stream =
        TcpStream::connect_timeout(&address, Duration::from_secs(1)).map_err(|_| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "Interactive launcher is not active for the signed-in Windows user",
            )
        })?;
    stream.set_read_timeout(Some(INTERACTIVE_LAUNCH_TIMEOUT))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;

    let mut request_bytes = [0u8; 8];
    getrandom::fill(&mut request_bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;
    let request_id: String = request_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    stream.write_all(format!("MINECRAFT_EDUCATION {request_id}\n").as_bytes())?;
    stream.shutdown(Shutdown::Write)?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    if response.trim() == format!("OK:{request_id}") {
        return Ok(MinecraftLaunchResult {
            schema: MINECRAFT_LAUNCH_SCHEMA,
            state: MinecraftLaunchState::Launched,
        });
    }
    if response.starts_with(&format!("ERROR:{request_id}:")) {
        return Err(io::Error::new(io::ErrorKind::Other, response.trim().to_string()).into());
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Interactive launcher returned an invalid response",
    )
    .into())
}

#[cfg(target_os = "windows")]
fn register_interactive_launcher() -> Result<(), Box<dyn std::error::Error>> {
    let app_data = std::env::var_os("APPDATA").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "APPDATA is unavailable for the interactive Windows user",
        )
    })?;
    let startup = std::path::PathBuf::from(app_data)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup");
    std::fs::create_dir_all(&startup)?;
    let executable = std::env::current_exe()?;
    let launcher = startup.join("M-Bedrock Virtual Interactive Launcher.cmd");
    let command = format!(
        "@echo off\r\nstart \"\" /min \"{}\" --interactive-launcher\r\n",
        executable.display()
    );
    std::fs::write(&launcher, command)?;
    if !interactive_launcher_ready() {
        std::process::Command::new(&executable)
            .arg("--interactive-launcher")
            .spawn()?;
    }
    println!("{}", launcher.display());
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn register_interactive_launcher() -> Result<(), Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Interactive launcher registration currently targets Windows guests",
    )
    .into())
}

#[cfg(target_os = "windows")]
fn request_interactive_shutdown() -> Result<(), Box<dyn std::error::Error>> {
    let address = interactive_launcher_address().parse()?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_millis(300))?;
    stream.set_write_timeout(Some(Duration::from_millis(300)))?;
    stream.write_all(b"SHUTDOWN\n")?;
    Ok(())
}

fn run_interactive_launcher() -> Result<(), Box<dyn std::error::Error>> {
    if !interactive_session_available() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Interactive launcher must run in an interactive Windows user session",
        )
        .into());
    }
    let listener = TcpListener::bind(("127.0.0.1", INTERACTIVE_LAUNCHER_PORT))?;
    for incoming in listener.incoming() {
        let Ok(mut stream) = incoming else {
            continue;
        };
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(INTERACTIVE_LAUNCH_TIMEOUT))?;
        let mut request = String::new();
        // Read::take consumes its receiver. Borrow the socket so the same
        // connection can send an explicit response after the bounded read.
        (&mut stream)
            .take(MAX_INTERACTIVE_REQUEST_BYTES + 1)
            .read_to_string(&mut request)?;
        if request.len() as u64 > MAX_INTERACTIVE_REQUEST_BYTES {
            stream.write_all(b"ERROR:INVALID:request too large\n")?;
            continue;
        }
        if request.trim().is_empty() {
            continue;
        }
        let mut parts = request.split_whitespace();
        let verb = parts.next().unwrap_or_default();
        let request_id = parts.next().unwrap_or_default();
        let request_id_valid = request_id.len() == 16
            && request_id
                .chars()
                .all(|character| character.is_ascii_hexdigit())
            && parts.next().is_none();
        let response = if verb == "PING" && request_id_valid {
            format!("READY:{request_id}\n")
        } else if verb == "MINECRAFT_EDUCATION" && request_id_valid {
            match launch_minecraft_interactive() {
                Ok(()) => format!("OK:{request_id}\n"),
                Err(error) => format!("ERROR:{request_id}:{error}\n"),
            }
        } else {
            "ERROR:INVALID:unsupported interactive action\n".to_string()
        };
        stream.write_all(response.as_bytes())?;
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn run_interactive_launcher() -> Result<(), Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Interactive launcher currently targets Windows guests",
    )
    .into())
}

// TCP may split headers across multiple reads. Do not wait for EOF because
// the host waits for a response without half-closing its write side.
fn read_guest_request(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let deadline = Instant::now() + REQUEST_HEADER_TIMEOUT;
    let mut request = Vec::with_capacity(512);
    let mut chunk = [0_u8; 512];
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Guest Agent header timed out",
            ));
        }
        stream.set_read_timeout(Some(remaining))?;
        let limit = (MAX_GUEST_REQUEST_BYTES + 1 - request.len()).min(chunk.len());
        let size = stream.read(&mut chunk[..limit])?;
        if size == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Guest Agent header incomplete",
            ));
        }
        request.extend_from_slice(&chunk[..size]);
        if request.len() > MAX_GUEST_REQUEST_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Guest Agent request exceeds size limit",
            ));
        }
        if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            return Ok(request);
        }
    }
}

fn handle(
    mut stream: TcpStream,
    token: &str,
    launch_gate: &Mutex<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let request = match read_guest_request(&mut stream) {
        Ok(request) => request,
        Err(error) => {
            let response = match error.kind() {
                io::ErrorKind::InvalidData =>
                    b"HTTP/1.1 413 Payload Too Large\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".as_slice(),
                io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock =>
                    b"HTTP/1.1 408 Request Timeout\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".as_slice(),
                _ =>
                    b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".as_slice(),
            };
            stream.write_all(response)?;
            return Ok(());
        }
    };
    let request = match std::str::from_utf8(&request) {
        Ok(value) => value,
        Err(_) => {
            stream.write_all(
                b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )?;
            return Ok(());
        }
    };
    let (headers, body) = request
        .split_once("\r\n\r\n")
        .expect("read_guest_request returned a complete header");
    let first_line = headers.lines().next().unwrap_or_default();
    if !body.is_empty() {
        stream.write_all(
            b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        )?;
        return Ok(());
    }

    let supplied: Vec<&str> = headers
        .lines()
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("X-Virtual-Clients-Token")
                .then(|| value.trim())
        })
        .collect();
    let request_ids: Vec<&str> = headers
        .lines()
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case(REQUEST_ID_HEADER)
                .then(|| value.trim())
        })
        .collect();

    let host_count = headers
        .lines()
        .filter(|line| {
            line.split_once(':')
                .is_some_and(|(name, _)| name.eq_ignore_ascii_case("Host"))
        })
        .count();
    let content_lengths: Vec<&str> = headers
        .lines()
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("Content-Length")
                .then(|| value.trim())
        })
        .collect();
    let transfer_encoding = headers.lines().any(|line| {
        line.split_once(':')
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case("Transfer-Encoding"))
    });
    if host_count != 1
        || transfer_encoding
        || content_lengths.len() > 1
        || content_lengths.first().is_some_and(|value| *value != "0")
    {
        stream.write_all(
            b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        )?;
        return Ok(());
    }

    if supplied.len() != 1 || !timing_safe_token_eq(supplied[0], token) {
        stream.write_all(
            b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        )?;
        return Ok(());
    }

    if request_ids.len() != 1
        || request_ids[0].len() != 32
        || !request_ids[0]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        stream.write_all(
            b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        )?;
        return Ok(());
    }
    let request_id = request_ids[0];

    match first_line {
        "GET /status HTTP/1.1" => write_json(&mut stream, request_id, &current_guest_status()),
        "POST /minecraft/launch HTTP/1.1" => {
            // One interactive launch at a time; status requests remain independent.
            let _launch = match launch_gate.try_lock() {
                Ok(guard) => guard,
                Err(_) => {
                    stream.write_all(SERVICE_UNAVAILABLE)?;
                    return Ok(());
                }
            };
            match launch_minecraft() {
                Ok(result) => write_json(&mut stream, request_id, &result),
                Err(error) => {
                    eprintln!("Minecraft Education launch request failed: {error}");
                    stream.write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")?;
                    Ok(())
                }
            }
        }
        _ => {
            stream.write_all(
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )?;
            Ok(())
        }
    }
}

fn write_json(
    stream: &mut TcpStream,
    request_id: &str,
    value: &impl serde::Serialize,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::to_string(value)?;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n{REQUEST_ID_HEADER}: {request_id}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn interactive_session_available() -> bool {
    // A hung Windows status probe cannot occupy a Guest Agent worker forever.
    bounded_command_output(
        std::path::Path::new("powershell.exe"),
        [
            "-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
            "$session = (Get-Process -Id $PID).SessionId; $interactive = @(Get-Process explorer -ErrorAction SilentlyContinue | Where-Object { $_.SessionId -eq $session }).Count -gt 0; ($session -ne 0) -and $interactive",
        ],
        Duration::from_secs(4),
    )
    .is_ok_and(|value| value.eq_ignore_ascii_case("True"))
}

#[cfg(target_os = "windows")]
fn minecraft_process_running() -> bool {
    // Avoid starting PowerShell merely to check for an existing game process.
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    system.processes().values().any(|process| {
        let name = process.name().to_string_lossy().to_ascii_lowercase();
        name.contains("minecraft") || name.contains("education")
    })
}

#[cfg(target_os = "windows")]
fn launch_minecraft(
) -> Result<m_bedrock_virtual_clients_core::MinecraftLaunchResult, Box<dyn std::error::Error>> {
    request_interactive_minecraft_launch()
}

#[cfg(target_os = "windows")]
fn launch_minecraft_interactive() -> Result<(), Box<dyn std::error::Error>> {
    if !interactive_session_available() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Interactive launcher is not running in an interactive Windows user session",
        )
        .into());
    }
    if minecraft_process_running() {
        return Ok(());
    }

    let profile = guest_agent_minecraft_profile().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Minecraft Education installation is unavailable",
        )
    })?;

    let command = match profile.install_type {
        m_bedrock_virtual_clients_core::MinecraftInstallType::Store => {
            "Start-Process 'shell:AppsFolder\\Microsoft.MinecraftEducationEdition_8wekyb3d8bbwe!Microsoft.MinecraftEducationEdition'"
        }
        _ => {
            "$app = Get-StartApps | Where-Object { $_.Name -like '*Minecraft Education*' } | Select-Object -First 1; if (-not $app) { throw 'Minecraft Education Start menu entry is unavailable' }; Start-Process ('shell:AppsFolder\\' + $app.AppID)"
        }
    };
    bounded_command_output(
        std::path::Path::new("powershell.exe"),
        [
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            command,
        ],
        Duration::from_secs(10),
    )
    .map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("Minecraft Education launch failed: {error}"),
        )
    })?;

    let deadline = std::time::Instant::now() + MINECRAFT_PROCESS_START_TIMEOUT;
    while std::time::Instant::now() < deadline {
        if minecraft_process_running() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "Minecraft Education did not start within 30 seconds",
    )
    .into())
}

#[cfg(not(target_os = "windows"))]
fn launch_minecraft(
) -> Result<m_bedrock_virtual_clients_core::MinecraftLaunchResult, Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Minecraft launch currently targets Windows guests",
    )
    .into())
}

#[cfg(target_os = "windows")]
fn guest_machine_identity() -> Option<String> {
    let machine_guid = bounded_command_output(
        std::path::Path::new("powershell.exe"),
        [
            "-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
            "(Get-ItemProperty -LiteralPath 'HKLM:\\SOFTWARE\\Microsoft\\Cryptography' -Name MachineGuid -ErrorAction Stop).MachineGuid",
        ],
        Duration::from_secs(4),
    ).ok()?;
    let machine_guid = machine_guid.trim();
    if machine_guid.is_empty() || machine_guid.len() > 128 {
        return None;
    }
    Some(
        Sha256::digest(machine_guid.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
    )
}

#[cfg(not(target_os = "windows"))]
fn guest_machine_identity() -> Option<String> {
    None
}

fn timing_safe_token_eq(candidate: &str, expected: &str) -> bool {
    if candidate.len() != expected.len() {
        return false;
    }

    candidate
        .as_bytes()
        .iter()
        .zip(expected.as_bytes())
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

#[cfg(target_os = "windows")]
fn guest_agent_token() -> io::Result<String> {
    let program_files = std::env::var_os("ProgramFiles")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "ProgramFiles is unavailable"))?;
    let vmtoolsd = std::path::PathBuf::from(program_files)
        .join("VMware")
        .join("VMware Tools")
        .join("vmtoolsd.exe");
    if !vmtoolsd.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "VMware Tools vmtoolsd.exe is missing: {}",
                vmtoolsd.display()
            ),
        ));
    }
    let token = bounded_command_output(
        &vmtoolsd,
        ["--cmd", "info-get guestinfo.virtualclients.token"],
        Duration::from_secs(10),
    )
    .map_err(|error| {
        io::Error::new(
            error.kind(),
            "Guest Agent token could not be read from VMware Tools",
        )
    })?;
    if token.len() != 64 || !token.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Guest Agent token has an invalid format",
        ));
    }
    Ok(token)
}

#[cfg(not(target_os = "windows"))]
fn guest_agent_token() -> io::Result<String> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Virtual Guest Agent currently targets Windows guests",
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        read_guest_request, run_mode, timing_safe_token_eq, RunMode, MAX_ACTIVE_REQUESTS,
        MAX_GUEST_REQUEST_BYTES, MAX_INTERACTIVE_REQUEST_BYTES,
    };
    use std::{
        io::{self, Write},
        net::{Shutdown, TcpListener, TcpStream},
        thread,
    };

    #[test]
    fn transport_limits_are_small_and_explicit() {
        assert_eq!(MAX_GUEST_REQUEST_BYTES, 4096);
        assert_eq!(MAX_ACTIVE_REQUESTS, 8);
        assert_eq!(MAX_INTERACTIVE_REQUEST_BYTES, 512);
    }

    #[test]
    fn bounded_http_reader_handles_segmented_headers_and_rejects_bad_frames() {
        fn exchange(chunks: &[&[u8]]) -> io::Result<Vec<u8>> {
            let listener = TcpListener::bind(("127.0.0.1", 0))?;
            let address = listener.local_addr()?;
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept()?;
                read_guest_request(&mut stream)
            });
            let mut client = TcpStream::connect(address)?;
            for part in chunks {
                client.write_all(part)?;
                thread::yield_now();
            }
            client.shutdown(Shutdown::Write)?;
            server.join().expect("server thread panicked")
        }

        let header = b"GET /status HTTP/1.1\r\nHost: virtual-client\r\n\r\n";
        assert_eq!(
            exchange(&[
                b"GET /status HTTP/1.1\r\nHost:",
                b" virtual-client\r\n",
                b"\r\n"
            ])
            .unwrap(),
            header
        );
        assert_eq!(
            exchange(&[&vec![b'X'; MAX_GUEST_REQUEST_BYTES + 1]])
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(
            exchange(&[b"GET /status HTTP/1.1\r\nHost: virtual-client\r\n"])
                .unwrap_err()
                .kind(),
            io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn ssh_one_shot_modes_do_not_fall_through_to_vmware_service() {
        assert_eq!(run_mode(&[]).unwrap(), RunMode::VmwareService);
        assert_eq!(
            run_mode(&["--ssh-status".into()]).unwrap(),
            RunMode::SshStatus
        );
        assert_eq!(
            run_mode(&["--ssh-launch-minecraft".into()]).unwrap(),
            RunMode::SshLaunch
        );
        assert_eq!(
            run_mode(&["--interactive-launcher".into()]).unwrap(),
            RunMode::InteractiveLauncher
        );
        for invalid in [
            vec!["--ssh-status".into(), "--interactive-launcher".into()],
            vec!["--unknown-flag".into()],
            vec!["--ssh-launch-minecraft".into(), "other-command".into()],
        ] {
            assert!(run_mode(&invalid).is_err());
        }
    }

    #[test]
    fn guest_token_comparison_requires_exact_match() {
        let token = "a".repeat(64);
        assert!(timing_safe_token_eq(&token, &token));
        assert!(!timing_safe_token_eq(&"b".repeat(64), &token));
        assert!(!timing_safe_token_eq("short", &token));
    }
}
