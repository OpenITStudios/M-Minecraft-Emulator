use std::{
    ffi::OsStr,
    io,
    io::{Read, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(45);

// Pipe readers may outlive a direct child if an unrelated descendant inherits
// stdout/stderr. Cap retained workers rather than allowing repeated timed-out
// inspections to accumulate unbounded detached threads. This is a safety
// ceiling, not proof that OS pipe cancellation or descendant cleanup works.
const MAX_BOUNDED_IO_WORKERS: usize = 24;
static BOUNDED_IO_WORKERS: AtomicUsize = AtomicUsize::new(0);

struct IoWorkerSlot;

impl Drop for IoWorkerSlot {
    fn drop(&mut self) {
        BOUNDED_IO_WORKERS.fetch_sub(1, Ordering::AcqRel);
    }
}

fn reserve_io_workers(count: usize) -> io::Result<Vec<IoWorkerSlot>> {
    BOUNDED_IO_WORKERS
        .try_update(Ordering::AcqRel, Ordering::Acquire, |current| {
            current
                .checked_add(count)
                .filter(|next| *next <= MAX_BOUNDED_IO_WORKERS)
        })
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::WouldBlock,
                "too many active provider I/O workers; previous subprocess pipes may still be open",
            )
        })?;
    Ok((0..count).map(|_| IoWorkerSlot).collect())
}

pub(crate) fn command_output<I, S>(program: &Path, args: I) -> io::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    command_output_with_timeout(program, args, COMMAND_TIMEOUT)
}

pub(crate) fn command_output_with_timeout<I, S>(
    program: &Path,
    args: I,
    timeout: Duration,
) -> io::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    bounded_process(program, args, timeout, None)
}

// Keep long, fixed PowerShell source off the Windows command line. The
// canonical bounded runner owns its subprocess, streams and timeout.
pub(crate) fn command_output_with_stdin<I, S>(
    program: &Path,
    args: I,
    timeout: Duration,
    input: &[u8],
) -> io::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    const MAX_SCRIPT_BYTES: usize = 128 * 1024;
    if input.is_empty() || input.len() > MAX_SCRIPT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "bounded process input is empty or exceeds 128 KiB",
        ));
    }
    bounded_process(program, args, timeout, Some(input))
}

fn bounded_process<I, S>(
    program: &Path,
    args: I,
    timeout: Duration,
    input: Option<&[u8]>,
) -> io::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    // Reserve before spawning the process. An unavailable slot refuses work
    // without starting another helper or increasing resource pressure.
    let slots = reserve_io_workers(if input.is_some() { 3 } else { 2 })?;
    let mut slots = slots.into_iter();
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Background helpers launched by the GUI must not allocate a console.
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let mut child = command
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // Write stdin concurrently with stdout/stderr drainage. This prevents
    // deadlocks when the script is larger than the child's stdin pipe.
    let input_result = if let Some(bytes) = input {
        let mut writer = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing bounded command stdin"))?;
        let data = bytes.to_vec();
        let (tx, rx) = mpsc::sync_channel(1);
        let slot = slots.next().expect("reserved stdin worker slot");
        spawn_worker_or_reap(&mut child, program, || {
            thread::Builder::new().spawn(move || {
                let _slot = slot;
                let _ = tx.send(writer.write_all(&data));
            })
        })?;
        Some(rx)
    } else {
        None
    };

    // Drain both streams while the child is running. Waiting before reading can
    // deadlock when the child fills an OS pipe buffer.
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing stdout pipe"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing stderr pipe"))?;
    let (stdout_sender, stdout_reader) = mpsc::sync_channel(1);
    let (stderr_sender, stderr_reader) = mpsc::sync_channel(1);
    // An overflowing helper is already invalid evidence. Stop reading it
    // rather than retaining a worker until an arbitrary descendant closes EOF.
    let oversized_output = Arc::new(AtomicBool::new(false));
    let stdout_overflow = Arc::clone(&oversized_output);
    let stderr_overflow = Arc::clone(&oversized_output);
    let stdout_slot = slots.next().expect("reserved stdout worker slot");
    let stderr_slot = slots.next().expect("reserved stderr worker slot");
    spawn_worker_or_reap(&mut child, program, || {
        thread::Builder::new().spawn(move || {
            let _slot = stdout_slot;
            let _ = stdout_sender.send(drain_output(stdout, &stdout_overflow));
        })
    })?;
    spawn_worker_or_reap(&mut child, program, || {
        thread::Builder::new().spawn(move || {
            let _slot = stderr_slot;
            let _ = stderr_sender.send(drain_output(stderr, &stderr_overflow));
        })
    })?;

    let started = Instant::now();
    loop {
        // Never wait for EOF from a stream whose bounded capture already
        // overflowed. Terminate ONLY our direct helper, not game descendants.
        if oversized_output.load(Ordering::Acquire) {
            terminate_direct_child(&mut child, program, "oversized output")?;
            return Err(oversized_output_error(program));
        }
        let observed_status = match child.try_wait() {
            Ok(status) => status,
            Err(error) => {
                terminate_direct_child(&mut child, program, "status check")?;
                return Err(error);
            }
        };
        if let Some(status) = observed_status {
            if let Some(receiver) = &input_result {
                let remaining = timeout.saturating_sub(started.elapsed());
                receiver.recv_timeout(remaining).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::TimedOut,
                        "bounded command input writer did not complete",
                    )
                })??;
            }
            // A descendant may inherit a pipe even after vmrun exits. Do not
            // block indefinitely waiting for EOF from an unrelated process.
            let remaining = timeout.saturating_sub(started.elapsed());
            let stdout = receive_output(&stdout_reader, remaining, &oversized_output)?;
            let remaining = timeout.saturating_sub(started.elapsed());
            let stderr = receive_output(&stderr_reader, remaining, &oversized_output)?;
            if oversized_output.load(Ordering::Acquire)
                || stdout.len() > OUTPUT_LIMIT
                || stderr.len() > OUTPUT_LIMIT
            {
                return Err(oversized_output_error(program));
            }
            return require_success(program, status.success(), &stdout, &stderr);
        }
        if started.elapsed() >= timeout {
            terminate_direct_child(&mut child, program, "timeout")?;
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!(
                    "{} exceeded {}s timeout; child process reaped",
                    program.display(),
                    timeout.as_secs()
                ),
            ));
        }
        thread::sleep(Duration::from_millis(100));
    }
}

// Thread creation can fail under resource pressure. Convert allocation
// failures into ordinary errors and reap only the directly owned helper.
// Dropped successful JoinHandles detach; IoWorkerSlot stays owned by the thread.
fn spawn_worker_or_reap(
    child: &mut Child,
    program: &Path,
    spawn: impl FnOnce() -> io::Result<thread::JoinHandle<()>>,
) -> io::Result<()> {
    match spawn() {
        Ok(handle) => {
            drop(handle);
            Ok(())
        }
        Err(error) => {
            terminate_direct_child(child, program, "I/O worker allocation failure")
                .map_err(|cleanup| io::Error::new(
                    cleanup.kind(),
                    format!(
                        "{} could not create an I/O worker ({error}); helper cleanup failed: {cleanup}",
                        program.display()
                    ),
                ))?;
            Err(io::Error::new(
                error.kind(),
                format!(
                    "{} could not create an I/O worker: {error}",
                    program.display()
                ),
            ))
        }
    }
}

// Only manage the direct child created here, never VMware descendants.
// A failed kill must not lead to an unconditional, potentially infinite wait.
// An already-exited child is the harmless race between try_wait and kill.
fn terminate_direct_child(child: &mut Child, program: &Path, context: &str) -> io::Result<()> {
    match child.kill() {
        Ok(()) => child.wait().map(|_| ()).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("{} {context}; child reap unconfirmed: {error}", program.display()),
            )
        }),
        Err(kill_error) => match child.try_wait() {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err(io::Error::new(
                kill_error.kind(),
                format!("{} {context}; child is still running and could not be terminated: {kill_error}", program.display()),
            )),
            Err(probe_error) => Err(io::Error::new(
                probe_error.kind(),
                format!("{} {context}; child status unknown (kill: {kill_error}; probe: {probe_error})", program.display()),
            )),
        },
    }
}

// Keep diagnostic memory bounded. Oversize output is invalid evidence, so
// release that pipe promptly rather than draining unbounded producer output.
const OUTPUT_LIMIT: usize = 64 * 1024;

fn oversized_output_error(program: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "{} output exceeded {} bytes per stream; refusing truncated provider evidence",
            program.display(),
            OUTPUT_LIMIT
        ),
    )
}

fn drain_output(mut pipe: impl Read, oversized: &AtomicBool) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        // A peer reader may have invalidated the entire helper observation.
        // When this reader can make progress, stop before another read instead
        // of retaining the worker or consuming additional, useless bytes.
        // This does NOT cancel an OS read already blocked on an inherited pipe.
        if oversized.load(Ordering::Acquire) {
            break;
        }
        let count = pipe.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        // Keep one sentinel byte, then close this reader: additional output
        // cannot make an already-invalid helper result trustworthy again.
        let remaining = (OUTPUT_LIMIT + 1).saturating_sub(output.len());
        output.extend_from_slice(&chunk[..count.min(remaining)]);
        if output.len() > OUTPUT_LIMIT {
            oversized.store(true, Ordering::Release);
            break;
        }
    }
    Ok(output)
}

fn receive_output(
    reader: &mpsc::Receiver<io::Result<Vec<u8>>>,
    timeout: Duration,
    oversized: &AtomicBool,
) -> io::Result<Vec<u8>> {
    let started = Instant::now();
    loop {
        // A second stream may exceed the cap while the first still has an
        // inherited open handle. Do not wait out the whole deadline in that case.
        if oversized.load(Ordering::Acquire) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "output exceeded provider limit; refusing truncated provider evidence",
            ));
        }
        let remaining = timeout.saturating_sub(started.elapsed());
        match reader.recv_timeout(remaining.min(Duration::from_millis(100))) {
            Ok(result) => return result,
            Err(mpsc::RecvTimeoutError::Timeout) if started.elapsed() >= timeout => {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "command exited but an output pipe remained open beyond the deadline",
                ));
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "command output reader exited without a result",
                ));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}

fn require_success(
    program: &Path,
    success: bool,
    stdout: &[u8],
    stderr: &[u8],
) -> io::Result<String> {
    if success {
        return Ok(String::from_utf8_lossy(stdout).trim().to_string());
    }
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();
    let detail = if !stderr.is_empty() { stderr } else { stdout };
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("{} failed: {}", program.display(), detail),
    ))
}

pub(crate) fn wait_for_state<F>(
    mut predicate: F,
    expected: bool,
    timeout: Duration,
) -> io::Result<()>
where
    F: FnMut() -> io::Result<bool>,
{
    let started = Instant::now();
    loop {
        if predicate()? == expected {
            return Ok(());
        }
        if started.elapsed() >= timeout {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "virtual machine did not reach the expected power state",
            ));
        }
        thread::sleep(Duration::from_millis(250));
    }
}

#[cfg(test)]
mod tests {
    use super::{command_output_with_timeout, terminate_direct_child};
    use std::process::{Command, Stdio};
    use std::{io, path::Path, time::Duration};

    #[test]
    fn worker_allocation_failure_reaps_only_direct_helper() {
        #[cfg(windows)]
        let (program, args): (&Path, &[&str]) =
            (Path::new("cmd.exe"), &["/C", "ping -n 10 127.0.0.1 >NUL"]);
        #[cfg(not(windows))]
        let (program, args): (&Path, &[&str]) = (Path::new("sh"), &["-c", "sleep 5"]);

        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start a disposable, directly owned test helper");

        // Inject thread allocation failure without exhausting host resources.
        let error = super::spawn_worker_or_reap(&mut child, program, || {
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "synthetic thread quota",
            ))
        })
        .expect_err("worker allocation failure must return, not panic");
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
        assert!(error.to_string().contains("could not create an I/O worker"));
        assert!(
            child.try_wait().unwrap().is_some(),
            "direct helper must be reaped"
        );
    }

    #[test]
    fn bounded_stdin_command_delivers_data_without_command_line_embedding() {
        #[cfg(windows)]
        let (program, args): (&Path, &[&str]) = (
            Path::new("powershell.exe"),
            &[
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "[Console]::In.ReadToEnd()",
            ],
        );
        #[cfg(not(windows))]
        let (program, args): (&Path, &[&str]) = (Path::new("cat"), &[]);

        // Cold-starting PowerShell on a shared Windows CI runner can exceed
        // five seconds under parallel Rust tests even for a tiny stdin value.
        // Only the test budget is extended; production and the separate
        // timed-out-child reaping regression retain their existing limits.
        let timeout = if cfg!(windows) {
            Duration::from_secs(20)
        } else {
            Duration::from_secs(5)
        };
        let output = super::command_output_with_stdin(
            program,
            args.iter().copied(),
            timeout,
            b"short-input",
        )
        .expect("bounded stdin should close with EOF and produce output");
        assert_eq!(output, "short-input");
    }

    #[test]
    fn excessive_pipe_workers_are_rejected_before_starting_another_helper() {
        // An impossible request must be rejected independently of other
        // concurrently executing tests and without spawning any child.
        assert!(super::reserve_io_workers(super::MAX_BOUNDED_IO_WORKERS + 1).is_err());
    }

    #[test]
    fn oversized_stdin_is_rejected_before_child_launch() {
        let error = super::command_output_with_stdin(
            Path::new("not-a-real-executable"),
            std::iter::empty::<&str>(),
            Duration::from_secs(1),
            &vec![b'x'; 128 * 1024 + 1],
        )
        .expect_err("input beyond bound must fail before spawn");
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn successful_command_returns_complete_output() {
        #[cfg(windows)]
        let (program, args): (&Path, &[&str]) = (Path::new("cmd.exe"), &["/C", "echo command-ok"]);
        #[cfg(not(windows))]
        let (program, args): (&Path, &[&str]) = (Path::new("sh"), &["-c", "printf command-ok"]);

        let output =
            command_output_with_timeout(program, args.iter().copied(), Duration::from_secs(5))
                .expect("short successful command must complete");
        assert_eq!(output, "command-ok");
    }

    #[test]
    fn failed_command_preserves_stderr_diagnostic() {
        #[cfg(windows)]
        let (program, args): (&Path, &[&str]) = (
            Path::new("cmd.exe"),
            &["/C", "echo expected-failure 1>&2 & exit /B 7"],
        );
        #[cfg(not(windows))]
        let (program, args): (&Path, &[&str]) = (
            Path::new("sh"),
            &["-c", "printf expected-failure >&2; exit 7"],
        );

        let error =
            command_output_with_timeout(program, args.iter().copied(), Duration::from_secs(5))
                .expect_err("failed command must not be treated as success");
        assert!(error.to_string().contains("expected-failure"));
    }

    #[test]
    fn output_reader_has_bounded_wait_without_eof() {
        let (_sender, receiver) = std::sync::mpsc::sync_channel(1);
        let error = super::receive_output(
            &receiver,
            Duration::from_millis(1),
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }

    #[test]
    fn output_drain_is_bounded_even_for_large_streams() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let input = vec![b'x'; super::OUTPUT_LIMIT * 4];
        let mut cursor = std::io::Cursor::new(input);
        let overflow = AtomicBool::new(false);
        let output = super::drain_output(&mut cursor, &overflow).unwrap();
        assert_eq!(output.len(), super::OUTPUT_LIMIT + 1);
        assert!(output.iter().all(|byte| *byte == b'x'));
        assert!(overflow.load(Ordering::Acquire));
        // The reader stopped before EOF even though the source had more data.
        assert!(cursor.position() < (super::OUTPUT_LIMIT * 4) as u64);
    }

    #[test]
    fn peer_overflow_stops_reader_after_current_read() {
        use std::sync::atomic::{AtomicBool, Ordering};

        struct PeerOverflowReader<'a> {
            flag: &'a AtomicBool,
            reads: usize,
        }

        impl io::Read for PeerOverflowReader<'_> {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                self.reads += 1;
                assert_eq!(self.reads, 1, "reader must stop before another read");
                buf[0] = b'x';
                // Simulate the other pipe crossing its cap during this read.
                self.flag.store(true, Ordering::Release);
                Ok(1)
            }
        }

        let flag = AtomicBool::new(false);
        let mut reader = PeerOverflowReader {
            flag: &flag,
            reads: 0,
        };
        let output = super::drain_output(&mut reader, &flag)
            .expect("cooperatively cancelled peer reader should return");
        assert_eq!(reader.reads, 1);
        assert_eq!(output, b"x");
        assert!(flag.load(Ordering::Acquire));
    }

    #[test]
    fn peer_overflow_prevents_initial_read() {
        use std::sync::atomic::AtomicBool;
        struct NeverRead;
        impl io::Read for NeverRead {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                panic!("cancelled reader must not issue a blocking read");
            }
        }

        let flag = AtomicBool::new(true);
        assert!(super::drain_output(NeverRead, &flag).unwrap().is_empty());
    }

    #[test]
    fn inherited_reader_can_be_refused_once_other_stream_overflows() {
        let (_sender, receiver) = std::sync::mpsc::sync_channel(1);
        let overflowing = std::sync::atomic::AtomicBool::new(true);
        let started = std::time::Instant::now();
        let error = super::receive_output(&receiver, Duration::from_secs(5), &overflowing)
            .expect_err("an oversized parallel stream must stop waiting for EOF");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn verbose_command_does_not_block_on_full_pipes() {
        #[cfg(windows)]
        let (program, args): (&Path, &[&str]) = (
            Path::new("cmd.exe"),
            &[
                "/C",
                "for /L %i in (1,1,12000) do @echo some-output-to-fill-the-pipe",
            ],
        );
        #[cfg(not(windows))]
        let (program, args): (&Path, &[&str]) = (
            Path::new("sh"),
            &["-c", "yes some-output-to-fill-the-pipe | head -c 200000"],
        );
        let error =
            command_output_with_timeout(program, args.iter().copied(), Duration::from_secs(15))
                .expect_err("oversized command output must not be treated as complete");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error
            .to_string()
            .contains("refusing truncated provider evidence"));
    }

    #[cfg(windows)]
    #[test]
    fn bounded_helper_has_no_console_window() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let directory =
            std::env::temp_dir().join(format!("mivubi-console-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let host = directory.join("gui-host.exe");
        let compiled = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .arg("--edition=2021")
            .arg(root.join("src/provider/fixtures/gui_helper_host.rs"))
            .arg("-o")
            .arg(&host)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let result = directory.join("console.txt");
        let status = Command::new(&host).arg(&result).status().unwrap();
        assert!(status.success());
        let observed = std::fs::read_to_string(&result).unwrap();
        let _ = std::fs::remove_file(&host);
        let _ = std::fs::remove_file(&result);
        let _ = std::fs::remove_file(directory.join("gui-host.pdb"));
        let _ = std::fs::remove_dir(&directory);
        assert_eq!(
            observed.trim(),
            "0",
            "a GUI manager's background helper must not allocate a console window"
        );
    }

    #[test]
    fn cleanup_accepts_child_that_already_exited() {
        #[cfg(windows)]
        let (program, args): (&Path, &[&str]) = (Path::new("cmd.exe"), &["/C", "exit /B 0"]);
        #[cfg(not(windows))]
        let (program, args): (&Path, &[&str]) = (Path::new("sh"), &["-c", "exit 0"]);
        let mut child = Command::new(program)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        child.wait().unwrap();
        terminate_direct_child(&mut child, program, "already exited").unwrap();
    }

    #[test]
    fn timed_out_command_is_reaped_before_reporting_timeout() {
        #[cfg(windows)]
        let (program, args): (&Path, &[&str]) =
            (Path::new("cmd.exe"), &["/C", "ping -n 10 127.0.0.1 >NUL"]);
        #[cfg(not(windows))]
        let (program, args): (&Path, &[&str]) = (Path::new("sh"), &["-c", "sleep 5"]);

        let error =
            command_output_with_timeout(program, args.iter().copied(), Duration::from_millis(100))
                .expect_err("long-running command must time out");
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(error.to_string().contains("child process reaped"));
    }
}
