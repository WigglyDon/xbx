// SPDX-License-Identifier: MIT
//! Original paused-device control, using retained QEMU's actual QTest/PCI/APU.
//! No title, proprietary DSP program, guest instruction observer or save input.
//! Registers: pinned xemu hw/xbox/{xbox.c,mcpx/apu/apu_regs.h}; QTest protocol:
//! system/qtest.c. Generated zero words are DSP NOPs; uninitialized P RAM retains
//! the interpreter's existing invalid-word sentinel. It must not be fetched
//! while the whole VM is stopped.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::{
        fs::OpenOptionsExt,
        net::UnixStream,
        process::{CommandExt, ExitStatusExt},
    },
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
const APU_PATCH: &str = "9becd99168277eda978bdf25e9b644a347a5122300946b7b26bfaf26d9faba09";

/// Keep the retained source/build immutable. Reuse only its copied object graph
/// in a separate repair tree, prevent regeneration into the original absolute
/// Meson directory, and refuse an unexpectedly broad native build.
pub(super) fn correct_runtime(repo: &Path, root: &Path) -> Result<(), Error> {
    let red: Value = c::read_json(&root.join("apu-quiescence-v1/baseline-paused/result.json"))?;
    if red["core"] != resources::LOCAL_IO_CORE
        || red["game_loaded"] != false
        || red["vm_stopped_verified"] != true
        || red["same_public_DSP_read_assertion_reproduced"] != true
    {
        return Err("actual original paused-APU failure is required before correction".into());
    }
    let patch = repo.join("patches/xemu-apu-paused-reset.patch");
    if c::file_digest(&patch)? != APU_PATCH {
        return Err("APU correction patch differs".into());
    }
    let base = root.join("rebuilt-runtime-v1");
    let old_core = base.join("build/qemu-system-i386");
    if c::file_digest(&old_core)? != resources::LOCAL_IO_CORE {
        return Err("retained core differs".into());
    }
    let relative = "hw/xbox/mcpx/apu/apu.c";
    let old_source = c::file_digest(&base.join("source").join(relative))?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(base.join("source"))
            .env("GIT_CEILING_DIRECTORIES", base.canonicalize()?)
            .args(["apply", "--check"])
            .arg(&patch),
    )?;
    let dir = root.join("apu-quiescence-v1/corrected-runtime-v1");
    if !dir.exists() {
        c::mkdir(&dir)?;
        c::store::checked(
            Command::new("cp")
                .args(["-a", "--reflink=always"])
                .arg(base.join("source"))
                .arg(base.join("build"))
                .arg(&dir),
        )?;
        let source = dir.join("source");
        c::store::checked(
            Command::new("git")
                .arg("-C")
                .arg(&source)
                .env("GIT_CEILING_DIRECTORIES", dir.canonicalize()?)
                .arg("apply")
                .arg(&patch),
        )?;
        let ninja = dir.join("build/build.ninja");
        let graph = fs::read_to_string(&ninja)?;
        let regeneration = "build build.ninja: REGENERATE_BUILD ";
        if graph.matches(regeneration).count() != 1 {
            return Err("unexpected retained Ninja regeneration edge".into());
        }
        fs::write(
            &ninja,
            graph.replacen(
                regeneration,
                "build xbx-preserved-regeneration-edge: REGENERATE_BUILD ",
                1,
            ),
        )?;
        c::write_new(
            &dir.join("inputs.json"),
            &c::json(&json!({
                "base_core":resources::LOCAL_IO_CORE,"base_inputs":c::file_digest(&base.join("inputs.json"))?,
                "base_apu_source":old_source,"patched_apu_source":c::file_digest(&source.join(relative))?,
                "patch":APU_PATCH,"compiler_jobs":1,"additional_peak_bound_bytes":2147483648u64,
                "cloned_ninja_graph":c::file_digest(&ninja)?,
                "old_source_and_build_mutated":false,"object_graph_reused_in_separate_tree":true,
                "regeneration_edge_renamed_only_to_prevent_original_absolute_Meson_reentry":true
            }))?,
        )?;
    }
    let inputs: Value = c::read_json(&dir.join("inputs.json"))?;
    if inputs["base_core"] != resources::LOCAL_IO_CORE
        || inputs["patch"] != APU_PATCH
        || inputs["base_apu_source"] != old_source
        || inputs["patched_apu_source"] != c::file_digest(&dir.join("source").join(relative))?
        || inputs["cloned_ninja_graph"] != c::file_digest(&dir.join("build/build.ninja"))?
    {
        return Err("reconcile preserved partial APU build; source bindings differ".into());
    }
    let build = dir.join("build");
    if !dir.join("identity.json").exists() {
        let dry = Command::new("ninja")
            .current_dir(&build)
            .args(["-n", "qemu-system-i386"])
            .env("NINJA_STATUS", "[%f/%t] ")
            .env("GIT_CEILING_DIRECTORIES", &dir)
            .output()?;
        let mut planned_output = dry.stdout.clone();
        planned_output.extend_from_slice(&dry.stderr);
        c::write_new(
            &dir.join(format!("planned-build-{}.log", c::new_id()?)),
            &planned_output,
        )?;
        let plan = String::from_utf8(dry.stdout)?;
        let steps = plan.lines().filter(|line| line.starts_with('[')).count();
        let started = dir.join("build-started.json").exists();
        if !dry.status.success()
            || steps > 32
            || !started && !plan.contains("apu.c.o")
            || plan.to_ascii_lowercase().contains("regenerating build")
        {
            return Err(
                "refuse an unqualified broad or regenerating native build; preserve plan".into(),
            );
        }
        if !started {
            c::write_new(
                &dir.join("build-started.json"),
                b"{\"one_native_compiler\":true}\n",
            )?;
        }
        if !plan.contains("no work to do") {
            let log = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .mode(0o600)
                .open(dir.join("build.log"))?;
            c::store::checked(
                Command::new("ninja")
                    .current_dir(&build)
                    .args(["-j1", "qemu-system-i386"])
                    .env("GIT_CEILING_DIRECTORIES", &dir)
                    .stdout(log.try_clone()?)
                    .stderr(log),
            )?;
        }
        let new_core = c::file_digest(&build.join("qemu-system-i386"))?;
        if new_core == resources::LOCAL_IO_CORE
            || c::file_digest(&old_core)? != resources::LOCAL_IO_CORE
            || c::file_digest(&base.join("source").join(relative))? != old_source
        {
            return Err(
                "corrected identity did not change or retained source/build changed".into(),
            );
        }
        c::write_new(
            &dir.join("identity.json"),
            &c::json(&json!({
                "core":new_core,"inputs":c::file_digest(&dir.join("inputs.json"))?,
                "native_qualified":false,"ordinary_default_changed":false,"retained_core_unchanged":true
            }))?,
        )?;
    }
    let identity: Value = c::read_json(&dir.join("identity.json"))?;
    if identity["core"] != c::file_digest(&build.join("qemu-system-i386"))?
        || identity["inputs"] != c::file_digest(&dir.join("inputs.json"))?
    {
        return Err("corrected native artifact/inputs differ".into());
    }
    let package = root.join("apu-quiescence-v1/package");
    if !package.exists() {
        c::mkdir(&package)?;
        fs::create_dir_all(package.join("native/subprojects/slirp"))?;
        let old = root.join("local-io-package-baseline");
        let mut manifest: resources::Manifest = c::read_json(&old.join("runtime.json"))?;
        fs::copy(build.join("qemu-system-i386"), package.join("native/core"))?;
        for name in [
            "open.bin",
            "native/qemu-img",
            "native/subprojects/slirp/libslirp.so.0",
        ] {
            fs::copy(old.join(name), package.join(name))?;
        }
        manifest.core = identity["core"]
            .as_str()
            .ok_or("corrected core identity")?
            .into();
        c::write_new(&package.join("runtime.json"), &c::json(&manifest)?)?;
    }
    println!(
        "{}",
        json!({"identity":dir.join("identity.json"),"original_paused_regression_reproduced":true,"correction_built":true,"native_qualification_pending":true})
    );
    Ok(())
}

struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

fn connect(path: &Path, child: &mut Child) -> Result<UnixStream, Error> {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        match UnixStream::connect(path) {
            Ok(socket) => return Ok(socket),
            Err(error) => {
                if child.try_wait()?.is_some() || Instant::now() >= deadline {
                    return Err(
                        format!("original device-control socket unavailable: {error}").into(),
                    );
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

struct QTest {
    writer: UnixStream,
    reader: BufReader<UnixStream>,
}
impl QTest {
    fn new(socket: UnixStream) -> Result<Self, Error> {
        socket.set_read_timeout(Some(Duration::from_secs(3)))?;
        socket.set_write_timeout(Some(Duration::from_secs(3)))?;
        Ok(Self {
            reader: BufReader::new(socket.try_clone()?),
            writer: socket,
        })
    }
    fn call(&mut self, command: &str) -> Result<String, Error> {
        writeln!(self.writer, "{command}")?;
        let mut reply = String::new();
        self.reader.read_line(&mut reply)?;
        if !reply.starts_with("OK") || reply.len() > 4096 {
            return Err(format!("original QTest command refused: {command}: {reply}").into());
        }
        Ok(reply)
    }
    fn write(&mut self, address: u64, value: u32) -> Result<(), Error> {
        self.call(&format!("writel 0x{address:x} 0x{value:x}"))?;
        Ok(())
    }
}

pub(super) fn baseline(repo: &Path, root: &Path) -> Result<(), Error> {
    let resources = resources::Resources::local_io(&root.join("local-io-package-baseline"), false)?;
    run(repo, root, resources, "baseline-paused", true, false)
}

pub(super) fn corrected(repo: &Path, root: &Path, running: bool) -> Result<(), Error> {
    let r = resources::Resources::apu_quiescent(&root.join("apu-quiescence-v1/package"))?;
    run(
        repo,
        root,
        r,
        if running {
            "corrected-running"
        } else {
            "corrected-paused"
        },
        running,
        running,
    )
}

fn corrected_controls(root: &Path) -> Result<(), Error> {
    for (name, field) in [
        ("corrected-paused", "original_paused_worker_contract_passed"),
        (
            "corrected-running",
            "active_DSP_fetch_assertion_control_passed",
        ),
    ] {
        let result: Value = c::read_json(
            &root
                .join("apu-quiescence-v1")
                .join(name)
                .join("result.json"),
        )?;
        if result["core"] != resources::APU_QUIESCENT_CORE
            || result[field] != true
            || result["game_loaded"] != false
            || result["native_reaped"] != true
            || result["seeds_unchanged"] != true
            || result["kill_attempted"] != false
        {
            return Err("actual corrected original APU controls and retirement required".into());
        }
    }
    Ok(())
}

pub(super) fn original_io(repo: &Path, root: &Path) -> Result<(), Error> {
    corrected_controls(root)?;
    let r = resources::Resources::apu_quiescent(&root.join("apu-quiescence-v1/package"))?;
    let dir = root.join("apu-quiescence-v1/original-io");
    c::mkdir(&dir)?;
    super::file_read::observe_with(
        repo,
        root,
        &dir,
        &root.join("file-io-errors-v1/original.iso"),
        false,
        r,
    )
}

pub(super) fn fable(repo: &Path, root: &Path, returning: bool) -> Result<(), Error> {
    corrected_controls(root)?;
    let io: Value = c::read_json(&root.join("apu-quiescence-v1/original-io/original.result.json"))?;
    if io["core"] != resources::APU_QUIESCENT_CORE
        || io["open_system"] != resources::LOCAL_BASE_OPEN
        || io["guest_passed"] != true
        || io["data_verified"] != true
        || io["native_clean"] != true
        || io["seeds_unchanged"] != true
    {
        return Err("changed core requires its actual original startup/I-O requalification".into());
    }
    let r = resources::Resources::apu_quiescent(&root.join("apu-quiescence-v1/package"))?;
    if returning {
        let prior: Value = c::read_json(&root.join("fable-apu-corrected.native-started.json"))?;
        let app = Path::new(prior["app"].as_str().ok_or("corrected cartridge app")?);
        let prefix = format!("fable-apu-corrected-return-{}", c::new_id()?);
        super::build::observe_local_play(repo, root, r, &prefix, Some(app.into()), true)
    } else {
        let prefix = "fable-apu-corrected";
        if root.join(format!("{prefix}.native-started.json")).exists() {
            return Err(
                "corrected game launch already started; reconcile its existing session".into(),
            );
        }
        super::build::observe_local_play(repo, root, r, prefix, None, false)
    }
}

fn run(
    repo: &Path,
    root: &Path,
    r: resources::Resources,
    label: &str,
    expect_red: bool,
    running_control: bool,
) -> Result<(), Error> {
    let parent = root.join("apu-quiescence-v1");
    fs::create_dir_all(&parent)?;
    let dir = parent.join(label);
    c::mkdir(&dir)?;
    let lease = crate::storage::ProfileLease::acquire(&dir)?;
    let seed_disk = repo.join(".local/dashboard-v20260516-0955/xbox_hdd.qcow2");
    let seed_eeprom = repo.join(".local/quick-reboot-d87cfc33-run1/eeprom.bin");
    let before = (c::file_digest(&seed_disk)?, c::file_digest(&seed_eeprom)?);
    c::store::copy_sparse(&seed_disk, &dir.join("hdd.qcow2"))?;
    c::store::copy_sparse(&seed_eeprom, &dir.join("eeprom.bin"))?;
    let quote = |path: PathBuf| serde_json::to_string(&path).unwrap();
    let config = format!(
        "[general]\nshow_welcome=false\n[general.updates]\ncheck=false\n[display]\nrenderer=\"OPENGL\"\n[audio]\nuse_dsp=true\nuse_dsp_jit=false\n[sys]\nmem_limit=\"64\"\n[sys.files]\nbootrom_path=\"\"\nflashrom_path={}\nhdd_path={}\neeprom_path={}\ndvd_path=\"\"\n",
        quote(r.root.join("open.bin")),
        quote(dir.join("hdd.qcow2")),
        quote(dir.join("eeprom.bin"))
    );
    c::write_new(&dir.join("xemu.toml"), config.as_bytes())?;
    let qmp = PathBuf::from("/tmp").join(format!("xbx-apu-qmp-{}.sock", c::new_id()?));
    let qtest = PathBuf::from("/tmp").join(format!("xbx-apu-test-{}.sock", c::new_id()?));
    let log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(dir.join("native.log"))?;
    let mut command = Command::new(r.core());
    command
        .current_dir(&dir)
        .env("XBX_ISOLATED_PROCESS_EXIT", "1")
        .env("SDL_AUDIO_DRIVER", "dummy")
        .env("SDL_VIDEO_DRIVER", "x11")
        .env("LIBGL_ALWAYS_SOFTWARE", "0")
        .args([
            "-S",
            "-accel",
            "tcg",
            "-nic",
            "none",
            "-display",
            "none",
            "-config_path",
        ])
        .arg(dir.join("xemu.toml"))
        .arg("-qmp")
        .arg(format!("unix:{},server=on,wait=off", qmp.display()))
        .arg("-qtest")
        .arg(format!("unix:{},server=on,wait=off", qtest.display()))
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log);
    for (key, name) in [
        ("XDG_DATA_HOME", "data"),
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_CACHE_HOME", "cache"),
        ("MESA_SHADER_CACHE_DIR", "mesa"),
        ("XDG_STATE_HOME", "state"),
    ] {
        command.env(key, dir.join(name));
    }
    // Synthetic failure controls must not create unbounded host core dumps.
    unsafe extern "C" {
        fn setrlimit(resource: i32, limit: *const [u64; 2]) -> i32;
    }
    unsafe {
        command.pre_exec(|| {
            if setrlimit(4, &[0, 0]) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    lease.inherit_for(&mut command);
    let mut child = Process(command.spawn()?);
    c::write_new(&dir.join("native.pid"), child.0.id().to_string().as_bytes())?;
    let mut paused_verified = false;
    let mut setup_completed = false;
    let mut quit_acknowledged = false;
    let mut running_verified = false;
    let operation = (|| -> Result<(), Error> {
        let socket = connect(&qmp, &mut child.0)?;
        let timeout = socket.try_clone()?;
        let mut monitor = c::monitor::Monitor::connected(socket)?;
        timeout.set_read_timeout(Some(Duration::from_secs(5)))?;
        let mut io = QTest::new(connect(&qtest, &mut child.0)?)?;
        if monitor.call("query-status", json!({}))?["running"] != false {
            return Err("original VM must start stopped".into());
        }
        monitor.call("system_reset", json!({}))?;
        std::thread::sleep(Duration::from_millis(100));
        paused_verified = monitor.call("query-status", json!({}))?["running"] == false;
        if !paused_verified {
            return Err("reset changed stopped VM state".into());
        }
        // PCI0:5.0 is MCPX APU. Assign an isolated MMIO aperture explicitly.
        io.call("outl 0xcf8 0x80002810")?;
        io.call("outl 0xcfc 0xfe800000")?;
        io.call("outl 0xcf8 0x80002804")?;
        io.call("outw 0xcfc 0x0002")?;
        io.call("memset 0x10000 0x3000 0x00")?;
        io.write(0x10000, 0x11000)?;
        io.write(0x10008, 0x12000)?;
        let apu = 0xfe800000;
        for offset in [0x2054, 0x2060, 0x206c] {
            io.write(apu + offset, 0xffff)?;
        }
        io.write(apu + 0x2040, 0x10000)?; // GP scratch scatter-gather table
        io.write(apu + 0x20d4, 1)?; // two generated zero/NOP pages
        io.write(apu + 0x30000 + 0xfffc, 3)?; // bootstrap GP; no proprietary code
        let readback = io.call("readl 0xfe83a000")?;
        let word = readback
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.strip_prefix("0x"));
        if word.and_then(|s| u64::from_str_radix(s, 16).ok()) != Some(0) {
            return Err("generated DSP NOP bootstrap did not read back".into());
        }
        setup_completed = true;
        if running_control {
            monitor.call("cont", json!({}))?;
            running_verified = monitor.call("query-status", json!({}))?["running"] == true;
            if !running_verified {
                return Err("original active DSP control did not resume".into());
            }
        }
        io.write(apu + 0x2000, 8)?; // enable APU engine while VM remains stopped
        let deadline = Instant::now() + Duration::from_secs(3);
        while child.0.try_wait()?.is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        if child.0.try_wait()?.is_none() {
            quit_acknowledged = monitor.call("quit", json!({})).is_ok();
        }
        Ok(())
    })();
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut kill_attempted = false;
    let status = loop {
        if let Some(status) = child.0.try_wait()? {
            break status;
        }
        if operation.is_err() || Instant::now() >= deadline {
            kill_attempted = true;
            child.0.kill()?;
            break child.0.wait()?;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    c::store::sync_files(&dir)?;
    let native_log = fs::read_to_string(dir.join("native.log"))?;
    let read_assertion = native_log.contains("read_memory_p") && native_log.contains("Assertion");
    let fetch_failure =
        paused_verified && setup_completed && status.signal() == Some(6) && read_assertion;
    let red_reproduced = fetch_failure && !running_control;
    let passed = paused_verified
        && setup_completed
        && operation.is_ok()
        && quit_acknowledged
        && status.success()
        && !kill_attempted;
    let seeds_unchanged = before == (c::file_digest(&seed_disk)?, c::file_digest(&seed_eeprom)?);
    c::write_new(
        &dir.join("result.json"),
        &c::json(&json!({
            "original_generated_data_only":true,"game_loaded":false,"core":r.manifest.core,"open_system":r.manifest.open_system,
            "vm_stopped_verified":paused_verified,"setup_completed":setup_completed,"original_paused_worker_contract_passed":passed,
            "running_control":running_control,"running_verified":running_verified,
            "active_DSP_fetch_assertion_control_passed":running_control && running_verified && fetch_failure,
            "same_public_DSP_read_assertion_reproduced":red_reproduced,"operation_error":operation.err().map(|e|e.to_string()),
            "native_exit":status.to_string(),"native_reaped":true,"qmp_acknowledged":quit_acknowledged,"kill_attempted":kill_attempted,
            "storage_synced":true,"seeds_unchanged":seeds_unchanged,"core_dump_limit":0
        }))?,
    )?;
    let _ = fs::remove_file(qmp);
    let _ = fs::remove_file(qtest);
    println!(
        "{}",
        json!({"result":dir.join("result.json"),"paused_contract_passed":passed,"red_reproduced":red_reproduced})
    );
    let expected_boundary = if running_control {
        running_verified && fetch_failure
    } else if expect_red {
        red_reproduced
    } else {
        passed
    };
    if !seeds_unchanged || !expected_boundary {
        return Err(
            "original APU control did not establish the expected boundary; retain actual result"
                .into(),
        );
    }
    Ok(())
}
