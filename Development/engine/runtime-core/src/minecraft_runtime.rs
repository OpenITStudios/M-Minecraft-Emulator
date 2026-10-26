use std::{io, time::Duration};

use crate::{
    client::ClientId,
    guest::{guest_agent_launch_compatible, GUEST_AGENT_PROTOCOL_VERSION},
    provider::Provider,
};

const GUEST_STATUS_TIMEOUT: Duration = Duration::from_secs(2);
const MINECRAFT_LAUNCH_REQUEST_TIMEOUT: Duration = Duration::from_secs(40);

pub fn ensure_running(provider: &dyn Provider, client: ClientId) -> io::Result<()> {
    let status = provider
        .guest_status(client, GUEST_STATUS_TIMEOUT)?
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                format!("{} Minecraft guest status is unavailable", client.as_str()),
            )
        })?;
    if !guest_agent_launch_compatible(status.protocol_version) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "{} Guest Agent protocol {} does not support Minecraft auto-launch; protocol {} is required",
                client.as_str(),
                status.protocol_version,
                GUEST_AGENT_PROTOCOL_VERSION
            ),
        ));
    }
    if status.interactive_launcher_ready != Some(true) {
        return Err(io::Error::new(
            io::ErrorKind::NotConnected,
            format!(
                "{} interactive launcher is not active; complete per-user launcher setup inside that Virtual",
                client.as_str()
            ),
        ));
    }
    provider.launch_minecraft(client, MINECRAFT_LAUNCH_REQUEST_TIMEOUT)
}
