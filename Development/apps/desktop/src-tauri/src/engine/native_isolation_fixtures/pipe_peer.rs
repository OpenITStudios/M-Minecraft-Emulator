//! Source-only Windows named-pipe peer correlation for the independent
//! engine. It does not create a pipe, install a service, authenticate users,
//! modify a game or grant ANY production authority.
//!
//! WARNING: The PID returned by GetNamedPipeClientProcessId identifies the
//! connected pipe client, but it does not authenticate each message, prove
//! that the peer holds a unique pipe handle or establish its user SID.
//! A privileged server needs a restrictive explicit pipe DACL, token/session
//! checks and safe impersonation/revert before any storage operation.
//! This module returns PENDING identity evidence ONLY.

#![allow(dead_code)]

use std::{io, mem, ptr};
use virtual_clients_core::independent_engine::{
    ProcessIdentity, UntrustedBrokerProbe,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, FILETIME, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::{
        GetProcessTimes, OpenProcess, WaitForSingleObject,
        PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
    },
};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetNamedPipeClientProcessId(pipe: HANDLE, pid: *mut u32) -> i32;
}

struct ProcessHandle(HANDLE);

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: Exactly one owner of the handle returned by OpenProcess.
            unsafe { CloseHandle(self.0) };
        }
    }
}

/// Diagnostic only. Never use as an authorization or "provider ready" flag.
/// Values must not be logged as identity, credential or sign-in evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PendingPipePeer {
    pub claimed_identity_matches_os_client: bool,
}

/// Called only with an already connected SERVER-side named-pipe HANDLE.
/// This code is deliberately not attached to a Tauri route or service.
/// A future authenticated broker must separately inspect client tokens,
/// pipe ACL, session and actual message provenance.
pub(super) fn inspect_connected_server_pipe(
    server_pipe: HANDLE,
    untrusted: UntrustedBrokerProbe,
) -> io::Result<PendingPipePeer> {
    if server_pipe.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput, "No OS server-side pipe capability",
        ));
    }
    if untrusted.slot.is_host_native()
        || untrusted.claimed_process.pid == 0
        || untrusted.claimed_process.created_file_time == 0
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput, "Untrusted probe has no valid isolated process",
        ));
    }
    let mut peer_pid = 0u32;
    // SAFETY: Server HANDLE must be created by CreateNamedPipeW and
    // owned by the caller. This function neither creates nor mutates it.
    if unsafe { GetNamedPipeClientProcessId(server_pipe, &mut peer_pid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if peer_pid == 0 || peer_pid != untrusted.claimed_process.pid {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Pipe client process differs from untrusted claimed PID",
        ));
    }
    // SAFETY: Acquire only query/synchronize rights to the actual
    // OS-reported client PID; no termination, token duplication or launch.
    let raw = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0, peer_pid,
        )
    };
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    let retained = ProcessHandle(raw);
    let mut creation: FILETIME = unsafe { mem::zeroed() };
    let mut exit: FILETIME = unsafe { mem::zeroed() };
    let mut kernel: FILETIME = unsafe { mem::zeroed() };
    let mut user: FILETIME = unsafe { mem::zeroed() };
    // SAFETY: Four correctly aligned FILETIME out parameters live for
    // the duration of this query against a retained process handle.
    if unsafe {
        GetProcessTimes(
            retained.0, &mut creation, &mut exit, &mut kernel, &mut user,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    let observed = ProcessIdentity::checked(
        peer_pid,
        ((creation.dwHighDateTime as u64) << 32)
            | (creation.dwLowDateTime as u64),
    ).ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, "No usable native pipe peer process identity",
    ))?;
    if observed != untrusted.claimed_process {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied, "Pipe client creation FILETIME differs",
        ));
    }
    // SAFETY: A zero-time wait observes whether THIS retained process has
    // exited; a PID being reused cannot change our HANDLE identity.
    match unsafe { WaitForSingleObject(retained.0, 0) } {
        WAIT_TIMEOUT => {}
        WAIT_OBJECT_0 => return Err(io::Error::new(
            io::ErrorKind::NotFound, "Native pipe peer process already exited",
        )),
        _ => return Err(io::Error::last_os_error()),
    }
    let mut still_peer_pid = 0u32;
    if unsafe { GetNamedPipeClientProcessId(server_pipe, &mut still_peer_pid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if still_peer_pid != peer_pid {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied, "Native pipe peer changed during inspection",
        ));
    }
    // This is NOT a trusted peer capability: the connection and its token
    // require a separate service-owned authenticated channel.
    Ok(PendingPipePeer { claimed_identity_matches_os_client: true })
}

#[cfg(test)]
mod tests {
    use super::*;
    use virtual_clients_core::independent_engine::{BrokerProbeKind, SessionSlot};

    #[test]
    fn missing_server_handle_refused_before_native_io() {
        let probe = UntrustedBrokerProbe {
            kind: BrokerProbeKind::ObserveExactProcess,
            slot: SessionSlot::Mc02,
            claimed_process: ProcessIdentity::checked(1, 1).unwrap(),
            correlation_id: 1,
        };
        assert_eq!(
            inspect_connected_server_pipe(ptr::null_mut(), probe)
                .unwrap_err().kind(),
            io::ErrorKind::InvalidInput,
        );
    }
}
