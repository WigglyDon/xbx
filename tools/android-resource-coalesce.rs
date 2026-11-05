// SPDX-License-Identifier: MIT
//! XBX-only immutable asset maintenance, never profile/checkpoint cleanup.
//! Execute as the app UID under normal custody with both engines absent.
#[allow(dead_code)]
#[path = "../crates/host/src/storage.rs"]
mod storage;
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    process::Command,
};
fn hexadecimal(value: &str, count: usize) -> bool {
    value.len() == count && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn real_directory(path: &Path) -> io::Result<()> {
    let m = fs::symlink_metadata(path)?;
    if !m.is_dir() || m.file_type().is_symlink() {
        return Err(io::Error::other("real XBX directory required"));
    }
    Ok(())
}
fn digest(path: &Path) -> io::Result<String> {
    let program = if cfg!(target_os = "android") {
        "/system/bin/sha256sum"
    } else {
        "/usr/bin/sha256sum"
    };
    let out = Command::new(program).arg(path).output()?;
    if !out.status.success() || out.stdout.len() > 4096 {
        return Err(io::Error::other("immutable resource digest unavailable"));
    }
    let text = String::from_utf8(out.stdout).map_err(io::Error::other)?;
    let value = text.split_whitespace().next().unwrap_or("");
    if !hexadecimal(value, 64) {
        return Err(io::Error::other("invalid immutable resource digest"));
    }
    Ok(value.to_ascii_lowercase())
}
struct Candidate {
    path: PathBuf,
    hash: String,
    metadata: fs::Metadata,
}
struct Plan {
    _leases: Vec<storage::ProfileLease>,
    duplicates: Vec<(Candidate, Candidate)>,
    bytes: u64,
}
fn plan(files: &Path, pins: &[(&str, &str)]) -> io::Result<Plan> {
    real_directory(files)?;
    if pins.iter().any(|(name, expected)| {
        !matches!(*name, "open.bin" | "empty.qcow2") || !hexadecimal(expected, 64)
    }) {
        return Err(io::Error::other(
            "only pinned immutable boot/seed assets allowed",
        ));
    }
    let owner = fs::metadata(files)?.uid();
    let mut parents = Vec::new();
    for entry in fs::read_dir(files)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name != "xbx-cartridges-v1"
            && !name
                .strip_prefix("xbx-window-fresh-")
                .is_some_and(|n| hexadecimal(n, 32))
        {
            continue;
        }
        real_directory(&entry.path())?;
        let resources = entry.path().join("resources");
        if !resources.try_exists()? {
            continue;
        }
        real_directory(&resources)?;
        let lock = fs::symlink_metadata(resources.join("writer.lock"))?;
        if !lock.is_file() || lock.file_type().is_symlink() {
            return Err(io::Error::other("existing real resource lease required"));
        }
        parents.push(resources);
    }
    parents.sort();
    let leases = parents
        .iter()
        .map(|p| storage::ProfileLease::acquire(p))
        .collect::<io::Result<Vec<_>>>()?;
    let mut anchors: BTreeMap<(String, u64, u32, u32, u32, u64), Candidate> = BTreeMap::new();
    let mut duplicates = Vec::new();
    let mut bytes: u64 = 0;
    for parent in parents {
        let mut versions = fs::read_dir(parent)?
            .map(|e| e.map(|e| e.path()))
            .collect::<io::Result<Vec<_>>>()?;
        versions.sort();
        for version in versions {
            if !version
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| hexadecimal(s, 64))
            {
                continue;
            }
            real_directory(&version)?;
            for &(name, expected) in pins {
                let path = version.join(name);
                let metadata = fs::symlink_metadata(&path)?;
                if !metadata.is_file()
                    || metadata.file_type().is_symlink()
                    || metadata.uid() != owner
                {
                    return Err(io::Error::other("regular immutable resource required"));
                }
                let hash = digest(&path)?;
                if hash != expected {
                    continue;
                }
                let key = (
                    hash.clone(),
                    metadata.len(),
                    metadata.mode(),
                    metadata.uid(),
                    metadata.gid(),
                    metadata.dev(),
                );
                if let Some(anchor) = anchors.get(&key) {
                    if metadata.ino() == anchor.metadata.ino() || metadata.nlink() != 1 {
                        continue;
                    }
                    bytes = bytes
                        .checked_add(
                            metadata
                                .blocks()
                                .checked_mul(512)
                                .ok_or_else(|| io::Error::other("allocation overflow"))?,
                        )
                        .ok_or_else(|| io::Error::other("allocation overflow"))?;
                    duplicates.push((
                        Candidate {
                            path: anchor.path.clone(),
                            hash: anchor.hash.clone(),
                            metadata: anchor.metadata.clone(),
                        },
                        Candidate {
                            path,
                            hash,
                            metadata,
                        },
                    ));
                } else {
                    anchors.insert(
                        key,
                        Candidate {
                            path,
                            hash,
                            metadata,
                        },
                    );
                }
            }
        }
    }
    Ok(Plan {
        _leases: leases,
        duplicates,
        bytes,
    })
}
fn coalesce(files: &Path, pins: &[(&str, &str)], nonce: &str) -> io::Result<(usize, u64)> {
    if !hexadecimal(nonce, 32) {
        return Err(io::Error::other("exact maintenance identity required"));
    }
    let plan = plan(files, pins)?;
    let journal_path = files.join(format!("xbx-resource-coalesce-{nonce}.log"));
    let mut journal = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(journal_path)?;
    File::open(files)?.sync_all()?;
    for (anchor, duplicate) in &plan.duplicates {
        let m = fs::symlink_metadata(&duplicate.path)?;
        let a = fs::symlink_metadata(&anchor.path)?;
        if !m.is_file()
            || m.file_type().is_symlink()
            || m.ino() != duplicate.metadata.ino()
            || m.nlink() != 1
            || m.len() != duplicate.metadata.len()
            || !a.is_file()
            || a.file_type().is_symlink()
            || a.ino() != anchor.metadata.ino()
            || digest(&anchor.path)? != anchor.hash
            || digest(&duplicate.path)? != duplicate.hash
        {
            return Err(io::Error::other(
                "immutable resource changed; preserved without replacement",
            ));
        }
        let parent = duplicate
            .path
            .parent()
            .ok_or_else(|| io::Error::other("resource parent missing"))?;
        let temp = parent.join(format!(
            "coalesce-{nonce}-{}.pending",
            duplicate.path.file_name().unwrap().to_string_lossy()
        ));
        writeln!(
            journal,
            "prepared {:?} {:?} {}",
            duplicate.path, anchor.path, duplicate.hash
        )?;
        journal.sync_all()?;
        fs::hard_link(&anchor.path, &temp)?;
        File::open(parent)?.sync_all()?;
        if digest(&temp)? != duplicate.hash {
            return Err(io::Error::other(
                "linked resource verification failed; original preserved",
            ));
        }
        fs::rename(&temp, &duplicate.path)?;
        File::open(&duplicate.path)?.sync_all()?;
        File::open(parent)?.sync_all()?;
        if digest(&duplicate.path)? != duplicate.hash
            || fs::metadata(&duplicate.path)?.ino() != a.ino()
        {
            return Err(io::Error::other(
                "resource link readback failed; retain journal",
            ));
        }
        writeln!(journal, "verified {:?} {}", duplicate.path, duplicate.hash)?;
        journal.sync_all()?;
    }
    Ok((plan.duplicates.len(), plan.bytes))
}
fn main() -> io::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 6 {
        return Err(io::Error::other(
            "mode files nonce open-sha256 seed-sha256 required",
        ));
    }
    let pins = [("open.bin", a[4].as_str()), ("empty.qcow2", a[5].as_str())];
    let files = Path::new(&a[2]);
    #[cfg(target_os = "android")]
    if files != Path::new("files")
        || std::env::current_dir()?
            .file_name()
            .and_then(|s| s.to_str())
            != Some("com.machinerhythm.xbx")
    {
        return Err(io::Error::other(
            "run only as the assigned XBX application UID",
        ));
    }
    let (count, bytes) = match a[1].as_str() {
        "inspect" => {
            let p = plan(files, &pins)?;
            (p.duplicates.len(), p.bytes)
        }
        "coalesce" => coalesce(files, &pins, &a[3])?,
        _ => return Err(io::Error::other("unsupported immutable maintenance mode")),
    };
    println!(
        "{{\"identical_resource_copies\":{count},\"separate_inode_allocation_bytes\":{bytes},\"profile_or_checkpoint_files_touched\":false}}"
    );
    Ok(())
}

#[cfg(test)]
mod coalesce_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    struct Fixture {
        root: PathBuf,
        a: PathBuf,
        b: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "xbx-coalesce-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let a = root
                .join("xbx-cartridges-v1/resources")
                .join("a".repeat(64));
            let b = root
                .join(format!("xbx-window-fresh-{}/resources", "1".repeat(32)))
                .join("b".repeat(64));
            for path in [&a, &b] {
                fs::create_dir_all(path).unwrap();
                fs::write(path.parent().unwrap().join("writer.lock"), []).unwrap();
                fs::write(path.join("open.bin"), b"original immutable boot fixture").unwrap();
                fs::write(
                    path.join("empty.qcow2"),
                    b"original immutable empty seed fixture",
                )
                .unwrap();
            }
            Self { root, a, b }
        }
        fn pins(&self) -> (String, String) {
            (
                digest(&self.a.join("open.bin")).unwrap(),
                digest(&self.a.join("empty.qcow2")).unwrap(),
            )
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
    #[test]
    fn coalesce_preserves_every_asset_and_actual_product_copy_is_private() {
        let f = Fixture::new();
        let (open, seed) = f.pins();
        let mutable = f
            .root
            .join("xbx-cartridges-v1/profiles/synthetic/storage/hdd.qcow2");
        fs::create_dir_all(mutable.parent().unwrap()).unwrap();
        xbx::cartridge::store::copy_sparse(&f.a.join("empty.qcow2"), &mutable).unwrap();
        let original = fs::read(&mutable).unwrap();
        let inode = fs::metadata(&mutable).unwrap().ino();
        assert_eq!(
            coalesce(
                &f.root,
                &[("open.bin", &open), ("empty.qcow2", &seed)],
                &"2".repeat(32)
            )
            .unwrap()
            .0,
            2
        );
        for name in ["open.bin", "empty.qcow2"] {
            assert_eq!(
                fs::read(f.a.join(name)).unwrap(),
                fs::read(f.b.join(name)).unwrap()
            );
            assert_eq!(
                fs::metadata(f.a.join(name)).unwrap().ino(),
                fs::metadata(f.b.join(name)).unwrap().ino()
            );
        }
        assert_eq!(fs::metadata(&mutable).unwrap().ino(), inode);
        let after = mutable.with_file_name("after.qcow2");
        xbx::cartridge::store::copy_sparse(&f.a.join("empty.qcow2"), &after).unwrap();
        assert_ne!(
            fs::metadata(&after).unwrap().ino(),
            fs::metadata(f.a.join("empty.qcow2")).unwrap().ino()
        );
        fs::write(&after, b"later ordinary save").unwrap();
        fs::write(&mutable, b"new ordinary save").unwrap();
        assert_eq!(fs::read(f.a.join("empty.qcow2")).unwrap(), original);
        assert_eq!(fs::read(f.b.join("empty.qcow2")).unwrap(), original);
    }
    #[test]
    fn coalesce_busy_resource_lease_refuses_before_changes() {
        let f = Fixture::new();
        let (open, seed) = f.pins();
        let before = fs::metadata(f.b.join("open.bin")).unwrap().ino();
        let _held = storage::ProfileLease::acquire(f.a.parent().unwrap()).unwrap();
        assert!(
            coalesce(
                &f.root,
                &[("open.bin", &open), ("empty.qcow2", &seed)],
                &"3".repeat(32)
            )
            .is_err()
        );
        assert_eq!(fs::metadata(f.b.join("open.bin")).unwrap().ino(), before);
    }
    #[test]
    fn coalesce_never_accepts_profile_files_or_unpinned_content() {
        let f = Fixture::new();
        let (open, seed) = f.pins();
        fs::write(f.b.join("open.bin"), b"different retained build").unwrap();
        let before = fs::metadata(f.b.join("open.bin")).unwrap().ino();
        assert_eq!(
            coalesce(
                &f.root,
                &[("open.bin", &open), ("empty.qcow2", &seed)],
                &"4".repeat(32)
            )
            .unwrap()
            .0,
            1
        );
        assert_eq!(fs::metadata(f.b.join("open.bin")).unwrap().ino(), before);
        assert!(plan(&f.root, &[("hdd.qcow2", &seed)]).is_err());
    }
    #[test]
    fn coalesce_symlink_refuses_before_any_asset_replacement() {
        let f = Fixture::new();
        let (open, seed) = f.pins();
        let before = fs::metadata(f.a.join("open.bin")).unwrap().ino();
        fs::remove_file(f.b.join("empty.qcow2")).unwrap();
        std::os::unix::fs::symlink(f.a.join("empty.qcow2"), f.b.join("empty.qcow2")).unwrap();
        assert!(
            coalesce(
                &f.root,
                &[("open.bin", &open), ("empty.qcow2", &seed)],
                &"5".repeat(32)
            )
            .is_err()
        );
        assert_eq!(fs::metadata(f.a.join("open.bin")).unwrap().ino(), before);
    }
}
