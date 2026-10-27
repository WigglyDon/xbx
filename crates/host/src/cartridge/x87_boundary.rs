// SPDX-License-Identifier: MIT
//! Original-test-supported x87 state at precise instruction restart boundaries.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const TRANSLATE: &str = "9a13a807f92ff591e36297ba81c9201c7f6596c68a566bfc8c4b6893c06b50b7";
fn retained_or_new(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    if path.exists() {
        if fs::read(path)? != bytes {
            return Err("prepared original input changed; preserve failed build".into());
        }
    } else {
        c::write_new(path, bytes)?;
    }
    Ok(())
}
pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    // Paths enter a retained Ninja compile command; refuse shell metacharacters.
    for path in [repo, root] {
        if !path
            .as_os_str()
            .as_encoded_bytes()
            .iter()
            .all(|v| v.is_ascii_alphanumeric() || b"/_-.".contains(v))
        {
            return Err("unsupported native build path".into());
        }
    }
    let project = root.parent().ok_or("project")?;
    let old = project.join("ramht-v1/build-v2-retry2");
    let src = project.join("ramht-v1/source-v2");
    let baseline = root.join("native-v3");
    let normal = resources::Resources::open(&root.join("package-v3"))?;
    let original = src.join("target/i386/tcg/translate.c");
    if c::file_digest(&original)? != TRANSLATE
        || c::file_digest(&baseline.join("core"))? != resources::CORE
        || c::file_digest(&normal.core())? != resources::CORE
    {
        return Err("retained qualified native/source identity differs".into());
    }
    let dir = root.join("x87-boundary-v1");
    if dir.join("build.identity.json").exists() {
        return Err("candidate already built; reuse its actual bytes".into());
    }
    if !dir.exists() {
        c::mkdir(&dir)?;
    }
    let source = dir.join("source");
    fs::create_dir_all(source.join("target/i386/tcg"))?;
    let subject_source = source.join("target/i386/tcg/translate.c");
    let before = fs::read(&original)?;
    let patch = repo.join("patches/xemu-x87-instruction-boundary.patch");
    let patch_hash = c::file_digest(&patch)?;
    // A failed compile may be resumed without recopying source or rebuilding
    // accepted native units. The immutable preparation binds its source/recipe.
    let prepared = dir.join("preparation.json");
    if !prepared.exists() {
        retained_or_new(&subject_source, &before)?;
        c::store::checked(
            Command::new("git")
                .arg("-C")
                .arg(&source)
                .arg("apply")
                .arg(&patch),
        )?;
        let changed = fs::read_to_string(&subject_source)?;
        let expected = String::from_utf8(before.clone())?.replacen(
            "    dc->prev_insn_start = dc->base.insn_start;",
            "    /* A memory access can restart this instruction after invalidating its TB.\n     * Keep earlier x87 results in CPU state before that precise boundary. */\n    gen_flush_fp(dc);\n    dc->prev_insn_start = dc->base.insn_start;", 1);
        if changed != expected {
            return Err("native patch changes beyond the supported boundary".into());
        }
        c::write_new(
            &prepared,
            &c::json(
                &json!({"patch":patch_hash,"before":TRANSLATE,"after":c::file_digest(&subject_source)?}),
            )?,
        )?;
    }
    let preparation: Value = c::read_json(&prepared)?;
    if preparation["patch"] != patch_hash
        || preparation["before"] != TRANSLATE
        || preparation["after"] != c::file_digest(&subject_source)?
    {
        return Err("retained failed native preparation differs".into());
    }
    // The retained compiler database is a pinned build input, not a small
    // untrusted session manifest. Keep the latter's64KiB limit unchanged.
    let database = old.join("compile_commands.json");
    if fs::symlink_metadata(&database)?.file_type().is_symlink()
        || fs::metadata(&database)?.len() > 16 * 1024 * 1024
    {
        return Err("retained compiler database extent refused".into());
    }
    let commands: Vec<Value> = serde_json::from_slice(&fs::read(&database)?)?;
    let subject = "libqemu-i386-softmmu.a.p/target_i386_tcg_translate.c.o";
    let cc = commands
        .iter()
        .find(|v| v["output"] == subject)
        .and_then(|v| v["command"].as_str())
        .ok_or("retained translate compile command")?;
    if cc
        .matches("../source-v2/target/i386/tcg/translate.c")
        .count()
        != 1
    {
        return Err("unexpected translate command".into());
    }
    let object = dir.join("translate.o");
    let dep = dir.join("translate.o.d");
    let command = cc
        .replace(&format!("{subject}.d"), dep.to_str().ok_or("dep path")?)
        .replace(subject, object.to_str().ok_or("object path")?)
        .replace(
            "../source-v2/target/i386/tcg/translate.c",
            subject_source.to_str().ok_or("source path")?,
        );
    let command = format!("{command} -I{}", src.join("target/i386/tcg").display());
    let ninja = fs::read_to_string(baseline.join("build.ninja"))?;
    let rule = ninja
        .split("rule link_core\n")
        .nth(1)
        .and_then(|s| s.split("\nbuild ").next())
        .ok_or("retained native link rule")?;
    let target = format!("build {}: link_core ", baseline.join("core").display());
    let inputs = ninja
        .lines()
        .find_map(|s| s.strip_prefix(&target))
        .ok_or("retained native inputs")?;
    if inputs.split_whitespace().filter(|p| *p == subject).count() != 1 {
        return Err("native translation input differs".into());
    }
    let inputs = inputs.replace(subject, object.to_str().ok_or("object path")?);
    let build = format!(
        "builddir = {}\nrule compile_translate\n command = {command}\nrule link_core\n{rule}\nbuild {}: compile_translate {}\nbuild {}: link_core {inputs}\n",
        dir.display(),
        object.display(),
        subject_source.display(),
        dir.join("core").display()
    );
    retained_or_new(&dir.join("build.ninja"), build.as_bytes())?;
    c::store::checked(
        Command::new("ninja")
            .current_dir(&old)
            .arg("-f")
            .arg(dir.join("build.ninja"))
            .args(["-j", "1"])
            .arg(dir.join("core")),
    )?;
    fs::File::open(dir.join("core"))?.sync_all()?;
    c::write_new(
        &dir.join("build.identity.json"),
        &c::json(&json!({
            "source_pin":"03b988c9d76553b33ed712ba00822ffdf49f0e7d","base_core":resources::CORE,
            "core":c::file_digest(&dir.join("core"))?,"patch":patch_hash,"source_before":TRANSLATE,"source_after":preparation["after"],
            "retained_compile_commands":c::file_digest(&old.join("compile_commands.json"))?,"retained_native_recipe":c::file_digest(&baseline.join("build.ninja"))?,
            "retained_ui_object":c::file_digest(&root.join("native-v2/ui.o"))?,"retained_hmp_object":c::file_digest(&baseline.join("hmp.o"))?,
            "recompiled_units":["target/i386/tcg/translate.c"],"compiler_jobs":1,"prior_source_builds_unchanged":true,
            "original_register_regression_qualified":false,"Fable_cause_established":false
        }))?,
    )?;
    Ok(())
}

pub(super) fn package(repo: &Path, root: &Path) -> Result<resources::Resources, Error> {
    let candidate = root.join("x87-boundary-v1");
    let identity: Value = c::read_json(&candidate.join("build.identity.json"))?;
    if identity["core"] != resources::X87_RESTART_CORE
        || identity["patch"]
            != c::file_digest(&repo.join("patches/xemu-x87-instruction-boundary.patch"))?
        || c::file_digest(&candidate.join("core"))? != resources::X87_RESTART_CORE
    {
        return Err("exact public correction candidate differs".into());
    }
    let normal = resources::Resources::open(&root.join("package-v3"))?;
    let package = root.join("package-x87-npx-v1");
    if !package.exists() {
        c::mkdir(&package)?;
        fs::create_dir_all(package.join("native/subprojects/slirp"))?;
        fs::hard_link(candidate.join("core"), package.join("native/core"))?;
        for name in ["native/qemu-img", "native/subprojects/slirp/libslirp.so.0"] {
            fs::hard_link(normal.root.join(name), package.join(name))?;
        }
        let firmware = root.join("npx-owner-port-v1/build-logging/bios.bin");
        if c::file_digest(&firmware)? != resources::NPX_LOGGING_OPEN {
            return Err("retained repaired logging image differs".into());
        }
        fs::copy(firmware, package.join("open.bin"))?;
        let mut manifest = normal.manifest.clone();
        manifest.core = resources::X87_RESTART_CORE.into();
        manifest.open_system = resources::NPX_LOGGING_OPEN.into();
        c::write_new(&package.join("runtime.json"), &c::json(&manifest)?)?;
    }
    Ok(resources::Resources::x87_logging(&package)?)
}
pub(super) fn original(repo: &Path, root: &Path) -> Result<(), Error> {
    use std::{
        io::Write,
        time::{Duration, Instant},
    };
    let dir = root.join("x87-boundary-v1");
    if [
        "original.result.json",
        "flat.result.json",
        "separate.result.json",
    ]
    .iter()
    .any(|name| dir.join(name).exists())
    {
        return Err("original candidate already attempted; preserve actual result".into());
    }
    // Reuse the clean actual old-core negative observation, not a new old run.
    let previous =
        std::path::PathBuf::from(std::env::var("XBX_X87_RETAINED_FLAT_RESULT")?).canonicalize()?;
    if !previous.starts_with(root.join("npx-owner-port-v1").canonicalize()?)
        || previous
            .file_name()
            .is_none_or(|s| s != "after.result.json")
    {
        return Err("retained original result outside assigned evidence".into());
    }
    let before: Value = c::read_json(&previous)?;
    if before["guest_passed"] != false
        || before["native_clean"] != true
        || before["firmware_sha256"] != resources::NPX_LOGGING_OPEN
        || before["same_original_fixture"]
            != "fe624afdafb26645f8beb088559988981c557b2bda357b899a275e32f0b62b1f"
    {
        return Err("retained original negative binding differs".into());
    }
    let profile = Path::new(before["profile"].as_str().ok_or("before profile")?).canonicalize()?;
    if profile.parent() != Some(root.canonicalize()?.as_path()) {
        return Err("retained original profile outside workspace".into());
    }
    let old_log = fs::read_to_string(profile.join("kernel.log"))?;
    let old_return = fs::read_to_string(profile.join("termination.txt"))?;
    let old_capture = fs::read_to_string(profile.join("serial-capture.txt"))?;
    if old_log.len() > 4 * 1024 * 1024
        || !old_capture.contains("truncated=false\n")
        || !old_capture.contains("capture_ok=true\n")
        || super::npx_port::outcome(&old_log) != Some(false)
        || !old_return.contains("forced=false\n")
        || !old_return.contains("qmp=Ok(())\n")
        || !old_return.contains("storage_synced=true\n")
    {
        return Err("retained old-core original negative not cleanly observed".into());
    }
    let disk = repo.join(".local/dashboard-v20260516-0955/xbox_hdd.qcow2");
    let eeprom = repo.join(".local/quick-reboot-d87cfc33-run1/eeprom.bin");
    let seed_before = (c::file_digest(&disk)?, c::file_digest(&eeprom)?);
    if seed_before.0 != "00d7df7a2bc235f8801764f00b7f40e194d1e392f7a9619d6b2396c89770f6dd" {
        return Err("matched original seed differs".into());
    }
    let mut results = Vec::new();
    for (case, relative, pin) in [
        (
            "flat",
            "npx-owner-port-v1/original.iso",
            "fe624afdafb26645f8beb088559988981c557b2bda357b899a275e32f0b62b1f",
        ),
        (
            "separate",
            "npx-owner-port-v1/layout-v1/original.iso",
            "372b08bfa9b9022b74985a088ecae10eeea7e25cf61823db6b0242f23b19123e",
        ),
    ] {
        let media = root.join(relative);
        if c::file_digest(&media)? != pin {
            return Err("retained original fixture differs".into());
        }
        let app = root.join(format!("x87-{case}-{}", c::new_id()?));
        let candidate = package(repo, root)?;
        if resources::Resources::open(&candidate.root).is_ok()
            || resources::Resources::npx_logging(&candidate.root).is_ok()
        {
            return Err(
                "developer candidate unexpectedly accepted by an ordinary/old runtime route".into(),
            );
        }
        let mut player = c::desktop::Player::with_resources(&app, candidate)?;
        let game = player.add(&media)?;
        let entry = c::library::Library::read(&app)?.resolve(&game)?;
        {
            let _profile =
                c::store::Profile::open_seeded(&app, &entry, &player.resources, &disk, &eeprom)?;
        }
        let start = Instant::now();
        player.launch(&game)?;
        let run = player
            .live
            .as_ref()
            .ok_or("live original")?
            .native
            .run
            .clone();
        let mut log = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(app.join("kernel.log"))?;
        let mut text = String::new();
        let observed = (|| -> Result<Option<bool>, Error> {
            loop {
                let native = &mut player.live.as_mut().ok_or("live original")?.native;
                let new = native.serial()?;
                log.write_all(new.as_bytes())?;
                text.push_str(&new);
                if text.len() > 4 * 1024 * 1024 {
                    return Err("original output bound".into());
                }
                if let Some(v) = super::npx_port::outcome(&text) {
                    return Ok(Some(v));
                }
                if native.exited()? || start.elapsed() > Duration::from_secs(90) {
                    return Ok(None);
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        })();
        log.sync_all()?;
        let returned = player.stop();
        let native: Value = c::read_json(&run.join("native-return.json"))?;
        let clean = returned.is_ok() && native["clean"] == true;
        let passed = observed.as_ref().is_ok_and(|v| *v == Some(true));
        let result = json!({"case":case,"guest_passed":passed,"native_clean":clean,"native_return":native,
            "elapsed_ms":start.elapsed().as_millis(),"fixture":pin,"app":app,"native_run":run,
            "observation_error":observed.err().map(|e|e.to_string()),"return_error":returned.err().map(|e|e.to_string())});
        c::write_new(&dir.join(format!("{case}.result.json")), &c::json(&result)?)?;
        println!("{result}");
        results.push(result);
        if !passed || !clean {
            return Err(
                "original restart regression/control failed; retain first actual result".into(),
            );
        }
    }
    if seed_before != (c::file_digest(&disk)?, c::file_digest(&eeprom)?) {
        return Err("original seeds changed".into());
    }
    c::write_new(
        &dir.join("original.result.json"),
        &c::json(
            &json!({"core":resources::X87_RESTART_CORE,"open":resources::NPX_LOGGING_OPEN,"before_core":resources::CORE,"retained_negative":previous,"results":results,"seeds_unchanged":true,"Fable_cause_established":false}),
        )?,
    )?;
    c::write_new(&dir.join("original.qualified"),b"same original flat image negative on retained core; flat and separated checks pass corrected core with clean native returns; no Fable attribution\n")?;
    Ok(())
}
