// SPDX-License-Identifier: MIT
//! Isolated original-regression-backed user-buffer residency correction.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
fn copy(source: &Path, dest: &Path) -> Result<(), Error> {
    fs::create_dir_all(dest.parent().ok_or("source parent")?)?;
    c::write_new(dest, &fs::read(source)?)?;
    Ok(())
}
pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("buffer-probe-v1");
    c::mkdir(&dir)?;
    let lock: Value = c::read_json(&repo.join("cartridge-buffer-probe.lock.json"))?;
    let negative: Value = c::read_json(&root.join("file-cache-demand-v3/original.result.json"))?;
    if negative["guest_passed"] != false
        || negative["data_verified"] != false
        || negative["native_clean"] != true
        || negative["seeds_unchanged"] != true
        || negative["fixture"] != lock["original_negative_fixture"]
        || c::file_digest(&root.join("file-cache-demand-v3/original.iso"))?
            != lock["original_negative_fixture"]
        || negative["core"] != resources::X87_RESTART_CORE
        || negative["open_system"] != resources::FLUSH_LOGGING_OPEN
        || !fs::read_to_string(root.join("file-cache-demand-v3/serial.log"))?.contains(
            "FAIL stage=00000011 status=00000000 count=00004000 first=00000000 faults=00000000",
        )
    {
        return Err("exact clean original negative required".into());
    }
    if !fs::read_to_string(root.join("file-cache-demand-v3/serial.log"))?
        .contains("XBX FILE READ DEMAND HEADER PASS")
    {
        return Err("actual original handler control required".into());
    }
    super::flush_io::qualified(root)?;
    let old = root.join("flush-io-v1/source");
    let manifest = old.join("source.objects");
    let patch = repo.join("patches/oxos-buffer-probe.patch");
    if c::file_digest(&manifest)? != lock["retained_corrected_source_objects_sha256"]
        || c::file_digest(&patch)? != lock["patch_sha256"]
    {
        return Err("retained source or functional patch differs".into());
    }
    let source = dir.join("source");
    c::mkdir(&source)?;
    let objects = fs::read_to_string(&manifest)?;
    for line in objects.lines() {
        let (metadata, name) = line.split_once('\t').ok_or("source manifest")?;
        if name.starts_with('/') || name.split('/').any(|p| p == "..") {
            return Err("source path".into());
        }
        let input = old.join(name);
        if let Some(expected) = lock["accepted_source"].get(name).and_then(Value::as_str) {
            if c::file_digest(&input)? != expected {
                return Err("accepted functional source repair differs".into());
            }
        } else {
            let got = Command::new("git")
                .arg("hash-object")
                .arg(&input)
                .output()?;
            if !got.status.success()
                || String::from_utf8(got.stdout)?.trim()
                    != metadata.split_whitespace().nth(2).ok_or("source blob")?
            {
                return Err("retained public source bytes differ".into());
            }
        }
        copy(&input, &source.join(name))?;
    }
    copy(&manifest, &source.join("source.objects"))?;
    let archive = root
        .parent()
        .ok_or("project")?
        .join("open-startup-v1/cromwell-v3.tar");
    if c::file_digest(&archive)?
        != "280b2b1dce7d73ba40d9e4d027fb8f54624fa7db0724e2f431d813f85ef29dcb"
    {
        return Err("retained public boot archive differs".into());
    }
    fs::create_dir_all(source.join("third_party/cromwell"))?;
    c::store::checked(
        Command::new("tar")
            .args(["-x", "--no-same-owner", "-f"])
            .arg(&archive)
            .arg("-C")
            .arg(source.join("third_party/cromwell")),
    )?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .arg("apply")
            .arg(&patch),
    )?;
    let mut changed = Vec::new();
    for line in objects.lines() {
        let (_, name) = line.split_once('\t').ok_or("source manifest")?;
        if fs::read(old.join(name))? != fs::read(source.join(name))? {
            if c::file_digest(&old.join(name))? != lock["changed"][name]["before"]
                || c::file_digest(&source.join(name))? != lock["changed"][name]["after"]
            {
                return Err("functional source extent differs".into());
            }
            changed.push(name);
        }
    }
    if changed != ["kernel/src/mm/memory.cpp"] {
        return Err("unrelated source change".into());
    }
    let out = dir.join("build-logging");
    c::mkdir(&out)?;
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let mut paths = vec![llvm.clone()];
    if let Some(path) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&path));
    }
    c::store::checked(
        Command::new("make")
            .arg("-C")
            .arg(source.join("kernel"))
            .env("PATH", std::env::join_paths(paths)?)
            .args([
                "-j1",
                "bios",
                "IMAGE=open",
                "SELFTEST=0",
                "TRACE_IMPORTS=0",
                "DEBUG=0",
                "SPLASH=",
                "MCPX16=0",
                "BIOS_BASE=",
                "MCPX_ROM=",
                "STOCK_MCPX=",
                "SERIAL=1",
            ])
            .arg(format!("BUILD={}", out.display()))
            .arg(format!("CC={}", llvm.join("clang").display()))
            .arg(format!(
                "CXX={} -Wno-error=cast-function-type-mismatch",
                llvm.join("clang++").display()
            ))
            .arg(format!("LD={}", llvm.join("lld-link").display()))
            .arg(format!("OBJCOPY={}", llvm.join("llvm-objcopy").display())),
    )?;
    for name in ["bios.bin", "xboxkrnl.exe", "xboxkrnl.map"] {
        fs::File::open(out.join(name))?.sync_all()?;
    }
    c::write_new(
        &dir.join("build.identity.json"),
        &c::json(&json!({
            "source":lock["commit"],"cromwell":lock["cromwell_commit"],"functional_patch":lock["patch_sha256"],"changed":changed,
            "firmware":c::file_digest(&out.join("bios.bin"))?,"kernel":c::file_digest(&out.join("xboxkrnl.exe"))?,"map":c::file_digest(&out.join("xboxkrnl.map"))?,
            "retained_core":resources::X87_RESTART_CORE,"compiler":c::file_digest(&llvm.join("clang"))?,"linker":c::file_digest(&llvm.join("lld-link"))?,
            "compiler_jobs":1,"serial":true,"trace_imports":false,"original_negative":negative["fixture"],"native_regression_passed":false,"Fable_cause_established":false
        }))?,
    )?;
    Ok(())
}

pub(super) fn package(root: &Path) -> Result<resources::Resources, Error> {
    let base = super::flush_io::package(root)?;
    let firmware = root.join("buffer-probe-v1/build-logging/bios.bin");
    if c::file_digest(&firmware)? != resources::BUFFER_PROBE_LOGGING_OPEN {
        return Err("buffer-probe build identity changed".into());
    }
    let dir = root.join("package-buffer-probe-v1");
    if !dir.exists() {
        c::mkdir(&dir)?;
        fs::create_dir_all(dir.join("native/subprojects/slirp"))?;
        for name in [
            "native/core",
            "native/qemu-img",
            "native/subprojects/slirp/libslirp.so.0",
        ] {
            fs::hard_link(base.root.join(name), dir.join(name))?;
        }
        fs::hard_link(firmware, dir.join("open.bin"))?;
        let mut manifest = base.manifest.clone();
        manifest.open_system = resources::BUFFER_PROBE_LOGGING_OPEN.into();
        c::write_new(&dir.join("runtime.json"), &c::json(&manifest)?)?;
    }
    Ok(resources::Resources::buffer_probe_logging(&dir)?)
}

pub(super) fn qualified(root: &Path) -> Result<(), Error> {
    for (name, fixture) in [
        (
            "demand-regression",
            "ccc5d1c6f07024ff18c95e7e3f3d4508c92a4c24f601d14b06ca96777cef7126",
        ),
        (
            "allocated-regression",
            "da22e0e3ef9444f110a5961c2ae43ba795ba22c513823f230f02c3ed2091d951",
        ),
    ] {
        let dir = root.join("buffer-probe-v1").join(name);
        let r: Value = c::read_json(&dir.join("original.result.json"))?;
        if r["guest_passed"] != true
            || r["data_verified"] != true
            || r["native_clean"] != true
            || r["seeds_unchanged"] != true
            || r["fixture"] != fixture
            || r["core"] != resources::X87_RESTART_CORE
            || r["open_system"] != resources::BUFFER_PROBE_LOGGING_OPEN
        {
            return Err("exact original buffer-residency regressions required".into());
        }
        if name == "demand-regression" {
            let serial = fs::read_to_string(dir.join("serial.log"))?;
            if !serial.contains("XBX FILE READ DEMAND HEADER PASS")
                || !serial.contains("XBX FILE READ DEMAND PAGES PASS")
            {
                return Err("actual demand commits during I/O required".into());
            }
        }
    }
    Ok(())
}

fn original_state(player: &mut c::desktop::Player, expected: &str) -> Result<String, Error> {
    let started = std::time::Instant::now();
    let mut text = String::new();
    while started.elapsed().as_secs() < 20 {
        text.push_str(
            &player
                .live
                .as_mut()
                .ok_or("no original runtime")?
                .native
                .serial()?,
        );
        if text.len() > 65536 {
            return Err("original output bound".into());
        }
        if text.contains(expected) {
            return Ok(text);
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    Err("original state response deadline".into())
}

/// Real native original-only proof of bounded disk identity and finite-owner
/// cleanup. The product keeps its live session on publication failure; only
/// this explicitly failed qualification attempt is then retired.
pub(super) fn storage_proof(repo: &Path, root: &Path) -> Result<(), Error> {
    qualified(root)?;
    let dir = root.join("bounded-storage-v1");
    c::mkdir(&dir)?;
    let app = dir.join("original-app");
    let media = root.join("fixtures-v2/a.iso");
    if c::file_digest(&media)? != resources::ORIGINAL_FIXTURES[0] {
        return Err("retained original cartridge differs".into());
    }
    let disk = repo.join(".local/dashboard-v20260516-0955/xbox_hdd.qcow2");
    let eeprom = repo.join(".local/quick-reboot-d87cfc33-run1/eeprom.bin");
    let seeds = (c::file_digest(&disk)?, c::file_digest(&eeprom)?);
    let mut first = c::desktop::Player::with_resources(&app, package(root)?)?;
    let game = first.add(&media)?;
    let entry = c::library::Library::read(&app)?.resolve(&game)?;
    {
        let _profile =
            c::store::Profile::open_seeded(&app, &entry, &first.resources, &disk, &eeprom)?;
    }
    let first_result = (|| -> Result<_, Error> {
        first.launch(&game)?;
        let run = first
            .live
            .as_ref()
            .ok_or("original live")?
            .native
            .run
            .clone();
        original_state(&mut first, "CART A state=00000000 save=00000000")?;
        first.live.as_mut().unwrap().native.fixture_input("as a")?;
        let state = original_state(
            &mut first,
            "CART A state=00000002 save=00000001 cookie=57F090E4",
        )?;
        c::write_new(&dir.join("first-state.log"), state.as_bytes())?;
        first.depart()?;
        let returned: Value = c::read_json(&run.join("native-return.json"))?;
        if returned["clean"] != true {
            return Err("original first native return".into());
        }
        Ok((run, returned))
    })();
    if let Err(e) = first_result {
        let cleanup = super::build::retire_failed_observation(&mut first);
        c::write_new(
            &dir.join("first-failure.json"),
            &c::json(&json!({"error":e.to_string(),"cleanup":cleanup}))?,
        )?;
        return Err(e);
    }
    let (first_run, first_return) = first_result?;
    drop(first);
    let mut second = c::desktop::Player::with_resources(&app, package(root)?)?;
    let observed = (|| -> Result<_, Error> {
        if !second.launch(&game)? {
            return Err("original checkpoint was not restored".into());
        }
        let run = second
            .live
            .as_ref()
            .ok_or("restored live")?
            .native
            .run
            .clone();
        if fs::read(first_run.join("native.pid"))? == fs::read(run.join("native.pid"))? {
            return Err("native process must be genuinely fresh".into());
        }
        second.live.as_mut().unwrap().native.fixture_input("?a")?;
        let state = original_state(
            &mut second,
            "CART A state=00000003 save=00000001 cookie=6B482CC3",
        )?;
        if !state.contains("CART A state=00000002 save=00000001 cookie=57F090E4") {
            return Err("restored unsaved state not observed before new action".into());
        }
        c::write_new(&dir.join("restored-state.log"), state.as_bytes())?;
        second.pause(false)?;
        let live = second.live.as_ref().unwrap();
        let profile = live.profile.root.clone();
        let before = c::store::signature(&live.profile.current()?, &second.resources)?;
        let pointer = fs::read(profile.join("session.json"))?;
        let session: c::checkpoint::Session = serde_json::from_slice(&pointer)?;
        let old_checkpoint = profile
            .join("checkpoints")
            .join(session.checkpoint)
            .join("hdd.qcow2");
        let old_digest = c::file_digest(&old_checkpoint)?;
        c::mkdir(&profile.join("session.pending"))?;
        let error = second
            .depart()
            .err()
            .ok_or("publication injection did not fail")?;
        if second.live.is_none() || second.live.as_mut().unwrap().native.exited()? {
            return Err("product must preserve the live session on save failure".into());
        }
        Ok((
            run,
            profile,
            before,
            pointer,
            old_checkpoint,
            old_digest,
            error.to_string(),
        ))
    })();
    let cleanup = super::build::retire_failed_observation(&mut second);
    let checked = (|| -> Result<Value, Error> {
        let (run, profile, before, pointer, old_checkpoint, old_digest, error) = observed?;
        let live = second.live.as_ref().ok_or("failed session retained")?;
        let returned: Value = c::read_json(&run.join("native-return.json"))?;
        let safe = before == c::store::signature(&live.profile.current()?, &second.resources)?
            && pointer == fs::read(profile.join("session.json"))?
            && old_digest == c::file_digest(&old_checkpoint)?
            && profile.join("session.pending").is_dir();
        let unchanged = seeds == (c::file_digest(&disk)?, c::file_digest(&eeprom)?);
        let pass = safe
            && unchanged
            && returned["clean"] == true
            && cleanup.as_ref().is_some_and(|v| v["native_clean"] == true);
        Ok(
            json!({"passed":pass,"original_only":true,"fixture":resources::ORIGINAL_FIXTURES[0],"core":resources::X87_RESTART_CORE,"open_system":resources::BUFFER_PROBE_LOGGING_OPEN,
            "unsaved2_save1_fresh_native_restore_then_action3":true,"product_keeps_live_session_on_publication_failure":true,
            "current_saves_old_checkpoint_pointer_and_failure_evidence_unchanged":safe,"seeds_unchanged":unchanged,
            "expected_publication_error":error,"first_native_run":first_run,"first_native_return":first_return,
            "native_run":run,"native_return":returned,"explicit_failed_observation_cleanup":cleanup,"commercial_acceptance":false}),
        )
    })();
    let result = match checked {
        Ok(v) => v,
        Err(e) => json!({"passed":false,"error":e.to_string(),"cleanup":cleanup}),
    };
    c::write_new(&dir.join("original.result.json"), &c::json(&result)?)?;
    if result["passed"] != true {
        return Err(
            "bounded storage or native failure cleanup proof failed; preserve evidence".into(),
        );
    }
    Ok(())
}
