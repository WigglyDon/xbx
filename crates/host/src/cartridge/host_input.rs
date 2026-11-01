// SPDX-License-Identifier: MIT
//! Rust-owned complete controller frames; the native adapter copies the existing
//! Xbox controller representation under BQL. No desktop key injection or guest
//! memory access. Qualification reuses retained builds and original fixtures.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{
    fs, io,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
pub(super) const CORE: &str = match option_env!("XBX_CONTROLLER_CORE_SHA256") {
    Some(v) => v,
    None => "unqualified-controller-build",
};
pub(super) const TTL_MS: u64 = 400;
/// SDL indices in the existing Rust/Android Pad, in native Xbox bit order.
const BUTTONS: [u8; 14] = [0, 1, 2, 3, 13, 11, 14, 12, 4, 6, 9, 10, 7, 8];
pub(super) fn frame(p: crate::input::Pad) -> io::Result<(u16, [i16; 6])> {
    let allowed = BUTTONS.iter().fold(0u32, |v, b| v | (1 << b));
    if p.buttons & !allowed != 0 || p.axes[4] < 0 || p.axes[5] < 0 {
        return Err(io::Error::other(
            "unsupported controller button or trigger range",
        ));
    }
    let mut buttons = 0u16;
    for (i, b) in BUTTONS.iter().enumerate() {
        if p.buttons & (1 << b) != 0 {
            buttons |= 1 << i;
        }
    }
    // The retained SDL adapter reverses Y. Keep the exact signed endpoints and
    // a true neutral center when adapting the shared Rust Pad to Xbox axes.
    let invert = |v: i16| if v == 0 { 0 } else { -1 - v };
    Ok((
        buttons,
        [
            p.axes[4],
            p.axes[5],
            p.axes[0],
            invert(p.axes[1]),
            p.axes[2],
            invert(p.axes[3]),
        ],
    ))
}
pub(super) fn pulse(
    player: &mut c::desktop::Player,
    pad: crate::input::Pad,
    duration: Duration,
) -> io::Result<()> {
    if duration > Duration::from_secs(5) {
        return Err(io::Error::other("controller pulse bound"));
    }
    let start = Instant::now();
    let applied = (|| -> io::Result<()> {
        while start.elapsed() < duration {
            player.input(pad)?;
            std::thread::sleep(
                Duration::from_millis(80).min(duration.saturating_sub(start.elapsed())),
            );
        }
        Ok(())
    })();
    let neutral = player.input(crate::input::Pad::default());
    applied.and(neutral)
}
pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let base = root.join("host-kvm-v1");
    if c::file_digest(&base.join("build/qemu-system-i386"))? != resources::HOST_KVM_CORE {
        return Err("retained qualified core differs".into());
    }
    let dir = root.join("host-input-v1");
    c::mkdir(&dir)?;
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(base.join("source"))
            .arg(base.join("build"))
            .arg(&dir),
    )?;
    let paths = ["ui/xemu-input.c", "monitor/hmp-target.c", "hmp-commands.hx"];
    let mut before = Vec::new();
    for p in paths {
        before.push(json!({"path":p,"sha256":c::file_digest(&base.join("source").join(p))?}));
    }
    let patch = repo.join("patches/xemu-controller-frame.patch");
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(dir.join("source"))
            .env("GIT_CEILING_DIRECTORIES", dir.canonicalize()?)
            .arg("apply")
            .arg(&patch),
    )?;
    let ninja = dir.join("build/build.ninja");
    let graph = fs::read_to_string(&ninja)?;
    let edge = "build build.ninja: REGENERATE_BUILD ";
    if graph.matches(edge).count() != 1 {
        return Err("unexpected retained regeneration edge".into());
    }
    fs::write(
        &ninja,
        graph.replacen(
            edge,
            "build xbx-preserved-regeneration-edge: REGENERATE_BUILD ",
            1,
        ),
    )?;
    let build = dir.join("build");
    let dry = Command::new("ninja")
        .current_dir(&build)
        .args(["-n", "qemu-system-i386"])
        .env("NINJA_STATUS", "[%f/%t] ")
        .output()?;
    let plan = String::from_utf8(dry.stdout)?;
    c::write_new(&dir.join("build-plan.log"), plan.as_bytes())?;
    let steps = plan.lines().filter(|s| s.starts_with('[')).count();
    c::write_new(
        &dir.join("inputs.json"),
        &c::json(
            &json!({"base_core":resources::HOST_KVM_CORE,"patch":c::file_digest(&patch)?,"changed_sources":before,"planned_steps":steps,"no_machine_state_layout_changes":true,"no_new_dependencies_or_downloads":true,"additional_peak_bound_bytes":2147483648u64,"compiler_jobs":1}),
        )?,
    )?;
    if !dry.status.success()
        || steps > 64
        || !plan.contains("ui_xemu-input.c.o")
        || plan.to_ascii_lowercase().contains("regenerating build")
    {
        return Err("refuse broad or regenerating controller build; preserve plan".into());
    }
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
        .current_dir(&build)
        .stdout(log.try_clone()?)
        .stderr(log)
        .status()?;
    let mut unchanged =
        c::file_digest(&base.join("build/qemu-system-i386"))? == resources::HOST_KVM_CORE;
    for v in &before {
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
            &json!({"passed":status.success(),"exit":status.code(),"retained_core_and_sources_unchanged":unchanged}),
        )?,
    )?;
    if !status.success() || !unchanged {
        return Err("controller build failed; preserve all intermediates".into());
    }
    c::write_new(
        &dir.join("identity.json"),
        &c::json(
            &json!({"core":c::file_digest(&build.join("qemu-system-i386"))?,"base_core":resources::HOST_KVM_CORE,"open_system":resources::LOCAL_BASE_OPEN,"patch":c::file_digest(&patch)?,"original_or_game_qualified":false}),
        )?,
    )?;
    Ok(())
}

pub(super) fn package(root: &Path) -> Result<(), Error> {
    let dir = root.join("host-input-v1");
    let v: Value = c::read_json(&dir.join("identity.json"))?;
    let core = dir.join("build/qemu-system-i386");
    if v["core"] != CORE || c::file_digest(&core)? != CORE {
        return Err("actual bound controller core required".into());
    }
    let base = root.join("host-kvm-v1/package");
    let old = resources::Resources::host_kvm(&base)?;
    let target = dir.join("package-signed");
    c::mkdir(&target)?;
    fs::create_dir_all(target.join("native/subprojects/slirp"))?;
    for (a, b) in [
        (core, target.join("native/core")),
        (base.join("open.bin"), target.join("open.bin")),
        (base.join("native/qemu-img"), target.join("native/qemu-img")),
        (
            base.join("native/subprojects/slirp/libslirp.so.0"),
            target.join("native/subprojects/slirp/libslirp.so.0"),
        ),
    ] {
        c::store::checked(
            Command::new("cp")
                .args(["-a", "--reflink=always"])
                .arg(a)
                .arg(b),
        )?;
    }
    let mut manifest = old.manifest;
    manifest.core = CORE.into();
    c::write_new(&target.join("runtime.json"), &c::json(&manifest)?)?;
    let new = resources::Resources::controller_candidate(&target)?;
    if new.identity != old.identity {
        return Err("explicit state lineage differs".into());
    }
    Ok(())
}
fn wait(player: &mut c::desktop::Player, expected: &str, log: &mut String) -> Result<(), Error> {
    let start = Instant::now();
    let mut text = String::new();
    while start.elapsed() < Duration::from_secs(30) {
        let next = player
            .live
            .as_mut()
            .ok_or("original live session")?
            .native
            .serial()?;
        text.push_str(&next);
        log.push_str(&next);
        if log.len() > 65536 {
            return Err("original serial output bound".into());
        }
        if text.contains(expected) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Err("original restore response deadline".into())
}
pub(super) fn baseline(root: &Path) -> Result<(), Error> {
    let out = root.join("host-input-v1/visible-baseline");
    let r = resources::Resources::host_kvm(&root.join("host-kvm-v1/package"))?;
    baseline_at(root, r, &out)
}
pub(super) fn baseline_at(root: &Path, r: resources::Resources, out: &Path) -> Result<(), Error> {
    c::mkdir(out)?;
    let core = r.manifest.core.clone();
    let mut player = c::desktop::Player::with_resources(&out.join("app"), r)?;
    let media = root.join("fixtures-v2/a.iso");
    if c::file_digest(&media)? != resources::ORIGINAL_FIXTURES[0] {
        return Err("retained original input differs".into());
    }
    let game = player.add(&media)?;
    let mut run = None;
    let mut log = String::new();
    let observed = (|| -> Result<(), Error> {
        if player.launch(&game)? {
            return Err("fresh visible baseline required".into());
        }
        run = Some(player.live.as_ref().ok_or("live")?.native.run.clone());
        wait(&mut player, "CART A state=00000000 save=00000000", &mut log)?;
        player
            .live
            .as_mut()
            .ok_or("live")?
            .native
            .fixture_input("as a")?;
        wait(
            &mut player,
            "CART A state=00000002 save=00000001 cookie=57F090E4",
            &mut log,
        )?;
        Ok(())
    })();
    let returned = if observed.is_ok() {
        player.depart()
    } else {
        player.stop()
    };
    let native: Value = match &run {
        Some(p) => c::read_json(&p.join("native-return.json"))?,
        None => Value::Null,
    };
    let passed = observed.is_ok() && returned.is_ok() && native["clean"] == true;
    c::write_new(
        &out.join("result.json"),
        &c::json(
            &json!({"passed":passed,"core":core,"controller_topology":"visible Controller S; fresh generated original storage","native_run":run,"native_return":native,"guest_output":log,"observation_error":observed.err().map(|e|e.to_string()),"return_error":returned.err().map(|e|e.to_string())}),
        )?,
    )?;
    if !passed {
        return Err("original visible controller baseline failed".into());
    }
    Ok(())
}
pub(super) fn original(root: &Path, returning: bool) -> Result<(), Error> {
    let dir = root.join("host-input-v1");
    let out = dir.join(if returning {
        "original-visible-return-v3"
    } else {
        "original-visible-cross-core-v3"
    });
    c::mkdir(&out)?;
    let baseline: Value = c::read_json(&dir.join("visible-baseline/result.json"))?;
    if baseline["passed"] != true || baseline["core"] != resources::HOST_KVM_CORE {
        return Err("matched Controller S original baseline required".into());
    }
    let old = dir.join("visible-baseline/app");
    let app = dir.join("original-visible-app-v3");
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
        let first: Value = c::read_json(&dir.join("original-visible-cross-core-v3/result.json"))?;
        if first["passed"] != true || first["core"] != CORE {
            return Err("actual cross-core original proof required".into());
        }
    }
    let r = resources::Resources::controller_candidate(&dir.join("package-signed"))?;
    original_at(
        &old,
        &app,
        &out,
        r,
        resources::HOST_KVM_CORE,
        &old_library,
        returning,
    )
}
pub(super) fn original_at(
    old: &Path,
    app: &Path,
    out: &Path,
    r: resources::Resources,
    base_core: &str,
    old_library: &str,
    returning: bool,
) -> Result<(), Error> {
    let core = r.manifest.core.clone();
    let mut player = c::desktop::Player::with_resources(app, r)?;
    let game = player.last_game()?.ok_or("retained original entry")?;
    let mut log = String::new();
    let mut run = None;
    let observed = (|| -> Result<(), Error> {
        if !player.launch(&game)? {
            return Err("actual complete old checkpoint restoration required".into());
        }
        run = Some(player.live.as_ref().ok_or("live")?.native.run.clone());
        player
            .live
            .as_mut()
            .ok_or("live")?
            .native
            .fixture_input("?")?;
        wait(
            &mut player,
            if returning {
                "CART A state=00000003 save=00000001 cookie=6B482CC3"
            } else {
                "CART A state=00000002 save=00000001 cookie=57F090E4"
            },
            &mut log,
        )?;
        let pad = crate::input::Pad {
            buttons: (1 << 6) | (1 << 0),
            axes: [-32768, 32767, 1234, -2345, 32767, 16384],
        };
        player.input(pad)?;
        let native = &mut player.live.as_mut().ok_or("live")?.native;
        let (bits, axes) = frame(pad)?;
        let applied = native.input_state()?;
        if applied["buttons"] != bits || applied["axes"] != json!(axes) {
            return Err("native full-frame mapping differs".into());
        }
        // No further renewal: loss of the producer must neutralize every channel.
        std::thread::sleep(Duration::from_millis(TTL_MS + 100));
        let expired = native.input_state()?;
        if expired["buttons"] != 0 || expired["axes"] != json!([0, 0, 0, 0, 0, 0]) {
            return Err("expired input did not neutralize".into());
        }
        native.input(pad)?;
        let invalid = native.call(
            "human-monitor-command",
            json!({"command-line":"xbx-pad-frame 16384 0 0 0 0 0 0 400"}),
        )?;
        if !invalid.as_str().is_some_and(|s| s.contains("invalid"))
            || native.input_state()?["buttons"] != bits
        {
            return Err("invalid native frame mutated accepted input".into());
        }
        player.pause(true)?;
        if player.input(pad).is_ok() {
            return Err("paused product session accepted input".into());
        }
        if player.live.as_mut().ok_or("live")?.native.input_state()?["buttons"] != 0 {
            return Err("pause did not release controller".into());
        }
        player.resume()?;
        c::write_new(
            &out.join("host-controller.json"),
            &c::json(
                &json!({"applied":applied,"expired":expired,"invalid_refused":true,"pause_neutralized":true,"paused_product_input_refused":true}),
            )?,
        )?;
        if !returning {
            player
                .live
                .as_mut()
                .ok_or("live")?
                .native
                .fixture_input("a")?;
            wait(
                &mut player,
                "CART A state=00000003 save=00000001 cookie=6B482CC3",
                &mut log,
            )?;
        }
        Ok(())
    })();
    let returned = if observed.is_ok() {
        player.depart()
    } else {
        player.stop()
    };
    c::write_new(&out.join("serial.log"), log.as_bytes())?;
    let native: Value = match &run {
        Some(p) => c::read_json(&p.join("native-return.json"))?,
        None => Value::Null,
    };
    let preserved = old_library == c::file_digest(&old.join("library.json"))?;
    let passed = observed.is_ok() && returned.is_ok() && native["clean"] == true && preserved;
    c::write_new(
        &out.join("result.json"),
        &c::json(
            &json!({"passed":passed,"core":core,"base_state_core":base_core,"runtime":player.resources.identity,"returning":returning,"native_run":run,"native_return":native,"old_library_unchanged":preserved,"checkpoint_metadata_retagged":false,"guest_output":log,"observation_error":observed.err().map(|e|e.to_string()),"return_error":returned.err().map(|e|e.to_string()),"game_qualified":false}),
        )?,
    )?;
    if !passed {
        return Err(
            "original controller/state compatibility proof failed; preserve all data".into(),
        );
    }
    Ok(())
}

pub(super) fn fable(repo: &Path, root: &Path, vsync: bool) -> Result<(), Error> {
    for name in [
        "original-visible-cross-core-v3",
        "original-visible-return-v3",
    ] {
        let v: Value = c::read_json(&root.join("host-input-v1").join(name).join("result.json"))?;
        if v["passed"] != true
            || v["core"] != CORE
            || v["base_state_core"] != resources::HOST_KVM_CORE
        {
            return Err("native cross-core and fresh-process original proofs required".into());
        }
    }
    let prior: Value = c::read_json(&root.join("fable-host-kvm.native-started.json"))?;
    let app = prior["app"].as_str().ok_or("retained KVM Fable app")?;
    let mut r =
        resources::Resources::controller_candidate(&root.join("host-input-v1/package-signed"))?;
    r.presentation_vsync = vsync;
    super::build::observe_local_play(
        repo,
        root,
        r,
        &format!("fable-controller-vsync{}-{}", u8::from(vsync), c::new_id()?),
        Some(app.into()),
        true,
    )
}

pub(super) fn fable_xbox_tsc(repo: &Path, root: &Path, returning: bool) -> Result<(), Error> {
    let mut r =
        resources::Resources::controller_candidate(&root.join("host-input-v1/package-signed"))?;
    r.select_xbox_tsc();
    r.presentation_vsync = false;
    for name in ["fixed-kvm-cold", "fixed-kvm-return"] {
        let v: Value = c::read_json(&root.join("host-tsc-v1").join(name).join("result.json"))?;
        if v["passed"] != true
            || v["core"] != CORE
            || v["fixed_xbox_tsc"] != true
            || v["runtime"] != r.identity
            || v["actual_tsc"]["actual_hz"] != 733333000u64
        {
            return Err(
                "actual fixed-clock original and fresh-process return proofs required".into(),
            );
        }
    }
    let original: Value =
        c::read_json(&root.join("host-input-v1/original-visible-return-v3/result.json"))?;
    if original["passed"] != true || original["core"] != CORE {
        return Err("qualified native controller transport required".into());
    }
    let app = if returning {
        let prior: Value = c::read_json(&root.join("fable-controller-tsc.native-started.json"))?;
        Some(
            prior["app"]
                .as_str()
                .ok_or("retained fixed-clock app")?
                .into(),
        )
    } else {
        if root
            .join("fable-controller-tsc.native-started.json")
            .exists()
        {
            return Err("fixed-clock game already started; continue its retained profile".into());
        }
        None
    };
    super::build::observe_local_play(
        repo,
        root,
        r,
        &if returning {
            format!("fable-controller-tsc-return-{}", c::new_id()?)
        } else {
            "fable-controller-tsc".into()
        },
        app,
        returning,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_pad_maps_every_xbox_button_and_preserves_analog_channels() {
        for (bit, sdl) in BUTTONS.iter().enumerate() {
            let p = crate::input::Pad {
                buttons: 1 << sdl,
                ..Default::default()
            };
            assert_eq!(frame(p).unwrap().0, 1 << bit);
        }
        assert_eq!(frame(crate::input::Pad::default()).unwrap(), (0, [0; 6]));
        let p = crate::input::Pad {
            buttons: 0,
            axes: [-32768, 32767, 123, -32768, 32767, 17000],
        };
        assert_eq!(
            frame(p).unwrap().1,
            [32767, 17000, -32768, -32768, 123, 32767]
        );
        assert!(
            frame(crate::input::Pad {
                buttons: 1 << 31,
                ..Default::default()
            })
            .is_err()
        );
        assert!(
            frame(crate::input::Pad {
                axes: [0, 0, 0, 0, -1, 0],
                ..Default::default()
            })
            .is_err()
        );
    }
}
