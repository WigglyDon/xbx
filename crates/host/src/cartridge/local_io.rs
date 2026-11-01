// SPDX-License-Identifier: MIT
//! Current local reconstruction uses the accepted product owners and fixtures.
//! No historical workspace, profiles or qualification markers are fabricated.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{fs, path::Path};
type Error = Box<dyn std::error::Error>;
const DEMAND: &str = "ccc5d1c6f07024ff18c95e7e3f3d4508c92a4c24f601d14b06ca96777cef7126";

fn package(root: &Path, reporting: bool) -> Result<resources::Resources, Error> {
    let native = root.join("rebuilt-runtime-v1");
    let identity: Value = c::read_json(&native.join("identity.json"))?;
    let core = native.join("build/qemu-system-i386");
    if identity["core"] != resources::LOCAL_IO_CORE
        || c::file_digest(&core)? != resources::LOCAL_IO_CORE
        || identity["inputs"] != c::file_digest(&native.join("inputs.json"))?
    {
        return Err("current reconstructed native identity differs".into());
    }
    let firmware = root
        .join("rebuilt-io-images-v3")
        .join(if reporting { "reporting" } else { "baseline" })
        .join("bios.bin");
    let expected = if reporting {
        resources::LOCAL_IO_OPEN
    } else {
        resources::LOCAL_BASE_OPEN
    };
    if c::file_digest(&firmware)? != expected {
        return Err("current reconstructed open image differs".into());
    }
    let dir = root.join(if reporting {
        "local-io-package-reporting"
    } else {
        "local-io-package-baseline"
    });
    if !dir.exists() {
        c::mkdir(&dir)?;
        fs::create_dir_all(dir.join("native/subprojects/slirp"))?;
        fs::hard_link(core, dir.join("native/core"))?;
        fs::hard_link(firmware, dir.join("open.bin"))?;
        fs::copy("/usr/bin/qemu-img", dir.join("native/qemu-img"))?;
        fs::copy(
            native.join("build/subprojects/slirp/libslirp.so.0"),
            dir.join("native/subprojects/slirp/libslirp.so.0"),
        )?;
        let manifest = resources::Manifest {
            schema: 1,
            core: resources::LOCAL_IO_CORE.into(),
            open_system: expected.into(),
            disk_tool: c::file_digest(&dir.join("native/qemu-img"))?,
            slirp: c::file_digest(&dir.join("native/subprojects/slirp/libslirp.so.0"))?,
            state_abi: 1,
        };
        c::write_new(&dir.join("runtime.json"), &c::json(&manifest)?)?;
    }
    Ok(resources::Resources::local_io(&dir, reporting)?)
}

pub(super) fn control(
    repo: &Path,
    root: &Path,
    reporting: bool,
    demand: bool,
) -> Result<(), Error> {
    let media = root.join(if demand {
        "file-cache-demand-v3/original.iso"
    } else {
        "file-io-errors-v1/original.iso"
    });
    let pin = c::file_digest(&media)?;
    if demand {
        if pin != DEMAND {
            return Err("exact reconstructed original demand image differs".into());
        }
    } else {
        let build: Value = c::read_json(&root.join("file-io-errors-v1/build.json"))?;
        if build["fixture"] != pin
            || build["expected_public_io_error_control"] != true
            || build["source"] != c::file_digest(&repo.join("native/fixtures/file-read.S"))?
        {
            return Err("current original I/O control bindings differ".into());
        }
    }
    let r = package(root, reporting)?;
    let dir = root.join("local-io-conformance").join(if demand {
        "demand"
    } else if reporting {
        "reporting"
    } else {
        "baseline"
    });
    fs::create_dir_all(dir.parent().ok_or("control parent")?)?;
    c::mkdir(&dir)?;
    super::file_read::observe_with(repo, root, &dir, &media, demand, r)
}

fn qualified(root: &Path) -> Result<(), Error> {
    let fixture = c::file_digest(&root.join("file-io-errors-v1/original.iso"))?;
    for (name, open, media) in [
        ("baseline", resources::LOCAL_BASE_OPEN, fixture.as_str()),
        ("reporting", resources::LOCAL_IO_OPEN, fixture.as_str()),
        ("demand", resources::LOCAL_IO_OPEN, DEMAND),
    ] {
        let dir = root.join("local-io-conformance").join(name);
        let result: Value = c::read_json(&dir.join("original.result.json"))?;
        if result["core"] != resources::LOCAL_IO_CORE
            || result["open_system"] != open
            || result["fixture"] != media
            || result["guest_passed"] != true
            || result["data_verified"] != true
            || result["native_clean"] != true
            || result["seeds_unchanged"] != true
            || (name != "demand" && result["public_io_error_control_passed"] != true)
        {
            return Err("current local original conformance or return failed".into());
        }
        let serial = fs::read_to_string(dir.join("serial.log"))?.to_ascii_lowercase();
        if name == "reporting" {
            for message in [
                "xbxio: kind=0 status=c0000034",
                "xbxio: kind=1 status=c0000011",
                "xbxio: kind=2 status=c0000011",
            ] {
                if !serial.contains(message) {
                    return Err("actual public error status reporting required".into());
                }
            }
        }
        if name == "demand" && !serial.contains("demand pages pass") {
            return Err("actual original demand-commit verification required".into());
        }
    }
    Ok(())
}

/// Two finite ordinary integration inputs that differ from the accepted seeded
/// full-disc desktop result. Never rerun the completed intro/menu campaign.
pub(super) fn input_check(repo: &Path, root: &Path, view: bool) -> Result<(), Error> {
    use std::time::{Duration, Instant};
    qualified(root)?;
    let name = if view {
        "input-media-view-v1"
    } else {
        "input-generated-profile-v1"
    };
    let out = root.join(name);
    c::mkdir(&out)?;
    let r = package(root, false)?;
    let media_root = repo.join(".local/cartridge-media-20261029-v1");
    let media = media_root.join(if view {
        "game-view-v1/selected-game-partition.iso"
    } else {
        "media/selected-disc.iso"
    });
    let identity: Value = c::read_json(&if view {
        repo.join(".local/evidence/cartridge-v1/pixel-demo-20261101-v1/prepared-game-view.json")
    } else {
        media_root.join("media/verified.identity.json")
    })?;
    let mut player = c::desktop::Player::with_resources(&out.join("app"), r)?;
    let game = player.add(&media)?;
    let entry = c::library::Library::read(&player.app)?.resolve(&game)?;
    if entry.media_sha256
        != identity["sha256"]
            .as_str()
            .ok_or("verified media identity")?
        || entry.partition != if view { 0 } else { 405_798_912 }
    {
        return Err("integration input differs; native not started".into());
    }
    let seed = repo.join(".local/dashboard-v20260516-0955/xbox_hdd.qcow2");
    let eeprom = repo.join(".local/quick-reboot-d87cfc33-run1/eeprom.bin");
    let seed_before = (c::file_digest(&seed)?, c::file_digest(&eeprom)?);
    if view {
        if seed_before.0 != "00d7df7a2bc235f8801764f00b7f40e194d1e392f7a9619d6b2396c89770f6dd" {
            return Err("matched seed differs".into());
        }
        let _p =
            c::store::Profile::open_seeded(&player.app, &entry, &player.resources, &seed, &eeprom)?;
    }
    let metadata = fs::metadata(&media)?;
    player.launch(&game)?;
    let run = player
        .live
        .as_ref()
        .ok_or("native machine")?
        .native
        .run
        .clone();
    let core = player.resources.core();
    let start = Instant::now();
    let mut serial = String::new();
    let mut captures = Vec::new();
    let mut next = 0;
    let observed = (|| -> Result<(), Error> {
        while start.elapsed() < Duration::from_secs(45) {
            serial.push_str(&player.live.as_mut().ok_or("live")?.native.serial()?);
            if serial.len() > 128 * 1024 {
                return Err("finite startup log bound".into());
            }
            if next < 4 && start.elapsed().as_secs() >= [3, 10, 20, 40][next] {
                let result = std::process::Command::new("python3")
                    .arg(repo.join(".local/capture-ramht-window-v2.py"))
                    .arg(&run)
                    .arg(&core)
                    .output()?;
                captures.push(json!({"elapsed_ms":start.elapsed().as_millis(),"exit":result.status.code(),
                    "path":String::from_utf8_lossy(&result.stdout).trim(),"actual_review_required":true}));
                next += 1;
            }
            if serial.contains("fatal error 21") {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        Ok(())
    })();
    c::write_new(&out.join("kernel.log"), serial.as_bytes())?;
    let stopped = player.stop();
    drop(player);
    let native: Value = c::read_json(&run.join("native-return.json"))?;
    let unchanged = seed_before == (c::file_digest(&seed)?, c::file_digest(&eeprom)?)
        && fs::metadata(&media)?.len() == metadata.len()
        && fs::metadata(&media)?.modified()? == metadata.modified()?;
    c::write_new(
        &out.join("result.json"),
        &c::json(&json!({
            "case":name,"core":resources::LOCAL_IO_CORE,"open_system":resources::LOCAL_BASE_OPEN,
            "media_view":view,"matched_reference_storage":view,"ordinary_generated_storage":!view,
            "maximum_guest_observation_seconds":45,"elapsed_ms":start.elapsed().as_millis(),
            "default_xbe_loaded":serial.contains("load \\Device\\CdRom0\\default.xbe: 0x0"),
            "quick_reboot_observed":serial.contains(": quick boot"),"fatal21_observed":serial.contains("fatal error 21"),
            "captures":captures,"native_return":native,"inputs_preserved":unchanged,
            "observation_error":observed.as_ref().err().map(ToString::to_string),
            "stop_error":stopped.as_ref().err().map(ToString::to_string),
            "gameplay_or_performance_acceptance":false,"phone_operated":false
        }))?,
    )?;
    observed?;
    stopped?;
    if native["clean"] != true || native["native_reaped"] != true || !unchanged {
        return Err(
            "integration observation return/input verification failed; preserve outputs".into(),
        );
    }
    Ok(())
}

pub(super) fn fable(repo: &Path, root: &Path) -> Result<(), Error> {
    qualified(root)?;
    let prefix = "fable-local-io-logging";
    if root.join(format!("{prefix}.native-started.json")).exists() {
        return Err(
            "this local reporting observation already started; reconcile instead of repeating"
                .into(),
        );
    }
    let r = package(root, true)?;
    c::write_new(
        &root.join(format!("{prefix}.started.json")),
        &c::json(&json!({
            "core":r.manifest.core,"open_system":r.manifest.open_system,"source_reconstruction":true,
            "actual_original_conformance_passed":true,"ordinary_product_policy":true,
            "memory_mib":64,"network":"none","fp":"native","graphics":"Intel OpenGL",
            "public_io_failure_diagnostics_only":true,"historical_campaign_repeated":false
        }))?,
    )?;
    super::build::observe_logging_fable(repo, root, r, prefix, true, true, true)
}

/// Ordinary non-reporting open build, with manual normal controller input and
/// a finite observation owner. No existing profile is migrated or overwritten.
pub(super) fn play(repo: &Path, root: &Path) -> Result<(), Error> {
    qualified(root)?;
    let prior: Value = c::read_json(&root.join("fable-local-io-logging.native-started.json"))?;
    let app = Path::new(prior["app"].as_str().ok_or("prior observation app")?);
    let result: Value = c::read_json(&app.join("observation.json"))?;
    if result["normal_retirement"] != true || result["immutable_inputs_unchanged"] != true {
        return Err("reporting observation and return must be reconciled first".into());
    }
    let prefix = "fable-local-play";
    if root.join(format!("{prefix}.native-started.json")).exists() {
        return Err("ordinary local play already started; reconcile instead of repeating".into());
    }
    let r = package(root, false)?;
    c::write_new(
        &root.join(format!("{prefix}.started.json")),
        &c::json(&json!({
            "core":r.manifest.core,"open_system":r.manifest.open_system,
            "prior_reporting_result":c::file_digest(&app.join("observation.json"))?,
            "actual_original_conformance_passed":true,"ordinary_product_policy":true,
            "memory_mib":64,"network":"none","fp":"native","graphics":"Intel OpenGL",
            "public_io_failure_diagnostics":false,"maximum_seconds":3600,
            "input":"manual normal controller, actual visible review required",
            "departure":"operator-depart or ordinary budget expiry requests the product checkpoint; actual healthy restoration remains separately unqualified"
        }))?,
    )?;
    super::build::observe_local_play(repo, root, r, prefix, None, false)
}

pub(super) fn continue_play(repo: &Path, root: &Path) -> Result<(), Error> {
    qualified(root)?;
    let prior: Value = c::read_json(&root.join("fable-local-play.native-started.json"))?;
    let app = Path::new(prior["app"].as_str().ok_or("prior ordinary app")?);
    let result: Value = c::read_json(&app.join("observation.json"))?;
    if result["normal_retirement"] != true
        || result["immutable_inputs_unchanged"] != true
        || result["continuation_checkpoint_published"] != false
    {
        return Err("reconcile the preserved ordinary attempt before explicit restart".into());
    }
    let returned: Value = c::read_json(
        &Path::new(result["native_run"].as_str().ok_or("prior native run")?)
            .join("native-return.json"),
    )?;
    if returned["clean"] != true || returned["native_reaped"] != true {
        return Err("prior ordinary native return is incomplete".into());
    }
    // Each ordinary restart retains its own run receipt. The actual app/profile
    // leases still refuse a second live owner; no prior attempt is overwritten.
    let prefix = format!("fable-local-play-continue-{}", c::new_id()?);
    let r = package(root, false)?;
    super::build::observe_local_play(repo, root, r, &prefix, Some(app.into()), false)
}

/// Continue the same ordinary cartridge only through its actual coherent saved
/// machine. Exclusive product/profile leases and checkpoint validation apply.
pub(super) fn return_play(repo: &Path, root: &Path) -> Result<(), Error> {
    qualified(root)?;
    let prior: Value = c::read_json(&root.join("fable-local-play.native-started.json"))?;
    let app = Path::new(prior["app"].as_str().ok_or("ordinary cartridge app")?);
    let prefix = format!("fable-local-play-return-{}", c::new_id()?);
    let r = package(root, false)?;
    super::build::observe_local_play(repo, root, r, &prefix, Some(app.into()), true)
}
