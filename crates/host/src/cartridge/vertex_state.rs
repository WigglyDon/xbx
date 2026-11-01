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
pub(super) const KVM_CORE: &str = match option_env!("XBX_VERTEX_KVM_CORE_SHA256") {
    Some(v) => v,
    None => "unqualified-vertex-kvm-core",
};
pub(super) fn kvm_core(repo: &Path, root: &Path) -> Result<(), Error> {
    core_variant(repo, root, 3)
}
pub(super) fn vblank_core(repo: &Path, root: &Path) -> Result<(), Error> {
    core_variant(repo, root, 4)
}
pub(super) fn core(repo: &Path, root: &Path, corrected: bool) -> Result<(), Error> {
    core_variant(repo, root, if corrected { 2 } else { 0 })
}
pub(super) fn prelaunch_core(repo: &Path, root: &Path) -> Result<(), Error> {
    core_variant(repo, root, 1)
}
fn core_variant(repo: &Path, root: &Path, variant: u8) -> Result<(), Error> {
    let corrected = variant >= 2;
    fs::create_dir_all(root.join("vertex-state-v1"))?;
    let base = root.join(match variant {
        4 => "vertex-state-v1/kvm-runtime",
        3 => "host-input-v1",
        2 => "vertex-state-v1/prelaunch-runtime",
        1 => "vertex-state-v1/base-runtime",
        _ => "smc-v1/runtime",
    });
    let old_core = c::file_digest(&base.join("build/qemu-system-i386"))?;
    let expected = if variant == 4 {
        KVM_CORE.to_owned()
    } else if variant == 3 {
        super::host_input::CORE.to_owned()
    } else if variant > 0 {
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
        4 => "vblank-v1/runtime",
        3 => "vertex-state-v1/kvm-runtime",
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
    let patch_names: &[&str] = match variant {
        4 => &["patches/xemu-vblank-timer.patch"],
        3 => &[
            "patches/xemu-vertex-fixture.patch",
            "patches/xemu-prelaunch-savevm.patch",
            "patches/xemu-vertex-state.patch",
        ],
        2 => &["patches/xemu-vertex-state.patch"],
        1 => &["patches/xemu-prelaunch-savevm.patch"],
        _ => &["patches/xemu-vertex-fixture.patch"],
    };
    if variant == 0 || variant == 3 {
        fs::copy(
            repo.join("native/fixtures/nv2a-vertex-state.inc"),
            dir.join("source/hw/xbox/nv2a/xbx-original-vertex-fixture.inc"),
        )?;
    }
    let mut patches = Vec::new();
    for name in patch_names {
        let patch = repo.join(name);
        c::store::checked(
            Command::new("git")
                .arg("-C")
                .arg(dir.join("source"))
                .env("GIT_CEILING_DIRECTORIES", dir.canonicalize()?)
                .arg("apply")
                .arg(&patch),
        )?;
        patches.push(json!({"path":name,"sha256":c::file_digest(&patch)?}));
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
            &json!({"base_core":old_core,"base_source":old_source,"patches":patches,"fixture_source":c::file_digest(&repo.join("native/fixtures/nv2a-vertex-state.inc"))?,"planned_steps":steps,"compiler_jobs":1,"new_dependencies_or_downloads":false,"state_layout_changed":corrected,"additional_peak_bound_bytes":2147483648u64}),
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
    package_variant(root, corrected, false)
}
pub(super) fn kvm_package(root: &Path) -> Result<(), Error> {
    package_variant(root, true, true)
}
fn package_variant(root: &Path, corrected: bool, kvm: bool) -> Result<(), Error> {
    let dir = root.join(if kvm {
        "vertex-state-v1/kvm-runtime"
    } else if corrected {
        "vertex-state-v1/runtime-v2"
    } else {
        "vertex-state-v1/prelaunch-runtime"
    });
    let pin = if kvm {
        KVM_CORE
    } else if corrected {
        CORE
    } else {
        BASE
    };
    if c::file_digest(&dir.join("build/qemu-system-i386"))? != pin {
        return Err("exact compiled vertex pin required".into());
    }
    let old_root = root.join("smc-v1/package");
    let mut manifest = resources::Resources::smc_fresh(&old_root)?.manifest;
    let out = root.join(if kvm {
        "vertex-state-v1/kvm-package"
    } else if corrected {
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
    if kvm {
        resources::Resources::vertex_kvm(&out, false)?;
    } else {
        resources::Resources::vertex_candidate(&out, corrected, false)?;
    }
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
    observe_variant(root, corrected, returning, false, false)
}
pub(super) fn kvm_observe(root: &Path, returning: bool) -> Result<(), Error> {
    observe_variant(root, true, returning, true, false)
}
pub(super) fn clone_app(source: &Path, destination: &Path) -> Result<Value, Error> {
    if destination.exists() {
        return Err("preserve existing comparison app".into());
    }
    let _app = crate::storage::ProfileLease::acquire(source)?;
    let library = c::library::Library::read(source)?;
    if library.entries.len() != 1 {
        return Err("one original/control entry required".into());
    }
    let profile = source.join("profiles").join(&library.entries[0].profile);
    let _profile = crate::storage::ProfileLease::acquire(&profile)?;
    let before = c::file_digest(&source.join("library.json"))?;
    let storage = c::file_digest(&profile.join("storage.json"))?;
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(source)
            .arg(destination),
    )?;
    if before != c::file_digest(&source.join("library.json"))?
        || storage != c::file_digest(&profile.join("storage.json"))?
    {
        return Err("source changed during owned comparison copy".into());
    }
    Ok(
        json!({"source_app":source,"app":destination,"source_library_unchanged":true,"source_storage_record_unchanged":true,"app_and_profile_leases_held":true,"media_copied":false,"checkpoint_metadata_retagged":false}),
    )
}
pub(super) fn kvm_vulkan_state(root: &Path) -> Result<(), Error> {
    let dir = root.join("vertex-state-v1");
    let proof: Value = c::read_json(&dir.join("kvm-return/result.json"))?;
    if proof["passed"] != true || proof["core"] != KVM_CORE {
        return Err("actual GL device-state source required".into());
    }
    let copy = clone_app(&dir.join("kvm-app"), &dir.join("kvm-vulkan-app"))?;
    c::write_new(&dir.join("kvm-vulkan-source.json"), &c::json(&copy)?)?;
    observe_variant(root, true, true, true, true)
}
fn observe_variant(
    root: &Path,
    corrected: bool,
    returning: bool,
    kvm: bool,
    vulkan: bool,
) -> Result<(), Error> {
    let dir = root.join("vertex-state-v1");
    let label = if vulkan {
        "kvm-vulkan"
    } else if kvm {
        "kvm"
    } else if corrected {
        "corrected"
    } else {
        "baseline-v4"
    };
    let out = dir.join(format!(
        "{label}-{}",
        if returning { "return" } else { "first" }
    ));
    if corrected {
        let red: Value = c::read_json(&dir.join("baseline-v4-return/result.json"))?;
        if red["passed"] != true || red["missing_state_reproduced"] != true {
            return Err("actual original negative required".into());
        }
    }
    let mut r = if kvm {
        resources::Resources::vertex_kvm(&dir.join("kvm-package"), true)?
    } else {
        resources(root, corrected, true)?
    };
    r.presentation_vulkan = vulkan;
    device_at(
        root,
        r,
        &dir.join(format!("{label}-app")),
        &out,
        corrected,
        returning,
    )
}
pub(super) fn device_at(
    root: &Path,
    r: resources::Resources,
    app: &Path,
    out: &Path,
    corrected: bool,
    returning: bool,
) -> Result<(), Error> {
    c::mkdir(out)?;
    let media = root.join("fixtures-v2/a.iso");
    if c::file_digest(&media)? != resources::ORIGINAL_FIXTURES[0] {
        return Err("original fixture changed".into());
    }
    let core = r.manifest.core.clone();
    let runtime = r.identity.clone();
    let mut player = c::desktop::Player::with_resources(app, r)?;
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
    original_variant(root, returning, false, false)
}
pub(super) fn kvm_original(root: &Path, returning: bool) -> Result<(), Error> {
    original_variant(root, returning, true, false)
}
pub(super) fn kvm_vulkan_original(root: &Path) -> Result<(), Error> {
    original_variant(root, true, true, true)
}
fn original_variant(root: &Path, returning: bool, kvm: bool, vulkan: bool) -> Result<(), Error> {
    let dir = root.join("vertex-state-v1");
    let proof: Value = c::read_json(&dir.join(if vulkan {
        "kvm-vulkan-return/result.json"
    } else if kvm {
        "kvm-return/result.json"
    } else {
        "corrected-return/result.json"
    }))?;
    if proof["passed"] != true {
        return Err("device-state return gate required".into());
    }
    let mut r = if kvm {
        resources::Resources::vertex_kvm(&dir.join("kvm-package"), false)?
    } else {
        resources(root, true, false)?
    };
    r.presentation_vulkan = vulkan;
    let core = r.manifest.core.clone();
    let label = if vulkan {
        "original-kvm-vulkan"
    } else if kvm {
        "original-kvm"
    } else {
        "original"
    };
    if !returning {
        return super::host_input::baseline_at(root, r, &dir.join(format!("{label}-first")));
    }
    let out = dir.join(format!("{label}-return"));
    c::mkdir(&out)?;
    let old = dir.join(if vulkan {
        "original-kvm-first/app".to_owned()
    } else {
        format!("{label}-first/app")
    });
    let app = dir.join(format!("{label}-return-app"));
    let _lease = crate::storage::ProfileLease::acquire(&old)?;
    let library = c::file_digest(&old.join("library.json"))?;
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(&old)
            .arg(&app),
    )?;
    super::host_input::original_at(&old, &app, &out, r, &core, &library, false)
}
pub(super) fn kvm_vulkan_fable(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("vertex-state-v1");
    let proof: Value = c::read_json(&dir.join("original-kvm-vulkan-return/result.json"))?;
    if proof["passed"] != true || proof["core"] != KVM_CORE {
        return Err("actual cross-renderer original GPU/CPU/save/input return required".into());
    }
    let source: Value = c::read_json(&root.join("fable-vertex-kvm.native-started.json"))?;
    let source = Path::new(source["app"].as_str().ok_or("retained source app")?);
    let app = dir.join("fable-kvm-vulkan-app");
    let copy = clone_app(source, &app)?;
    c::write_new(&dir.join("fable-kvm-vulkan-source.json"), &c::json(&copy)?)?;
    let mut r = resources::Resources::vertex_kvm(&dir.join("kvm-package"), false)?;
    r.presentation_vulkan = true;
    super::build::observe_local_play(repo, root, r, "fable-vertex-kvm-vulkan", Some(app), true)
}
pub(super) fn fable(repo: &Path, root: &Path, returning: bool) -> Result<(), Error> {
    fable_variant(repo, root, returning, false)
}
pub(super) fn kvm_fable(repo: &Path, root: &Path, returning: bool) -> Result<(), Error> {
    fable_variant(repo, root, returning, true)
}
fn fable_variant(repo: &Path, root: &Path, returning: bool, kvm: bool) -> Result<(), Error> {
    let core = if kvm { KVM_CORE } else { CORE };
    let proof: Value = c::read_json(&root.join(if kvm {
        "vertex-state-v1/original-kvm-return/result.json"
    } else {
        "vertex-state-v1/original-return/result.json"
    }))?;
    if proof["passed"] != true || proof["core"] != core {
        return Err("exact corrected original CPU/storage/input return gate required".into());
    }
    let r = if kvm {
        resources::Resources::vertex_kvm(&root.join("vertex-state-v1/kvm-package"), false)?
    } else {
        resources(root, true, false)?
    };
    let tag = if kvm {
        "fable-vertex-kvm"
    } else {
        "fable-vertex"
    };
    let app = if returning {
        let v: Value = c::read_json(&root.join(format!("{tag}.native-started.json")))?;
        Some(v["app"].as_str().ok_or("retained app")?.into())
    } else {
        None
    };
    let prefix = if returning {
        format!("{tag}-return-{}", c::new_id()?)
    } else {
        tag.into()
    };
    super::build::observe_local_play(repo, root, r, &prefix, app, returning)
}
