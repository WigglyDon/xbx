// SPDX-License-Identifier: MIT
//! Exact-image original relaunch through the existing cartridge product owner.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
pub(super) fn relaunch(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("npx-owner-port-v1");
    let identity: Value = c::read_json(&dir.join("build.identity.json"))?;
    let firmware = dir.join("build-logging/bios.bin");
    if identity["firmware"] != resources::NPX_LOGGING_OPEN
        || c::file_digest(&firmware)? != resources::NPX_LOGGING_OPEN
        || !fs::read_to_string(dir.join("regression.qualified"))?.contains("current release red;")
    {
        return Err("exact original-regression-qualified functional port required".into());
    }
    let fixture = repo.join(".local/build-relaunch-title/relaunch.iso");
    if c::file_digest(&fixture)?
        != "8efe94b6f8a26f63247f44bc42da9ed437e431a5ed03180ba73044140cb88491"
    {
        return Err("retained original relaunch fixture differs".into());
    }
    if dir.join("relaunch-product-v1.result.json").exists()
        || dir.join("relaunch.qualified").exists()
    {
        return Err(
            "this product qualification already attempted; preserve and reconcile it".into(),
        );
    }
    // Reuse the product's existing monitor, leases, importer and native owner.
    // The separate candidate resource remains developer-only; no default change.
    let normal = resources::Resources::open(&root.join("package-v3"))?;
    let package = root.join("package-npx-logging-v1");
    if !package.exists() {
        c::mkdir(&package)?;
        fs::create_dir_all(package.join("native/subprojects/slirp"))?;
        for name in [
            "native/core",
            "native/qemu-img",
            "native/subprojects/slirp/libslirp.so.0",
        ] {
            fs::hard_link(normal.root.join(name), package.join(name))?;
        }
        fs::copy(&firmware, package.join("open.bin"))?;
        let mut manifest = normal.manifest.clone();
        manifest.open_system = resources::NPX_LOGGING_OPEN.into();
        c::write_new(&package.join("runtime.json"), &c::json(&manifest)?)?;
    }
    let app = root.join(format!("npx-relaunch-{}", c::new_id()?));
    let mut player =
        c::desktop::Player::with_resources(&app, resources::Resources::npx_logging(&package)?)?;
    let game = player.add(&fixture)?;
    let entry = c::library::Library::read(&app)?.resolve(&game)?;
    let disk = repo.join(".local/dashboard-v20260516-0955/xbox_hdd.qcow2");
    let eeprom = repo.join(".local/quick-reboot-d87cfc33-run1/eeprom.bin");
    let seed_before = (c::file_digest(&disk)?, c::file_digest(&eeprom)?);
    if seed_before.0 != "00d7df7a2bc235f8801764f00b7f40e194d1e392f7a9619d6b2396c89770f6dd" {
        return Err("matched disk seed differs".into());
    }
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
    let observed = (|| -> Result<bool, Error> {
        loop {
            let native = &mut player.live.as_mut().ok_or("live original")?.native;
            let new = native.serial()?;
            log.write_all(new.as_bytes())?;
            text.push_str(&new);
            if text.len() > 4 * 1024 * 1024 {
                return Err("original output bound".into());
            }
            if completed(&text) {
                return Ok(true);
            }
            if native.exited()? || start.elapsed() > Duration::from_secs(90) {
                return Ok(false);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    })();
    log.sync_all()?;
    let returned = player.stop();
    let native: Value = c::read_json(&run.join("native-return.json"))?;
    let seeds_unchanged = seed_before == (c::file_digest(&disk)?, c::file_digest(&eeprom)?);
    let clean = returned.is_ok() && native["clean"] == true && seeds_unchanged;
    let complete = observed.as_ref().is_ok_and(|v| *v);
    let result = json!({"firmware":resources::NPX_LOGGING_OPEN,"first_stage_relaunch_completed":complete,
        "second_stage_execution_qualified":false,"native_clean":clean,"native_return":native,
        "elapsed_ms":start.elapsed().as_millis(),"fixture_unchanged":true,"seeds_unchanged":seeds_unchanged,
        "product_session_path":true,"app":app,"native_run":run,
        "observation_error":observed.err().map(|e|e.to_string()),"return_error":returned.err().map(|e|e.to_string()),
        "prior_forced_profile_retained":true,"paused_diagnostics":false});
    c::write_new(
        &dir.join("relaunch-product-v1.result.json"),
        &c::json(&result)?,
    )?;
    println!("{result}");
    if !complete || !clean {
        return Err(
            "exact-image original product qualification failed; preserve actual result".into(),
        );
    }
    c::write_new(&dir.join("relaunch.qualified"),format!("bios_sha256={}\nretained_original_first_stage_relaunch_completed=true\nsecond_stage_execution=unqualified\nproduct_session_path=true\n",resources::NPX_LOGGING_OPEN).as_bytes())?;
    Ok(())
}
fn completed(text: &str) -> bool {
    text.contains(": cold boot")
        && text.contains(": quick boot")
        && text.contains("load \\Device\\CdRom0\\default.xbe: 0x0")
        && text.contains("load \\Device\\CdRom0\\media\\second.xbe: 0x0")
}
