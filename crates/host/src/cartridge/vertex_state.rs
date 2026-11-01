// SPDX-License-Identifier: MIT
//! Existing NV2A serializer repair and original fresh-process product proof.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
pub(super) const CORE: &str = match option_env!("XBX_VERTEX_CORE_SHA256") {
    Some(v) => v,
    None => "unqualified-vertex-core",
};
pub(super) const BASE: &str = match option_env!("XBX_VERTEX_BASE_SHA256") {
    Some(v) => v,
    None => "unqualified-vertex-base",
};
pub(super) fn core(repo: &Path, root: &Path, corrected: bool) -> Result<(), Error> {
    core_variant(repo, root, if corrected { 2 } else { 0 })
}
pub(super) fn prelaunch_core(repo: &Path, root: &Path) -> Result<(), Error> {
    core_variant(repo, root, 1)
}
fn core_variant(repo: &Path, root: &Path, variant: u8) -> Result<(), Error> {
    let corrected = variant == 2;
    fs::create_dir_all(root.join("vertex-state-v1"))?;
    let base = root.join(match variant {
        2 => "vertex-state-v1/prelaunch-runtime",
        1 => "vertex-state-v1/base-runtime",
        _ => "smc-v1/runtime",
    });
    let old_core = c::file_digest(&base.join("build/qemu-system-i386"))?;
    let expected = if variant > 0 {
        let v: Value = c::read_json(&base.join("identity.json"))?;
        v["core"].as_str().ok_or("base core")?.to_owned()
    } else {
        super::smc::CORE.to_owned()
    };
    if old_core != expected {
        return Err("retained vertex base differs".into());
    }
    let unit = "hw/xbox/nv2a/nv2a.c";
    let old_source = c::file_digest(&base.join("source").join(unit))?;
    let dir = root.join(match variant {
        2 => "vertex-state-v1/runtime-v2",
        1 => "vertex-state-v1/prelaunch-runtime",
        _ => "vertex-state-v1/base-runtime",
    });
    c::mkdir(&dir)?;
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(base.join("source"))
            .arg(base.join("build"))
            .arg(&dir),
    )?;
    let patch = repo.join(match variant {
        2 => "patches/xemu-vertex-state.patch",
        1 => "patches/xemu-prelaunch-savevm.patch",
        _ => "patches/xemu-vertex-fixture.patch",
    });
    if variant == 0 {
        fs::copy(
            repo.join("native/fixtures/nv2a-vertex-state.inc"),
            dir.join("source/hw/xbox/nv2a/xbx-original-vertex-fixture.inc"),
        )?;
    }
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(dir.join("source"))
            .env("GIT_CEILING_DIRECTORIES", dir.canonicalize()?)
            .arg("apply")
            .arg(&patch),
    )?;
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
        || !plan.contains(if variant == 1 {
            "migration-hmp-cmds.c"
        } else {
            "nv2a.c"
        })
        || plan.to_lowercase().contains("regenerating build")
    {
        return Err("refuse broad vertex build; preserve plan".into());
    }
    c::write_new(
        &dir.join("inputs.json"),
        &c::json(
            &json!({"base_core":old_core,"base_source":old_source,"patch":c::file_digest(&patch)?,"fixture_source":c::file_digest(&repo.join("native/fixtures/nv2a-vertex-state.inc"))?,"planned_steps":steps,"compiler_jobs":1,"new_dependencies_or_downloads":false,"state_layout_changed":corrected,"additional_peak_bound_bytes":2147483648u64}),
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
    let unchanged = c::file_digest(&base.join("build/qemu-system-i386"))? == old_core
        && c::file_digest(&base.join("source").join(unit))? == old_source;
    c::write_new(
        &dir.join("build-result.json"),
        &c::json(
            &json!({"passed":status.success(),"retained_source_and_core_unchanged":unchanged,"exit":status.code()}),
        )?,
    )?;
    if !status.success() || !unchanged {
        return Err("vertex build failed; preserve artifacts".into());
    }
    c::write_new(
        &dir.join("identity.json"),
        &c::json(
            &json!({"core":c::file_digest(&dir.join("build/qemu-system-i386"))?,"base_core":old_core,"open_system":resources::LOCAL_BASE_OPEN,"state_layout_changed":corrected,"qualified":false}),
        )?,
    )?;
    Ok(())
}
pub(super) fn package(root: &Path, corrected: bool) -> Result<(), Error> {
    let dir = root.join(if corrected {
        "vertex-state-v1/runtime-v2"
    } else {
        "vertex-state-v1/prelaunch-runtime"
    });
    let pin = if corrected { CORE } else { BASE };
    if c::file_digest(&dir.join("build/qemu-system-i386"))? != pin {
        return Err("exact compiled vertex pin required".into());
    }
    let old_root = root.join("smc-v1/package");
    let mut manifest = resources::Resources::smc_fresh(&old_root)?.manifest;
    let out = root.join(if corrected {
        "vertex-state-v1/package-v2"
    } else {
        "vertex-state-v1/base-package-v2"
    });
    c::mkdir(&out)?;
    fs::create_dir_all(out.join("native/subprojects/slirp"))?;
    for (from, to) in [
        (dir.join("build/qemu-system-i386"), out.join("native/core")),
        (old_root.join("open.bin"), out.join("open.bin")),
        (
            old_root.join("native/qemu-img"),
            out.join("native/qemu-img"),
        ),
        (
            old_root.join("native/subprojects/slirp/libslirp.so.0"),
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
    manifest.core = pin.into();
    c::write_new(&out.join("runtime.json"), &c::json(&manifest)?)?;
    resources::Resources::vertex_candidate(&out, corrected, false)?;
    Ok(())
}
fn resources(root: &Path, corrected: bool, fixture: bool) -> Result<resources::Resources, Error> {
    Ok(resources::Resources::vertex_candidate(
        &root.join(if corrected {
            "vertex-state-v1/package-v2"
        } else {
            "vertex-state-v1/base-package-v2"
        }),
        corrected,
        fixture,
    )?)
}
pub(super) fn observe(root: &Path, corrected: bool, returning: bool) -> Result<(), Error> {
    let dir = root.join("vertex-state-v1");
    let label = if corrected {
        "corrected"
    } else {
        "baseline-v4"
    };
    let out = dir.join(format!(
        "{label}-{}",
        if returning { "return" } else { "first" }
    ));
    c::mkdir(&out)?;
    if corrected {
        let red: Value = c::read_json(&dir.join("baseline-v4-return/result.json"))?;
        if red["passed"] != true || red["missing_state_reproduced"] != true {
            return Err("actual original negative required".into());
        }
    }
    let media = root.join("fixtures-v2/a.iso");
    if c::file_digest(&media)? != resources::ORIGINAL_FIXTURES[0] {
        return Err("original fixture changed".into());
    }
    let r = resources(root, corrected, true)?;
    let core = r.manifest.core.clone();
    let runtime = r.identity.clone();
    let mut player = c::desktop::Player::with_resources(&dir.join(format!("{label}-app")), r)?;
    let game = if returning {
        player.last_game()?.ok_or("original entry")?
    } else {
        player.add(&media)?
    };
    let mut state = Value::Null;
    let mut run = None;
    let observed = (|| -> Result<(), Error> {
        if player.launch_quiesced(&game)? != returning {
            return Err("original cold/restore boundary differs".into());
        }
        let native = &mut player.live.as_mut().ok_or("live")?.native;
        run = Some(native.run.clone());
        let action = if returning { "check" } else { "seed" };
        let v = native.call(
            "human-monitor-command",
            json!({"command-line":format!("xbx-original-vertex {action}")}),
        )?;
        state = serde_json::from_str(v.as_str().ok_or("fixture response")?)?;
        let expected = corrected || !returning;
        if state["passed"] != expected
            || (!expected && (state["vertex_bytes"] != 0 || state["vertices"] != 3))
        {
            return Err("original vertex-state expectation failed".into());
        }
        Ok(())
    })();
    if observed.is_ok() && std::env::var("XBX_VERTEX_DIAGNOSTIC_HOLD").as_deref() == Ok("1") {
        c::write_new(
            &out.join("diagnostic-ready.json"),
            &c::json(&json!({"native_run":run,"state":state}))?,
        )?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        while !out.join("diagnostic-continue").is_file() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    let returned = if observed.is_ok() && (corrected || !returning) {
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
    let passed = observed.is_ok() && returned.is_ok() && native["clean"] == true;
    c::write_new(
        &out.join("result.json"),
        &c::json(
            &json!({"passed":passed,"corrected":corrected,"returning":returning,"core":core,"runtime":runtime,"state":state,"missing_state_reproduced":!corrected&&returning&&passed,"native_return":native,"cleanup":cleanup,"observation_error":observed.err().map(|e|e.to_string()),"return_error":returned.err().map(|e|e.to_string()),"original_only":true,"native_serializer_used":true,"commercial_acceptance":false}),
        )?,
    )?;
    if !passed {
        return Err("original vertex-state proof failed; preserve evidence".into());
    }
    Ok(())
}
pub(super) fn original(root: &Path, returning: bool) -> Result<(), Error> {
    let dir = root.join("vertex-state-v1");
    let proof: Value = c::read_json(&dir.join("corrected-return/result.json"))?;
    if proof["passed"] != true {
        return Err("device-state return gate required".into());
    }
    let r = resources(root, true, false)?;
    if !returning {
        return super::host_input::baseline_at(root, r, &dir.join("original-first"));
    }
    let out = dir.join("original-return");
    c::mkdir(&out)?;
    let old = dir.join("original-first/app");
    let app = dir.join("original-return-app");
    let _lease = crate::storage::ProfileLease::acquire(&old)?;
    let library = c::file_digest(&old.join("library.json"))?;
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(&old)
            .arg(&app),
    )?;
    super::host_input::original_at(&old, &app, &out, r, CORE, &library, false)
}
pub(super) fn fable(repo: &Path, root: &Path, returning: bool) -> Result<(), Error> {
    let proof: Value = c::read_json(&root.join("vertex-state-v1/original-return/result.json"))?;
    if proof["passed"] != true || proof["core"] != CORE {
        return Err("exact corrected original CPU/storage/input return gate required".into());
    }
    let r = resources(root, true, false)?;
    let app = if returning {
        let v: Value = c::read_json(&root.join("fable-vertex.native-started.json"))?;
        Some(v["app"].as_str().ok_or("retained app")?.into())
    } else {
        None
    };
    let prefix = if returning {
        format!("fable-vertex-return-{}", c::new_id()?)
    } else {
        "fable-vertex".into()
    };
    super::build::observe_local_play(repo, root, r, &prefix, app, returning)
}
