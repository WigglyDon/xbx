// SPDX-License-Identifier: MIT
//! Bounded integration/packaging for XBX-CARTRIDGE-1, retained inputs only.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::io::Write;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use std::{
    io::{BufRead, BufReader},
    process::{Child, ChildStdin, ChildStdout, Stdio},
    time::{Duration, Instant},
};
struct BuildPaths {
    root: std::path::PathBuf,
}
type Error = Box<dyn std::error::Error>;
fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
fn fixture(repo: &Path, w: &BuildPaths) -> Result<(), Error> {
    let dir = w.root.join("fixtures-v2");
    c::mkdir(&dir)?;
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    for (label, tag) in [("a", 65), ("b", 66)] {
        let obj = dir.join(format!("{label}.o"));
        let bin = dir.join(format!("{label}.bin"));
        c::store::checked(
            Command::new(llvm.join("clang"))
                .args(["--target=i386-none-elf", "-march=pentium3", "-c"])
                .arg(format!("-DCARTRIDGE_TAG={tag}"))
                .arg(repo.join("native/fixtures/cartridge.S"))
                .arg("-o")
                .arg(&obj),
        )?;
        c::store::checked(
            Command::new(llvm.join("ld.lld"))
                .args([
                    "-m",
                    "elf_i386",
                    "-Ttext=0x11000",
                    "--oformat=binary",
                    "--entry=_start",
                ])
                .arg(&obj)
                .arg("-o")
                .arg(&bin),
        )?;
        let code = fs::read(bin)?;
        if code.is_empty() || code.len() > 4096 {
            return Err("fixture extent".into());
        }
        let mut xbe = vec![0; 4096];
        xbe[..4].copy_from_slice(b"XBEH");
        for (at, v) in [
            (0x104, 0x10000),
            (0x108, 0x1000),
            (0x10c, 0x2000),
            (0x110, 0x178),
            (0x118, 0x10178),
            (0x11c, 1),
            (0x120, 0x10348),
            (0x128, 0x11000 ^ 0xa8fc57ab),
            (0x130, 0x10000),
            (0x134, 0x100000),
            (0x138, 0x1000),
            (0x158, 0x10300 ^ 0x5b6d40b6),
            (0x178, 0x1d0),
            (0x180, 0x58424300 + tag as u32),
            (0x214, 0xffffff),
            (0x218, 0x7fffffff),
        ] {
            put(&mut xbe, at, v)
        }
        let name = format!("xbx cartridge {label}");
        for (i, ch) in name.encode_utf16().enumerate() {
            xbe[0x184 + i * 2..0x186 + i * 2].copy_from_slice(&ch.to_le_bytes())
        }
        xbe[0x400..0x406].copy_from_slice(b".text\0");
        for (at, v) in [
            (0, 7),
            (4, 0x11000),
            (8, 4096),
            (12, 4096),
            (16, code.len() as u32),
            (20, 0x10400),
            (28, 0x10410),
            (32, 0x10412),
        ] {
            put(&mut xbe, 0x348 + at, v)
        }
        xbe[0x348 + 36..0x348 + 56].copy_from_slice(&section_digest(&code)?);
        xbe.extend_from_slice(&code);
        c::write_new(&dir.join(format!("{label}.xbe")), &xbe)?;
        let mut iso = vec![0; 64 * 2048];
        let at = 32 * 2048;
        let magic = b"MICROSOFT*XBOX*MEDIA";
        iso[at..at + 20].copy_from_slice(magic);
        put(&mut iso, at + 20, 33);
        put(&mut iso, at + 24, 2048);
        iso[at + 2028..at + 2048].copy_from_slice(magic);
        let root = 33 * 2048;
        put(&mut iso, root + 4, 34);
        put(&mut iso, root + 8, xbe.len() as u32);
        iso[root + 12] = 0x20;
        iso[root + 13] = 11;
        iso[root + 14..root + 25].copy_from_slice(b"default.xbe");
        iso[34 * 2048..34 * 2048 + xbe.len()].copy_from_slice(&xbe);
        c::write_new(&dir.join(format!("{label}.iso")), &iso)?;
    }
    Ok(())
}
fn package(repo: &Path, w: &BuildPaths) -> Result<(), Error> {
    let project = w.root.parent().ok_or("workspace parent")?;
    let dir = w.root.join("package-v3");
    c::mkdir(&dir)?;
    c::mkdir(&dir.join("native"))?;
    fs::create_dir_all(dir.join("native/subprojects/slirp"))?;
    let core = w.root.join("native-v3/core");
    let open = project.join("open-startup-v1/build-release-v3/bios.bin");
    if c::file_digest(&core)? != resources::CORE || c::file_digest(&open)? != resources::OPEN {
        return Err("accepted input identity differs".into());
    }
    fs::copy(core, dir.join("native/core"))?;
    fs::copy(open, dir.join("open.bin"))?;
    fs::copy("/usr/bin/qemu-img", dir.join("native/qemu-img"))?;
    fs::copy(
        project.join("ramht-v1/build-v2-retry2/subprojects/slirp/libslirp.so.0"),
        dir.join("native/subprojects/slirp/libslirp.so.0"),
    )?;
    let manifest = resources::Manifest {
        schema: 1,
        core: resources::CORE.into(),
        open_system: resources::OPEN.into(),
        disk_tool: c::file_digest(&dir.join("native/qemu-img"))?,
        slirp: c::file_digest(&dir.join("native/subprojects/slirp/libslirp.so.0"))?,
        state_abi: 1,
    };
    c::write_new(&dir.join("runtime.json"), &c::json(&manifest)?)?;
    fs::copy(repo.join("THIRD_PARTY.md"), dir.join("THIRD_PARTY.md"))?;
    resources::Resources::open(&dir)?;
    Ok(())
}
pub fn command(repo: &Path, root: &Path, action: &str) -> Result<(), Error> {
    let w = BuildPaths { root: root.into() };
    match action {
        "cartridge-package" => package(repo, &w)?,
        "cartridge-fixture" => fixture(repo, &w)?,
        "cartridge-rebuild-io-images" => super::rebuild::io_images(repo, root)?,
        "cartridge-rebuild-native" => super::rebuild::native(repo, root)?,
        "cartridge-recover-media" => super::rebuild::media(repo, root)?,
        "cartridge-local-io-baseline" => super::local_io::control(repo, root, false, false)?,
        "cartridge-local-io-reporting" => super::local_io::control(repo, root, true, false)?,
        "cartridge-local-io-demand" => super::local_io::control(repo, root, true, true)?,
        "cartridge-local-io-fable" => super::local_io::fable(repo, root)?,
        "cartridge-local-fable-play" => super::local_io::play(repo, root)?,
        "cartridge-local-fable-continue" => super::local_io::continue_play(repo, root)?,
        "cartridge-local-fable-return" => super::local_io::return_play(repo, root)?,
        "cartridge-apu-paused-original" => super::apu_quiesce::baseline(repo, root)?,
        "cartridge-apu-correction-build" => super::apu_quiesce::correct_runtime(repo, root)?,
        "cartridge-apu-paused-corrected" => super::apu_quiesce::corrected(repo, root, false)?,
        "cartridge-apu-running-control" => super::apu_quiesce::corrected(repo, root, true)?,
        "cartridge-apu-original-io" => super::apu_quiesce::original_io(repo, root)?,
        "cartridge-apu-fable-play" => super::apu_quiesce::fable(repo, root, false)?,
        "cartridge-apu-fable-return" => super::apu_quiesce::fable(repo, root, true)?,
        "cartridge-file-highva-build" => super::file_read::build_high_va(repo, root)?,
        "cartridge-file-highva-observe" => super::file_read::observe_high_va(repo, root)?,
        "cartridge-file-demand-build" => super::file_read::build_demand(repo, root)?,
        "cartridge-file-demand-negative" => super::file_read::demand_negative(repo, root)?,
        "cartridge-buffer-probe-build" => super::buffer_probe::build(repo, root)?,
        "cartridge-buffer-probe-demand" => super::file_read::probe_regression(repo, root, true)?,
        "cartridge-buffer-probe-allocated" => {
            super::file_read::probe_regression(repo, root, false)?
        }
        "cartridge-buffer-probe-fable" => fable_logging(repo, &w, LoggingRuntime::BufferProbe)?,
        "cartridge-bounded-storage-proof" => super::buffer_probe::storage_proof(repo, root)?,
        "cartridge-io-errors-build" => super::buffer_probe::build_io_errors(repo, root)?,
        "cartridge-file-io-errors-build" => super::file_read::build_io_errors(repo, root)?,
        "cartridge-io-errors-baseline-control" => {
            super::file_read::io_error_control(repo, root, false)?
        }
        "cartridge-io-errors-control" => super::file_read::io_error_control(repo, root, true)?,
        "cartridge-io-errors-demand" => super::file_read::io_demand_regression(repo, root)?,
        "cartridge-io-errors-fable" => fable_logging(repo, &w, LoggingRuntime::IoErrors)?,
        "cartridge-file-read-build" => super::file_read::build(repo, root, false)?,
        "cartridge-file-read-observe" => super::file_read::observe(repo, root, false)?,
        "cartridge-flush-cache" => super::file_read::flush_regression(repo, root, true)?,
        "cartridge-flush-disc" => super::file_read::flush_regression(repo, root, false)?,
        "cartridge-flush-fable" => fable_logging(repo, &w, LoggingRuntime::Flush)?,
        "cartridge-flush-build" => super::flush_io::build(repo, root)?,
        "cartridge-file-cache-build" => super::file_read::build(repo, root, true)?,
        "cartridge-file-cache-observe" => super::file_read::observe(repo, root, true)?,
        "cartridge-proof" => proof(&w)?,
        "cartridge-native" => native(repo, &w)?,
        "cartridge-partial-load" => partial(&w)?,
        "cartridge-policy-proof" => policy_proof(&w)?,
        "cartridge-paused-contract" => paused_contract(repo, &w)?,
        "cartridge-native-paused" => native_paused(repo, &w)?,
        "cartridge-fable" => fable(repo, &w)?,
        "cartridge-lifetime-build" => super::lifetime::build(repo, root)?,
        "cartridge-lifetime-observe" => super::lifetime::observe(repo, root)?,
        "cartridge-npx-build" => super::npx_port::build(repo, root)?,
        "cartridge-x87-build" => super::x87_boundary::build(repo, root)?,
        "cartridge-x87-original" => super::x87_boundary::original(repo, root)?,
        "cartridge-npx-observe" => super::npx_port::observe(repo, root)?,
        "cartridge-npx-relaunch" => super::npx_followup::relaunch(repo, root)?,
        "cartridge-npx-fable" => fable_logging(repo, &w, LoggingRuntime::Npx)?,
        "cartridge-x87-relaunch" => super::npx_followup::relaunch_with_core(repo, root, true)?,
        "cartridge-x87-fable" => fable_logging(repo, &w, LoggingRuntime::X87)?,
        "cartridge-rsp-original" => super::fault_context::observe(repo, root, true, true)?,
        "cartridge-rsp-fable" => super::fault_context::observe(repo, root, false, true)?,
        "cartridge-fault-original" => super::fault_context::observe(repo, root, true, false)?,
        "cartridge-fault-fable" => super::fault_context::observe(repo, root, false, false)?,
        "cartridge-npx-values-build" => super::npx_port::values_fixture(repo, root)?,
        "cartridge-npx-layout-build" => super::npx_port::layout_fixture(repo, root)?,
        "cartridge-android-native" => super::android_build::native(repo, root)?,
        "cartridge-android-link" => super::android_build::link_adapters(repo, root)?,
        "cartridge-android-helper" => super::android_build::image_helper(repo, root)?,
        "cartridge-android-bridge" => super::android_build::bridge(repo, root)?,
        "cartridge-android-socket" => super::android_build::socket_driver(repo, root)?,
        "cartridge-android-monitor" => super::android_build::monitor_base(repo, root)?,
        "cartridge-android-session-monitor" => super::android_build::session_monitor(repo, root)?,
        "cartridge-android-monitor-contract" => super::android_build::monitor_contract(repo, root)?,
        "cartridge-android-package" => super::android_package::build(repo, root)?,
        "cartridge-android-package-socket" => super::android_package::build_socket(repo, root)?,
        "cartridge-android-package-monitor" => super::android_package::build_monitor(repo, root)?,
        "cartridge-android-package-session" => super::android_package::build_session(repo, root)?,
        "cartridge-android-package-back" => super::android_package::build_back(repo, root)?,
        "cartridge-android-sign" => super::android_package::sign_retained(repo, root)?,
        "cartridge-capability" => capability(&w)?,
        _ => return Err("unknown cartridge build action".into()),
    };
    Ok(())
}

fn capability(w: &BuildPaths) -> Result<(), Error> {
    use std::fs::OpenOptions;
    let app = w.root.join(format!("capability-{}", c::new_id()?));
    c::mkdir(&app)?;
    let _lease = crate::storage::ProfileLease::acquire(&app)?;
    let original = app.join("original.iso");
    fs::copy(w.root.join("fixtures-v2/a.iso"), &original)?;
    let file = fs::File::open(&original)?;
    let moved = app.join("moved.iso");
    fs::rename(&original, &moved)?;
    let mut library = c::library::Library::read(&app)?;
    let id = library.add_open(&app, "content://xbx-original/a", file)?;
    let e = library.resolve(&id)?;
    c::library::Library::verify_open(&e, fs::File::open(&moved)?)?;
    if c::library::Library::verify_open(&e, OpenOptions::new().read(true).write(true).open(&moved)?)
        .is_ok()
    {
        return Err("writable media capability admitted".into());
    }
    let before = fs::read(app.join("library.json"))?;
    if library
        .relink_open(
            &app,
            &id,
            "content://xbx-original/wrong",
            fs::File::open(w.root.join("fixtures-v2/b.iso"))?,
        )
        .is_ok()
        || fs::read(app.join("library.json"))? != before
    {
        return Err("wrong media changed collection identity".into());
    }
    library.relink_open(
        &app,
        &id,
        "content://xbx-original/relinked",
        fs::File::open(&moved)?,
    )?;
    let after = library.resolve(&id)?;
    if after.profile != e.profile || after.media_sha256 != e.media_sha256 || after.id != e.id {
        return Err("relink changed durable identity".into());
    }
    c::write_new(&app.join("capability.qualified"),b"actual original read-only descriptor import survives pathname move; no content URI reopening; writable descriptor refused; wrong relink preserves collection; exact relink keeps game/profile identity\n")?;
    println!("{}", json!({"original_capability_proof":app,"passed":true}));
    Ok(())
}

/// One evidence-selected, corrected logging observation. This developer-only
/// resource route is never exposed by xbx-player or the ordinary manifest.
fn fable(repo: &Path, w: &BuildPaths) -> Result<(), Error> {
    fable_logging(repo, w, LoggingRuntime::Corrected)
}
#[derive(Clone, Copy)]
enum LoggingRuntime {
    Corrected,
    Npx,
    X87,
    Flush,
    BufferProbe,
    IoErrors,
}
fn fable_logging(repo: &Path, w: &BuildPaths, runtime: LoggingRuntime) -> Result<(), Error> {
    let npx = !matches!(runtime, LoggingRuntime::Corrected);
    let flush = matches!(runtime, LoggingRuntime::Flush);
    let probe = matches!(runtime, LoggingRuntime::BufferProbe);
    let io_errors = matches!(runtime, LoggingRuntime::IoErrors);
    let functional = flush || probe || io_errors;
    let x87 = matches!(
        runtime,
        LoggingRuntime::X87
            | LoggingRuntime::Flush
            | LoggingRuntime::BufferProbe
            | LoggingRuntime::IoErrors
    );
    if io_errors {
        super::buffer_probe::qualified_io(&w.root)?;
    }
    if probe {
        super::buffer_probe::qualified(&w.root)?;
    }
    if flush {
        super::flush_io::qualified(&w.root)?;
    }
    let core = if x87 {
        resources::X87_RESTART_CORE
    } else {
        resources::CORE
    };
    if x87 && !w.root.join("x87-boundary-v1/original.qualified").exists() {
        return Err("corrected-core original qualification required".into());
    }
    let prefix = if io_errors {
        "fable-io-errors-logging"
    } else if probe {
        "fable-buffer-probe-logging"
    } else if flush {
        "fable-flush-logging"
    } else if x87 {
        "fable-x87-logging"
    } else if npx {
        "fable-npx-logging"
    } else {
        "fable-logging"
    };
    if w.root
        .join(format!("{prefix}.native-started.json"))
        .exists()
    {
        return Err("the single corrected logging guest attempt already started; reconcile its actual result instead of repeating it".into());
    }
    let package = w.root.join("package-v3");
    let normal = resources::Resources::open(&package)?;
    let proof = qualified(w, &normal)?;
    let partial: Value = c::read_json(&proof.join("partial-native-load.json"))?;
    if partial["authoritative_disk_eeprom_checkpoint_pointer_generation_unchanged"] != true
        || partial["native_return_clean"] != true
        || !fs::read_dir(&w.root)?
            .filter_map(Result::ok)
            .any(|e| e.path().join("policy.qualified").is_file())
    {
        return Err("complete original continuity/storage-safety slice required".into());
    }
    let project = w.root.parent().ok_or("project")?;
    let (logging, logging_pin, qualification) = if io_errors {
        (
            w.root.join("io-errors-v1/build-logging/bios.bin"),
            resources::IO_ERRORS_LOGGING_OPEN,
            w.root.join("io-errors-v1/build.identity.json"),
        )
    } else if probe {
        (
            w.root.join("buffer-probe-v1/build-logging/bios.bin"),
            resources::BUFFER_PROBE_LOGGING_OPEN,
            w.root.join("buffer-probe-v1/build.identity.json"),
        )
    } else if flush {
        (
            w.root.join("flush-io-v1/build-logging/bios.bin"),
            resources::FLUSH_LOGGING_OPEN,
            w.root.join("flush-io-v1/build.identity.json"),
        )
    } else if npx {
        let port = w.root.join("npx-owner-port-v1");
        if !fs::read_to_string(port.join("regression.qualified"))?.contains("current release red;")
        {
            return Err("functional-port original regression required".into());
        }
        (
            port.join("build-logging/bios.bin"),
            resources::NPX_LOGGING_OPEN,
            if x87 {
                w.root.join("x87-boundary-v1/relaunch.qualified")
            } else {
                port.join("relaunch.qualified")
            },
        )
    } else {
        (
            project.join("open-startup-v1/build-basic-v3/bios.bin"),
            resources::LOGGING_OPEN,
            project.join("open-startup-v1/fixture-v3.qualified"),
        )
    };
    if x87
        && !functional
        && !fs::read_to_string(&qualification)?.contains(&format!("core_sha256={core}\n"))
    {
        return Err("exact corrected-core relaunch qualification required".into());
    }
    if c::file_digest(&logging)? != logging_pin
        || (!functional
            && !fs::read_to_string(qualification)?
                .contains(&format!("bios_sha256={logging_pin}\n")))
    {
        return Err(
            "this exact corrected logging input needs retained-fixture qualification".into(),
        );
    }
    if !w.root.join(format!("{prefix}.started.json")).exists() {
        c::write_new(
            &w.root.join(format!("{prefix}.started.json")),
            &c::json(&json!({
            "source":"Master6399143636 after XBX-CARTRIDGE-1 original proof",
            "core":core,"open_system":logging_pin,"accepted_npx_functional_port":npx,
            "ordinary_product_policy":true,"memory_mib":64,"network":"none",
            "fp":"native","graphics":"Intel OpenGL","audio":"dummy_unqualified"
            }))?,
        )?;
    }
    let dev = if io_errors {
        super::buffer_probe::io_package(&w.root)?.root
    } else if probe {
        super::buffer_probe::package(&w.root)?.root
    } else if flush {
        super::flush_io::package(&w.root)?.root
    } else if x87 {
        super::x87_boundary::package(repo, &w.root)?.root
    } else {
        w.root.join(if npx {
            "package-npx-logging-v1"
        } else {
            "package-logging-v1"
        })
    };
    if !dev.exists() {
        c::mkdir(&dev)?;
        fs::create_dir_all(dev.join("native/subprojects/slirp"))?;
        for name in [
            "native/core",
            "native/qemu-img",
            "native/subprojects/slirp/libslirp.so.0",
        ] {
            fs::hard_link(package.join(name), dev.join(name))?;
        }
        fs::copy(logging, dev.join("open.bin"))?;
        let mut manifest = normal.manifest.clone();
        manifest.open_system = logging_pin.into();
        manifest.core = core.into();
        c::write_new(&dev.join("runtime.json"), &c::json(&manifest)?)?;
    }
    let r = if io_errors {
        resources::Resources::io_errors_logging(&dev)?
    } else if probe {
        resources::Resources::buffer_probe_logging(&dev)?
    } else if flush {
        resources::Resources::flush_logging(&dev)?
    } else if x87 {
        resources::Resources::x87_logging(&dev)?
    } else if npx {
        resources::Resources::npx_logging(&dev)?
    } else {
        resources::Resources::logging(&dev)?
    };
    observe_logging_fable(repo, &w.root, r, prefix, npx, x87, io_errors)
}

pub(super) fn observe_logging_fable(
    repo: &Path,
    root: &Path,
    r: resources::Resources,
    prefix: &str,
    npx: bool,
    x87: bool,
    io_errors: bool,
) -> Result<(), Error> {
    observe_fable(
        repo,
        root,
        r,
        prefix,
        FableObservation {
            matched_media: npx,
            checkpoint_on_budget: x87 && !io_errors,
            io_errors,
            interactive: false,
            existing_app: None,
            restore_existing: false,
        },
    )
}

pub(super) fn observe_local_play(
    repo: &Path,
    root: &Path,
    r: resources::Resources,
    prefix: &str,
    existing_app: Option<PathBuf>,
    restore_existing: bool,
) -> Result<(), Error> {
    observe_fable(
        repo,
        root,
        r,
        prefix,
        FableObservation {
            matched_media: true,
            checkpoint_on_budget: true,
            io_errors: false,
            interactive: true,
            existing_app,
            restore_existing,
        },
    )
}

struct FableObservation {
    matched_media: bool,
    checkpoint_on_budget: bool,
    io_errors: bool,
    interactive: bool,
    existing_app: Option<PathBuf>,
    restore_existing: bool,
}

fn observe_fable(
    repo: &Path,
    root: &Path,
    r: resources::Resources,
    prefix: &str,
    recipe: FableObservation,
) -> Result<(), Error> {
    let w = BuildPaths { root: root.into() };
    let continuing = recipe.existing_app.is_some();
    let app = match &recipe.existing_app {
        Some(app) => app.clone(),
        None => w.root.join(format!("{prefix}-{}", c::new_id()?)),
    };
    let mut player = c::desktop::Player::with_resources(&app, r)?;
    let media = std::path::PathBuf::from(std::env::var("XBX_REFERENCE_MEDIA")?);
    let admission_started = Instant::now();
    let game = if continuing {
        player.last_game()?.ok_or("existing ordinary game target")?
    } else {
        player.add(&media)?
    };
    println!(
        "{}",
        json!({"stage":if continuing {"existing_collection_target"} else {"game_imported_without_copy"},"elapsed_ms":admission_started.elapsed().as_millis()})
    );
    let entry = c::library::Library::read(&app)?.resolve(&game)?;
    if recipe.matched_media && entry.media_sha256 != std::env::var("XBX_REFERENCE_MEDIA_SHA256")? {
        return Err("matched reference media identity differs; no native launch".into());
    }
    // The compatibility comparison retains its previously qualified seeds. This
    // is a fresh developer-control profile, never a firmware/storage migration.
    // The ordinary product path still generates its own empty storage.
    let disk = repo.join(".local/dashboard-v20260516-0955/xbox_hdd.qcow2");
    if c::file_digest(&disk)? != "00d7df7a2bc235f8801764f00b7f40e194d1e392f7a9619d6b2396c89770f6dd"
    {
        return Err("matched storage seed differs".into());
    }
    let eeprom = repo.join(".local/quick-reboot-d87cfc33-run1/eeprom.bin");
    let seed_before = (c::file_digest(&disk)?, c::file_digest(&eeprom)?);
    if !continuing {
        let _profile =
            c::store::Profile::open_seeded(&app, &entry, &player.resources, &disk, &eeprom)?;
    }
    let media_before = fs::metadata(&media)?;
    c::write_new(
        &w.root.join(format!("{prefix}.native-started.json")),
        &c::json(
            &json!({"app":app,"same_seed":true,"media_import_ms":admission_started.elapsed().as_millis()}),
        )?,
    )?;
    if continuing && !recipe.restore_existing {
        // The finite prior owner explicitly stopped without a checkpoint. Use
        // the product's explicit restart against current storage, never reset
        // that storage to its original seed or silently bypass dirty recovery.
        player.restart(&game)?;
    }
    let restored = if recipe.restore_existing {
        let restored = player.launch_quiesced(&game)?;
        if !restored {
            player.stop()?;
            return Err("a validated complete checkpoint is required for return".into());
        }
        if !player.live.as_ref().ok_or("restored live session")?.paused {
            player.resume()?;
        }
        true
    } else {
        player.launch(&game)?
    };
    let run = player.live.as_ref().ok_or("live")?.native.run.clone();
    println!(
        "{}",
        json!({"product_app":app,"fresh_app":!continuing,"native_run":run,"same_seed":true,"restored":restored,"controller_attachment":"verified_native_inventory"})
    );
    let observation_dir = if continuing { &run } else { &app };
    let mut log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(observation_dir.join("kernel.log"))?;
    let started = Instant::now();
    let mut text = String::new();
    let mut events = Vec::new();
    let mut io_messages = Vec::new();
    let mut io_pending = String::new();
    let mut next_capture = 0;
    let mut captures = vec![
        3, 10, 25, 75, 115, 145, 180, 240, 300, 330, 360, 390, 420, 480, 510, 550, 585,
    ];
    let maximum_seconds = if recipe.interactive { 3600 } else { 600 };
    if recipe.interactive {
        captures.retain(|second| *second < 300);
        captures.extend((300..3600).step_by(30));
    }
    let mut next_input = 0;
    let mut inputs = vec![
        (315, "Return"),
        (345, "Down"),
        (375, "Up"),
        (405, "a"),
        (495, "Return"),
    ];
    if recipe.interactive {
        inputs.clear();
    }
    // One admitted finite operator session can send normal commands without
    // launching another owner or waiting for per-key host admission. The task
    // owner remains responsible for native cleanup after client EOF/timeouts.
    let (operator_tx, operator_rx) = std::sync::mpsc::sync_channel(4);
    if recipe.interactive {
        std::thread::spawn(move || {
            for line in std::io::stdin().lock().lines().take(100) {
                let Ok(line) = line else { break };
                if line.len() > 4096 || operator_tx.send(line).is_err() {
                    break;
                }
            }
        });
    }
    let observe = (|| -> Result<&str, Error> {
        loop {
            let mut operator_capture = false;
            if let Ok(line) = operator_rx.try_recv() {
                let command: Value = serde_json::from_str(&line)?;
                match command["op"].as_str() {
                    Some("capture") => operator_capture = true,
                    Some("depart") => return Ok("operator_requested_product_departure"),
                    Some("stop") => return Ok("operator_requested_stop"),
                    Some("pause") => player.pause(true)?,
                    Some("resume") => player.resume()?,
                    Some("key") => {
                        let key = command["key"].as_str().ok_or("normal controller key")?;
                        if ![
                            "Return",
                            "BackSpace",
                            "Up",
                            "Down",
                            "Left",
                            "Right",
                            "a",
                            "b",
                            "x",
                            "y",
                            "w",
                            "s",
                            "d",
                            "q",
                            "e",
                            "i",
                            "j",
                            "k",
                            "l",
                        ]
                        .contains(&key)
                        {
                            return Err("unsupported normal controller key".into());
                        }
                        let result = Command::new("python3")
                            .arg(repo.join(".local/ramht-window-input-v2.py"))
                            .arg(&run)
                            .arg(player.resources.core())
                            .arg(key)
                            .output()?;
                        let event = json!({"elapsed_ms":started.elapsed().as_millis(),"normal_key":key,"operator_exit":result.status.code(),"visible_response_unqualified":true});
                        println!("{event}");
                        events.push(event);
                        operator_capture = true;
                    }
                    _ => return Err("unsupported finite operator command".into()),
                }
            }
            let native = &mut player.live.as_mut().ok_or("live")?.native;
            let new = native.serial()?;
            log.write_all(new.as_bytes())?;
            text.push_str(&new);
            if text.len() > 1024 * 1024 {
                return Err("ordinary output bound reached".into());
            }
            if recipe.io_errors {
                io_pending.push_str(&new);
                while let Some(end) = io_pending.find('\n') {
                    let line: String = io_pending.drain(..=end).collect();
                    if line.starts_with("xbxio: ") && io_messages.len() < 99 {
                        io_messages.push(json!({"host_received_ms":started.elapsed().as_millis(),"message":line.trim_end()}));
                    }
                }
            }
            let terminal =
                text.contains("*** bug check") || text.contains("*** unexpected exception ");
            let elapsed = started.elapsed().as_secs();
            let scheduled_capture =
                next_capture < captures.len() && elapsed >= captures[next_capture];
            if terminal || operator_capture || scheduled_capture {
                let result = Command::new("python3")
                    .arg(repo.join(".local/capture-ramht-window-v2.py"))
                    .arg(&run)
                    .arg(player.resources.core())
                    .output()?;
                events.push(json!({"elapsed_ms":started.elapsed().as_millis(),"capture_exit":result.status.code(),"output":String::from_utf8_lossy(&result.stdout),"error":String::from_utf8_lossy(&result.stderr),"visible_review_required":true}));
                if operator_capture {
                    println!(
                        "{}",
                        json!({"capture":String::from_utf8_lossy(&result.stdout).trim(),"exit":result.status.code(),"visible_review_required":true})
                    );
                }
                next_capture += usize::from(scheduled_capture);
            }
            if terminal {
                return Ok("guest_terminal_startup_failure");
            }
            if native.exited()? {
                return Ok("native_exited_during_observation");
            }
            if next_input < inputs.len() && elapsed >= inputs[next_input].0 {
                let key = inputs[next_input].1;
                let result = Command::new("python3")
                    .arg(repo.join(".local/ramht-window-input-v1.py"))
                    .arg(&run)
                    .arg(player.resources.core())
                    .arg(key)
                    .output()?;
                events.push(json!({"elapsed_ms":started.elapsed().as_millis(),"normal_key":key,"operator_exit":result.status.code(),"visible_response_unqualified":true}));
                next_input += 1;
            }
            // Private finite-task operator request after actual visible review.
            // The product owner still performs the complete durable departure.
            if recipe.interactive && observation_dir.join("operator-depart").is_file() {
                return Ok("operator_requested_product_departure");
            }
            if elapsed >= maximum_seconds {
                return Ok("observation_budget_expired_requires_visual_review");
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    })();
    log.sync_all()?;
    let reason = match observe {
        Ok(v) => v.to_owned(),
        Err(e) => format!("observation_error:{e}"),
    };
    // An observed terminal machine is retained as diagnostic evidence, not
    // published as a healthy automatic-return checkpoint.
    // Reporting stops with ordinary storage retained. Ordinary interactive
    // expiry uses product departure to preserve unsaved state. Publication
    // itself does not establish a healthy gameplay/continuity milestone.
    let continuation_checkpoint = reason == "operator_requested_product_departure"
        || recipe.checkpoint_on_budget
            && reason == "observation_budget_expired_requires_visual_review";
    let returned = if continuation_checkpoint {
        player.depart()
    } else {
        player.stop()
    };
    // Product departure deliberately keeps the only live session when saving
    // fails. This finite qualification owner must explicitly retire its failed
    // attempt before returning an error, without clearing current saves/state.
    let failed_operation_cleanup = if returned.is_err() {
        retire_failed_observation(&mut player)
    } else {
        None
    };
    let media_after = fs::metadata(&media)?;
    let immutable_inputs_unchanged = seed_before
        == (c::file_digest(&disk)?, c::file_digest(&eeprom)?)
        && media_before.len() == media_after.len()
        && media_before.modified()? == media_after.modified()?;
    c::write_new(
        &observation_dir.join("observation.json"),
        &c::json(&json!({
            "reason":reason,"elapsed_ms":started.elapsed().as_millis(),"maximum_seconds":maximum_seconds,
            "native_run":run,"events":events,"normal_retirement":returned.is_ok(),
            "return_error":returned.as_ref().err().map(ToString::to_string),"continuation_checkpoint_published":continuation_checkpoint && returned.is_ok(),"immutable_inputs_unchanged":immutable_inputs_unchanged,
            "failed_operation_cleanup":failed_operation_cleanup,
            "public_io_error_diagnostics_only":recipe.io_errors,
            "interactive_visible_review_recipe":recipe.interactive,
            "explicit_restart_of_preserved_profile":continuing && !recipe.restore_existing,
            "complete_checkpoint_restored":restored,
            "public_io_failure_messages":io_messages,
            "fable_menu_controls_gameplay_sound_save_resume_qualified":false,"title_memory_or_instruction_inspection":false
        }))?,
    )?;
    println!(
        "{}",
        json!({"result":observation_dir.join("observation.json"),"reason":reason,"normal_retirement":returned.is_ok()})
    );
    returned?;
    if !immutable_inputs_unchanged {
        return Err("immutable inputs changed".into());
    }
    Ok(())
}
pub(super) fn retire_failed_observation(player: &mut c::desktop::Player) -> Option<Value> {
    player
        .live
        .as_mut()
        .map(|live| match c::player::Engine::retire(&mut live.native) {
            Ok(clean) => json!({"attempted":true,"native_clean":clean}),
            Err(e) => json!({"attempted":true,"error":e.to_string()}),
        })
}
pub(super) fn section_digest(bytes: &[u8]) -> Result<[u8; 20], Error> {
    use std::process::Stdio;
    let mut child = Command::new("sha1sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut input = child.stdin.take().ok_or("digest stdin")?;
    input.write_all(&(bytes.len() as u32).to_le_bytes())?;
    input.write_all(bytes)?;
    drop(input);
    let out = child.wait_with_output()?;
    if !out.status.success() {
        return Err("digest failed".into());
    }
    let s = String::from_utf8(out.stdout)?;
    let mut result = [0; 20];
    for (i, b) in result.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)?
    }
    Ok(result)
}

struct Transport {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}
impl Transport {
    fn spawn(app: &Path, package: &Path) -> Result<Self, Error> {
        let binary = std::env::current_exe()?.with_file_name("xbx-player");
        let mut child = Command::new(binary)
            .arg(app)
            .arg(package)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        Ok(Self {
            input: child.stdin.take().ok_or("stdin")?,
            output: BufReader::new(child.stdout.take().ok_or("stdout")?),
            child,
        })
    }
    fn send(&mut self, v: Value) -> Result<Value, Error> {
        let started = Instant::now();
        writeln!(self.input, "{v}")?;
        self.input.flush()?;
        let mut line = String::new();
        self.output.read_line(&mut line)?;
        let response: Value = serde_json::from_str(&line)?;
        println!(
            "{}",
            json!({"request":v,"response":response,"elapsed_ms":started.elapsed().as_secs_f64()*1000.0,"host_pid":self.child.id(),"memory":memory(self.child.id())})
        );
        if response.get("error").is_some() {
            return Err(format!("operation refused: {response}").into());
        }
        Ok(response["ok"].clone())
    }
    fn refusal(&mut self, v: Value) -> Result<String, Error> {
        writeln!(self.input, "{v}")?;
        self.input.flush()?;
        let mut line = String::new();
        self.output.read_line(&mut line)?;
        let response: Value = serde_json::from_str(&line)?;
        println!(
            "{}",
            json!({"request":v,"expected_refusal":response,"host_pid":self.child.id(),"memory":memory(self.child.id())})
        );
        response["error"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "expected operation was not refused".into())
    }
    fn wait(&mut self, expected: &str) -> Result<String, Error> {
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut output = String::new();
        while Instant::now() < deadline {
            let v = self.send(json!({"op":"serial"}))?;
            output.push_str(v["text"].as_str().unwrap_or(""));
            if output.contains(expected) {
                return Ok(output);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Err(format!("guest completion missing: {expected}; got {output}").into())
    }
    fn quit(mut self) -> Result<(), Error> {
        self.send(json!({"op":"quit"}))?;
        if !self.child.wait()?.success() {
            return Err("player did not retire".into());
        }
        Ok(())
    }
}
impl Drop for Transport {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn proof(w: &BuildPaths) -> Result<(), Error> {
    let app = w.root.join(format!("proof-{}", c::new_id()?));
    let package = w.root.join("package-v3");
    println!(
        "{}",
        json!({"fresh_app":app,"packaged_runtime":package,"original_program":true,"phone":false})
    );
    let mut first = Transport::spawn(&app, &package)?;
    let a = first.send(json!({"op":"add","media":w.root.join("fixtures-v2/a.iso")}))?["game"]
        .as_str()
        .ok_or("game")?
        .to_owned();
    let b = first.send(json!({"op":"add","media":w.root.join("fixtures-v2/b.iso")}))?["game"]
        .as_str()
        .ok_or("game")?
        .to_owned();
    first.send(json!({"op":"open","game":a}))?;
    first.wait("CART A state=00000000 save=00000000")?;
    first.send(json!({"op":"fixture-input","text":"as a"}))?;
    first.wait("CART A state=00000002 save=00000001 cookie=57F090E4")?;
    first.send(json!({"op":"commit"}))?;
    first.quit()?;
    let mut second = Transport::spawn(&app, &package)?;
    if second.send(json!({"op":"open","game":a}))?["restored"] != true {
        return Err("fresh process did not restore".into());
    }
    second.send(json!({"op":"fixture-input","text":"?"}))?;
    second.wait("CART A state=00000002 save=00000001 cookie=57F090E4")?;
    second.send(json!({"op":"fixture-input","text":"a"}))?;
    second.wait("CART A state=00000003 save=00000001 cookie=6B482CC3")?;
    second.send(json!({"op":"commit"}))?;
    second.quit()?;
    let mut third = Transport::spawn(&app, &package)?;
    third.send(json!({"op":"open","game":a}))?;
    third.send(json!({"op":"fixture-input","text":"?"}))?;
    third.wait("CART A state=00000003 save=00000001 cookie=6B482CC3")?;
    third.send(json!({"op":"open","game":b}))?;
    third.wait("CART B state=00000000 save=00000000")?;
    third.send(json!({"op":"fixture-input","text":"a"}))?;
    third.wait("CART B state=00000001 save=00000000")?;
    third.send(json!({"op":"open","game":a}))?;
    third.send(json!({"op":"fixture-input","text":"?"}))?;
    third.wait("CART A state=00000003 save=00000001 cookie=6B482CC3")?;
    third.send(json!({"op":"fixture-input","text":"f"}))?;
    third.wait("CART COMPLETE")?;
    third.quit()?;
    c::write_new(
        &app.join("runtime.qualified"),
        resources::Resources::open(&package)?.identity.as_bytes(),
    )?;
    c::write_new(&app.join("continuity.qualified"),b"original A unsaved2/save1 fresh-process restore; action3; second fresh restore3; A-B-A; real native snapshots; original UART input only\n")?;
    Ok(())
}

fn native(repo: &Path, w: &BuildPaths) -> Result<(), Error> {
    use serde_json::{Value, json};
    let project = w.root.parent().ok_or("project")?;
    let old = project.join("ramht-v1/build-v2-retry2");
    let src = project.join("ramht-v1/source-v2");
    let new = w.root.join("native-v2");
    c::mkdir(&new)?;
    c::mkdir(&new.join("ui"))?;
    if c::file_digest(&old.join("qemu-system-i386"))? != resources::REFERENCE_CORE {
        return Err("retained core differs".into());
    }
    fs::copy(src.join("ui/xemu.c"), new.join("ui/xemu.c"))?;
    let patch = repo.join("patches/xemu-headless-session.patch");
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&new)
            .args(["apply", "--unsafe-paths"])
            .arg(&patch),
    )?;
    let commands: Vec<Value> =
        serde_json::from_slice(&fs::read(old.join("compile_commands.json"))?)?;
    let cc = commands
        .iter()
        .find(|v| v["output"] == "libsystem.a.p/ui_xemu.c.o")
        .ok_or("compile command missing")?["command"]
        .as_str()
        .ok_or("command missing")?;
    let object = new.join("ui.o");
    let dep = new.join("ui.o.d");
    let cc = cc
        .replace("libsystem.a.p/ui_xemu.c.o.d", dep.to_str().unwrap())
        .replace("libsystem.a.p/ui_xemu.c.o", object.to_str().unwrap())
        .replace(
            "../source-v2/ui/xemu.c",
            new.join("ui/xemu.c").to_str().unwrap(),
        );
    let cc = format!("{cc} -I{}", src.join("ui").display());
    let ninja = fs::read_to_string(old.join("build.ninja"))?;
    let edge = ninja
        .split("\n\n")
        .find(|s| s.starts_with("build qemu-system-i386:"))
        .ok_or("link edge missing")?;
    let input = edge
        .lines()
        .next()
        .ok_or("link inputs missing")?
        .strip_prefix("build qemu-system-i386: cpp_LINKER_RSP ")
        .ok_or("unexpected link rule")?
        .split(" | ")
        .next()
        .ok_or("inputs")?;
    let input = input.replace("libsystem.a.p/ui_xemu.c.o", object.to_str().unwrap());
    let link = edge
        .lines()
        .find_map(|s| s.strip_prefix(" LINK_ARGS = "))
        .ok_or("link args missing")?;
    let link = link.replace(
        &format!(":{}", repo.join(".local/host-sysroot/usr/lib64").display()),
        "",
    );
    let build = format!(
        "builddir = {}\nrule compile_ui\n command = {cc}\nrule link_core\n command = c++ -m64 @$out.rsp\n rspfile = $out.rsp\n rspfile_content = -o $out $in {link}\nbuild {}: compile_ui {}\nbuild {}: link_core {input}\n",
        new.display(),
        object.display(),
        new.join("ui/xemu.c").display(),
        new.join("core").display()
    );
    c::write_new(&new.join("build.ninja"), build.as_bytes())?;
    c::store::checked(
        Command::new("ninja")
            .current_dir(&old)
            .arg("-f")
            .arg(new.join("build.ninja"))
            .args(["-j", "1"])
            .arg(new.join("core")),
    )?;
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(
            &json!({"base_core_sha256":resources::REFERENCE_CORE,"base_source_pin":"03b988c9d76553b33ed712ba00822ffdf49f0e7d","prior_base_and_RAMHT_patches_retained":true,"patch_sha256":c::file_digest(&patch)?,"retained_compile_commands_sha256":c::file_digest(&old.join("compile_commands.json"))?,"retained_ninja_sha256":c::file_digest(&old.join("build.ninja"))?,"modified_source_sha256":c::file_digest(&new.join("ui/xemu.c"))?,"core_sha256":c::file_digest(&new.join("core"))?,"recompiled_units":["ui/xemu.c"],"reused_other_objects":true,"old_build_mutated":false,"jobs":1}),
        )?,
    )?;
    Ok(())
}

fn partial(w: &BuildPaths) -> Result<(), Error> {
    let r = resources::Resources::open(&w.root.join("package-v3"))?;
    let app = qualified(w, &r)?;
    let library = c::library::Library::read(&app)?;
    let entry = library.entries.first().ok_or("original entry")?;
    let receipt = c::desktop::original_partial_load_test(&app, &r, entry)?;
    c::write_new(&app.join("partial-native-load.json"), &c::json(&receipt)?)?;
    println!("{}", serde_json::to_string(&receipt)?);
    Ok(())
}

fn memory(parent: u32) -> Vec<Value> {
    let mut pids = vec![parent];
    if let Ok(children) = fs::read_to_string(format!("/proc/{parent}/task/{parent}/children")) {
        pids.extend(
            children
                .split_whitespace()
                .filter_map(|s| s.parse::<u32>().ok()),
        );
    }
    pids.into_iter()
        .filter_map(|pid| {
            let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
            let fields: serde_json::Map<String, Value> = status
                .lines()
                .filter(|s| {
                    s.starts_with("VmHWM:") || s.starts_with("VmRSS:") || s.starts_with("VmPeak:")
                })
                .filter_map(|s| {
                    let (k, v) = s.split_once(':')?;
                    Some((
                        format!("{k}_kib"),
                        json!(v.split_whitespace().next()?.parse::<u64>().ok()?),
                    ))
                })
                .collect();
            Some(json!({"ephemeral_pid":pid,"readings":fields}))
        })
        .collect()
}
fn qualified(w: &BuildPaths, r: &resources::Resources) -> Result<std::path::PathBuf, Error> {
    fs::read_dir(&w.root)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.join("continuity.qualified").is_file()
                && fs::read_to_string(p.join("runtime.qualified")).is_ok_and(|id| id == r.identity)
        })
        .ok_or_else(|| "no completed original continuity proof for this exact runtime".into())
}

fn clone_original(w: &BuildPaths, r: &resources::Resources) -> Result<std::path::PathBuf, Error> {
    let source = qualified(w, r)?;
    let _app = crate::storage::ProfileLease::acquire(&source)?;
    let library = c::library::Library::read(&source)?;
    let dest = w.root.join(format!("policy-{}", c::new_id()?));
    c::mkdir(&dest)?;
    c::mkdir(&dest.join("profiles"))?;
    fs::copy(source.join("library.json"), dest.join("library.json"))?;
    for entry in &library.entries {
        if !resources::ORIGINAL_FIXTURES.contains(&entry.media_sha256.as_str()) {
            return Err("original fixtures only".into());
        }
        let p = c::store::Profile::open(&source, entry, r)?;
        let v = c::checkpoint::load(
            &p,
            r,
            &c::Binding {
                game: entry.id.clone(),
                profile: entry.profile.clone(),
                media: entry.media_sha256.clone(),
                partition: entry.partition,
                runtime: r.identity.clone(),
                state_abi: 1,
            },
        )?
        .ok_or("original checkpoint")?;
        let target = dest.join("profiles").join(&entry.profile);
        c::mkdir(&target)?;
        c::mkdir(&target.join("storage"))?;
        c::mkdir(&target.join("checkpoints"))?;
        for name in ["storage.json", "session.json", "generation"] {
            fs::copy(p.root.join(name), target.join(name))?;
        }
        let current = p.current()?;
        let disk = target
            .join("storage")
            .join(current.file_name().ok_or("storage")?);
        c::mkdir(&disk)?;
        c::store::copy_machine(&current, &disk)?;
        let checkpoint = target.join("checkpoints").join(&v.id);
        c::mkdir(&checkpoint)?;
        c::store::copy_machine(&v.directory, &checkpoint)?;
        fs::copy(
            v.directory.join("record.json"),
            checkpoint.join("record.json"),
        )?;
    }
    Ok(dest)
}
fn policy_proof(w: &BuildPaths) -> Result<(), Error> {
    let package = w.root.join("package-v3");
    let r = resources::Resources::open(&package)?;
    let app = clone_original(w, &r)?;
    let library = c::library::Library::read(&app)?;
    let a = library.entries.first().ok_or("game")?.id.clone();
    let profile = app.join("profiles").join(&library.entries[0].profile);
    println!(
        "{}",
        json!({"isolated_synthetic_app":app,"source_preserved":qualified(w,&r)?,"shared_policy":true})
    );
    let mut first = Transport::spawn(&app, &package)?;
    first.send(json!({"op":"open","game":a}))?;
    first.send(json!({"op":"fixture-input","text":"?"}))?;
    first.wait("CART A state=00000003 save=00000001 cookie=6B482CC3")?;
    first.send(json!({"op":"pause"}))?;
    first.quit()?;
    let mut second = Transport::spawn(&app, &package)?;
    second.send(json!({"op":"return"}))?;
    second.send(json!({"op":"fixture-input","text":"?"}))?;
    if second.send(json!({"op":"serial"}))?["text"]
        .as_str()
        .is_some_and(|s| !s.is_empty())
    {
        return Err("explicit Pause did not remain paused".into());
    }
    second.send(json!({"op":"resume"}))?;
    second.wait("CART A state=00000003 save=00000001 cookie=6B482CC3")?;
    second.send(json!({"op":"fixture-input","text":"a"}))?;
    second.wait("CART A state=00000004 save=00000001 cookie=7E9FC8A2")?;
    second.send(json!({"op":"commit"}))?;
    let old = fs::read(profile.join("session.json"))?;
    let session: c::checkpoint::Session = serde_json::from_slice(&old)?;
    let state = profile
        .join("checkpoints")
        .join(&session.checkpoint)
        .join("hdd.qcow2");
    let state_hash = c::file_digest(&state)?;
    second.send(json!({"op":"resume"}))?;
    second.send(json!({"op":"fixture-input","text":"s"}))?;
    second.wait("CART A state=00000004 save=00000004 cookie=7E9FC8A2")?;
    second.send(json!({"op":"stop"}))?;
    second.quit()?;
    let (_global, lock) = (
        crate::storage::ProfileLease::acquire(&app)?,
        crate::storage::ProfileLease::acquire(&profile)?,
    );
    c::atomic(&profile.join("session.json"), &old)?;
    let current: c::store::Current = c::read_json(&profile.join("storage.json"))?;
    let current = profile.join("storage").join(current.directory);
    let before = (
        c::file_digest(&current.join("hdd.qcow2"))?,
        c::file_digest(&current.join("eeprom.bin"))?,
        lock.generation()?,
        c::file_digest(&state)?,
        fs::read(profile.join("session.json"))?,
    );
    drop(lock);
    drop(_global);
    let mut third = Transport::spawn(&app, &package)?;
    let refusal = third.refusal(json!({"op":"open","game":a}))?;
    if !refusal.contains("generation") {
        return Err(format!("wrong refusal: {refusal}").into());
    }
    third.quit()?;
    let lock = crate::storage::ProfileLease::acquire(&profile)?;
    let after = (
        c::file_digest(&current.join("hdd.qcow2"))?,
        c::file_digest(&current.join("eeprom.bin"))?,
        lock.generation()?,
        c::file_digest(&state)?,
        fs::read(profile.join("session.json"))?,
    );
    drop(lock);
    if before != after || c::file_digest(&state)? != state_hash {
        return Err("newer ordinary save or rejected checkpoint was changed".into());
    }
    let mut fourth = Transport::spawn(&app, &package)?;
    fourth.send(json!({"op":"restart","game":a}))?;
    fourth.send(json!({"op":"open","game":a}))?;
    fourth.wait("CART A state=00000000 save=00000004")?;
    fourth.send(json!({"op":"fixture-input","text":"a"}))?;
    fourth.wait("CART A state=00000001 save=00000004")?;
    fourth.quit()?;
    if c::file_digest(&state)? != state_hash {
        return Err("explicit restart changed rejected checkpoint".into());
    }
    c::write_new(&app.join("policy.qualified"),b"shared policy actual fresh-process explicit Pause; normal Resume; newer guest disk save4 rejects old checkpoint with both unchanged; explicit restart cold-loads current save4; native returns clean\n")?;
    Ok(())
}

fn paused_contract(repo: &Path, w: &BuildPaths) -> Result<(), Error> {
    let root = w.root.join("paused-capture-contract-v2");
    c::mkdir(&root)?;
    let source = w
        .root
        .parent()
        .ok_or("project")?
        .join("ramht-v1/source-v2/migration/migration-hmp-cmds.c");
    let text = fs::read_to_string(&source)?;
    for (label, corrected) in [("before", false), ("after", true)] {
        let dir = root.join(label);
        c::mkdir(&dir)?;
        c::mkdir(&dir.join("migration"))?;
        fs::copy(&source, dir.join("migration/migration-hmp-cmds.c"))?;
        if corrected {
            c::store::checked(
                Command::new("git")
                    .arg("-C")
                    .arg(&dir)
                    .args(["apply", "--unsafe-paths"])
                    .arg(repo.join("patches/xemu-paused-savevm.patch")),
            )?;
        }
        let actual = fs::read_to_string(dir.join("migration/migration-hmp-cmds.c"))?;
        let start = actual
            .find("void hmp_savevm(")
            .ok_or("capture function missing")?;
        let end = start
            + actual[start..]
                .find("\nvoid hmp_delvm")
                .ok_or("capture function extent")?;
        c::write_new(&dir.join("subject.inc"), &actual.as_bytes()[start..end])?;
        fs::copy(
            repo.join("fixtures/paused_capture_shim.c"),
            dir.join("shim.c"),
        )?;
        fs::copy(
            repo.join("fixtures/paused_capture_contract.rs"),
            dir.join("contract.rs"),
        )?;
        c::store::checked(
            Command::new("cc")
                .args([
                    "-std=c11", "-O2", "-fPIC", "-Wall", "-Wextra", "-Werror", "-c",
                ])
                .arg(dir.join("shim.c"))
                .arg("-o")
                .arg(dir.join("subject.o")),
        )?;
        c::store::checked(
            Command::new("rustc")
                .args(["--edition=2024", "--test", "-Dwarnings"])
                .arg(dir.join("contract.rs"))
                .arg("-C")
                .arg(format!("link-arg={}", dir.join("subject.o").display()))
                .arg("-o")
                .arg(dir.join("test")),
        )?;
        let output = Command::new(dir.join("test"))
            .arg("--test-threads=1")
            .output()?;
        c::write_new(
            &dir.join("test.log"),
            &[output.stdout.as_slice(), output.stderr.as_slice()].concat(),
        )?;
        if output.status.success() != corrected {
            return Err(
                format!("unexpected capture contract {label} result; evidence retained").into(),
            );
        }
    }
    c::write_new(
        &root.join("result.json"),
        &c::json(
            &json!({"public_source_sha256":c::file_digest(&source)?,"public_source_file_bytes":text.len(),"patch_sha256":c::file_digest(&repo.join("patches/xemu-paused-savevm.patch"))?,"before_expected_failure":true,"after_pass":true,"native_or_game_input":false,"scope":"actual HMP wrapper with original controlled runstate/device-preparation/error dependencies, not native serialization or vCPU concurrency"}),
        )?,
    )?;
    Ok(())
}

fn native_paused(repo: &Path, w: &BuildPaths) -> Result<(), Error> {
    let project = w.root.parent().ok_or("project")?;
    let old = project.join("ramht-v1/build-v2-retry2");
    let src = project.join("ramht-v1/source-v2");
    let new = w.root.join("native-v3");
    let contract = w.root.join("paused-capture-contract-v2/result.json");
    let result: Value = c::read_json(&contract)?;
    if result["after_pass"] != true
        || result["patch_sha256"] != c::file_digest(&repo.join("patches/xemu-paused-savevm.patch"))?
    {
        return Err("original paused-capture regression gate missing".into());
    }
    c::mkdir(&new)?;
    c::mkdir(&new.join("migration"))?;
    fs::copy(
        src.join("migration/migration-hmp-cmds.c"),
        new.join("migration/migration-hmp-cmds.c"),
    )?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&new)
            .args(["apply", "--unsafe-paths"])
            .arg(repo.join("patches/xemu-paused-savevm.patch")),
    )?;
    let commands: Vec<Value> =
        serde_json::from_slice(&fs::read(old.join("compile_commands.json"))?)?;
    let subject = "libsystem.a.p/migration_migration-hmp-cmds.c.o";
    let command = commands
        .iter()
        .find(|v| v["output"] == subject)
        .ok_or("HMP compile command")?["command"]
        .as_str()
        .ok_or("compile command")?;
    let object = new.join("hmp.o");
    let dep = new.join("hmp.o.d");
    let command = command
        .replace(&format!("{subject}.d"), dep.to_str().unwrap())
        .replace(subject, object.to_str().unwrap())
        .replace(
            "../source-v2/migration/migration-hmp-cmds.c",
            new.join("migration/migration-hmp-cmds.c").to_str().unwrap(),
        );
    let command = format!("{command} -I{}", src.join("migration").display());
    let ninja = fs::read_to_string(old.join("build.ninja"))?;
    let edge = ninja
        .split("\n\n")
        .find(|s| s.starts_with("build qemu-system-i386:"))
        .ok_or("native link edge")?;
    let input = edge
        .lines()
        .next()
        .ok_or("inputs")?
        .strip_prefix("build qemu-system-i386: cpp_LINKER_RSP ")
        .ok_or("native rule")?
        .split(" | ")
        .next()
        .ok_or("inputs")?;
    let ui = w.root.join("native-v2/ui.o");
    let identity: Value = c::read_json(&w.root.join("native-v2/build.identity.json"))?;
    if identity["core_sha256"] != resources::HEADLESS_CORE
        || identity["patch_sha256"]
            != c::file_digest(&repo.join("patches/xemu-headless-session.patch"))?
    {
        return Err("retained headless repair binding differs".into());
    }
    let input = input
        .replace(subject, object.to_str().unwrap())
        .replace("libsystem.a.p/ui_xemu.c.o", ui.to_str().unwrap());
    let link = edge
        .lines()
        .find_map(|s| s.strip_prefix(" LINK_ARGS = "))
        .ok_or("link args")?
        .replace(
            &format!(":{}", repo.join(".local/host-sysroot/usr/lib64").display()),
            "",
        );
    let build = format!(
        "builddir = {}\nrule compile_hmp\n command = {command}\nrule link_core\n command = c++ -m64 @$out.rsp\n rspfile = $out.rsp\n rspfile_content = -o $out $in {link}\nbuild {}: compile_hmp {}\nbuild {}: link_core {input}\n",
        new.display(),
        object.display(),
        new.join("migration/migration-hmp-cmds.c").display(),
        new.join("core").display()
    );
    c::write_new(&new.join("build.ninja"), build.as_bytes())?;
    c::store::checked(
        Command::new("ninja")
            .current_dir(&old)
            .arg("-f")
            .arg(new.join("build.ninja"))
            .args(["-j", "1"])
            .arg(new.join("core")),
    )?;
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(
            &json!({"base_pin":"03b988c9d76553b33ed712ba00822ffdf49f0e7d","prior_core_sha256":resources::HEADLESS_CORE,"prior_headless_object_sha256":c::file_digest(&ui)?,"paused_capture_patch_sha256":c::file_digest(&repo.join("patches/xemu-paused-savevm.patch"))?,"regression_result_sha256":c::file_digest(&contract)?,"core_sha256":c::file_digest(&new.join("core"))?,"recompiled_units":["migration/migration-hmp-cmds.c"],"reused_previous_headless_repair":true,"accepted_base_RAMHT_retained":true,"prior_builds_untouched":true}),
        )?,
    )?;
    Ok(())
}
