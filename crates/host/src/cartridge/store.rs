// SPDX-License-Identifier: MIT
//! Xbox HDD/EEPROM generations. Qcow2 metadata is not a guest-save generation.
use super::*;
use crate::storage::ProfileLease;
use std::{
    io::{Seek, SeekFrom},
    path::{Path, PathBuf},
    process::Command,
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Current {
    pub directory: String,
}
pub struct Profile {
    pub root: PathBuf,
    pub lease: ProfileLease,
}
impl Profile {
    pub fn open(app: &Path, entry: &library::Entry, r: &resources::Resources) -> io::Result<Self> {
        if !valid_id(&entry.profile) || !valid_id(&entry.id) {
            return Err(io::Error::other("invalid collection identity"));
        }
        let profiles = app.join("profiles");
        if !profiles.exists() {
            mkdir(&profiles)?
        }
        let root = profiles.join(&entry.profile);
        if !root.exists() {
            mkdir(&root)?
        }
        if fs::symlink_metadata(&root)?.file_type().is_symlink() {
            return Err(io::Error::other("profile symlink refused"));
        }
        let lease = ProfileLease::acquire(&root)?;
        let p = Self { root, lease };
        if !p.root.join("storage.json").exists() {
            if p.root.join("storage").exists() {
                return Err(io::Error::other("incomplete profile retained"));
            }
            mkdir(&p.root.join("storage"))?;
            mkdir(&p.root.join("checkpoints"))?;
            let id = new_id()?;
            let dir = p.root.join("storage").join(&id);
            mkdir(&dir)?;
            crate::disk::create_empty_raw(&dir.join("initial.raw"))?;
            checked(
                Command::new(r.image_tool())
                    .args(["convert", "-f", "raw", "-O", "qcow2", "-S", "4096"])
                    .arg(dir.join("initial.raw"))
                    .arg(dir.join("hdd.qcow2")),
            )?;
            sync_files(&dir)?;
            atomic(
                &p.root.join("storage.json"),
                &json(&Current { directory: id })?,
            )?;
        }
        Ok(p)
    }
    pub fn current(&self) -> io::Result<PathBuf> {
        let c: Current = read_json(&self.root.join("storage.json"))?;
        if !valid_id(&c.directory) {
            return Err(io::Error::other("invalid storage identity"));
        }
        let p = self.root.join("storage").join(c.directory);
        if fs::symlink_metadata(&p)?.file_type().is_symlink() {
            return Err(io::Error::other("storage symlink"));
        }
        Ok(p)
    }
    pub fn select(&self, dir: &Path) -> io::Result<()> {
        let id = dir
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| io::Error::other("storage name"))?;
        if !valid_id(id) || dir.parent() != Some(self.root.join("storage").as_path()) {
            return Err(io::Error::other("storage path escaped"));
        }
        sync_files(dir)?;
        atomic(
            &self.root.join("storage.json"),
            &json(&Current {
                directory: id.into(),
            })?,
        )
    }
    pub fn attempt(&self) -> io::Result<PathBuf> {
        let p = self.root.join("storage").join(new_id()?);
        mkdir(&p)?;
        Ok(p)
    }
}
pub fn checked(c: &mut Command) -> io::Result<()> {
    let out = c.output()?;
    if !out.status.success() {
        return Err(io::Error::other(format!(
            "disk tool failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(())
}
pub fn sync_files(dir: &Path) -> io::Result<()> {
    for name in ["hdd.qcow2", "eeprom.bin"] {
        let p = dir.join(name);
        if p.exists() {
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(p)?
                .sync_all()?
        }
    }
    File::open(dir)?.sync_all()
}
/// Exact ordinary copy, with zero blocks represented by holes when available.
/// This works without host reflinks and preserves qcow2 internal VM snapshots.
pub fn copy_sparse(from: &Path, to: &Path) -> io::Result<()> {
    let mut src = File::open(from)?;
    let mut dst = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(to)?;
    let len = src.metadata()?.len();
    dst.set_len(len)?;
    let mut buf = [0; 65536];
    let mut offset = 0;
    while offset < len {
        let n = src.read(&mut buf)?;
        if n == 0 {
            return Err(io::Error::other("source changed during copy"));
        }
        if buf[..n].iter().any(|b| *b != 0) {
            dst.seek(SeekFrom::Start(offset))?;
            dst.write_all(&buf[..n])?
        }
        offset += n as u64;
    }
    if src.metadata()?.len() != len {
        return Err(io::Error::other("source length changed"));
    }
    dst.sync_all()
}
pub fn copy_machine(from: &Path, to: &Path) -> io::Result<()> {
    for name in ["hdd.qcow2", "eeprom.bin"] {
        copy_sparse(&from.join(name), &to.join(name))?
    }
    File::open(to)?.sync_all()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveSignature {
    pub disk: String,
    pub eeprom: String,
}
/// Hash logical disk sectors, ignoring qcow2 snapshot metadata. Conversion is
/// read-only and only admitted while native execution/storage is quiesced.
pub fn signature(dir: &Path, r: &resources::Resources) -> io::Result<SaveSignature> {
    let raw = dir.join(format!("logical-{}.raw", new_id()?));
    checked(
        Command::new(r.image_tool())
            .args(["convert", "-U", "-f", "qcow2", "-O", "raw", "-S", "4096"])
            .arg(dir.join("hdd.qcow2"))
            .arg(&raw),
    )?;
    let hash = logical_digest(&raw)?;
    fs::remove_file(&raw)?;
    Ok(SaveSignature {
        disk: hash,
        eeprom: file_digest(&dir.join("eeprom.bin"))?,
    })
}
/// Canonical 64KiB nonzero logical blocks plus their offsets. Sparse allocation
/// layout does not affect identity. SEEK_DATA is an optimization, not a reliance
/// on reflinks or a requirement of the storage format.
pub fn logical_digest(raw: &Path) -> io::Result<String> {
    use std::os::fd::AsRawFd;
    unsafe extern "C" {
        fn lseek(fd: i32, offset: i64, whence: i32) -> i64;
    }
    let mut file = File::open(raw)?;
    let len = file.metadata()?.len();
    let mut hash = Sha256::new();
    hash.update(len.to_le_bytes());
    let mut offset = 0u64;
    let mut buffer = [0; 65536];
    let mut seek_data = true;
    while offset < len {
        if seek_data {
            let next = unsafe { lseek(file.as_raw_fd(), offset as i64, 3) };
            if next < 0 {
                let e = io::Error::last_os_error();
                if e.raw_os_error() == Some(6) {
                    break;
                }
                seek_data = false;
            } else {
                offset = (next as u64 / 65536) * 65536;
            }
        }
        if offset >= len {
            break;
        }
        file.seek(SeekFrom::Start(offset))?;
        let count = (len - offset).min(65536) as usize;
        file.read_exact(&mut buffer[..count])?;
        if buffer[..count].iter().any(|b| *b != 0) {
            hash.update(offset.to_le_bytes());
            hash.update((count as u64).to_le_bytes());
            hash.update(&buffer[..count]);
        }
        offset += count as u64;
    }
    Ok(format!("{:x}", hash.finalize()))
}
