//! Profile-wide OS lease and conservative disk-generation publication.
//! Full machine checkpoints remain disabled until the engine's complete boundary is proven.
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};

pub struct ProfileLease {
    _lock: File,
    root: PathBuf,
}

impl ProfileLease {
    /// Carry the same open-file-description lease into a diagnostic native child.
    /// If the supervising Rust process dies, the native writer still owns the lease.
    pub fn inherit_for(&self, command: &mut std::process::Command) {
        let descriptor = self._lock.as_raw_fd();
        unsafe extern "C" {
            fn fcntl(fd: i32, command: i32, ...) -> i32;
        }
        // pre_exec changes only the child's descriptor flags; the parent stays CLOEXEC.
        unsafe {
            command.pre_exec(move || {
                const F_SETFD: i32 = 2;
                if fcntl(descriptor, F_SETFD, 0) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    /// The caller selects a private, existing directory. Never remove/replace its lock file.
    pub fn acquire(root: &Path) -> io::Result<Self> {
        let root = root.canonicalize()?;
        if !root.is_dir() {
            return Err(io::Error::other("profile is not a directory"));
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.join("writer.lock"))?;
        lock.try_lock()
            .map_err(|_| io::Error::other("profile already has a writer"))?;
        Ok(Self { _lock: lock, root })
    }

    pub fn generation(&self) -> io::Result<u64> {
        match fs::read_to_string(self.root.join("generation")) {
            Ok(s) => s
                .trim()
                .parse()
                .map_err(|_| io::Error::other("invalid disk generation")),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(0),
            Err(e) => Err(e),
        }
    }

    /// Advance *before* giving the native engine writable storage. A crash stays divergent.
    pub fn begin_writes(&self) -> io::Result<u64> {
        let next = self
            .generation()?
            .checked_add(1)
            .ok_or_else(|| io::Error::other("disk generation overflow"))?;
        let temporary = self.root.join("generation.pending");
        // A leftover temporary is recovery evidence, not permission to truncate it.
        let mut out = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        writeln!(out, "{next}")?;
        out.sync_all()?;
        fs::rename(&temporary, self.root.join("generation"))?;
        File::open(&self.root)?.sync_all()?;
        Ok(next)
    }

    pub fn require_generation(&self, checkpoint_generation: u64) -> io::Result<()> {
        if self.generation()? != checkpoint_generation {
            return Err(io::Error::other(
                "checkpoint disk generation differs; preserve it and cold boot current storage explicitly",
            ));
        }
        Ok(())
    }

    /// Call only after native teardown has joined every disk/EEPROM writer.
    pub fn sync_storage(&self) -> io::Result<()> {
        for name in ["hdd.qcow2", "eeprom.bin"] {
            let path = self.root.join(name);
            if path.exists() {
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(path)?
                    .sync_all()?;
            }
        }
        File::open(&self.root)?.sync_all()
    }
}
