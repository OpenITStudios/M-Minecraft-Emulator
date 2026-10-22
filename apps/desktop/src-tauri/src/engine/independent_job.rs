//! Original Windows process-lifecycle primitive for MIVUBI isolation.
//! NO Sandboxie binaries, service, driver, DLL or runtime dependencies.
//! A Job Object is NOT filesystem, registry, COM, IPC or WAM isolation.
//!
//! This module is deliberately unmounted from Tauri. Its only creation
//! path runs our OWN test executable. Never use it to launch Minecraft;
//! persistent game processes require a durable service and verified data
//! isolation, not this disposable fixture's kill-on-close policy.

#![allow(dead_code)]

use std::{io, mem, ptr};
#[cfg(test)]
use std::{ffi::OsStr, os::windows::ffi::OsStrExt, time::Duration};
#[cfg(test)]
use virtual_clients_core::independent_engine::ProcessIdentity;
use windows_sys::Win32::{
    Foundation::{CloseHandle, FILETIME, HANDLE, WAIT_OBJECT_0},
    System::{
        JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, IsProcessInJob,
            SetInformationJobObject, JobObjectExtendedLimitInformation,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        },
        Threading::{
            CreateProcessW, GetProcessId, GetProcessTimes, ResumeThread,
            TerminateProcess, WaitForSingleObject, CREATE_NO_WINDOW,
            CREATE_SUSPENDED, PROCESS_INFORMATION, STARTUPINFOW,
        },
    },
};

struct OwnedHandle(HANDLE);

impl OwnedHandle {
    fn checked(value: HANDLE) -> io::Result<Self> {
        if value.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self(value))
        }
    }

    fn raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: Exactly one Rust owner for this Windows handle.
            unsafe { CloseHandle(self.0) };
        }
    }
}

/// An unnamed, local-to-this-test Job Object, not a security sandbox.
/// The last handle closing terminates the disposable test process tree.
struct FixtureJob {
    handle: OwnedHandle,
}

impl FixtureJob {
    fn create() -> io::Result<Self> {
        // SAFETY: Null parameters create a new, unnamed job with the calling
        // process security context; it cannot collide with a Sandboxie name.
        let handle = OwnedHandle::checked(unsafe {
            CreateJobObjectW(ptr::null(), ptr::null())
        })?;
        // SAFETY: Windows POD struct, zero-initialized before passing to OS.
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION =
            unsafe { mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: Windows copies a valid struct at its exact ABI size.
        let ok = unsafe {
            SetInformationJobObject(
                handle.raw(),
                JobObjectExtendedLimitInformation,
                &limits as *const _ as *const _,
                mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { handle })
    }

    /// Only available to the Rust Windows test runner. Never accepts a
    /// game path, operator-provided executable, command or Microsoft token.
    #[cfg(test)]
    fn spawn_self_test(self) -> io::Result<SuspendedFixture> {
        let image = std::env::current_exe()?.canonicalize()?;
        let utf16: Vec<u16> = image.as_os_str().encode_wide().collect();
        if utf16.is_empty() || utf16.len() > 30_000 || utf16.contains(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput, "Unsafe fixture executable pathname",
            ));
        }
        let mut image_nul = utf16.clone();
        image_nul.push(0);
        // Fixed Rust test-runner --help invocation. No shell or untrusted
        // argument parsing. OS launch uses the *absolute* lpApplicationName.
        let mut command = Vec::with_capacity(utf16.len() + 12);
        command.push(b'"' as u16);
        command.extend_from_slice(&utf16);
        command.extend(OsStr::new("\" --help").encode_wide());
        command.push(0);
        // SAFETY: Both C structs are POD, pointers remain alive throughout
        // CreateProcessW, and no inheritable handle or user tokens are used.
        let mut startup: STARTUPINFOW = unsafe { mem::zeroed() };
        startup.cb = mem::size_of::<STARTUPINFOW>() as u32;
        let mut pi: PROCESS_INFORMATION = unsafe { mem::zeroed() };
        let started = unsafe {
            CreateProcessW(
                image_nul.as_ptr(), command.as_mut_ptr(),
                ptr::null(), ptr::null(), 0,
                CREATE_SUSPENDED | CREATE_NO_WINDOW,
                ptr::null(), ptr::null(), &startup, &mut pi,
            )
        };
        if started == 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: Successful CreateProcessW provides these two uniquely
        // owned handles. Invalid output is rejected rather than guessed.
        let process = match OwnedHandle::checked(pi.hProcess) {
            Ok(value) => value,
            Err(error) => {
                if !pi.hThread.is_null() {
                    unsafe { CloseHandle(pi.hThread) };
                }
                return Err(error);
            }
        };
        let thread = match OwnedHandle::checked(pi.hThread) {
            Ok(value) => value,
            Err(error) => {
                terminate_failed_child(process.raw());
                return Err(error);
            }
        };
        // The child's primary thread has not executed any instructions.
        // SAFETY: Attach only this newly created, suspended process; on
        // failure terminate only that child and preserve unrelated apps.
        if unsafe { AssignProcessToJobObject(self.handle.raw(), process.raw()) } == 0 {
            let error = io::Error::last_os_error();
            terminate_failed_child(process.raw());
            return Err(error);
        }
        let identity = match pinned_identity(process.raw(), pi.dwProcessId) {
            Ok(value) => value,
            Err(error) => {
                terminate_failed_child(process.raw());
                return Err(error);
            }
        };
        // Return the process STILL SUSPENDED. Tests and future reviewed
        // enrollment logic can inspect real OS membership before any child
        // instructions run; a --help process may otherwise exit too quickly.
        Ok(SuspendedFixture { job: self, process, thread, identity })
    }
}

#[cfg(test)]
fn terminate_failed_child(child: HANDLE) {
    // SAFETY: Child HANDLE was returned by THIS CreateProcessW invocation;
    // never a stale PID, foreign client, existing world or installed game.
    unsafe {
        TerminateProcess(child, 1);
        let _ = WaitForSingleObject(child, 5_000);
    }
}

#[cfg(test)]
fn pinned_identity(process: HANDLE, expected_pid: u32) -> io::Result<ProcessIdentity> {
    // SAFETY: GetProcessTimes fills four properly aligned FILETIME structs.
    let mut created: FILETIME = unsafe { mem::zeroed() };
    let mut exited: FILETIME = unsafe { mem::zeroed() };
    let mut kernel: FILETIME = unsafe { mem::zeroed() };
    let mut user: FILETIME = unsafe { mem::zeroed() };
    if unsafe {
        GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user)
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: Query the retained process HANDLE, not a discovered PID.
    let actual_pid = unsafe { GetProcessId(process) };
    let timestamp = ((created.dwHighDateTime as u64) << 32)
        | (created.dwLowDateTime as u64);
    if actual_pid == 0 || actual_pid != expected_pid {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Process identity changed",
        ));
    }
    ProcessIdentity::checked(actual_pid, timestamp).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Missing creation FILETIME")
    })
}

#[cfg(test)]
struct SuspendedFixture {
    // Dropping an un-resumed fixture closes the job first, killing its
    // suspended process and descendants without orphaning either HANDLE.
    job: FixtureJob,
    process: OwnedHandle,
    thread: OwnedHandle,
    identity: ProcessIdentity,
}

#[cfg(test)]
impl SuspendedFixture {
    fn member_of(&self, job: &FixtureJob) -> io::Result<bool> {
        let mut member = 0;
        // SAFETY: Both OS handles remain owned and live while our child
        // is suspended. Membership evidence cannot race a quick --help exit.
        if unsafe {
            IsProcessInJob(self.process.raw(), job.handle.raw(), &mut member)
        } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(member != 0)
        }
    }

    fn resume(self) -> io::Result<FixtureChild> {
        if pinned_identity(self.process.raw(), self.identity.pid)? != self.identity {
            terminate_failed_child(self.process.raw());
            return Err(io::Error::new(
                io::ErrorKind::InvalidData, "Suspended child identity changed",
            ));
        }
        // SAFETY: Resume exactly the retained CREATE_SUSPENDED primary thread
        // after OS job membership has been independently observed.
        if unsafe { ResumeThread(self.thread.raw()) } == u32::MAX {
            let error = io::Error::last_os_error();
            terminate_failed_child(self.process.raw());
            return Err(error);
        }
        let Self { job, process, thread, identity } = self;
        drop(thread);
        Ok(FixtureChild { job, process, identity })
    }
}

#[cfg(test)]
struct FixtureChild {
    // Rust fields drop in declaration order; last job handle closes before
    // the process handle, killing any remaining disposable descendants.
    job: FixtureJob,
    process: OwnedHandle,
    identity: ProcessIdentity,
}

#[cfg(test)]
impl FixtureChild {
    fn wait(&self, deadline: Duration) -> io::Result<()> {
        let ms = (deadline.as_millis().min((u32::MAX - 1) as u128)) as u32;
        // SAFETY: The process handle remains valid for this bounded wait.
        if unsafe { WaitForSingleObject(self.process.raw(), ms) } == WAIT_OBJECT_0 {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::TimedOut, "Fixture child did not exit",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn own_suspended_fixture_is_assigned_to_job_before_resume() {
        let suspended = FixtureJob::create().unwrap().spawn_self_test().unwrap();
        assert!(suspended.identity.pid > 0 && suspended.identity.created_file_time > 0);
        assert!(suspended.member_of(&suspended.job).unwrap());
        let child = suspended.resume().unwrap();
        child.wait(Duration::from_secs(20)).unwrap();
    }

    #[test]
    fn separate_job_cannot_claim_fixture_as_member() {
        let unrelated = FixtureJob::create().unwrap();
        let suspended = FixtureJob::create().unwrap().spawn_self_test().unwrap();
        assert!(suspended.member_of(&suspended.job).unwrap());
        assert!(!suspended.member_of(&unrelated).unwrap());
        let child = suspended.resume().unwrap();
        child.wait(Duration::from_secs(20)).unwrap();
    }
}
