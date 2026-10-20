use crate::{
    persistence::{read_text_recovering, write_text_transactional},
    policy::VIRTUAL_VCPUS,
    profile::BaseState,
    resources::VIRTUAL_MEMORY_LIMIT_MB,
};
use std::{io, path::Path};

pub(crate) const GUEST_TOKEN_KEY: &str = "guestinfo.virtualclients.token";
pub(crate) const BASE_STATE_KEY: &str = "guestinfo.virtualclients.baseState";

// VMX may temporarily exist only as an adjacent backup after a failed
// metadata replacement. Recover the canonical path before classifying it.
pub(crate) fn vmx_file_present(vmx: &Path) -> io::Result<bool> {
    match read_text_recovering(vmx) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

pub(crate) fn apply_virtual_hardware_policy(vmx: &Path) -> io::Result<()> {
    let source = read_text_recovering(vmx)?;
    let mut lines: Vec<String> = source.lines().map(ToOwned::to_owned).collect();
    set_vmx_value(&mut lines, "numvcpus", &VIRTUAL_VCPUS.to_string());
    set_vmx_value(&mut lines, "memsize", &VIRTUAL_MEMORY_LIMIT_MB.to_string());
    set_vmx_value(&mut lines, "mks.enable3d", "TRUE");
    set_vmx_value(&mut lines, "ethernet0.present", "TRUE");
    set_vmx_value(&mut lines, "ethernet0.startConnected", "TRUE");
    set_vmx_value(&mut lines, "answer.msg.uuid.altered", "I copied it");
    remove_vmx_value(&mut lines, BASE_STATE_KEY);
    // One VMX write: no intermediate clone configuration with inherited token.
    let token = generate_guest_token()?;
    set_vmx_value(&mut lines, GUEST_TOKEN_KEY, &token);
    let mut output = lines.join("\n");
    output.push('\n');
    write_text_transactional(vmx, &output)
}

pub(crate) fn read_vmx_value(vmx: &Path, key: &str) -> io::Result<Option<String>> {
    let source = read_text_recovering(vmx)?;
    let mut result = None;
    for line in source.lines() {
        let Some((name, value)) = vmx_assignment(line) else {
            continue;
        };
        if !name.eq_ignore_ascii_case(key) {
            continue;
        }
        if result.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("VMX contains multiple definitions for {key}"),
            ));
        }
        result = Some(value.trim_matches('"').to_owned());
    }
    Ok(result)
}

pub(crate) fn read_vmx_memory(vmx: &Path) -> io::Result<u64> {
    read_vmx_value(vmx, "memsize")?
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "memsize is missing from VMX"))?
        .parse::<u64>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid memsize in VMX"))
}

pub(crate) fn guest_token_for_path(vmx: &Path) -> io::Result<Option<String>> {
    read_vmx_value(vmx, GUEST_TOKEN_KEY)
}

pub(crate) fn base_state_for_path(vmx: &Path) -> io::Result<Option<BaseState>> {
    read_vmx_value(vmx, BASE_STATE_KEY)?
        .map(|value| BaseState::from_vmx_str(&value))
        .transpose()
}

pub(crate) fn set_base_state_for_path(vmx: &Path, state: BaseState) -> io::Result<()> {
    let source = read_text_recovering(vmx)?;
    let mut lines: Vec<String> = source.lines().map(ToOwned::to_owned).collect();
    set_vmx_value(&mut lines, BASE_STATE_KEY, state.as_vmx_str());
    let mut output = lines.join("\n");
    output.push('\n');
    write_text_transactional(vmx, &output)
}

pub(crate) fn ensure_guest_token_for_path(vmx: &Path) -> io::Result<String> {
    if let Some(token) = guest_token_for_path(vmx)? {
        if valid_guest_token(&token) {
            return Ok(token);
        }
    }
    rotate_guest_token_for_path(vmx)
}

pub(crate) fn rotate_guest_token_for_path(vmx: &Path) -> io::Result<String> {
    let token = generate_guest_token()?;
    let source = read_text_recovering(vmx)?;
    let mut lines: Vec<String> = source.lines().map(ToOwned::to_owned).collect();
    set_vmx_value(&mut lines, GUEST_TOKEN_KEY, &token);
    let mut output = lines.join("\n");
    output.push('\n');
    write_text_transactional(vmx, &output)?;
    Ok(token)
}

pub(crate) fn valid_guest_token(token: &str) -> bool {
    token.len() == 64 && token.chars().all(|character| character.is_ascii_hexdigit())
}

pub(crate) fn vm_identity_key(vmx: &Path) -> io::Result<Option<String>> {
    let uuid = read_vmx_value(vmx, "uuid.bios")?.filter(|value| !value.trim().is_empty());
    let mac =
        read_vmx_value(vmx, "ethernet0.generatedAddress")?.filter(|value| !value.trim().is_empty());
    Ok(match (uuid, mac) {
        (Some(uuid), Some(mac)) => Some(format!("{uuid}|{mac}")),
        _ => None,
    })
}

fn generate_guest_token() -> io::Result<String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|error| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("guest token entropy failed: {error}"),
        )
    })?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

// Exact left-side key; comments, unrelated similarly-prefixed fields
// and casing/spacing variants are never confused with one another.
fn vmx_assignment(line: &str) -> Option<(&str, &str)> {
    let (name, value) = line.split_once('=')?;
    let name = name.trim();
    if name.is_empty() || name.starts_with('#') {
        return None;
    }
    Some((name, value.trim()))
}

fn remove_vmx_value(lines: &mut Vec<String>, key: &str) {
    lines.retain(|line| {
        !vmx_assignment(line).is_some_and(|(name, _)| name.eq_ignore_ascii_case(key))
    });
}

fn set_vmx_value(lines: &mut Vec<String>, key: &str, value: &str) {
    remove_vmx_value(lines, key);
    lines.push(format!("{key} = \"{value}\""));
}
