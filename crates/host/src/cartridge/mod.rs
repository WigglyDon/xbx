// SPDX-License-Identifier: MIT
//! One product policy for imported games, integrated runtime and durable return.
//! GCX ac9693c5 informed generation checks/publication ordering; see THIRD_PARTY.
#[cfg(not(target_os = "android"))]
mod android_build;
#[cfg(not(target_os = "android"))]
mod android_package;
#[cfg(not(target_os = "android"))]
pub mod build;
pub mod checkpoint;
#[cfg(not(target_os = "android"))]
mod debug_stop;
#[cfg(not(target_os = "android"))]
pub mod desktop;
#[cfg(not(target_os = "android"))]
mod fault_context;
#[cfg(not(target_os = "android"))]
mod file_read;
#[cfg(not(target_os = "android"))]
mod flush_io;
pub mod library;
#[cfg(not(target_os = "android"))]
mod lifetime;
pub(crate) mod monitor;
#[cfg(not(target_os = "android"))]
mod npx_followup;
#[cfg(not(target_os = "android"))]
mod npx_port;
pub mod player;
pub mod resources;
pub mod store;
#[cfg(all(test, not(target_os = "android")))]
mod tests;
#[cfg(not(target_os = "android"))]
mod x87_boundary;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::Path,
};

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn file_digest(path: &Path) -> io::Result<String> {
    reader_digest(File::open(path)?)
}
pub fn reader_digest(mut file: impl Read) -> io::Result<String> {
    let mut hash = Sha256::new();
    let mut bytes = [0; 65536];
    loop {
        let n = file.read(&mut bytes)?;
        if n == 0 {
            break;
        }
        hash.update(&bytes[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
pub fn valid_id(id: &str) -> bool {
    id.len() == 32
        && id
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
pub fn new_id() -> io::Result<String> {
    let mut bytes = [0; 16];
    File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(bytes.iter().map(|x| format!("{x:02x}")).collect())
}
pub fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(bytes)?;
    f.sync_all()
}
/// Pending files are preserved on any error. Never erase an interrupted publication.
pub fn atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let pending = path.with_extension("pending");
    write_new(&pending, bytes)?;
    fs::rename(&pending, path)?;
    File::open(
        path.parent()
            .ok_or_else(|| io::Error::other("missing parent"))?,
    )?
    .sync_all()
}
pub fn read_json<T: for<'a> Deserialize<'a>>(p: &Path) -> io::Result<T> {
    if fs::symlink_metadata(p)?.file_type().is_symlink() || fs::metadata(p)?.len() > 65536 {
        return Err(io::Error::other("invalid metadata; retained"));
    }
    serde_json::from_slice(&fs::read(p)?).map_err(io::Error::other)
}
pub fn json<T: Serialize>(v: &T) -> io::Result<Vec<u8>> {
    serde_json::to_vec_pretty(v).map_err(io::Error::other)
}
pub fn mkdir(p: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new().mode(0o700).create(p)
}

/// Used only by an explicit recovery decision under the existing writer lease.
/// Retain interrupted control bytes/directories; never promote a pending value
/// or erase recovery evidence, and keep normal publication guards unchanged.
pub(crate) fn retain_pending(root: &Path, names: &[&str]) -> io::Result<()> {
    for name in names {
        if name.contains('/') || name.contains('\\') || *name == ".." {
            return Err(io::Error::other("invalid recovery control name"));
        }
        let path = root.join(name);
        match fs::symlink_metadata(&path) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err(io::Error::other("recovery control symlink retained"));
                }
                fs::rename(path, root.join(format!("interrupted-{name}-{}", new_id()?)))?;
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    File::open(root)?.sync_all()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub game: String,
    pub profile: String,
    pub media: String,
    pub partition: u64,
    pub runtime: String,
    pub state_abi: u32,
}

/// Lifecycle requests coalesce into one desired state; interruption does not
/// replace explicit Pause intent. All native work runs on one serialized owner.
#[derive(Default, Debug)]
pub struct Intent {
    pub user_paused: bool,
    pub foreground: bool,
    pub surface: bool,
    pub audio_focus: bool,
}
impl Intent {
    pub fn may_run(&self) -> bool {
        !self.user_paused && self.foreground && self.surface && self.audio_focus
    }
}
