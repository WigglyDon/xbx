// SPDX-License-Identifier: MIT
//! Private qualification utility, never a product storage-reset action.
//! Run as the XBX app UID under its admitted device lease, with both engine
//! processes absent. Keep the original directory/inode and every fresh output.
#[allow(dead_code)]
#[path = "../crates/host/src/storage.rs"]
mod storage;
use std::{
    fs, io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

struct Window {
    parent: PathBuf,
    current: PathBuf,
    original: PathBuf,
    fresh: PathBuf,
    journal: PathBuf,
}
impl Window {
    fn new(parent: &Path, nonce: &str) -> io::Result<Self> {
        if nonce.len() != 32 || !nonce.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(io::Error::other("exact private window identity required"));
        }
        let parent = parent.canonicalize()?;
        Ok(Self {
            current: parent.join("xbx-cartridges-v1"),
            original: parent.join(format!("xbx-window-original-{nonce}")),
            fresh: parent.join(format!("xbx-window-fresh-{nonce}")),
            journal: parent.join(format!("xbx-window-{nonce}.receipt")),
            parent,
        })
    }
    fn inode(path: &Path) -> io::Result<u64> {
        let meta = fs::symlink_metadata(path)?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Err(io::Error::other("real private directory required"));
        }
        Ok(meta.ino())
    }
    fn sync(&self) -> io::Result<()> {
        fs::File::open(&self.parent)?.sync_all()
    }
    fn begin(&self) -> io::Result<()> {
        let inode = Self::inode(&self.current)?;
        let _lease = storage::ProfileLease::acquire(&self.current)?;
        if self.original.try_exists()? || self.fresh.try_exists()? || self.journal.try_exists()? {
            return Err(io::Error::other(
                "existing window evidence retained; no replay",
            ));
        }
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut out = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&self.journal)?;
        writeln!(out, "{inode}")?;
        out.sync_all()?;
        self.sync()?;
        // The existing app lock follows the original directory across rename.
        fs::rename(&self.current, &self.original)?;
        self.sync()?;
        if Self::inode(&self.original)? != inode {
            return Err(io::Error::other("original identity changed"));
        }
        // The ordinary product creates its own fresh collection/storage here.
        Ok(())
    }
    fn restore(&self) -> io::Result<()> {
        let expected: u64 = fs::read_to_string(&self.journal)?
            .trim()
            .parse()
            .map_err(|_| io::Error::other("invalid retained window receipt"))?;
        if self.current.try_exists()? && Self::inode(&self.current)? == expected {
            if self.original.try_exists()? {
                return Err(io::Error::other("ambiguous original retained"));
            }
            let _lease = storage::ProfileLease::acquire(&self.current)?;
            return self.sync();
        }
        if Self::inode(&self.original)? != expected {
            return Err(io::Error::other(
                "preserved original identity differs; no replacement",
            ));
        }
        let _original = storage::ProfileLease::acquire(&self.original)?;
        if self.current.try_exists()? {
            Self::inode(&self.current)?;
            let _fresh = storage::ProfileLease::acquire(&self.current)?;
            if self.fresh.try_exists()? {
                return Err(io::Error::other(
                    "fresh destination already exists; preserve both",
                ));
            }
            fs::rename(&self.current, &self.fresh)?;
            self.sync()?;
        }
        // Also handles interruption after preserving fresh output but before
        // returning the original. No directory or receipt is removed/replaced.
        fs::rename(&self.original, &self.current)?;
        self.sync()?;
        if Self::inode(&self.current)? != expected {
            return Err(io::Error::other("original return identity differs"));
        }
        Ok(())
    }
}
fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err(io::Error::other(
            "expected begin|restore PRIVATE_FILES WINDOW_ID",
        ));
    }
    let window = Window::new(Path::new(&args[2]), &args[3])?;
    match args[1].as_str() {
        "probe" => {}
        "begin" => window.begin()?,
        "restore" => window.restore()?,
        _ => return Err(io::Error::other("unknown window operation")),
    }
    println!("window {} verified; all data retained", args[1]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Window {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("xbx-window-{}-{id}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let window = Window::new(&root, "00112233445566778899aabbccddeeff").unwrap();
        fs::create_dir(&window.current).unwrap();
        fs::write(
            window.current.join("saved-session"),
            b"ordinary save and accepted checkpoint",
        )
        .unwrap();
        window
    }
    #[test]
    fn fresh_output_and_original_save_both_survive_return() {
        let w = fixture();
        let old = Window::inode(&w.current).unwrap();
        w.begin().unwrap();
        fs::create_dir(&w.current).unwrap();
        fs::write(w.current.join("fresh-result"), b"independent observation").unwrap();
        w.restore().unwrap();
        w.restore().unwrap();
        assert_eq!(Window::inode(&w.current).unwrap(), old);
        assert_eq!(
            fs::read(w.current.join("saved-session")).unwrap(),
            b"ordinary save and accepted checkpoint"
        );
        assert_eq!(
            fs::read(w.fresh.join("fresh-result")).unwrap(),
            b"independent observation"
        );
        assert!(w.journal.exists());
        fs::remove_dir_all(w.parent).unwrap();
    }
    #[test]
    fn interrupted_return_preserves_both_generations() {
        let w = fixture();
        w.begin().unwrap();
        fs::create_dir(&w.current).unwrap();
        fs::write(w.current.join("fresh-result"), b"retained").unwrap();
        fs::rename(&w.current, &w.fresh).unwrap();
        w.restore().unwrap();
        assert!(w.current.join("saved-session").is_file());
        assert!(w.fresh.join("fresh-result").is_file());
        fs::remove_dir_all(w.parent).unwrap();
    }
    #[test]
    fn busy_original_and_colliding_destination_refuse_without_mutation() {
        let w = fixture();
        let lease = storage::ProfileLease::acquire(&w.current).unwrap();
        assert!(w.begin().is_err());
        assert!(!w.journal.exists());
        drop(lease);
        fs::create_dir(&w.original).unwrap();
        assert!(w.begin().is_err());
        assert!(w.current.join("saved-session").is_file());
        assert!(!w.journal.exists());
        fs::remove_dir_all(w.parent).unwrap();
    }
    #[test]
    fn interrupted_begin_without_fresh_launch_restores_original() {
        let w = fixture();
        w.begin().unwrap();
        w.restore().unwrap();
        assert!(w.current.join("saved-session").is_file());
        assert!(!w.fresh.exists());
        assert!(w.begin().is_err());
        fs::remove_dir_all(w.parent).unwrap();
    }
}
