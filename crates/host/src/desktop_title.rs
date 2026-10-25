//! Ordinary unchanged-title desktop observation. No guest memory/code probes or
//! synthetic title markers; visible progression requires separate image review.
use crate::probe::{OwnedChild, quote, response};
use crate::storage::ProfileLease;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::{
    fs::{DirBuilderExt, PermissionsExt},
    net::UnixStream,
};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn guest_memory(value: Option<&std::ffi::OsStr>) -> Result<u16, &'static str> {
    match value.and_then(std::ffi::OsStr::to_str) {
        None if value.is_none() => Ok(64),
        Some("64") => Ok(64),
        Some("128") => Ok(128),
        _ => Err("desktop guest memory must be 64 or 128 MiB"),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    engine: &Path,
    new_root: &Path,
    flash: &Path,
    disk: &Path,
    eeprom: &Path,
    media: &Path,
    seconds: u64,
    bootrom: Option<&Path>,
    journal_map: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    run_environment(
        engine,
        new_root,
        flash,
        disk,
        eeprom,
        Some(media),
        seconds,
        bootrom,
        journal_map,
    )
}

/// Ordinary no-disc boot of an independent console profile.
pub fn run_dashboard(
    engine: &Path,
    new_root: &Path,
    flash: &Path,
    disk: &Path,
    eeprom: &Path,
    seconds: u64,
    bootrom: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    run_environment(
        engine, new_root, flash, disk, eeprom, None, seconds, bootrom, None,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_environment(
    engine: &Path,
    new_root: &Path,
    flash: &Path,
    disk: &Path,
    eeprom: &Path,
    media: Option<&Path>,
    seconds: u64,
    bootrom: Option<&Path>,
    journal_map: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let basic_serial = match std::env::var_os("XBX_DESKTOP_BASIC_SERIAL") {
        None => false,
        Some(value) if value == "1" && journal_map.is_none() => true,
        _ => return Err("basic serial capture requires exact scope and no journal".into()),
    };
    let ceiling =
        match std::env::var_os("XBX_DESKTOP_REFERENCE_INTERVAL_SECONDS") {
            None => 300,
            Some(value) if value == "600" && media.is_some() => 600,
            _ => return Err(
                "extended reference interval requires a selected title and exact 600-second scope"
                    .into(),
            ),
        };
    if !(15..=ceiling).contains(&seconds) {
        return Err(format!("ordinary observation budget must be 15..{ceiling} seconds").into());
    }
    let memory_mib = guest_memory(std::env::var_os("XBX_DESKTOP_MEMORY_MIB").as_deref())?;
    let journal_format = journal_map
        .map(|map| crate::api_journal::verify_binding(map, flash))
        .transpose()?;
    let video = std::env::var("XBX_DESKTOP_VIDEO_DRIVER").unwrap_or_else(|_| "offscreen".into());
    if !matches!(video.as_str(), "offscreen" | "x11") {
        return Err("desktop video driver must be offscreen or x11".into());
    }
    let graphics = match std::env::var_os("XBX_DESKTOP_GRAPHICS") {
        None => "software",
        Some(value) if value == "software" => "software",
        Some(value) if value == "host" => "host",
        _ => return Err("desktop graphics must be software or host".into()),
    };
    if graphics == "host" && video != "x11" {
        return Err("qualified host graphics requires x11".into());
    }
    // An ordinary playable machine needs its first controller connected. The
    // retained frontend always creates the keyboard source, but binds it only
    // when this exact configuration entry is present.
    let controller = std::env::var("XBX_DESKTOP_CONTROLLER").unwrap_or_else(|_| "keyboard".into());
    if !matches!(controller.as_str(), "keyboard" | "none") {
        return Err("desktop controller must be keyboard or none".into());
    }
    let floating_point = std::env::var("XBX_DESKTOP_FP").unwrap_or_else(|_| "native".into());
    if !matches!(floating_point.as_str(), "native" | "software") {
        return Err("desktop floating-point path must be native or software".into());
    }
    let engine = engine.canonicalize()?;
    let mut media = media.map(File::open).transpose()?;
    let disc = media.as_mut().map(crate::disc::inspect_open).transpose()?;
    let incoming = media.as_ref().map(crate::disc::qemu_media_fd).transpose()?;
    fs::DirBuilder::new().mode(0o700).create(new_root)?;
    let root = new_root.canonicalize()?;
    let lease = ProfileLease::acquire(&root)?;
    for (source, name) in [
        (flash, "flash.bin"),
        (disk, "hdd.qcow2"),
        (eeprom, "eeprom.bin"),
    ] {
        fs::copy(source, root.join(name))?;
        // The pristine seeds may be read-only; only these private copies are
        // writable by the machine, never the seed or a predecessor profile.
        fs::set_permissions(root.join(name), fs::Permissions::from_mode(0o600))?;
    }
    let boot = if let Some(path) = bootrom {
        fs::copy(path, root.join("bootrom.bin"))?;
        root.join("bootrom.bin")
    } else {
        std::path::PathBuf::new()
    };
    // QEMU's existing raw block view on the retained read-only FD; no media copy,
    // pathname reopen, extraction, conversion or guest-code inspection.
    let view = disc.as_ref().map(|disc| format!(
        r#"json:{{"driver":"raw","offset":{},"read-only":true,"file":{{"driver":"file","filename":"/dev/fdset/1"}}}}"#,
        disc.partition_offset
    )).unwrap_or_default();
    let mut config = format!(
        "[general]\nshow_welcome=false\nskip_boot_anim=false\n[general.updates]\ncheck=false\n[display]\nrenderer=\"OPENGL\"\n[sys]\nmem_limit=\"{memory_mib}\"\n[sys.files]\nflashrom_path={}\nbootrom_path={}\nhdd_path={}\ndvd_path={}\neeprom_path={}\n",
        quote(&root.join("flash.bin"))?,
        quote(&boot)?,
        quote(&root.join("hdd.qcow2"))?,
        quote(Path::new(&view))?,
        quote(&root.join("eeprom.bin"))?
    );
    config.push_str("[input]\nauto_bind=false\n[input.bindings]\n");
    if controller == "keyboard" {
        config.push_str("port1=\"keyboard\"\nport1_driver=\"usb-xbox-gamepad-s\"\n");
    }
    config.push_str("[net]\nenable=false\n");
    if floating_point == "software" {
        config.push_str("[perf]\nfp_jit=false\n");
    }
    fs::write(root.join("xemu.toml"), config)?;
    if let Some(disc) = disc {
        fs::write(
            root.join("media-metadata.txt"),
            format!(
                "bytes={}\npartition_offset={}\ntitle={:?}\ntitle_id={:08x}\nregion={:08x}\nversion={}\n",
                disc.bytes,
                disc.partition_offset,
                disc.title.name,
                disc.title.id,
                disc.title.region,
                disc.title.version
            ),
        )?;
    }
    let qmp = root.join("qmp.sock");
    if qmp.as_os_str().len() > 100 {
        return Err("use a shorter new scratch path".into());
    }
    let (mut log_reader, log_writer) = UnixStream::pair()?;
    let mut log = File::create(root.join("native.log"))?;
    let mut serial_log = if basic_serial {
        Some(File::create(root.join("kernel.log"))?)
    } else {
        None
    };
    let mut command = Command::new(engine);
    command
        .current_dir(&root)
        .env("SDL_VIDEO_DRIVER", &video)
        .env("SDL_AUDIO_DRIVER", "dummy")
        .env("XBX_ISOLATED_PROCESS_EXIT", "1")
        .env(
            "LIBGL_ALWAYS_SOFTWARE",
            if graphics == "host" { "0" } else { "1" },
        );
    if floating_point == "software" {
        command.env("XEMU_TCG_DEFER_FIP", "0");
    }
    for (env, directory) in [
        ("XDG_DATA_HOME", "data"),
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_CACHE_HOME", "cache"),
        ("XDG_STATE_HOME", "state"),
        ("MESA_SHADER_CACHE_DIR", "mesa-shader-cache"),
    ] {
        command.env(env, root.join(directory));
    }
    command
        .arg("-config_path")
        .arg(root.join("xemu.toml"))
        .args(["-accel", "tcg"]);
    if let Some(incoming) = &incoming {
        command.arg("-add-fd").arg(format!(
            "fd={},set=1,opaque=xbx-readonly-disc",
            incoming.as_raw_fd()
        ));
    }
    // Both ordinary qualification routes are explicitly offline.
    command.args(["-nic", "none"]);
    command
        .arg("-qmp")
        .arg(format!("unix:{},server=on,wait=off", qmp.display()))
        .args(["-device", "lpc47m157"]);
    if basic_serial {
        // Existing QEMU serial transport, drained through the same finite
        // host-owned stream. This is ordinary output, never a guest probe.
        command.args(["-monitor", "none", "-serial", "stdio"]);
    } else {
        command
            .arg("-serial")
            .arg(format!("file:{}", root.join("kernel.log").display()));
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(std::os::fd::OwnedFd::from(
            log_writer.try_clone()?,
        )))
        .stderr(Stdio::from(std::os::fd::OwnedFd::from(log_writer)));
    lease.begin_writes()?;
    lease.inherit_for(&mut command);
    let mut owned = OwnedChild(command.spawn()?);
    drop(command);
    drop(incoming);
    let drain = std::thread::spawn(move || {
        let mut remaining = 4 * 1024 * 1024;
        let mut truncated = false;
        let mut capture_ok = true;
        let mut buffer = [0u8; 8192];
        loop {
            let n = match log_reader.read(&mut buffer) {
                Ok(n) => n,
                Err(_) => {
                    capture_ok = false;
                    break;
                }
            };
            if n == 0 {
                break;
            }
            let keep = n.min(remaining);
            capture_ok &= log.write_all(&buffer[..keep]).is_ok();
            if let Some(serial) = serial_log.as_mut() {
                capture_ok &= serial.write_all(&buffer[..keep]).is_ok();
            }
            truncated |= keep < n;
            remaining -= keep;
        }
        capture_ok &= log.sync_all().is_ok();
        if let Some(serial) = serial_log.as_mut() {
            capture_ok &= serial.sync_all().is_ok();
        }
        (truncated, capture_ok)
    });
    let start = Instant::now();
    let deadline = start + Duration::from_secs(seconds);
    let mut reason = "monitor_observation_failed";
    let mut monitor_keepalive = None;
    let mut capture_failure = None;
    let observation = (|| -> Result<(), Box<dyn std::error::Error>> {
        // Receipt publication can fail too; keep it inside the same native
        // termination/reaping/storage-sync path as monitor failures.
        fs::write(root.join("native.pid"), owned.0.id().to_string())?;
        while !qmp.exists() && Instant::now() < deadline {
            if owned.0.try_wait()?.is_some() {
                reason = "native_exited_before_monitor_readiness";
                return Err("native exited before monitor readiness".into());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let mut socket = UnixStream::connect(&qmp)?;
        socket.set_read_timeout(Some(Duration::from_secs(3)))?;
        socket.set_write_timeout(Some(Duration::from_secs(3)))?;
        let mut reader = BufReader::new(socket.try_clone()?);
        socket.write_all(b"{\"execute\":\"qmp_capabilities\"}\n")?;
        response(&mut reader)?;
        // Core-owned device inventory establishes attachment separately from
        // config/source intent. This is not guest input/gameplay acceptance.
        // QMP can be ready before the frontend finishes attaching its configured
        // controller. A first hub-only inventory is not a final rejection.
        let attachment_deadline = (Instant::now() + Duration::from_secs(10)).min(deadline);
        let mut attachment_checks = 0;
        let (usb, controller_attached) = loop {
            socket.write_all(b"{\"execute\":\"human-monitor-command\",\"arguments\":{\"command-line\":\"info usb\"}}\n")?;
            let usb = response(&mut reader)?;
            if attachment_checks == 0 {
                fs::write(root.join("usb-devices-initial.txt"), &usb)?;
            }
            attachment_checks += 1;
            let attached = usb.contains("Xbox") && usb.contains("Controller");
            if controller != "keyboard" || attached || Instant::now() >= attachment_deadline {
                break (usb, attached);
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        fs::write(root.join("usb-devices.txt"), &usb)?;
        fs::write(
            root.join("controller-attachment.txt"),
            format!(
                "requested={controller}\ncontroller_attached={controller_attached}\nattachment_checks={attachment_checks}\n"
            ),
        )?;
        if controller == "keyboard" && !controller_attached {
            reason = "requested_controller_not_attached";
            socket.write_all(b"{\"execute\":\"quit\"}\n")?;
            response(&mut reader)?;
            monitor_keepalive = Some((socket, reader));
            return Err("requested desktop Controller S not attached".into());
        }
        loop {
            let serial = root.join("kernel.log");
            let text = if serial.exists() {
                if fs::metadata(&serial)?.len() > 1024 * 1024 {
                    return Err("serial log exceeded observation bound".into());
                }
                fs::read_to_string(serial)?
            } else {
                String::new()
            };
            // Existing runtime status only. Exceptions alone do not establish a
            // terminal failure. An explicit error-dashboard request or bugcheck does.
            let terminal = text.contains("*** bug check")
                || (basic_serial && text.contains("*** unexpected exception "))
                || text.lines().any(|l| {
                    l.contains("dashboard_fields=1")
                        && l.contains("reason=00000001")
                        && l.contains("code=00000001")
                });
            if terminal {
                reason = "guest_terminal_startup_failure";
            } else if root.join("observation.complete").exists() {
                // Operator completion is a stop request, not a guest/menu pass.
                // Keep the same acknowledged stop/quit, reap and sync below.
                reason = "operator_completed_observation_requires_visual_review";
            } else if Instant::now() >= deadline {
                reason = "observation_budget_expired_requires_visual_review";
            }
            if owned.0.try_wait()?.is_some() {
                if !terminal {
                    reason = "native_exited_during_observation";
                }
                return Err("native exited during observation".into());
            }
            if terminal || root.join("observation.complete").exists() || Instant::now() >= deadline
            {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        socket.write_all(b"{\"execute\":\"stop\"}\n")?;
        response(&mut reader)?;
        if let Some(map) = journal_map {
            let capture = match journal_format {
                Some(crate::api_journal::JournalFormat::Context) => {
                    crate::contract::capture(&mut socket, &mut reader, map, &root)
                }
                Some(crate::api_journal::JournalFormat::Observer) => {
                    crate::observer::capture(&mut socket, &mut reader, map, &root)
                }
                _ => crate::api_journal::capture(
                    &mut socket,
                    &mut reader,
                    map,
                    &root.join("api-journal.txt"),
                ),
            };
            // A rejected capture is a failed observation, but must not prevent
            // acknowledged quit/reap/sync of an otherwise healthy native core.
            if let Err(error) = capture {
                capture_failure = Some(error.to_string());
            }
        }
        socket.write_all(b"{\"execute\":\"quit\"}\n")?;
        response(&mut reader)?;
        monitor_keepalive = Some((socket, reader));
        Ok(())
    })();
    let stop_deadline = Instant::now() + Duration::from_secs(5);
    let mut forced = false;
    let status = loop {
        if let Some(status) = owned.0.try_wait()? {
            break status;
        }
        if (observation.is_err() && monitor_keepalive.is_none()) || Instant::now() >= stop_deadline
        {
            forced = true;
            owned.0.kill()?;
            break owned.0.wait()?;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    drop(monitor_keepalive);
    let capture = drain.join();
    lease.sync_storage()?;
    if basic_serial {
        let (truncated, capture_ok) = capture.unwrap_or((true, false));
        fs::write(
            root.join("serial-capture.txt"),
            format!(
                "transport=ordinary_stdio_combined_with_native_output\nlimit_bytes=4194304\ntruncated={truncated}\ncapture_ok={capture_ok}\n"
            ),
        )?;
    }
    fs::write(
        root.join("termination.txt"),
        format!(
            "status={status}\nforced={forced}\nqmp={observation:?}\ncapture_failure={capture_failure:?}\nstorage_synced=true\n"
        ),
    )?;
    fs::write(
        root.join("observation.txt"),
        format!(
            "reason={reason}\nelapsed_seconds={:.3}\nbudget_seconds={seconds}\nvideo_driver={video}\ngraphics_request={graphics}\ncontroller={controller}\nfloating_point={floating_point}\nmemory_mib={memory_mib}\ndisc_present={}\nbootrom_present={}\nnetwork=none\nvisible_progress_requires_own_window_review=true\naudio=dummy_not_qualified\ngameplay_unqualified=true\n",
            start.elapsed().as_secs_f64(),
            media.is_some(),
            bootrom.is_some(),
        ),
    )?;
    let serial = fs::read_to_string(root.join("kernel.log"))?;
    fs::write(
        root.join("runtime-summary.txt"),
        crate::runtime_summary::Summary::parse(&serial).receipt(),
    )?;
    println!(
        "Ordinary title observation: {reason}; native={status}; forced={forced}; storage synced. Visual/gameplay acceptance is separate."
    );
    if observation.is_err() || !status.success() || forced {
        return Err("ordinary title runtime failed; scratch and evidence preserved".into());
    }
    if let Some(error) = capture_failure {
        return Err(format!("observer capture failed; evidence retained: {error}").into());
    }
    Ok(())
}
