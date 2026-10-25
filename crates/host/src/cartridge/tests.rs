// SPDX-License-Identifier: MIT
//! Original storage-policy fault cases. Native continuity is tested separately
//! through the original guest and fresh xbx-player processes, never mocked here.
use super::*;
use std::{path::PathBuf, process::Command};
struct Fixture {
    root: PathBuf,
    p: store::Profile,
    r: resources::Resources,
    b: Binding,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("xbx-policy-{}", new_id().unwrap()));
        mkdir(&root).unwrap();
        let runtime = root.join("runtime");
        mkdir(&runtime).unwrap();
        mkdir(&runtime.join("native")).unwrap();
        std::os::unix::fs::symlink("/usr/bin/qemu-img", runtime.join("native/qemu-img")).unwrap();
        let profile = root.join("profile");
        mkdir(&profile).unwrap();
        mkdir(&profile.join("storage")).unwrap();
        mkdir(&profile.join("checkpoints")).unwrap();
        let id = new_id().unwrap();
        let dir = profile.join("storage").join(&id);
        mkdir(&dir).unwrap();
        let mut raw = vec![0; 131072];
        raw[8192..8200].copy_from_slice(b"old save");
        write_new(&dir.join("seed.raw"), &raw).unwrap();
        store::checked(
            Command::new("/usr/bin/qemu-img")
                .args(["convert", "-f", "raw", "-O", "qcow2"])
                .arg(dir.join("seed.raw"))
                .arg(dir.join("hdd.qcow2")),
        )
        .unwrap();
        write_new(&dir.join("eeprom.bin"), &[0x47; 256]).unwrap();
        atomic(
            &profile.join("storage.json"),
            &json(&store::Current {
                directory: id,
                previous: None,
                managed: true,
            })
            .unwrap(),
        )
        .unwrap();
        let p = store::Profile {
            lease: crate::storage::ProfileLease::acquire(&profile).unwrap(),
            root: profile,
        };
        p.lease.begin_writes().unwrap();
        let r = resources::Resources {
            root: runtime,
            identity: "c".repeat(64),
            manifest: resources::Manifest {
                schema: 1,
                core: resources::CORE.into(),
                open_system: resources::OPEN.into(),
                disk_tool: String::new(),
                slirp: String::new(),
                state_abi: 1,
            },
        };
        let b = Binding {
            game: "a".repeat(32),
            profile: "b".repeat(32),
            media: "d".repeat(64),
            partition: 0,
            runtime: r.identity.clone(),
            state_abi: 1,
        };
        Self { root, p, r, b }
    }
    fn publish(&self) -> String {
        checkpoint::publish(&self.p, &self.r, &self.b, false, "xbx-session").unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn newer_write_epoch_refuses_stale_resume_before_any_disk_mutation() {
    let f = Fixture::new();
    let id = f.publish();
    let cp = f.p.root.join("checkpoints").join(&id).join("hdd.qcow2");
    let old = file_digest(&cp).unwrap();
    let current = f.p.current().unwrap();
    let disk = file_digest(&current.join("hdd.qcow2")).unwrap();
    let validated = checkpoint::load(&f.p, &f.r, &f.b).unwrap().unwrap();
    f.p.lease.begin_writes().unwrap();
    assert!(validated.recheck(&f.p, &f.r).is_err());
    assert!(checkpoint::load(&f.p, &f.r, &f.b).is_err());
    assert_eq!(file_digest(&cp).unwrap(), old);
    assert_eq!(file_digest(&current.join("hdd.qcow2")).unwrap(), disk);
}
#[test]
fn snapshot_metadata_alone_does_not_manufacture_newer_game_progress() {
    let f = Fixture::new();
    f.publish();
    let before = store::signature(&f.p.current().unwrap(), &f.r).unwrap();
    let epoch = f.p.lease.generation().unwrap();
    store::checked(
        Command::new("/usr/bin/qemu-img")
            .args(["snapshot", "-c", "metadata-only"])
            .arg(f.p.current().unwrap().join("hdd.qcow2")),
    )
    .unwrap();
    assert_eq!(
        store::signature(&f.p.current().unwrap(), &f.r).unwrap(),
        before
    );
    assert_eq!(f.p.lease.generation().unwrap(), epoch);
    assert!(checkpoint::load(&f.p, &f.r, &f.b).unwrap().is_some());
}
#[test]
fn interrupted_reference_publication_keeps_prior_checkpoint_and_pending_evidence() {
    let f = Fixture::new();
    let id = f.publish();
    let old = fs::read(f.p.root.join("session.json")).unwrap();
    mkdir(&f.p.root.join("session.pending")).unwrap();
    assert!(checkpoint::publish(&f.p, &f.r, &f.b, false, "xbx-session").is_err());
    assert_eq!(fs::read(f.p.root.join("session.json")).unwrap(), old);
    assert!(f.p.root.join("session.pending").is_dir());
    assert_eq!(checkpoint::load(&f.p, &f.r, &f.b).unwrap().unwrap().id, id);
}
#[test]
fn corruption_or_incompatible_runtime_never_rewrites_checkpoint_or_current_saves() {
    let f = Fixture::new();
    let id = f.publish();
    let mut b = f.b.clone();
    b.runtime = "e".repeat(64);
    assert!(checkpoint::load(&f.p, &f.r, &b).is_err());
    let path = f.p.root.join("checkpoints").join(id).join("hdd.qcow2");
    let mut bytes = fs::read(&path).unwrap();
    bytes[64] ^= 1;
    fs::write(&path, &bytes).unwrap();
    let disk = file_digest(&f.p.current().unwrap().join("hdd.qcow2")).unwrap();
    assert!(checkpoint::load(&f.p, &f.r, &f.b).is_err());
    assert_eq!(fs::read(path).unwrap(), bytes);
    assert_eq!(
        file_digest(&f.p.current().unwrap().join("hdd.qcow2")).unwrap(),
        disk
    );
}
#[test]
fn failed_load_copy_cannot_flush_over_authoritative_saves() {
    let f = Fixture::new();
    f.publish();
    let v = checkpoint::load(&f.p, &f.r, &f.b).unwrap().unwrap();
    let source = f.p.current().unwrap();
    let before = file_digest(&source.join("hdd.qcow2")).unwrap();
    let attempt = checkpoint::restore_copy(&v, &f.p).unwrap();
    fs::write(attempt.join("eeprom.bin"), b"partial native mutation").unwrap();
    fs::write(attempt.join("hdd.qcow2"), b"failed native machine").unwrap();
    store::sync_files(&attempt).unwrap();
    assert_eq!(f.p.current().unwrap(), source);
    assert_eq!(file_digest(&source.join("hdd.qcow2")).unwrap(), before);
    assert!(checkpoint::load(&f.p, &f.r, &f.b).unwrap().is_some());
}
#[test]
fn interruption_and_explicit_pause_have_distinct_return_intent() {
    let mut i = Intent {
        foreground: true,
        surface: true,
        audio_focus: true,
        user_paused: false,
    };
    assert!(i.may_run());
    i.foreground = false;
    assert!(!i.may_run());
    i.foreground = true;
    assert!(i.may_run());
    i.user_paused = true;
    i.foreground = false;
    i.foreground = true;
    assert!(!i.may_run());
    i.user_paused = false;
    i.surface = false;
    assert!(!i.may_run());
}

#[test]
fn only_known_managed_storage_retires_after_replacement_is_durable() {
    for managed in [false, true] {
        let f = Fixture::new();
        let mut current: store::Current = read_json(&f.p.root.join("storage.json")).unwrap();
        current.managed = managed;
        atomic(&f.p.root.join("storage.json"), &json(&current).unwrap()).unwrap();
        f.publish();
        let v = checkpoint::load(&f.p, &f.r, &f.b).unwrap().unwrap();
        let old = f.p.current().unwrap();
        let next = checkpoint::restore_copy(&v, &f.p).unwrap();
        f.p.select(&next).unwrap();
        assert!(old.exists());
        let id = f.publish();
        assert_eq!(old.exists(), !managed);
        assert_eq!(checkpoint::load(&f.p, &f.r, &f.b).unwrap().unwrap().id, id);
        assert_eq!(f.p.current().unwrap(), next);
    }
}
