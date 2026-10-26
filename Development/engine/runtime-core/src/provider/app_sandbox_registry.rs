//! App Sandbox v0.1.9 registry metadata: read-only and fail-closed.
//! The HTTP status field diskDirectory is a SHARED PARENT, not the VM folder.
//! vms.cfg stores the current VhdxPath and MacAddress. Neither value proves
//! HCS attachment or Minecraft account ownership.
use super::app_sandbox::normalized_disk_root;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

const MAX_CONFIG_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RegistryVm {
    pub name: String,
    pub active_vhdx: String,
    pub vm_root: String,
    pub shared_parent: String,
    pub mac: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Registry {
    entries: HashMap<String, RegistryVm>,
}

impl Registry {
    pub fn vm(&self, name: &str) -> Option<&RegistryVm> {
        self.entries.get(&name.to_ascii_lowercase())
    }
}

fn disk_path(name: &str, value: &str) -> Result<(String, String, String), &'static str> {
    let path = normalized_disk_root(value).ok_or("INVALID_VM_REGISTRY")?;
    let (dir, leaf) = path.rsplit_once('\\').ok_or("INVALID_VM_REGISTRY")?;
    let vm_dir = if leaf.eq_ignore_ascii_case("disk.vhdx") {
        dir
    } else if (leaf.starts_with("branch_") || leaf.starts_with("snapshot_"))
        && leaf.ends_with(".vhdx")
        && dir.ends_with("\\snapshots")
    {
        dir.rsplit_once('\\').ok_or("INVALID_VM_REGISTRY")?.0
    } else {
        return Err("INVALID_VM_REGISTRY");
    };
    let (parent, dirname) = vm_dir.rsplit_once('\\').ok_or("INVALID_VM_REGISTRY")?;
    if !dirname.eq_ignore_ascii_case(name) || parent.len() < 3 {
        return Err("INVALID_VM_REGISTRY");
    }
    let vm_dir = vm_dir.to_string();
    let parent = parent.to_string();
    Ok((path, vm_dir, parent))
}

fn normalize_mac(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    if bytes.len() != 17 {
        return None;
    }
    for (index, ch) in bytes.iter().copied().enumerate() {
        if index % 3 == 2 && ch != b'-' {
            return None;
        }
        if index % 3 != 2 && !ch.is_ascii_hexdigit() {
            return None;
        }
    }
    Some(input.to_ascii_lowercase())
}

fn finish(
    block: &HashMap<&str, &str>,
    entries: &mut HashMap<String, RegistryVm>,
    macs: &mut HashSet<String>,
    disks: &mut HashSet<String>,
    roots: &mut HashSet<String>,
) -> Result<(), &'static str> {
    let name = block.get("Name").ok_or("INVALID_VM_REGISTRY")?;
    let os = block.get("OsType").ok_or("INVALID_VM_REGISTRY")?;
    let max_name_len = match *os {
        "Windows" => 15,
        "Linux" => 63,
        _ => return Err("INVALID_VM_REGISTRY"),
    };
    if name.is_empty()
        || name.len() > max_name_len
        || name.starts_with('-')
        || name.ends_with('-')
        || name.bytes().all(|ch| ch.is_ascii_digit())
        || !name
            .bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == b'-')
        || (*os == "Linux" && name.bytes().any(|ch| ch.is_ascii_uppercase()))
    {
        return Err("INVALID_VM_REGISTRY");
    }
    let vhdx = block.get("VhdxPath").ok_or("INVALID_VM_REGISTRY")?;
    let (active_vhdx, vm_root, shared_parent) = disk_path(name, vhdx)?;
    let mac = match block.get("MacAddress") {
        Some(m) => Some(normalize_mac(m).ok_or("INVALID_VM_REGISTRY")?),
        None => None, // network-less guests need not have a MAC
    };
    if let Some(mac) = &mac {
        if !macs.insert(mac.clone()) {
            return Err("DUPLICATE_VM_MAC");
        }
    }
    if !disks.insert(active_vhdx.clone()) {
        return Err("DUPLICATE_VM_VHDX");
    }
    // Check VM-folder isolation independently of its active differencing disk.
    // Lexical paths are NOT immutable disk or HCS attachment attestation.
    if !roots.insert(vm_root.clone()) {
        return Err("DUPLICATE_VM_ROOT");
    }
    if entries
        .insert(
            name.to_ascii_lowercase(),
            RegistryVm {
                name: name.to_string(),
                active_vhdx,
                vm_root,
                shared_parent,
                mac,
            },
        )
        .is_some()
    {
        return Err("DUPLICATE_VM_NAME");
    }
    Ok(())
}

fn parse_config(source: &str) -> Result<Registry, &'static str> {
    if source.len() as u64 > MAX_CONFIG_BYTES || source.contains('\0') {
        return Err("INVALID_VM_REGISTRY");
    }
    let (mut entries, mut macs, mut disks, mut roots) = (
        HashMap::new(),
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
    );
    let mut block = HashMap::new();
    let mut in_vm = false;
    let mut count = 0;
    for row in source.lines() {
        let line = row.trim_end_matches('\r');
        if line.starts_with('[') && line.ends_with(']') {
            if in_vm {
                finish(&block, &mut entries, &mut macs, &mut disks, &mut roots)?;
            }
            block.clear();
            in_vm = line == "[VM]";
            if in_vm {
                count += 1;
                if count > 16 {
                    return Err("INVALID_VM_REGISTRY");
                }
            }
            continue;
        }
        if !in_vm || line.is_empty() {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or("INVALID_VM_REGISTRY")?;
        if ["Name", "OsType", "VhdxPath", "MacAddress"].contains(&key)
            && block.insert(key, value).is_some()
        {
            return Err("INVALID_VM_REGISTRY");
        }
    }
    if in_vm {
        finish(&block, &mut entries, &mut macs, &mut disks, &mut roots)?;
    }
    Ok(Registry { entries })
}

fn decode_config(bytes: &[u8]) -> Result<String, &'static str> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return String::from_utf8(bytes[3..].to_vec()).map_err(|_| "INVALID_VM_REGISTRY");
    }
    if bytes.starts_with(&[0xff, 0xfe]) {
        if (bytes.len() - 2) % 2 != 0 {
            return Err("INVALID_VM_REGISTRY");
        }
        let units: Vec<_> = bytes[2..]
            .chunks_exact(2)
            .map(|s| u16::from_le_bytes([s[0], s[1]]))
            .collect();
        return String::from_utf16(&units).map_err(|_| "INVALID_VM_REGISTRY");
    }
    String::from_utf8(bytes.to_vec()).map_err(|_| "INVALID_VM_REGISTRY")
}

#[cfg(target_os = "windows")]
fn config_file() -> Result<PathBuf, &'static str> {
    use std::os::windows::fs::MetadataExt;
    let root = PathBuf::from(std::env::var_os("ProgramData").ok_or("REGISTRY_UNAVAILABLE")?);
    if !root.is_absolute() {
        return Err("REGISTRY_UNAVAILABLE");
    }
    let directory = root.join("AppSandbox");
    let meta = fs::symlink_metadata(&directory).map_err(|_| "REGISTRY_UNAVAILABLE")?;
    if !meta.is_dir() || meta.file_attributes() & 0x400 != 0 {
        return Err("REGISTRY_UNAVAILABLE");
    }
    Ok(directory.join("vms.cfg"))
}
#[cfg(not(target_os = "windows"))]
fn config_file() -> Result<PathBuf, &'static str> {
    Err("WINDOWS_HOST_REQUIRED")
}

pub(super) fn read_registry() -> Result<Registry, &'static str> {
    read_at(&config_file()?)
}

fn read_at(file: &Path) -> Result<Registry, &'static str> {
    let meta = fs::symlink_metadata(file).map_err(|_| "REGISTRY_UNAVAILABLE")?;
    if !meta.file_type().is_file() || meta.len() > MAX_CONFIG_BYTES {
        return Err("REGISTRY_UNAVAILABLE");
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err("REGISTRY_UNAVAILABLE");
        }
    }
    let bytes = fs::read(file).map_err(|_| "REGISTRY_UNAVAILABLE")?;
    if bytes.len() as u64 > MAX_CONFIG_BYTES {
        return Err("REGISTRY_UNAVAILABLE");
    }
    parse_config(&decode_config(&bytes)?)
}

#[cfg(test)]
mod tests {
    use super::{decode_config, disk_path, parse_config};
    const FIRST: &str = "Name=Virtual-01\nOsType=Windows\nVhdxPath=C:\\VMs\\Virtual-01\\disk.vhdx\nMacAddress=02-AA-BB-CC-DD-01\n";
    const SECOND: &str = "Name=Virtual-02\nOsType=Windows\nVhdxPath=C:\\VMs\\Virtual-02\\snapshots\\branch_abc.vhdx\nMacAddress=02-AA-BB-CC-DD-02\n";
    #[test]
    fn common_parent_is_normal_but_vm_disks_and_macs_must_differ() {
        let cfg = format!("[Settings]\nSuppressTrayWarn=1\n[VM]\n{FIRST}\n[VM]\n{SECOND}\n");
        let registry = parse_config(&cfg).unwrap();
        let first = registry.vm("Virtual-01").unwrap();
        let second = registry.vm("Virtual-02").unwrap();
        assert_eq!(first.shared_parent, second.shared_parent);
        assert_ne!(first.vm_root, second.vm_root);
        assert_ne!(first.active_vhdx, second.active_vhdx);
        assert_ne!(first.mac, second.mac);
        assert_eq!(first.shared_parent, r"c:\vms");
    }
    #[test]
    fn reject_collisions_or_invalid_registry_records() {
        let duplicated_mac = SECOND.replace("02-AA-BB-CC-DD-02", "02-AA-BB-CC-DD-01");
        assert_eq!(
            parse_config(&format!("[VM]\n{FIRST}\n[VM]\n{duplicated_mac}")).unwrap_err(),
            "DUPLICATE_VM_MAC"
        );
        assert!(parse_config(&format!(
            "[VM]\n{FIRST}\n[VM]\n{}",
            FIRST.replace("Virtual-01", "virtual-01")
        ))
        .is_err());
        assert!(parse_config(&format!(
            "[VM]\n{}",
            FIRST.replace("disk.vhdx", r"..\disk.vhdx")
        ))
        .is_err());
        assert!(parse_config(&format!("[VM]\n{FIRST}MacAddress=02-AA-BB-CC-DD-09\n")).is_err());
        // Even different differencing VHDX leaves must not alias a VM folder.
        let branch = FIRST
            .replace("disk.vhdx", r"snapshots\branch_abc.vhdx")
            .replace("02-AA-BB-CC-DD-01", "02-AA-BB-CC-DD-02");
        assert!(parse_config(&format!("[VM]\n{FIRST}\n[VM]\n{branch}")).is_err());
    }
    #[test]
    fn unrelated_linux_hostnames_do_not_invalidate_valid_windows_vm_records() {
        let linux_name = "my-long-linux-virtual-machine-created-by-another-user";
        let linux =
            format!("Name={linux_name}\nOsType=Linux\nVhdxPath=C:\\VMs\\{linux_name}\\disk.vhdx\n");
        let report = parse_config(&format!("[VM]\n{FIRST}\n[VM]\n{linux}")).unwrap();
        assert_eq!(report.entries.len(), 2);
        assert!(report.vm("Virtual-01").is_some());
        assert!(parse_config(&format!(
            "[VM]\n{}",
            linux.replace("OsType=Linux", "OsType=Windows")
        ))
        .is_err());
    }
    #[test]
    fn accept_bom_encodings_and_differencing_disks_without_writing() {
        let content = format!("[VM]\n{FIRST}");
        let mut utf8 = vec![0xef, 0xbb, 0xbf];
        utf8.extend_from_slice(content.as_bytes());
        assert_eq!(
            parse_config(&decode_config(&utf8).unwrap())
                .unwrap()
                .entries
                .len(),
            1
        );
        let mut utf16 = vec![0xff, 0xfe];
        for unit in content.encode_utf16() {
            utf16.extend_from_slice(&unit.to_le_bytes());
        }
        assert_eq!(
            parse_config(&decode_config(&utf16).unwrap())
                .unwrap()
                .entries
                .len(),
            1
        );
        assert_eq!(
            disk_path("Virtual-01", r"C:\VMs\Virtual-01\disk.vhdx")
                .unwrap()
                .2,
            r"c:\vms"
        );
    }
}
