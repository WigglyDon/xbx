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

#[test]
#[ignore = "requires the source-built native engine and local socket/graphics permission"]
fn native_rejection_preserves_existing_eeprom_and_disk_after_resume() {
    let dir = scratch("native-rejected-profile");
    let engine = std::env::var_os("XBX_TEST_ENGINE").expect("select source-built engine");
    let eeprom = vec![0; 256];
    let disk = b"synthetic writable-disk predecessor; must remain unchanged";
    fs::write(dir.join("eeprom.bin"), &eeprom).unwrap();
    fs::write(dir.join("hdd.qcow2"), disk).unwrap();
    fs::write(dir.join("flash.bin"), vec![0xff; 1024 * 1024]).unwrap();
    let cfg = xbx::profile_config(&dir)
        .unwrap()
        .replace("\"vulkan\"", "\"OPENGL\"");
    fs::write(dir.join("xemu.toml"), cfg).unwrap();
    let lease = ProfileLease::acquire(&dir).unwrap();
    let log = File::create(dir.join("native.log")).unwrap();
    let mut command = Command::new(engine);
    command
        .arg("-config_path")
        .arg(dir.join("xemu.toml"))
        .env("SDL_VIDEO_DRIVER", "offscreen")
        .env("SDL_AUDIO_DRIVER", "dummy")
        .env("LIBGL_ALWAYS_SOFTWARE", "1")
        .env("XDG_DATA_HOME", dir.join("data"))
        .env("XDG_CONFIG_HOME", dir.join("config"))
        .env("XDG_CACHE_HOME", dir.join("cache"))
        .env("MESA_SHADER_CACHE_DIR", dir.join("mesa-cache"))
        .stdin(Stdio::null())
        .stdout(log.try_clone().unwrap())
        .stderr(log);
    lease.inherit_for(&mut command);
    let mut child = command.spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let result = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break Some(status);
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            break None;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let lifecycle = xbx::startup::Startup::new();
    lifecycle.report(-1);
    lifecycle.display_ready();
    lifecycle.foreground(false);
    lifecycle.foreground(true);
    assert!(!lifecycle.may_resume());
    assert!(
        result.is_some_and(|s| !s.success()),
        "native rejection must exit before guest start"
    );
    assert_eq!(
        result.unwrap().code(),
        Some(1),
        "terminal refusal must not be an abort/signal"
    );
    assert_eq!(fs::read(dir.join("eeprom.bin")).unwrap(), eeprom);
    assert_eq!(fs::read(dir.join("hdd.qcow2")).unwrap(), disk);
    let log = fs::read_to_string(dir.join("native.log")).unwrap();
    assert!(!log.contains("[CPU-PRE]"));
    assert!(log.contains("EEPROM missing/invalid"));
    println!(
        "PASS: native rejection exits before guest execution; foreground/resume stays gated and both predecessor files are unchanged. Scratch evidence retained."
    );
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
    let mut xbe = [0u8; 2048];
    xbe[..4].copy_from_slice(b"XBEH");
    xbe[0x104..0x108].copy_from_slice(&0x10000u32.to_le_bytes());
    xbe[0x108..0x10c].copy_from_slice(&2048u32.to_le_bytes());
    xbe[0x118..0x11c].copy_from_slice(&0x10180u32.to_le_bytes());
    xbe[0x180..0x184].copy_from_slice(&0x1d0u32.to_le_bytes());
    for (index, value) in "Synthetic title".encode_utf16().enumerate() {
        let offset = 0x18c + index * 2;
        xbe[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    file.write_all(&xbe).unwrap();
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
    bad.seek(SeekFrom::Start(34 * 2048 + 0x118)).unwrap();
    bad.write_all(&0xfffffff0u32.to_le_bytes()).unwrap();
    assert!(xbx::disc::inspect(&path).is_err());
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
fn callsite_probe_bounds_code_excludes_headers_and_preserves_existing_evidence() {
    let dir = scratch("callsite");
    let path = dir.join("fixture.xiso");
    fixture(&path, 0);
    let mut file = File::options().read(true).write(true).open(&path).unwrap();
    let start = 34 * 2048;
    let mut image = [0; 2048];
    file.seek(SeekFrom::Start(start)).unwrap();
    std::io::Read::read_exact(&mut file, &mut image).unwrap();
    let set = |bytes: &mut [u8], at: usize, value: u32| {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes())
    };
    set(&mut image, 0x108, 0x400);
    set(&mut image, 0x10c, 0x20000);
    set(&mut image, 0x11c, 1);
    set(&mut image, 0x120, 0x10360);
    set(&mut image, 0x128, 0x20000 ^ 0xa8fc57ab);
    set(&mut image, 0x158, 0x20300 ^ 0x5b6d40b6);
    set(&mut image, 0x360, 4);
    set(&mut image, 0x364, 0x20000);
    set(&mut image, 0x368, 0x400);
    set(&mut image, 0x36c, 0x400);
    set(&mut image, 0x370, 0x400);
    image[0x440..0x4c0].fill(0x90);
    set(&mut image, 0x700, 0x8000012b);
    file.seek(SeekFrom::Start(start)).unwrap();
    file.write_all(&image).unwrap();
    drop(file);
    let mut file = File::open(&path).unwrap();
    let disc = xbx::disc::inspect_open(&mut file).unwrap();
    let request = "xbx-callsite-v1 00020040 80";
    let (code, map) = xbx::callsite::observe(&mut file, &disc, request).unwrap();
    assert_eq!(code, vec![0x90; 128]);
    assert!(map.contains("import=00020300:299"));
    for invalid in [
        "xbx-callsite-v1 00010000 80",
        "xbx-callsite-v1 000203f0 80",
        "xbx-callsite-v1 00020000 8001",
        "xbx-callsite-v1 00020000 0",
        "xbx-callsite-v1 00020000 10 extra",
    ] {
        assert!(xbx::callsite::observe(&mut file, &disc, invalid).is_err());
    }
    fs::write(dir.join("callsite.request"), request).unwrap();
    assert!(xbx::callsite::requested(&dir, &mut file, &disc).is_err());
    fs::write(dir.join("branch.identity"), "synthetic branch").unwrap();
    assert!(xbx::callsite::requested(&dir, &mut file, &disc).unwrap());
    assert!(xbx::callsite::requested(&dir, &mut file, &disc).is_err());
    assert_eq!(fs::read(dir.join("callsite-evidence/code")).unwrap(), code);
    assert_eq!(fs::read(&path).unwrap()[start as usize..], image);
    // A forged section cannot export the certificate/header, nor non-code data.
    let mut writer = File::options().write(true).open(&path).unwrap();
    for (at, value) in [(0x36c, 0x180u32), (0x360, 0)] {
        writer.seek(SeekFrom::Start(start + at)).unwrap();
        writer.write_all(&value.to_le_bytes()).unwrap();
        assert!(xbx::callsite::observe(&mut file, &disc, request).is_err());
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn diagnostic_modes_cannot_alias_a_used_profile_or_choose_arbitrary_paths() {
    use xbx::branch::diagnostic_root;
    assert_eq!(diagnostic_root("ram_mib=64\n").unwrap(), None);
    let old = diagnostic_root("mode=launch-diagnostic-v1\n").unwrap();
    let next = diagnostic_root("mode=launch-diagnostic-v2\n").unwrap();
    assert_ne!(old, next);
    assert_ne!(old, Some("xbx-qualification"));
    assert_ne!(next, Some("xbx-qualification"));
    assert!(diagnostic_root("mode=../xbx-qualification\n").is_err());
    assert!(diagnostic_root("mode=launch-diagnostic-v2\nmode=launch-diagnostic-v1\n").is_err());
}

#[test]
fn retained_read_only_document_is_not_reopened_after_path_replacement() {
    let dir = scratch("retained-media");
    let path = dir.join("source.iso");
    fixture(&path, 0);
    let mut retained = File::open(&path).unwrap();
    fs::remove_file(&path).unwrap();
    fs::write(&path, b"a different file at the original pathname").unwrap();
    assert!(xbx::disc::inspect(&path).is_err());
    let disc = xbx::disc::inspect_open(&mut retained).unwrap();
    assert_eq!(disc.title.name, "Synthetic title");
    assert_eq!(disc.partition_offset, 0);
    assert!(retained.write_all(b"must fail").is_err());
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
    // Parallel fixtures may briefly inherit this CLOEXEC descriptor between
    // fork and exec. Reaping one PID does not prove the final inherited copy
    // has closed: actual acquisition, rather than PID inference, is the proof.
    let deadline = Instant::now() + Duration::from_secs(2);
    let released = loop {
        match ProfileLease::acquire(&dir) {
            Ok(lease) => break lease,
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("inherited writer did not release its lease: {error}"),
        }
    };
    drop(released);
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
#[ignore = "requires qemu-io; run explicitly after host preflight"]
fn raw_disc_partition_view_is_read_only_and_bounded() {
    let dir = scratch("raw-disc-view");
    let source = dir.join("full-disc.fixture");
    let mut original = vec![0xee; 4096];
    original.extend_from_slice(&[0x31; 4096]);
    original.extend_from_slice(&[0xaa; 4096]);
    fs::write(&source, &original).unwrap();
    let options = format!(
        "driver=raw,offset=4096,size=4096,file.driver=file,file.filename={}",
        source.display()
    );
    native(
        Command::new("qemu-io")
            .args(["--image-opts", "-r", "-c", "read -P 0x31 0 4096"])
            .arg(&options),
    );
    let rejected_write = Command::new("qemu-io")
        .args(["--image-opts", "-r", "-c", "write -P 0x00 0 512"])
        .arg(&options)
        .output()
        .unwrap();
    assert!(!rejected_write.status.success());
    let rejected_past_end = Command::new("qemu-io")
        .args(["--image-opts", "-r", "-c", "read 4096 512"])
        .arg(&options)
        .output()
        .unwrap();
    assert!(!rejected_past_end.status.success());
    assert_eq!(fs::read(&source).unwrap(), original);
    println!(
        "PASS: raw partition view reads only the selected region, rejects writes/out-of-range reads, and preserves the entire source fixture. Android execution remains untested."
    );
    fs::remove_dir_all(dir).unwrap();
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

#[test]
fn qemu_media_handoff_clears_only_duplicate_cloexec_and_stays_read_only() {
    use std::os::fd::AsRawFd;
    unsafe extern "C" {
        fn fcntl(fd: i32, command: i32, ...) -> i32;
    }
    let root = scratch("media-fd-flags");
    let path = root.join("disc.bin");
    fs::write(&path, b"synthetic read-only disc").unwrap();
    let grant = File::open(&path).unwrap();
    let original = unsafe { fcntl(grant.as_raw_fd(), 1) };
    let fd = xbx::disc::qemu_media_fd(&grant).unwrap();
    assert_eq!(unsafe { fcntl(fd.as_raw_fd(), 1) } & 1, 0);
    assert_eq!(unsafe { fcntl(grant.as_raw_fd(), 1) }, original);
    let mut copy = File::from(fd);
    assert!(copy.write_all(b"changed").is_err());
    assert_eq!(fs::read(&path).unwrap(), b"synthetic read-only disc");
    assert!(xbx::disc::qemu_media_fd(&File::options().write(true).open(&path).unwrap()).is_err());
    fs::remove_dir_all(root).unwrap();
}

fn fixture_sha(file: &mut File) -> std::io::Result<String> {
    file.seek(SeekFrom::Start(0))?;
    let out = Command::new("sha256sum")
        .stdin(Stdio::from(file.try_clone()?))
        .output()?;
    if !out.status.success() {
        return Err(std::io::Error::other("fixture hash failed"));
    }
    Ok(String::from_utf8(out.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned())
}
fn branch_fixture(root: &Path) -> (Vec<u8>, Vec<u8>) {
    for (name, data) in [
        ("flash.bin", b"baseline firmware".as_slice()),
        ("hdd.qcow2", b"newest durable saves"),
        ("eeprom.bin", b"stable synthetic identity"),
        ("generation", b"12\n"),
        ("media.uri", b"content://synthetic/retained-disc"),
    ] {
        fs::write(root.join(name), data).unwrap();
    }
    let mut disk = vec![0u8; 104];
    disk[..8].copy_from_slice(b"QFI\xfb\0\0\0\x03");
    disk.extend_from_slice(b"newest durable saves");
    fs::write(root.join("hdd.qcow2"), disk).unwrap();
    let baseline = fixture_sha(&mut File::open(root.join("flash.bin")).unwrap()).unwrap();
    fs::write(
        root.join("profile.identity"),
        format!("sha256={baseline}\nram_mib=64\n"),
    )
    .unwrap();
    fs::write(root.join("saved-session"),format!("xbx-saved-session-v1\ngame_id={}\nprofile_id={}\nmedia_sha256=fixture-media\nfirmware_sha256={baseline}\nram_mib=64\n","a".repeat(32),"b".repeat(32))).unwrap();
    let fw = b"instrumented diagnostic firmware".to_vec();
    let path = root.join("new-fw-fixture");
    fs::write(&path, &fw).unwrap();
    let new = fixture_sha(&mut File::open(path).unwrap()).unwrap();
    (fw,format!("xbx-qualification-profile-v1\nsha256={new}\nparent_firmware_sha256={baseline}\nram_mib=64\n").into_bytes())
}
#[test]
fn diagnostic_branch_preserves_parent_and_has_independent_writes_and_identity() {
    let root = scratch("coherent-branch");
    let parent = root.join("parent");
    fs::create_dir(&parent).unwrap();
    let target = root.join("diagnostic");
    let (fw, identity) = branch_fixture(&parent);
    let names = [
        "flash.bin",
        "hdd.qcow2",
        "eeprom.bin",
        "generation",
        "media.uri",
        "saved-session",
        "profile.identity",
    ];
    let before: Vec<_> = names
        .iter()
        .map(|n| fs::read(parent.join(n)).unwrap())
        .collect();
    let branch = xbx::branch::create(
        &parent,
        &target,
        &fw,
        &identity,
        &"c".repeat(32),
        fixture_sha,
    )
    .unwrap();
    assert_eq!(branch.generation().unwrap(), 12);
    assert!(ProfileLease::acquire(&target).is_err());
    let _parent = ProfileLease::acquire(&parent).unwrap();
    for (n, bytes) in names.iter().zip(before) {
        assert_eq!(fs::read(parent.join(n)).unwrap(), bytes);
    }
    assert_eq!(
        fs::read(parent.join("eeprom.bin")).unwrap(),
        fs::read(target.join("eeprom.bin")).unwrap()
    );
    let session = fs::read_to_string(target.join("saved-session")).unwrap();
    assert_eq!(
        xbx::branch::field(&session, "profile_id").unwrap(),
        "c".repeat(32)
    );
    assert_eq!(
        xbx::branch::field(&session, "game_id").unwrap(),
        "a".repeat(32)
    );
    assert!(
        fs::read_to_string(target.join("branch.identity"))
            .unwrap()
            .contains("parent_generation=12")
    );
    fs::write(target.join("hdd.qcow2"), b"diagnostic writes").unwrap();
    assert!(
        fs::read(parent.join("hdd.qcow2"))
            .unwrap()
            .ends_with(b"newest durable saves")
    );
}
#[test]
fn diagnostic_branch_refuses_owned_source_and_preserves_incomplete_target() {
    let root = scratch("refused-branch");
    let parent = root.join("parent");
    fs::create_dir(&parent).unwrap();
    let target = root.join("diagnostic");
    let (fw, identity) = branch_fixture(&parent);
    let owner = ProfileLease::acquire(&parent).unwrap();
    assert!(
        xbx::branch::create(
            &parent,
            &target,
            &fw,
            &identity,
            &"c".repeat(32),
            fixture_sha
        )
        .is_err()
    );
    assert!(!target.exists());
    drop(owner);
    assert!(
        xbx::branch::create(
            &parent,
            &target,
            b"wrong firmware bytes",
            &identity,
            &"c".repeat(32),
            fixture_sha
        )
        .is_err()
    );
    assert!(target.join("hdd.qcow2").is_file());
    assert!(!target.join("branch.identity").exists());
    assert!(
        xbx::branch::create(
            &parent,
            &target,
            &fw,
            &identity,
            &"c".repeat(32),
            fixture_sha
        )
        .is_err()
    );
    assert!(
        fs::read(parent.join("hdd.qcow2"))
            .unwrap()
            .ends_with(b"newest durable saves")
    );
}

#[test]
fn diagnostic_branch_rejects_external_disk_dependencies_before_copying() {
    let root = scratch("external-disk-branch");
    let parent = root.join("parent");
    fs::create_dir(&parent).unwrap();
    let (fw, identity) = branch_fixture(&parent);
    let original = fs::read(parent.join("hdd.qcow2")).unwrap();
    for (offset, value) in [(8, 1), (79, 4)] {
        let mut disk = original.clone();
        disk[offset] = value;
        fs::write(parent.join("hdd.qcow2"), &disk).unwrap();
        let target = root.join(format!("branch-{offset}"));
        assert!(
            xbx::branch::create(
                &parent,
                &target,
                &fw,
                &identity,
                &"c".repeat(32),
                fixture_sha
            )
            .is_err()
        );
        assert!(!target.exists());
        assert_eq!(fs::read(parent.join("hdd.qcow2")).unwrap(), disk);
    }
}
