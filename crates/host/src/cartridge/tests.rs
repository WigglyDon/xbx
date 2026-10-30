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
            window: false,
            presentation_vsync: true,
            presentation_vulkan: false,
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
#[cfg(target_os = "linux")]
fn logical_identity_survives_file_size_limit_without_full_raw_conversion() {
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    const LIMIT: u64 = 16 * 1024 * 1024;
    fn bounded(command: &mut Command) {
        unsafe extern "C" {
            fn setrlimit(resource: i32, limits: *const [u64; 2]) -> i32;
        }
        unsafe {
            command.pre_exec(|| {
                if setrlimit(1, &[LIMIT, LIMIT]) != 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    if let Some(dir) = std::env::var_os("XBX_TEST_SIGNATURE_DIR") {
        let f = Fixture::new();
        let dir = PathBuf::from(dir);
        let result = store::signature(&dir, &f.r).unwrap();
        assert_eq!(
            result.disk,
            std::env::var("XBX_TEST_SIGNATURE_EXPECTED").unwrap()
        );
        assert_eq!(result.eeprom, file_digest(&dir.join("eeprom.bin")).unwrap());
        assert!(!fs::read_dir(dir).unwrap().any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("logical-chunk-")
        }));
        return;
    }
    let f = Fixture::new();
    let dir = f.p.current().unwrap();
    let raw = dir.join("seed.raw");
    {
        use std::io::{Seek, SeekFrom};
        let mut file = OpenOptions::new().write(true).open(&raw).unwrap();
        file.set_len(1024 * 1024 * 1024 + 512).unwrap();
        file.seek(SeekFrom::Start(65530)).unwrap();
        file.write_all(b"crossing canonical block boundary")
            .unwrap();
        file.seek(SeekFrom::End(-512)).unwrap();
        file.write_all(b"synthetic progress after a large logical hole")
            .unwrap();
        file.sync_all().unwrap();
    }
    let expected = store::logical_digest(&raw).unwrap();
    store::checked(
        Command::new("/usr/bin/qemu-img")
            .args(["convert", "-f", "raw", "-O", "qcow2", "-S", "4096"])
            .arg(&raw)
            .arg(dir.join("hdd.qcow2")),
    )
    .unwrap();
    let disk_before = file_digest(&dir.join("hdd.qcow2")).unwrap();
    let partial = dir.join("old-full-conversion.raw");
    let mut old = Command::new("/usr/bin/qemu-img");
    old.args(["convert", "-U", "-f", "qcow2", "-O", "raw", "-S", "4096"])
        .arg(dir.join("hdd.qcow2"))
        .arg(&partial);
    bounded(&mut old);
    let negative = old.output().unwrap();
    assert!(!negative.status.success());
    assert!(
        negative.status.signal() == Some(25)
            || String::from_utf8_lossy(&negative.stderr).contains("File too large")
    );
    let mut child = Command::new(std::env::current_exe().unwrap());
    child.args(["--exact", "cartridge::tests::logical_identity_survives_file_size_limit_without_full_raw_conversion", "--nocapture"])
        .env("XBX_TEST_SIGNATURE_DIR", &dir)
        .env("XBX_TEST_SIGNATURE_EXPECTED", expected);
    bounded(&mut child);
    let output = child.output().unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(file_digest(&dir.join("hdd.qcow2")).unwrap(), disk_before);
    assert!(
        partial.exists(),
        "retain the original failed conversion until synthetic fixture cleanup"
    );
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
fn explicit_restart_preserves_interrupted_controls_and_current_saves() {
    let f = Fixture::new();
    let id = f.publish();
    let session = fs::read(f.p.root.join("session.json")).unwrap();
    let disk = file_digest(&f.p.current().unwrap().join("hdd.qcow2")).unwrap();
    let state = file_digest(&f.p.root.join("checkpoints").join(id).join("hdd.qcow2")).unwrap();
    write_new(&f.p.root.join("generation.pending"), b"2\n").unwrap();
    mkdir(&f.p.root.join("session.pending")).unwrap();
    assert!(f.p.lease.begin_writes().is_err());
    checkpoint::restart(&f.p).unwrap();
    assert_eq!(
        file_digest(&f.p.current().unwrap().join("hdd.qcow2")).unwrap(),
        disk
    );
    assert_eq!(
        file_digest(
            &f.p.root
                .join("checkpoints")
                .join(
                    serde_json::from_slice::<checkpoint::Session>(&session)
                        .unwrap()
                        .checkpoint
                )
                .join("hdd.qcow2")
        )
        .unwrap(),
        state
    );
    let paths: Vec<_> = fs::read_dir(&f.p.root)
        .unwrap()
        .map(|v| v.unwrap().path())
        .collect();
    assert!(paths.iter().any(|p| {
        p.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("interrupted-generation.pending-")
            && fs::read(p).unwrap() == b"2\n"
    }));
    assert!(paths.iter().any(|p| {
        p.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("interrupted-session.pending-")
            && p.is_dir()
    }));
    assert!(paths.iter().any(|p| {
        p.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("rejected-session-")
            && fs::read(p).unwrap() == session
    }));
    assert!(f.p.root.join("restart.intent.json").is_file());
    assert_eq!(f.p.lease.begin_writes().unwrap(), 2);
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

#[test]
fn matched_seed_initialization_preserves_newer_storage_and_failed_partial_copy() {
    let mut f = Fixture::new();
    let source = f.p.current().unwrap();
    let seed_disk = source.join("hdd.qcow2");
    let seed_eeprom = source.join("eeprom.bin");
    let original_seeds = (
        file_digest(&seed_disk).unwrap(),
        file_digest(&seed_eeprom).unwrap(),
    );
    // The seeded developer route must need no blank-disk generator at all.
    f.r.root = f.root.join("no-disk-generator");
    let app = f.root.join("seeded-app");
    mkdir(&app).unwrap();
    let mut entry = library::Entry {
        id: "1".repeat(32),
        profile: "2".repeat(32),
        name: "original seeded storage".into(),
        media: "unused".into(),
        media_sha256: "3".repeat(64),
        partition: 0,
    };
    let p = store::Profile::open_seeded(&app, &entry, &f.r, &seed_disk, &seed_eeprom).unwrap();
    let current = p.current().unwrap();
    assert!(!current.join("initial.raw").exists());
    fs::write(current.join("hdd.qcow2"), b"newer synthetic ordinary save").unwrap();
    fs::write(current.join("eeprom.bin"), b"current synthetic EEPROM").unwrap();
    store::sync_files(&current).unwrap();
    let pointer = fs::read(p.root.join("storage.json")).unwrap();
    let profile_root = p.root.clone();
    drop(p);
    let p = store::Profile::open_seeded(&app, &entry, &f.r, &seed_disk, &seed_eeprom).unwrap();
    assert_eq!(fs::read(p.root.join("storage.json")).unwrap(), pointer);
    assert_eq!(
        fs::read(p.current().unwrap().join("hdd.qcow2")).unwrap(),
        b"newer synthetic ordinary save"
    );
    assert_eq!(
        fs::read(p.current().unwrap().join("eeprom.bin")).unwrap(),
        b"current synthetic EEPROM"
    );
    drop(p);
    entry.profile = "4".repeat(32);
    assert!(
        store::Profile::open_seeded(
            &app,
            &entry,
            &f.r,
            &seed_disk,
            &f.root.join("missing-eeprom")
        )
        .is_err()
    );
    let failed = app.join("profiles").join(&entry.profile);
    assert!(failed.join("storage").is_dir());
    assert!(!failed.join("storage.json").exists());
    let partial: Vec<_> = fs::read_dir(failed.join("storage"))
        .unwrap()
        .map(|p| p.unwrap().path())
        .collect();
    assert_eq!(partial.len(), 1);
    assert_eq!(
        file_digest(&partial[0].join("hdd.qcow2")).unwrap(),
        original_seeds.0
    );
    assert!(store::Profile::open_seeded(&app, &entry, &f.r, &seed_disk, &seed_eeprom).is_err());
    assert_eq!(
        file_digest(&partial[0].join("hdd.qcow2")).unwrap(),
        original_seeds.0
    );
    assert_eq!(
        fs::read(profile_root.join("storage.json")).unwrap(),
        pointer
    );
    assert_eq!(
        (
            file_digest(&seed_disk).unwrap(),
            file_digest(&seed_eeprom).unwrap()
        ),
        original_seeds
    );
}
