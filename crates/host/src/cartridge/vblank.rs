// SPDX-License-Identifier: MIT
//! Original guest PCRTC timing independent of a host presentation listener.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
pub(super) const CORE: &str = match option_env!("XBX_VBLANK_CORE_SHA256") {
    Some(v) => v,
    None => "unqualified-vblank-core",
};
pub(super) fn core(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("vblank-v1");
    let negative: Value = c::read_json(&dir.join("headless-baseline/result.json"))?;
    let positive: Value = c::read_json(&dir.join("visible-baseline/result.json"))?;
    if negative["completed"] != true
        || negative["events_per_counter_second"] != 0
        || positive["completed"] != true
        || positive["within_45_to_75_event_window"] != true
        || negative["core"] != super::vertex_state::KVM_CORE
        || positive["core"] != super::vertex_state::KVM_CORE
    {
        return Err("actual presentation-coupled original control required".into());
    }
    super::vertex_state::vblank_core(repo, root)
}
pub(super) fn package(root: &Path) -> Result<(), Error> {
    let dir = root.join("vblank-v1/runtime");
    let core = dir.join("build/qemu-system-i386");
    if c::file_digest(&core)? != CORE {
        return Err("exact separately built VBlank core required".into());
    }
    let base = root.join("vertex-state-v1/kvm-package");
    let mut manifest = resources::Resources::vertex_kvm(&base, false)?.manifest;
    let out = root.join("vblank-v1/package");
    c::mkdir(&out)?;
    fs::create_dir_all(out.join("native/subprojects/slirp"))?;
    for (a, b) in [
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
                .arg(a)
                .arg(b),
        )?;
    }
    manifest.core = CORE.into();
    c::write_new(&out.join("runtime.json"), &c::json(&manifest)?)?;
    resources::Resources::vblank(&out, false)?;
    Ok(())
}
pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("vblank-v1");
    c::mkdir(&dir)?;
    let exports = root.join("source-inputs/exports.tsv");
    if c::file_digest(&exports)?
        != "aa5c07f0bb9a0c1053dc9aadd4d4271ba53d3a8b69c49356d8479faeb66d7e7b"
    {
        return Err("public ABI declaration differs".into());
    }
    let abi = fs::read_to_string(&exports)?;
    for line in [
        "126\tKeQueryPerformanceCounter\tstdcall\t0\tfunc",
        "127\tKeQueryPerformanceFrequency\tstdcall\t0\tfunc",
        "177\tMmMapIoSpace\tstdcall\t12\tfunc",
    ] {
        if !abi.lines().any(|v| v == line) {
            return Err("original PCRTC control ABI differs".into());
        }
    }
    let source = repo.join("native/fixtures/vblank-cadence.S");
    let uart_source = repo.join("native/fixtures/startup-lifetime.S");
    let uart = fs::read_to_string(&uart_source)?;
    let uart = uart
        .split_once("serial_init:\n")
        .ok_or("UART start")?
        .1
        .split_once(".balign 4\n")
        .ok_or("UART end")?
        .0;
    let generated = format!(
        "{}\n.text\nserial_init:\n{uart}",
        fs::read_to_string(&source)?
    );
    let fixture = super::cpu_performance::assemble_two_page(
        &root.join("fixtures-v2/a.iso"),
        &dir,
        &generated,
        "xbx original PCRTC cadence",
        0x58425642,
    )?;
    c::write_new(
        &dir.join("build.json"),
        &c::json(
            &json!({"source":c::file_digest(&source)?,"uart_source":c::file_digest(&uart_source)?,"fixture":fixture,"original_only":true,"contract":"PCRTC VBlank status events continue independently of a host presentation listener; event count per one 3375000Hz-counter second","guest_irq_delivery_disabled_during_count":true,"commercial_acceptance":false}),
        )?,
    )?;
    Ok(())
}
fn wait(player: &mut c::desktop::Player, marker: &str, log: &mut String) -> Result<(), Error> {
    let deadline = Instant::now() + Duration::from_secs(12);
    loop {
        log.push_str(
            &player
                .live
                .as_mut()
                .ok_or("live original")?
                .native
                .serial()?,
        );
        if log.len() > 8192
            || log.contains("XBX VBLANK FAIL")
            || log.contains("*** bug check")
            || log.contains("*** unexpected exception")
        {
            return Err("original PCRTC startup failed".into());
        }
        if log.contains(marker) {
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err("original PCRTC response deadline".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
pub(super) fn baseline(repo: &Path, root: &Path, visible: bool) -> Result<(), Error> {
    observe(repo, root, visible, false, false)
}
pub(super) fn candidate(
    repo: &Path,
    root: &Path,
    visible: bool,
    returning: bool,
) -> Result<(), Error> {
    observe(repo, root, visible, true, returning)
}
fn observe(
    repo: &Path,
    root: &Path,
    visible: bool,
    corrected: bool,
    returning: bool,
) -> Result<(), Error> {
    let dir = root.join("vblank-v1");
    let built: Value = c::read_json(&dir.join("build.json"))?;
    if built["source"] != c::file_digest(&repo.join("native/fixtures/vblank-cadence.S"))?
        || built["fixture"] != c::file_digest(&dir.join("original.iso"))?
    {
        return Err("original PCRTC input changed".into());
    }
    let label = if visible { "visible" } else { "headless" };
    let out = dir.join(if corrected {
        format!(
            "{label}-corrected-{}",
            if returning { "return" } else { "first" }
        )
    } else {
        format!("{label}-baseline")
    });
    c::mkdir(&out)?;
    let mut r = if corrected {
        resources::Resources::vblank(&dir.join("package"), false)?
    } else {
        resources::Resources::vertex_kvm(&root.join("vertex-state-v1/kvm-package"), false)?
    };
    r.window = visible;
    let core = r.manifest.core.clone();
    let app = if corrected {
        dir.join(format!("{label}-corrected-app"))
    } else {
        out.join("app")
    };
    let mut player = c::desktop::Player::with_resources(&app, r)?;
    let game = if returning {
        player.last_game()?.ok_or("retained original")?
    } else {
        player.add(&dir.join("original.iso"))?
    };
    let mut run = None;
    let mut log = String::new();
    let mut count = None;
    let mut query_ms = None;
    let observed = (|| -> Result<(), Error> {
        if player.launch(&game)? != returning {
            return Err("PCRTC cold/return boundary differs".into());
        }
        run = Some(player.live.as_ref().ok_or("live")?.native.run.clone());
        if !returning {
            wait(&mut player, "XBX VBLANK READY", &mut log)?;
        }
        let query_start = Instant::now();
        player
            .live
            .as_mut()
            .ok_or("live")?
            .native
            .fixture_input("v")?;
        wait(&mut player, "XBX VBLANK COUNT=", &mut log)?;
        // The UART may split immediately after the prefix; collect its newline.
        let deadline = Instant::now() + Duration::from_secs(3);
        while !log
            .split_once("XBX VBLANK COUNT=")
            .ok_or("count")?
            .1
            .contains('\n')
        {
            log.push_str(&player.live.as_mut().ok_or("live")?.native.serial()?);
            if Instant::now() > deadline {
                return Err("original PCRTC count incomplete".into());
            }
        }
        let hex = log
            .split_once("XBX VBLANK COUNT=")
            .ok_or("count")?
            .1
            .lines()
            .next()
            .ok_or("value")?;
        count = Some(u32::from_str_radix(hex.trim(), 16)?);
        query_ms = Some(query_start.elapsed().as_millis());
        if returning && query_ms.is_some_and(|v| v < 500) {
            return Err("counter reply arrived too early for a fresh one-second query".into());
        }
        Ok(())
    })();
    let returned = if corrected && observed.is_ok() {
        player.depart()
    } else {
        player.stop()
    };
    let cleanup = if returned.is_err() {
        super::build::retire_failed_observation(&mut player)
    } else {
        None
    };
    let native = match run {
        Some(p) => c::read_json::<Value>(&p.join("native-return.json"))?,
        None => Value::Null,
    };
    let completed = observed.is_ok() && returned.is_ok() && native["clean"] == true;
    let passed = completed && (!corrected || count.is_some_and(|v| (45..=75).contains(&v)));
    c::write_new(
        &out.join("result.json"),
        &c::json(
            &json!({"passed":passed,"completed":completed,"visible":visible,"corrected":corrected,"restored":returning,"core":core,"query_ms":query_ms,"events_per_counter_second":count,"within_45_to_75_event_window":count.is_some_and(|v|(45..=75).contains(&v)),"native_return":native,"cleanup":cleanup,"observation_error":observed.err().map(|e|e.to_string()),"return_error":returned.err().map(|e|e.to_string()),"guest_output":log,"original_only":true,"commercial_acceptance":false}),
        )?,
    )?;
    if !passed {
        return Err("original PCRTC control incomplete; preserve evidence".into());
    }
    Ok(())
}
fn require(root: &Path, relative: &str) -> Result<(), Error> {
    let v: Value = c::read_json(&root.join("vblank-v1").join(relative).join("result.json"))?;
    if v["passed"] != true || v["core"] != CORE {
        return Err("exact corrected original gate required".into());
    }
    Ok(())
}
pub(super) fn device(root: &Path, returning: bool) -> Result<(), Error> {
    require(root, "headless-corrected-return")?;
    require(root, "visible-corrected-first")?;
    let dir = root.join("vblank-v1");
    let r = resources::Resources::vblank(&dir.join("package"), true)?;
    super::vertex_state::device_at(
        root,
        r,
        &dir.join("gpu-app"),
        &dir.join(if returning { "gpu-return" } else { "gpu-first" }),
        true,
        returning,
    )
}
pub(super) fn original(root: &Path, returning: bool) -> Result<(), Error> {
    require(root, "gpu-return")?;
    let dir = root.join("vblank-v1");
    let r = resources::Resources::vblank(&dir.join("package"), false)?;
    if !returning {
        return super::host_input::baseline_at(root, r, &dir.join("cpu-first"));
    }
    let out = dir.join("cpu-return");
    c::mkdir(&out)?;
    let old = dir.join("cpu-first/app");
    let app = dir.join("cpu-return-app");
    let copy = super::vertex_state::clone_app(&old, &app)?;
    c::write_new(&out.join("source.json"), &c::json(&copy)?)?;
    let library = c::file_digest(&old.join("library.json"))?;
    super::host_input::original_at(&old, &app, &out, r, CORE, &library, false)
}
pub(super) fn fable(repo: &Path, root: &Path, returning: bool) -> Result<(), Error> {
    require(root, "cpu-return")?;
    let r = resources::Resources::vblank(&root.join("vblank-v1/package"), false)?;
    let app = if returning {
        let v: Value = c::read_json(&root.join("fable-vblank.native-started.json"))?;
        Some(v["app"].as_str().ok_or("retained corrected app")?.into())
    } else {
        None
    };
    let prefix = if returning {
        format!("fable-vblank-return-{}", c::new_id()?)
    } else {
        "fable-vblank".into()
    };
    super::build::observe_local_play(repo, root, r, &prefix, app, returning)
}
