// SPDX-License-Identifier: MIT
//! Original byte-range invalidation reproduction from actual village counters.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{
    fs, io,
    path::Path,
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
pub(super) const CORE: &str = match option_env!("XBX_SMC_CORE_SHA256") {
    Some(v) => v,
    None => "unqualified-smc-core",
};
pub(super) const INPUT_CORE: &str = match option_env!("XBX_INPUT_CONTROL_CORE_SHA256") {
    Some(v) => v,
    None => "unqualified-input-control-core",
};
pub(super) fn package(root: &Path) -> Result<(), Error> {
    package_variant(root, false)
}
pub(super) fn input_package(root: &Path) -> Result<(), Error> {
    package_variant(root, true)
}
fn package_variant(root: &Path, control: bool) -> Result<(), Error> {
    let pin = if control { INPUT_CORE } else { CORE };
    use std::process::Command;
    let dir = root.join(if control {
        "smc-v1/input-control-runtime"
    } else {
        "smc-v1/runtime"
    });
    let identity: Value = c::read_json(&dir.join("identity.json"))?;
    let core = dir.join("build/qemu-system-i386");
    if identity["core"] != pin || c::file_digest(&core)? != pin {
        return Err("exact separately built byte-range core required".into());
    }
    let base = root.join("apu-quiescence-v1/package");
    let old = resources::Resources::apu_quiescent(&base)?;
    let out = root.join(if control {
        "smc-v1/input-control-package"
    } else {
        "smc-v1/package"
    });
    c::mkdir(&out)?;
    fs::create_dir_all(out.join("native/subprojects/slirp"))?;
    for (from, to) in [
        (core, out.join("native/core")),
        (base.join("open.bin"), out.join("open.bin")),
        (base.join("native/qemu-img"), out.join("native/qemu-img")),
        (
            base.join("native/subprojects/slirp/libslirp.so.0"),
            out.join("native/subprojects/slirp/libslirp.so.0"),
        ),
    ] {
        c::store::checked(
            Command::new("cp")
                .args(["-a", "--reflink=always"])
                .arg(from)
                .arg(to),
        )?;
    }
    let mut manifest = old.manifest;
    manifest.core = pin.into();
    c::write_new(&out.join("runtime.json"), &c::json(&manifest)?)?;
    let new = if control {
        resources::Resources::input_control_candidate(&out)?
    } else {
        resources::Resources::smc_candidate(&out)?
    };
    if new.identity != old.identity {
        return Err("explicit native-state lineage differs".into());
    }
    Ok(())
}
fn require_candidate(root: &Path) -> Result<(), Error> {
    let v: Value = c::read_json(&root.join("smc-v1/candidate-v2/result.json"))?;
    if v["passed"] != true || v["core"] != CORE || v["false_invalidations_eliminated"] != true {
        return Err("unchanged original correctness and invalidation proof required".into());
    }
    Ok(())
}
pub(super) fn session_baseline(root: &Path) -> Result<(), Error> {
    require_candidate(root)?;
    let r = resources::Resources::apu_quiescent(&root.join("apu-quiescence-v1/package"))?;
    super::host_input::baseline_at(root, r, &root.join("smc-v1/session-baseline"))
}
pub(super) fn session(root: &Path, returning: bool) -> Result<(), Error> {
    session_variant(root, returning, false)
}
pub(super) fn input_session(root: &Path, returning: bool) -> Result<(), Error> {
    session_variant(root, returning, true)
}
fn session_variant(root: &Path, returning: bool, control: bool) -> Result<(), Error> {
    let pin = if control { INPUT_CORE } else { CORE };
    use std::process::Command;
    require_candidate(root)?;
    let dir = root.join("smc-v1");
    let baseline: Value = c::read_json(&dir.join("session-baseline/result.json"))?;
    if baseline["passed"] != true || baseline["core"] != resources::APU_QUIESCENT_CORE {
        return Err("actual matched visible original baseline required".into());
    }
    let old = dir.join("session-baseline/app");
    let app = dir.join(if control {
        "input-session-app"
    } else {
        "session-app"
    });
    let out = dir.join(if control {
        if returning {
            "input-session-return"
        } else {
            "input-session-cross"
        }
    } else if returning {
        "session-return"
    } else {
        "session-cross"
    });
    c::mkdir(&out)?;
    let old_library = c::file_digest(&old.join("library.json"))?;
    if !returning {
        let _lease = crate::storage::ProfileLease::acquire(&old)?;
        c::store::checked(
            Command::new("cp")
                .args(["-a", "--reflink=always"])
                .arg(&old)
                .arg(&app),
        )?;
    } else {
        let prior: Value = c::read_json(&dir.join(if control {
            "input-session-cross/result.json"
        } else {
            "session-cross/result.json"
        }))?;
        if prior["passed"] != true || prior["core"] != pin {
            return Err("actual original cross-build restore required".into());
        }
    }
    let r = if control {
        resources::Resources::input_control_candidate(&dir.join("input-control-package"))?
    } else {
        resources::Resources::smc_candidate(&dir.join("package"))?
    };
    super::host_input::original_at(
        &old,
        &app,
        &out,
        r,
        resources::APU_QUIESCENT_CORE,
        &old_library,
        returning,
    )
}
pub(super) fn fable(repo: &Path, root: &Path, returning: bool) -> Result<(), Error> {
    fable_variant(repo, root, returning, false, false)
}
pub(super) fn input_fable(repo: &Path, root: &Path, returning: bool) -> Result<(), Error> {
    fable_variant(repo, root, returning, true, false)
}
pub(super) fn keyed_fable(repo: &Path, root: &Path) -> Result<(), Error> {
    fable_variant(repo, root, false, false, true)
}
pub(super) fn base_copy_fable(repo: &Path, root: &Path) -> Result<(), Error> {
    use std::process::Command;
    let dir = root.join("smc-v1");
    let app = dir.join("fable-base-copy-app");
    if app.try_exists()? {
        return Err("preserve existing base-copy control".into());
    }
    let prior: Value = c::read_json(&root.join("fable-apu-corrected.native-started.json"))?;
    let old = std::path::PathBuf::from(prior["app"].as_str().ok_or("retained base app")?);
    let _app_lease = crate::storage::ProfileLease::acquire(&old)?;
    let library = c::library::Library::read(&old)?;
    if library.entries.len() != 1 {
        return Err("unambiguous retained base entry required".into());
    }
    let _profile_lease = crate::storage::ProfileLease::acquire(
        &old.join("profiles").join(&library.entries[0].profile),
    )?;
    let before = c::file_digest(&old.join("library.json"))?;
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(&old)
            .arg(&app),
    )?;
    if before != c::file_digest(&old.join("library.json"))? {
        return Err("canonical base app changed".into());
    }
    drop(_profile_lease);
    drop(_app_lease);
    let r = resources::Resources::apu_quiescent(&root.join("apu-quiescence-v1/package"))?;
    super::build::observe_local_held_return(
        repo,
        root,
        r,
        &format!("fable-same-core-copy-{}", c::new_id()?),
        app,
    )
}
fn fable_variant(
    repo: &Path,
    root: &Path,
    returning: bool,
    control: bool,
    cache_check: bool,
) -> Result<(), Error> {
    let pin = if control { INPUT_CORE } else { CORE };
    use std::process::Command;
    require_candidate(root)?;
    let dir = root.join("smc-v1");
    for name in if control {
        ["input-session-cross", "input-session-return"]
    } else {
        ["session-cross", "session-return"]
    } {
        let v: Value = c::read_json(&dir.join(name).join("result.json"))?;
        if v["passed"] != true
            || v["core"] != pin
            || v["base_state_core"] != resources::APU_QUIESCENT_CORE
        {
            return Err(
                "actual original cross-build and fresh-process restoration required".into(),
            );
        }
    }
    let app = dir.join(if cache_check {
        "fable-cache-key-app"
    } else if control {
        "fable-input-control-app"
    } else {
        "fable-app"
    });
    if !returning {
        if app.try_exists()? {
            return Err("preserve candidate game app; continue its existing session".into());
        }
        let prior: Value = c::read_json(&root.join("fable-apu-corrected.native-started.json"))?;
        let old = std::path::PathBuf::from(prior["app"].as_str().ok_or("retained village app")?);
        let _app_lease = crate::storage::ProfileLease::acquire(&old)?;
        let library = c::library::Library::read(&old)?;
        if library.entries.len() != 1 {
            return Err("unambiguous retained village entry required".into());
        }
        let _profile_lease = crate::storage::ProfileLease::acquire(
            &old.join("profiles").join(&library.entries[0].profile),
        )?;
        let before = c::file_digest(&old.join("library.json"))?;
        c::store::checked(
            Command::new("cp")
                .args(["-a", "--reflink=always"])
                .arg(&old)
                .arg(&app),
        )?;
        if before != c::file_digest(&old.join("library.json"))? {
            return Err("retained source app changed".into());
        }
        c::write_new(
            &dir.join(if cache_check {
                "fable-cache-key-source.json"
            } else if control {
                "fable-input-control-source.json"
            } else {
                "fable-source.json"
            }),
            &c::json(
                &json!({"source":old,"candidate":app,"source_library_unchanged":true,"source_app_and_profile_lease_held":true,"copy_on_write_required":true,"game_media_copied":false,"no_checkpoint_retag":true}),
            )?,
        )?;
    }
    let r = if control {
        resources::Resources::input_control_candidate(&dir.join("input-control-package"))?
    } else {
        resources::Resources::smc_candidate(&dir.join("package"))?
    };
    super::build::observe_local_play(
        repo,
        root,
        r,
        &format!(
            "fable-smc-{}return-{}",
            if cache_check {
                "cache-key-"
            } else if control {
                "input-control-"
            } else {
                ""
            },
            c::new_id()?
        ),
        Some(app),
        true,
    )
}
pub(super) fn core(repo: &Path, root: &Path) -> Result<(), Error> {
    build_core(repo, root, false)
}
pub(super) fn input_control_core(repo: &Path, root: &Path) -> Result<(), Error> {
    build_core(repo, root, true)
}
fn build_core(repo: &Path, root: &Path, control: bool) -> Result<(), Error> {
    use std::process::Command;
    let proof: Value = c::read_json(&root.join("smc-v1/baseline-v2/result.json"))?;
    if proof["passed"] != true || proof["core"] != resources::APU_QUIESCENT_CORE {
        return Err("completed original data and genuine-code-change controls required".into());
    }
    let base = root.join(if control {
        "smc-v1/runtime"
    } else {
        "apu-quiescence-v1/corrected-runtime-v1"
    });
    let base_core = if control {
        CORE
    } else {
        resources::APU_QUIESCENT_CORE
    };
    let original = base.join("build/qemu-system-i386");
    if c::file_digest(&original)? != base_core {
        return Err("retained base core differs".into());
    }
    let paths = [
        "accel/tcg/tb-maint.c",
        "ui/xemu-input.c",
        "monitor/hmp-target.c",
        "hmp-commands.hx",
    ];
    let mut sources = Vec::new();
    for path in paths {
        sources
            .push(json!({"path":path,"sha256":c::file_digest(&base.join("source").join(path))?}));
    }
    let dir = root.join(if control {
        "smc-v1/input-control-runtime"
    } else {
        "smc-v1/runtime"
    });
    c::mkdir(&dir)?;
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(base.join("source"))
            .arg(base.join("build"))
            .arg(&dir),
    )?;
    let mut patches = Vec::new();
    let names: &[&str] = if control {
        &["patches/xemu-smc-byte-range.patch"]
    } else {
        &[
            "patches/xemu-controller-frame.patch",
            "patches/xemu-smc-byte-range.patch",
        ]
    };
    for name in names {
        let path = repo.join(name);
        let mut cmd = Command::new("git");
        cmd.arg("-C")
            .arg(dir.join("source"))
            .env("GIT_CEILING_DIRECTORIES", dir.canonicalize()?)
            .arg("apply");
        if control {
            cmd.arg("--reverse");
        }
        c::store::checked(cmd.arg(&path))?;
        patches.push(json!({"path":name,"sha256":c::file_digest(&path)?,"reversed":control}));
    }
    if control {
        if c::file_digest(&dir.join("source/accel/tcg/tb-maint.c"))?
            != c::file_digest(
                &root.join("apu-quiescence-v1/corrected-runtime-v1/source/accel/tcg/tb-maint.c"),
            )?
        {
            return Err("input control must restore the exact retained invalidator".into());
        }
        for path in &paths[1..] {
            if c::file_digest(&dir.join("source").join(path))?
                != c::file_digest(&base.join("source").join(path))?
            {
                return Err("controller control differs outside the invalidator".into());
            }
        }
    }
    let ninja = dir.join("build/build.ninja");
    let graph = fs::read_to_string(&ninja)?;
    if graph
        .matches("build xbx-preserved-regeneration-edge: REGENERATE_BUILD ")
        .count()
        != 1
    {
        return Err("retained regeneration isolation differs".into());
    }
    let dry = Command::new("ninja")
        .current_dir(dir.join("build"))
        .args(["-n", "qemu-system-i386"])
        .env("NINJA_STATUS", "[%f/%t] ")
        .output()?;
    let plan = String::from_utf8(dry.stdout)?;
    c::write_new(&dir.join("build-plan.log"), plan.as_bytes())?;
    let steps = plan.lines().filter(|s| s.starts_with('[')).count();
    if !dry.status.success()
        || steps > 64
        || !plan.contains("tb-maint.c")
        || (!control && !plan.contains("ui_xemu-input.c"))
        || plan.to_lowercase().contains("regenerating build")
    {
        return Err("refuse broad byte-range build; preserve plan".into());
    }
    c::write_new(
        &dir.join("inputs.json"),
        &c::json(
            &json!({"base_core":base_core,"sources":sources,"patches":patches,"planned_steps":steps,"compiler_jobs":1,"new_dependencies_or_downloads":false,"new_vmstate_layout":false,"additional_peak_bound_bytes":2147483648u64}),
        )?,
    )?;
    let log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join("build.log"))?;
    let status = Command::new("timeout")
        .args([
            "--signal=TERM",
            "--kill-after=20s",
            "600s",
            "ninja",
            "-j1",
            "qemu-system-i386",
        ])
        .current_dir(dir.join("build"))
        .stdout(log.try_clone()?)
        .stderr(log)
        .status()?;
    let mut unchanged = c::file_digest(&original)? == base_core;
    for v in &sources {
        unchanged &= v["sha256"]
            == c::file_digest(
                &base
                    .join("source")
                    .join(v["path"].as_str().ok_or("source path")?),
            )?;
    }
    c::write_new(
        &dir.join("build-result.json"),
        &c::json(
            &json!({"passed":status.success(),"retained_source_and_core_unchanged":unchanged,"exit":status.code()}),
        )?,
    )?;
    if !status.success() || !unchanged {
        return Err("byte-range build failed; preserve all artifacts".into());
    }
    c::write_new(
        &dir.join("identity.json"),
        &c::json(
            &json!({"core":c::file_digest(&dir.join("build/qemu-system-i386"))?,"base_core":base_core,"open_system":resources::LOCAL_BASE_OPEN,"original_or_game_qualified":false,"builder_compiled_core_pin":CORE,"control":control}),
        )?,
    )?;
    Ok(())
}
pub(super) fn fixture(repo: &Path, root: &Path) -> Result<(), Error> {
    let parent = root.join("smc-v1");
    if !parent.try_exists()? {
        c::mkdir(&parent)?;
    }
    let dir = parent.join("original-v2");
    c::mkdir(&dir)?;
    let exports = root.join("source-inputs/exports.tsv");
    if c::file_digest(&exports)?
        != "aa5c07f0bb9a0c1053dc9aadd4d4271ba53d3a8b69c49356d8479faeb66d7e7b"
    {
        return Err("independent public physical-address ABI differs".into());
    }
    let abi = fs::read_to_string(exports)?;
    for declaration in [
        "173\tMmGetPhysicalAddress\tstdcall\t4\tfunc",
        "177\tMmMapIoSpace\tstdcall\t12\tfunc",
        "183\tMmUnmapIoSpace\tstdcall\t8\tfunc",
    ] {
        if !abi.lines().any(|v| v == declaration) {
            return Err("independent alias API declaration differs".into());
        }
    }
    let source = repo.join("native/fixtures/smc-data-range.S");
    let uart_source = repo.join("native/fixtures/startup-lifetime.S");
    let uart = fs::read_to_string(&uart_source)?;
    let uart = uart
        .split_once("serial_init:\n")
        .ok_or("UART start")?
        .1
        .split_once(".balign 4\n")
        .ok_or("UART end")?
        .0;
    let text = fs::read_to_string(&source)?;
    if text.matches("/*__XBX_ORIGINAL_UART__*/").count() != 1 {
        return Err("original UART insertion differs".into());
    }
    let generated = text.replace(
        "/*__XBX_ORIGINAL_UART__*/",
        &format!("serial_init:\n{uart}"),
    );
    let media = super::cpu_performance::assemble_two_page(
        &root.join("fixtures-v2/a.iso"),
        &dir,
        &generated,
        "xbx original byte-range control",
        0x5842534d,
    )?;
    c::write_new(
        &dir.join("build.json"),
        &c::json(&json!({
            "source":c::file_digest(&source)?,"uart_source":c::file_digest(&uart_source)?,
            "fixture":media,"iterations":10000,"original_content_only":true,
            "cases":"same-page writes outside code; genuine direct/physical-alias/cross-page/current-block code changes",
            "reference":"retained QEMU byte-range check and precise_smc behavior; public MmGetPhysicalAddress ABI",
            "commercial_acceptance":false
        }))?,
    )?;
    Ok(())
}
fn wait(player: &mut c::desktop::Player, marker: &str, log: &mut String) -> Result<(), Error> {
    let start = Instant::now();
    let mut current = String::new();
    while start.elapsed() < Duration::from_secs(30) {
        let s = player.live.as_mut().ok_or("live")?.native.serial()?;
        log.push_str(&s);
        current.push_str(&s);
        if log.len() > 32768 {
            return Err("original output bound".into());
        }
        if current.contains("XBX SMC FAIL")
            || current.contains("*** bug check")
            || current.contains("*** unexpected exception")
        {
            return Err("original byte-range functional control failed".into());
        }
        if current.contains(marker) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err("original byte-range response deadline".into())
}
fn stats(player: &mut c::desktop::Player) -> Result<Value, Error> {
    let s = player
        .live
        .as_mut()
        .ok_or("live")?
        .native
        .call("human-monitor-command", json!({"command-line":"info jit"}))?;
    let text = s.as_str().ok_or("native aggregate statistics")?;
    let count = |label: &str| -> Result<u64, Error> {
        let row = text
            .lines()
            .find_map(|l| l.trim().strip_prefix(label))
            .ok_or("native counter unavailable")?;
        Ok(row.trim().parse()?)
    };
    Ok(
        json!({"invalidations":count("TB invalidate count")?,"buffer_flushes":count("TB flush count")?}),
    )
}
pub(super) fn baseline(repo: &Path, root: &Path) -> Result<(), Error> {
    observe(repo, root, false)
}
pub(super) fn candidate(repo: &Path, root: &Path) -> Result<(), Error> {
    observe(repo, root, true)
}
fn observe(repo: &Path, root: &Path, candidate: bool) -> Result<(), Error> {
    let dir = root.join("smc-v1/original-v2");
    let built: Value = c::read_json(&dir.join("build.json"))?;
    let media = dir.join("original.iso");
    if built["fixture"] != c::file_digest(&media)?
        || built["source"] != c::file_digest(&repo.join("native/fixtures/smc-data-range.S"))?
    {
        return Err("original byte-range input differs".into());
    }
    let out = root.join(if candidate {
        "smc-v1/candidate-v2"
    } else {
        "smc-v1/baseline-v2"
    });
    c::mkdir(&out)?;
    let mut r = if candidate {
        resources::Resources::smc_candidate(&root.join("smc-v1/package"))?
    } else {
        resources::Resources::apu_quiescent(&root.join("apu-quiescence-v1/package"))?
    };
    let core = r.manifest.core.clone();
    r.window = false;
    let mut player = c::desktop::Player::with_resources(&out.join("app"), r)?;
    let game = player.add(&media)?;
    let mut run = None;
    let mut log = String::new();
    let mut phases = Vec::new();
    let observed = (|| -> Result<(), Error> {
        if player.launch(&game)? {
            return Err("fresh original profile required".into());
        }
        run = Some(player.live.as_ref().ok_or("live")?.native.run.clone());
        wait(&mut player, "XBX SMC READY", &mut log)?;
        for (command, marker) in [("c", "CONTROL"), ("d", "DATA"), ("m", "MUTATE")] {
            let before = stats(&mut player)?;
            let started = Instant::now();
            player
                .live
                .as_mut()
                .ok_or("live")?
                .native
                .fixture_input(command)?;
            wait(&mut player, &format!("XBX SMC {marker} PASS"), &mut log)?;
            phases.push(json!({"phase":marker,"before":before,"after":stats(&mut player)?,"wall_ms":started.elapsed().as_millis()}));
        }
        Ok(())
    })();
    let returned = player.stop();
    let native = match &run {
        Some(p) => c::read_json::<Value>(&p.join("native-return.json"))?,
        None => Value::Null,
    };
    let baseline: Value = if candidate {
        c::read_json(&root.join("smc-v1/baseline-v2/result.json"))?
    } else {
        Value::Null
    };
    let false_invalidations_eliminated = if candidate && phases.len() == 3 {
        let delta = |phase: &Value| -> Result<u64, Error> {
            phase["after"]["invalidations"]
                .as_u64()
                .ok_or("invalidation count")?
                .checked_sub(
                    phase["before"]["invalidations"]
                        .as_u64()
                        .ok_or("invalidation count")?,
                )
                .ok_or_else(|| "invalidation counter reversed".into())
        };
        baseline["passed"] == true && delta(&phases[1])? * 20 < delta(&baseline["phases"][1])?
    } else {
        false
    };
    let passed = observed.is_ok()
        && returned.is_ok()
        && native["clean"] == true
        && (!candidate || false_invalidations_eliminated);
    c::write_new(&out.join("serial.log"), log.as_bytes())?;
    c::write_new(
        &out.join("result.json"),
        &c::json(&json!({
        "passed":passed,"core":core,"fixture":built["fixture"],"phases":phases,"candidate":candidate,
        "false_invalidations_eliminated":false_invalidations_eliminated,
            "native_run":run,"native_return":native,"observation_error":observed.err().map(|e|e.to_string()),
            "return_error":returned.err().map(|e|e.to_string()),"generated_storage_only":true,"commercial_acceptance":false
        }))?,
    )?;
    if !passed {
        return Err(
            io::Error::other("original byte-range baseline failed; retain evidence").into(),
        );
    }
    Ok(())
}
