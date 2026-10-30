// SPDX-License-Identifier: MIT
//! Bounded desktop qualification of the retained core's existing KVM backend.
//! No host configuration, new donor, firmware build or Android claim.
use super::{self as c, resources};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;

pub(super) fn package(root: &Path) -> Result<(), Error> {
    let dir = root.join("host-kvm-v1");
    let identity: serde_json::Value = c::read_json(&dir.join("identity.json"))?;
    let core = dir.join("build/qemu-system-i386");
    if identity["core"] != resources::HOST_KVM_CORE
        || c::file_digest(&core)? != resources::HOST_KVM_CORE
    {
        return Err("exact compiled retained KVM core required".into());
    }
    let package = dir.join("package");
    c::mkdir(&package)?;
    fs::create_dir_all(package.join("native/subprojects/slirp"))?;
    let base = root.join("apu-quiescence-v1/package");
    let r = resources::Resources::apu_quiescent(&base)?;
    for (from, to) in [
        (core, package.join("native/core")),
        (base.join("open.bin"), package.join("open.bin")),
        (
            base.join("native/qemu-img"),
            package.join("native/qemu-img"),
        ),
        (
            dir.join("build/subprojects/slirp/libslirp.so.0")
                .canonicalize()?,
            package.join("native/subprojects/slirp/libslirp.so.0"),
        ),
    ] {
        c::store::checked(
            Command::new("cp")
                .args(["-a", "--reflink=always"])
                .arg(from)
                .arg(to),
        )?;
    }
    let mut manifest = r.manifest;
    manifest.core = resources::HOST_KVM_CORE.into();
    manifest.slirp = c::file_digest(&package.join("native/subprojects/slirp/libslirp.so.0"))?;
    c::write_new(&package.join("runtime.json"), &c::json(&manifest)?)?;
    let _validated = resources::Resources::host_kvm(&package)?;
    Ok(())
}

pub(super) fn original(repo: &Path, root: &Path) -> Result<(), Error> {
    let r = resources::Resources::host_kvm(&root.join("host-kvm-v1/package"))?;
    let result = super::cpu_performance::observe_resources(
        repo,
        root,
        r,
        &root.join("host-kvm-v1/original-split"),
        false,
    );
    c::write_new(
        &root.join("host-kvm-v1/original-split-attempt.json"),
        &c::json(
            &json!({"passed":result.is_ok(),"error":result.as_ref().err().map(ToString::to_string),"core":resources::HOST_KVM_CORE}),
        )?,
    )?;
    result
}

fn require_session(root: &Path) -> Result<(), Error> {
    let dir = root.join("host-kvm-v1");
    let a: serde_json::Value = c::read_json(&dir.join("session-first/result.json"))?;
    let b: serde_json::Value = c::read_json(&dir.join("session-return/result.json"))?;
    if a["passed"] != true
        || b["passed"] != true
        || b["restored"] != true
        || a["core"] != resources::HOST_KVM_CORE
        || b["core"] != resources::HOST_KVM_CORE
        || a["fresh_supervisor_pid"] == b["fresh_supervisor_pid"]
        || a["native_run"] == b["native_run"]
    {
        return Err("actual fresh-process original continuity required".into());
    }
    Ok(())
}

pub(super) fn io(repo: &Path, root: &Path) -> Result<(), Error> {
    require_session(root)?;
    let fixture = root.join("file-io-errors-v1/original.iso");
    if c::file_digest(&fixture)?
        != "f1e46094955b06096eada46bbc5a740f36e5ae1d50f91deb9d7bbfd6265945c6"
    {
        return Err("retained original I/O input differs".into());
    }
    let out = root.join("host-kvm-v1/original-io");
    c::mkdir(&out)?;
    let r = resources::Resources::host_kvm(&root.join("host-kvm-v1/package"))?;
    super::file_read::observe_with(repo, root, &out, &fixture, false, r)
}

pub(super) fn fable(repo: &Path, root: &Path, returning: bool) -> Result<(), Error> {
    require_session(root)?;
    let io: serde_json::Value =
        c::read_json(&root.join("host-kvm-v1/original-io/original.result.json"))?;
    if io["core"] != resources::HOST_KVM_CORE
        || io["guest_passed"] != true
        || io["data_verified"] != true
        || io["public_io_error_control_passed"] != true
        || io["native_clean"] != true
        || io["seeds_unchanged"] != true
    {
        return Err("changed-backend original I/O qualification required".into());
    }
    let r = resources::Resources::host_kvm(&root.join("host-kvm-v1/package"))?;
    if returning {
        let prior: serde_json::Value =
            c::read_json(&root.join("fable-host-kvm.native-started.json"))?;
        let app = prior["app"].as_str().ok_or("retained KVM game profile")?;
        super::build::observe_local_play(
            repo,
            root,
            r,
            &format!("fable-host-kvm-return-{}", c::new_id()?),
            Some(app.into()),
            true,
        )
    } else {
        if root.join("fable-host-kvm.native-started.json").exists() {
            return Err("KVM game already started; reconcile existing profile".into());
        }
        super::build::observe_local_play(repo, root, r, "fable-host-kvm", None, false)
    }
}

fn wait_original(
    player: &mut c::desktop::Player,
    log: &mut String,
    expected: &str,
) -> Result<(), Error> {
    let start = std::time::Instant::now();
    let mut chunk = String::new();
    while start.elapsed() < std::time::Duration::from_secs(30) {
        let text = player
            .live
            .as_mut()
            .ok_or("original live session")?
            .native
            .serial()?;
        chunk.push_str(&text);
        log.push_str(&text);
        if log.len() > 65536 {
            return Err("original session output bound".into());
        }
        if chunk.contains(expected) {
            return Ok(());
        }
        if chunk.contains("*** bug check") || chunk.contains("*** unexpected exception") {
            return Err("original guest terminal failure".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Err("original session response deadline; preserve actual output".into())
}

/// Invoke first and return as separate supervisor processes. The guest's RAM
/// counter/cookie differs from its ordinary disk marker; no frontend mock.
pub(super) fn session(root: &Path, returning: bool) -> Result<(), Error> {
    let dir = root.join("host-kvm-v1");
    let original: serde_json::Value = c::read_json(&dir.join("original-split/result.json"))?;
    if original["core"] != resources::HOST_KVM_CORE
        || original["guest_passed"] != true
        || original["clean"] != true
    {
        return Err("actual retained KVM original execution required".into());
    }
    let out = dir.join(if returning {
        "session-return"
    } else {
        "session-first"
    });
    c::mkdir(&out)?;
    let app = dir.join("continuity-app");
    let mut r = resources::Resources::host_kvm(&dir.join("package"))?;
    r.window = false;
    if returning {
        let first: serde_json::Value = c::read_json(&dir.join("session-first/result.json"))?;
        if first["passed"] != true || first["core"] != resources::HOST_KVM_CORE {
            return Err("completed original unsaved-state departure required".into());
        }
    }
    let mut player = c::desktop::Player::with_resources(&app, r)?;
    let fixture = root.join("fixtures-v2/a.iso");
    if c::file_digest(&fixture)? != resources::ORIGINAL_FIXTURES[0] {
        return Err("retained original session input differs".into());
    }
    let game = if returning {
        player.last_game()?.ok_or("retained original game target")?
    } else {
        player.add(&fixture)?
    };
    let mut log = String::new();
    let mut run = None;
    let observed = (|| -> Result<(), Error> {
        let restored = player.launch(&game)?;
        run = Some(
            player
                .live
                .as_ref()
                .ok_or("original live session")?
                .native
                .run
                .clone(),
        );
        if restored != returning {
            return Err("original restore/cold distinction failed".into());
        }
        if returning {
            player
                .live
                .as_mut()
                .ok_or("live")?
                .native
                .fixture_input("?")?;
            wait_original(
                &mut player,
                &mut log,
                "CART A state=00000002 save=00000001 cookie=57F090E4",
            )?;
            player
                .live
                .as_mut()
                .ok_or("live")?
                .native
                .fixture_input("a")?;
            wait_original(
                &mut player,
                &mut log,
                "CART A state=00000003 save=00000001 cookie=6B482CC3",
            )?;
        } else {
            wait_original(&mut player, &mut log, "CART A state=00000000 save=00000000")?;
            player
                .live
                .as_mut()
                .ok_or("live")?
                .native
                .fixture_input("as a")?;
            wait_original(
                &mut player,
                &mut log,
                "CART A state=00000002 save=00000001 cookie=57F090E4",
            )?;
        }
        Ok(())
    })();
    let returned = if observed.is_ok() {
        player.depart()
    } else {
        player.stop()
    };
    let native = match &run {
        Some(p) => c::read_json::<serde_json::Value>(&p.join("native-return.json"))?,
        None => serde_json::Value::Null,
    };
    let passed = observed.is_ok() && returned.is_ok() && native["clean"] == true;
    c::write_new(&out.join("serial.log"), log.as_bytes())?;
    c::write_new(
        &out.join("result.json"),
        &c::json(
            &json!({"passed":passed,"core":resources::HOST_KVM_CORE,"runtime":player.resources.identity,"native_run":run,"fresh_supervisor_pid":std::process::id(),"restored":returning&&observed.is_ok(),"native_return":native,"observation_error":observed.err().map(|e|e.to_string()),"return_error":returned.err().map(|e|e.to_string()),"fixture":resources::ORIGINAL_FIXTURES[0],"commercial_acceptance":false}),
        )?,
    )?;
    if !passed {
        return Err(
            "original KVM continuity failed; preserve profiles, partial work and exact result"
                .into(),
        );
    }
    Ok(())
}

pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Err("desktop Linux x86_64 qualification only".into());
    }
    // Availability is not a working-machine qualification. Do not change
    // groups, permissions, modules, host settings or an existing profile.
    let _device = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/kvm")?;
    let baseline = root.join("apu-quiescence-v1/corrected-runtime-v1");
    if c::file_digest(&baseline.join("build/qemu-system-i386"))? != resources::APU_QUIESCENT_CORE {
        return Err("retained corrected core differs".into());
    }
    let source_before = c::file_digest(&baseline.join("source/fpu/softfloat.c"))?;
    let apu_before = c::file_digest(&baseline.join("source/hw/xbox/mcpx/apu/apu.c"))?;
    let dsp = repo
        .join(".local/build-desktop/dsp/x86_64-unknown-linux-gnu/release/libdsp56300_emu_ffi.a");
    if c::file_digest(&dsp)? != "5e78afecce79231338bd6c1838a55d300886de3be4e38f3f82229013ac23f1a7" {
        return Err("retained DSP library differs".into());
    }
    let dir = root.join("host-kvm-v1");
    c::mkdir(&dir)?;
    let source = dir.join("source");
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(baseline.join("source"))
            .arg(&source),
    )?;
    let build = dir.join("build");
    c::mkdir(&build)?;
    c::write_new(
        &dir.join("inputs.json"),
        &c::json(&json!({
            "source_pin":"03b988c9d76553b33ed712ba00822ffdf49f0e7d",
            "source_reused_from_corrected_core":resources::APU_QUIESCENT_CORE,
            "arithmetic_candidate_included":false,
            "softfloat":source_before,"apu":apu_before,"dsp":c::file_digest(&dsp)?,
            "estimated_additional_allocation_bound_bytes":2u64*1024*1024*1024,
            "existing_source_allocated_kib":336904,"existing_build_allocated_kib":683244,
            "compiler_jobs":1,"new_dependency_or_download":false,
            "existing_builds_profiles_and_host_configuration_untouched":true,
            "native_or_game_qualified":false
        }))?,
    )?;
    let log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join("configure.log"))?;
    let python = repo.join(".local/tools/bin/python3");
    let status = Command::new(source.join("configure"))
        .current_dir(&build)
        .env("GIT_CEILING_DIRECTORIES", &dir)
        .env(
            "PKG_CONFIG_PATH",
            format!(
                "{}:{}",
                repo.join(".local/host-sysroot/usr/lib64/pkgconfig")
                    .display(),
                repo.join(".local/host-sysroot/usr/share/pkgconfig")
                    .display()
            ),
        )
        .env("PIP_CACHE_DIR", dir.join("pip-cache"))
        .env(
            "XBX_DSP_SOURCE",
            repo.join(".local/build-android/_deps/dsp56300_src-src"),
        )
        .env("XBX_DSP_LIBRARY", &dsp)
        .arg(format!("--python={}", python.display()))
        .args([
            "--target-list=i386-softmmu",
            "--extra-cflags=-DXBOX=1",
            "--enable-kvm",
            "--enable-tcg",
            "--disable-werror",
            "--disable-docs",
            "--disable-tools",
            "--disable-guest-agent",
            "--disable-rust",
            "--disable-download",
        ])
        .stdout(log.try_clone()?)
        .stderr(log)
        .status()?;
    c::write_new(
        &dir.join("configure-result.json"),
        &c::json(&json!({"exit":status.code(),"passed":status.success()}))?,
    )?;
    if !status.success() {
        return Err(
            "retained KVM configure failed; preserve inputs and exact dependency boundary".into(),
        );
    }
    let configured = fs::read_to_string(build.join("i386-softmmu-config-target.h"))?;
    if !configured.contains("#define CONFIG_KVM 1") || !configured.contains("#define CONFIG_TCG 1")
    {
        return Err("explicit retained KVM and TCG build required".into());
    }
    let site = Command::new(&python)
        .args([
            "-c",
            "import sysconfig;print(sysconfig.get_paths()['purelib'])",
        ])
        .output()?;
    if !site.status.success() {
        return Err("retained build Python unavailable".into());
    }
    let site = String::from_utf8(site.stdout)?;
    let dry = Command::new("ninja")
        .current_dir(&build)
        .args(["-n", "qemu-system-i386"])
        .env("PYTHONPATH", site.trim())
        .env("NINJA_STATUS", "[%f/%t] ")
        .output()?;
    let plan = String::from_utf8(dry.stdout)?;
    let steps = plan.lines().filter(|l| l.starts_with('[')).count();
    c::write_new(&dir.join("build-plan.log"), plan.as_bytes())?;
    if !dry.status.success()
        || steps == 0
        || steps > 4096
        || plan.to_ascii_lowercase().contains("regenerating build")
    {
        return Err("KVM compile exceeds admitted retained target".into());
    }
    let log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join("build.log"))?;
    let status = Command::new("timeout")
        .args([
            "--signal=TERM",
            "--kill-after=20s",
            "1800s",
            "ninja",
            "-j1",
            "qemu-system-i386",
        ])
        .current_dir(&build)
        .env("GIT_CEILING_DIRECTORIES", &dir)
        .env("PYTHONPATH", site.trim())
        .stdout(log.try_clone()?)
        .stderr(log)
        .status()?;
    let preserved = c::file_digest(&baseline.join("build/qemu-system-i386"))?
        == resources::APU_QUIESCENT_CORE
        && source_before == c::file_digest(&baseline.join("source/fpu/softfloat.c"))?
        && apu_before == c::file_digest(&baseline.join("source/hw/xbox/mcpx/apu/apu.c"))?;
    c::write_new(
        &dir.join("build-result.json"),
        &c::json(
            &json!({"exit":status.code(),"passed":status.success(),"planned_steps":steps,"retained_core_and_sources_unchanged":preserved,"compiler_jobs":1,"native_or_game_qualified":false}),
        )?,
    )?;
    if !status.success() || !preserved {
        return Err("KVM build failed or retained input differs; preserve partial work".into());
    }
    fs::File::open(build.join("qemu-system-i386"))?.sync_all()?;
    c::write_new(
        &dir.join("identity.json"),
        &c::json(
            &json!({"core":c::file_digest(&build.join("qemu-system-i386"))?,"source_pin":"03b988c9d76553b33ed712ba00822ffdf49f0e7d","open_system":resources::LOCAL_BASE_OPEN,"existing_kvm_backend_enabled":true,"tcg_retained":true,"native_or_game_qualified":false,"host_or_default_configuration_changed":false}),
        )?,
    )?;
    Ok(())
}
