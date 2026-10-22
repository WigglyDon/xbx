//! Explicit diagnostic branches. Never substitute firmware beneath a used profile.
use crate::storage::ProfileLease;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    path::Path,
};

pub fn field<'a>(text: &'a str, name: &str) -> io::Result<&'a str> {
    let prefix = format!("{name}=");
    let mut values = text.lines().filter_map(|l| l.strip_prefix(&prefix));
    let value = values
        .next()
        .ok_or_else(|| io::Error::other("missing profile binding"))?;
    if values.next().is_some() {
        return Err(io::Error::other("duplicate profile binding"));
    }
    Ok(value)
}

/// Each instrumented firmware campaign gets its own fixed root. Unknown or
/// duplicate modes cannot alias a previously used baseline/diagnostic profile.
pub fn diagnostic_root(identity: &str) -> io::Result<Option<&'static str>> {
    if !identity.lines().any(|line| line.starts_with("mode=")) {
        return Ok(None);
    }
    match field(identity, "mode")? {
        "launch-diagnostic-v1" => Ok(Some("xbx-launch-diagnostic-v1")),
        "launch-diagnostic-v2" => Ok(Some("xbx-launch-diagnostic-v2")),
        "launch-diagnostic-v3" => Ok(Some("xbx-launch-diagnostic-v3")),
        _ => Err(io::Error::other("unknown diagnostic profile mode")),
    }
}
fn put(root: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join(name))?;
    f.write_all(bytes)?;
    f.sync_all()
}
fn regular(root: &Path, name: &str) -> io::Result<File> {
    let path = root.join(name);
    let meta = fs::symlink_metadata(&path)?;
    if !meta.is_file() || meta.len() > 512 * 1024 * 1024 {
        return Err(io::Error::other("unexpected diagnostic source file"));
    }
    File::open(path)
}
fn hex_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit())
}

/// The supplied digest adapter is GLib on Android; no cryptographic policy lives in a shim.
/// On any failure the parent remains untouched and an incomplete branch is preserved.
pub fn create(
    parent: &Path,
    target: &Path,
    firmware: &[u8],
    identity: &[u8],
    profile_id: &str,
    mut digest: impl FnMut(&mut File) -> io::Result<String>,
) -> io::Result<ProfileLease> {
    if !hex_id(profile_id) {
        return Err(io::Error::other("invalid diagnostic profile id"));
    }
    let source = ProfileLease::acquire(parent)?;
    let session = fs::read_to_string(parent.join("saved-session"))?;
    let old_id = field(&session, "profile_id")?;
    if !hex_id(old_id) || !hex_id(field(&session, "game_id")?) || old_id == profile_id {
        return Err(io::Error::other("invalid or reused profile identity"));
    }
    let old_identity = fs::read_to_string(parent.join("profile.identity"))?;
    let new_identity = std::str::from_utf8(identity).map_err(io::Error::other)?;
    let firmware_sha = field(new_identity, "sha256")?;
    let old_firmware = field(&old_identity, "sha256")?;
    if field(&session, "firmware_sha256")? != old_firmware
        || field(new_identity, "parent_firmware_sha256")? != old_firmware
        || field(new_identity, "ram_mib")? != "64"
        || field(&session, "ram_mib")? != "64"
    {
        return Err(io::Error::other(
            "diagnostic ancestry or memory profile mismatch",
        ));
    }
    // This experiment supports the existing self-contained qcow2 profile only.
    // Copying a container with an external backing/data file would not isolate it.
    let mut header = [0u8; 104];
    regular(parent, "hdd.qcow2")?.read_exact(&mut header)?;
    let version = u32::from_be_bytes(header[4..8].try_into().unwrap());
    if &header[..4] != b"QFI\xfb"
        || ![2, 3].contains(&version)
        || header[8..20].iter().any(|b| *b != 0)
        || (version == 3 && u64::from_be_bytes(header[72..80].try_into().unwrap()) & 4 != 0)
    {
        return Err(io::Error::other(
            "diagnostic branching requires a self-contained qcow2 disk",
        ));
    }
    let mut inputs = Vec::new();
    for name in [
        "hdd.qcow2",
        "eeprom.bin",
        "flash.bin",
        "profile.identity",
        "saved-session",
        "generation",
        "media.uri",
    ] {
        let mut f = regular(parent, name)?;
        let hash = digest(&mut f)?;
        inputs.push((name, hash));
    }
    if inputs.iter().find(|(n, _)| *n == "flash.bin").unwrap().1 != old_firmware {
        return Err(io::Error::other(
            "parent firmware bytes disagree with identity",
        ));
    }
    let generation = source.generation()?;
    source.sync_storage()?;
    fs::DirBuilder::new().mode(0o700).create(target)?;
    let lease = ProfileLease::acquire(target)?;
    for name in ["hdd.qcow2", "eeprom.bin", "generation", "media.uri"] {
        let mut input = regular(parent, name)?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(target.join(name))?;
        io::copy(&mut input, &mut output)?;
        output.sync_all()?;
        if input.metadata()?.ino() == output.metadata()?.ino()
            && input.metadata()?.dev() == output.metadata()?.dev()
        {
            return Err(io::Error::other("branch shares an inode"));
        }
        let expected = &inputs.iter().find(|(n, _)| *n == name).unwrap().1;
        if &digest(&mut regular(target, name)?)? != expected {
            return Err(io::Error::other("diagnostic copy verification failed"));
        }
    }
    put(target, "flash.bin", firmware)?;
    if digest(&mut regular(target, "flash.bin")?)? != firmware_sha {
        return Err(io::Error::other("diagnostic firmware digest mismatch"));
    }
    put(target, "profile.identity", identity)?;
    let updated = session
        .lines()
        .map(|l| {
            if l.starts_with("profile_id=") {
                format!("profile_id={profile_id}")
            } else if l.starts_with("firmware_sha256=") {
                format!("firmware_sha256={firmware_sha}")
            } else {
                l.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    put(target, "saved-session", updated.as_bytes())?;
    let mut ancestry = format!(
        "xbx-diagnostic-branch-v1\nparent_profile_id={old_id}\nprofile_id={profile_id}\nparent_generation={}\n",
        generation
    );
    for (name, hash) in &inputs {
        if &digest(&mut regular(parent, name)?)? != hash {
            return Err(io::Error::other(
                "parent changed during diagnostic branching",
            ));
        }
        ancestry.push_str(&format!("parent_{name}_sha256={hash}\n"));
    }
    put(target, "branch.identity", ancestry.as_bytes())?;
    File::open(target)?.sync_all()?;
    if let Some(parent) = target.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(lease)
}
