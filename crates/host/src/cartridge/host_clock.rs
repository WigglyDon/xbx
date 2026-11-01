// SPDX-License-Identifier: MIT
//! Finite original clock/timer comparison motivated by a stalled KVM scene.
//! The retained cores and open system are reused; no game content is inspected.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
const EXPORTS: &str = "aa5c07f0bb9a0c1053dc9aadd4d4271ba53d3a8b69c49356d8479faeb66d7e7b";

pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("host-clock-v1");
    c::mkdir(&dir)?;
    let exports = root.join("source-inputs/exports.tsv");
    if c::file_digest(&exports)? != EXPORTS {
        return Err("retained independent public ABI declaration differs".into());
    }
    let abi = fs::read_to_string(exports)?;
    for declaration in [
        "126\tKeQueryPerformanceCounter\tstdcall\t0\tfunc",
        "127\tKeQueryPerformanceFrequency\tstdcall\t0\tfunc",
        "99\tKeDelayExecutionThread\tstdcall\t12\tfunc",
        "125\tKeQueryInterruptTime\tstdcall\t0\tfunc",
        "156\tKeTickCount\t\t\tdata",
    ] {
        if !abi.lines().any(|v| v == declaration) {
            return Err("original clock ABI declaration mismatch".into());
        }
    }
    let source = repo.join("native/fixtures/clock-continuity.S");
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
        "xbx original clock continuity",
        0x5842434b,
    )?;
    c::write_new(
        &dir.join("build.json"),
        &c::json(&json!({
            "source":c::file_digest(&source)?,"uart_source":c::file_digest(&uart_source)?,
            "exports":EXPORTS,"fixture":fixture,"original_content_only":true,
            "contract":"3375000Hz performance counter; about 1000 tick/10000000 interrupt units per performance-counter second; one-second non-alertable delay; repeat after native checkpoint return",
            "timing_bounds":"wide 0.5–2.0 seconds; host timestamps are serial receipt times, not a benchmark",
            "no_core_or_kernel_rebuild":true,"commercial_acceptance":false
        }))?,
    )?;
    Ok(())
}

pub(super) fn observe(repo: &Path, root: &Path, kvm: bool, returning: bool) -> Result<(), Error> {
    if returning && !kvm {
        return Err("only the observed KVM return is assigned".into());
    }
    let dir = root.join("host-clock-v1");
    let built: Value = c::read_json(&dir.join("build.json"))?;
    let media = dir.join("original.iso");
    if built["fixture"] != c::file_digest(&media)?
        || built["source"] != c::file_digest(&repo.join("native/fixtures/clock-continuity.S"))?
        || built["exports"] != EXPORTS
    {
        return Err("original clock input differs".into());
    }
    if returning {
        let prior: Value = c::read_json(&dir.join("kvm-cold/result.json"))?;
        if prior["passed"] != true || prior["core"] != super::host_input::CORE {
            return Err("actual same-core cold clock proof required".into());
        }
    }
    let label = if returning {
        "kvm-return"
    } else if kvm {
        "kvm-cold"
    } else {
        "tcg-cold"
    };
    let out = dir.join(label);
    c::mkdir(&out)?;
    let mut r = if kvm {
        resources::Resources::controller_candidate(&root.join("host-input-v1/package-signed"))?
    } else {
        resources::Resources::apu_quiescent(&root.join("apu-quiescence-v1/package"))?
    };
    r.window = false;
    let core = r.manifest.core.clone();
    let app = dir.join(if kvm { "kvm-app" } else { "tcg-app" });
    let mut player = c::desktop::Player::with_resources(&app, r)?;
    let game = if returning {
        player.last_game()?.ok_or("clock entry")?
    } else {
        player.add(&media)?
    };
    let mut run = None;
    let mut text = String::new();
    let mut events = Vec::new();
    let expected = format!("XBX CLOCK PASS cycle={:08X}", if returning { 2 } else { 1 });
    let observed = (|| -> Result<(), Error> {
        if player.launch(&game)? != returning {
            return Err("clock cold/restore boundary differs".into());
        }
        run = Some(player.live.as_ref().ok_or("live")?.native.run.clone());
        if returning {
            player
                .live
                .as_mut()
                .ok_or("live")?
                .native
                .fixture_input("c")?;
        }
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(20) {
            let next = player.live.as_mut().ok_or("live")?.native.serial()?;
            if !next.is_empty() {
                events.push(json!({"host_received_ms":start.elapsed().as_millis(),"text":next}));
                text.push_str(&next);
            }
            if text.len() > 16384 {
                return Err("clock output bound".into());
            }
            if text.contains("XBX CLOCK FAIL")
                || text.contains("*** bug check")
                || text.contains("*** unexpected exception")
            {
                return Err("original clock contract failed".into());
            }
            if text.contains(&expected) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Err("original clock response deadline".into())
    })();
    let returned = if observed.is_ok() {
        player.depart()
    } else {
        player.stop()
    };
    let native = match &run {
        Some(run) => c::read_json::<Value>(&run.join("native-return.json"))?,
        None => Value::Null,
    };
    c::write_new(&out.join("serial.log"), text.as_bytes())?;
    let passed = observed.is_ok() && returned.is_ok() && native["clean"] == true;
    c::write_new(
        &out.join("result.json"),
        &c::json(&json!({
            "passed":passed,"core":core,"open_system":resources::LOCAL_BASE_OPEN,
            "fixture":built["fixture"],"kvm":kvm,"returning":returning,"native_run":run,
            "events":events,"native_return":native,"observation_error":observed.err().map(|e|e.to_string()),
            "return_error":returned.err().map(|e|e.to_string()),"generated_storage_only":true,
            "commercial_acceptance":false,"guest_code_or_data_inspection":false
        }))?,
    )?;
    if !passed {
        return Err("clock qualification failed; preserve results and current data".into());
    }
    Ok(())
}
