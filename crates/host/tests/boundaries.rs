use std::fs::{self, File};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use xbx::storage::ProfileLease;

fn scratch(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/scratch")
        .join(format!("{label}-{}-{unique}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

fn fixture(path: &Path, partition: u64) {
    let mut file = File::create(path).unwrap();
    file.set_len(partition + 35 * 2048).unwrap();
    let mut header = [0u8; 2048];
    header[..20].copy_from_slice(b"MICROSOFT*XBOX*MEDIA");
    header[2028..].copy_from_slice(b"MICROSOFT*XBOX*MEDIA");
    header[20..24].copy_from_slice(&33u32.to_le_bytes());
    header[24..28].copy_from_slice(&32u32.to_le_bytes());
    file.seek(SeekFrom::Start(partition + 32 * 2048)).unwrap();
    file.write_all(&header).unwrap();
    let mut entry = [0u8; 32];
    entry[4..8].copy_from_slice(&34u32.to_le_bytes());
    entry[8..12].copy_from_slice(&2048u32.to_le_bytes());
    entry[13] = 11;
    entry[14..25].copy_from_slice(b"default.xbe");
    file.seek(SeekFrom::Start(partition + 33 * 2048)).unwrap();
    file.write_all(&entry).unwrap();
    file.seek(SeekFrom::Start(partition + 34 * 2048)).unwrap();
    file.write_all(b"XBEH").unwrap();
}

#[test]
fn structural_disc_checks_preserve_media_and_reject_partial_download() {
    let dir = scratch("disc");
    let path = dir.join("fixture.xiso");
    for partition in [0, 0x18300000] {
        fixture(&path, partition);
        let before = fs::metadata(&path).unwrap().len();
        assert_eq!(
            xbx::disc::inspect(&path).unwrap().partition_offset,
            partition
        );
        assert_eq!(fs::metadata(&path).unwrap().len(), before);
    }
    fixture(&path, 0);
    let mut bad = File::options().write(true).open(&path).unwrap();
    bad.seek(SeekFrom::Start(34 * 2048)).unwrap();
    bad.write_all(b"XEX2").unwrap();
    assert!(xbx::disc::inspect(&path).is_err());
    fixture(&path, 0);
    File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(33 * 2048)
        .unwrap();
    assert!(xbx::disc::inspect(&path).is_err());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn incomplete_generation_publication_is_preserved() {
    let dir = scratch("interrupted-publication");
    let lease = ProfileLease::acquire(&dir).unwrap();
    assert_eq!(lease.begin_writes().unwrap(), 1);
    fs::write(dir.join("generation.pending"), b"incomplete evidence").unwrap();
    assert!(lease.begin_writes().is_err());
    assert_eq!(lease.generation().unwrap(), 1);
    assert_eq!(
        fs::read(dir.join("generation.pending")).unwrap(),
        b"incomplete evidence"
    );
    drop(lease);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn child_holds_lease() {
    let Some(root) = std::env::var_os("XBX_TEST_PROFILE") else {
        return;
    };
    let _lease = ProfileLease::acquire(Path::new(&root)).unwrap();
    fs::write(Path::new(&root).join("ready"), b"ready").unwrap();
    loop {
        std::thread::park();
    }
}

#[test]
fn writer_is_exclusive_across_processes_and_death_releases_it() {
    let dir = scratch("writer-death");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "child_holds_lease", "--nocapture"])
        .env("XBX_TEST_PROFILE", &dir)
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !dir.join("ready").exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    let ready = dir.join("ready").exists();
    let refused = ProfileLease::acquire(&dir).is_err();
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(
        ready && refused,
        "live child must hold the exclusive OS lease"
    );
    let lease = ProfileLease::acquire(&dir).unwrap();
    drop(lease);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn native_child_keeps_lease_when_supervisor_closes_its_copy() {
    let dir = scratch("inherited-writer");
    let lease = ProfileLease::acquire(&dir).unwrap();
    let mut command = Command::new("sleep");
    command.arg("30");
    lease.inherit_for(&mut command);
    let mut child = command.spawn().unwrap();
    drop(lease);
    let refused = ProfileLease::acquire(&dir).is_err();
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(
        refused,
        "native writer must retain ownership after supervisor loss"
    );
    drop(ProfileLease::acquire(&dir).unwrap());
    fs::remove_dir_all(dir).unwrap();
}

fn native(command: &mut Command) {
    let result = command
        .output()
        .expect("qemu-img and qemu-io must be installed for this explicit native storage probe");
    assert!(
        result.status.success(),
        "native tool failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

/// Explicit integration probe; only disposable synthetic block data, never game saves.
#[test]
#[ignore = "requires qemu-img and qemu-io; run explicitly after host preflight"]
fn qcow2_current_progress_survives_reopen_and_old_snapshot_is_refused() {
    let dir = scratch("qcow2");
    let base = dir.join("base.qcow2");
    let disk = dir.join("hdd.qcow2");
    let rollback = dir.join("deliberate-rollback-copy.qcow2");
    native(
        Command::new("qemu-img")
            .args(["create", "-f", "qcow2"])
            .arg(&base)
            .arg("16M"),
    );
    native(
        Command::new("qemu-img")
            .args(["create", "-f", "qcow2", "-F", "qcow2", "-b"])
            .arg(&base)
            .arg(&disk),
    );
    let lease = ProfileLease::acquire(&dir).unwrap();
    let old_generation = lease.begin_writes().unwrap();
    native(
        Command::new("qemu-io")
            .args(["-f", "qcow2", "-c", "write -P 0x31 0 4096"])
            .arg(&disk),
    );
    native(
        Command::new("qemu-img")
            .args(["snapshot", "-c", "old"])
            .arg(&disk),
    );
    lease.begin_writes().unwrap();
    native(
        Command::new("qemu-io")
            .args(["-f", "qcow2", "-c", "write -P 0x72 0 4096"])
            .arg(&disk),
    );
    lease.sync_storage().unwrap();
    drop(lease);
    let lease = ProfileLease::acquire(&dir).unwrap();
    assert!(lease.require_generation(old_generation).is_err());
    native(
        Command::new("qemu-io")
            .args(["-f", "qcow2", "-r", "-c", "read -P 0x72 0 4096"])
            .arg(&disk),
    );
    native(
        Command::new("qemu-io")
            .args(["-f", "qcow2", "-r", "-c", "read -P 0x00 0 4096"])
            .arg(&base),
    );
    // Prove the dangerous behavior on a separate scratch copy; accepted data stays current.
    fs::copy(&disk, &rollback).unwrap();
    native(
        Command::new("qemu-img")
            .args(["snapshot", "-a", "old"])
            .arg(&rollback),
    );
    native(
        Command::new("qemu-io")
            .args(["-f", "qcow2", "-r", "-c", "read -P 0x31 0 4096"])
            .arg(&rollback),
    );
    native(
        Command::new("qemu-io")
            .args(["-f", "qcow2", "-r", "-c", "read -P 0x72 0 4096"])
            .arg(&disk),
    );
    native(Command::new("qemu-img").arg("check").arg(&disk));
    println!(
        "PASS: reopened disk has new bytes; base unchanged; old-generation resume refused; rollback copy demonstrates loss of new bytes. Not an in-game save or full-machine resume test."
    );
    drop(lease);
    // Retain this named synthetic experiment for review (under ignored target/scratch).
    println!("retained synthetic evidence: {}", dir.display());
}
