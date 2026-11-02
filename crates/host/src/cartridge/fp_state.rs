// SPDX-License-Identifier: MIT
//! Original FP machine-state continuity on the changed native build. The
//! retained open system is unchanged; no title instructions or data are inputs.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    build_for(repo, root, false)
}
pub(super) fn build_sse1(repo: &Path, root: &Path) -> Result<(), Error> {
    build_for(repo, root, true)
}
fn build_for(repo: &Path, root: &Path, sse1: bool) -> Result<(), Error> {
    let dir = root.join(if sse1 {
        "fp-state-sse1-v1"
    } else {
        "fp-state-v1"
    });
    c::mkdir(&dir)?;
    let source = repo.join("native/fixtures/fp-continuity.S");
    let uart_source = repo.join("native/fixtures/startup-lifetime.S");
    let uart = fs::read_to_string(&uart_source)?;
    let uart = uart
        .split_once("serial_init:\n")
        .ok_or("UART start")?
        .1
        .split_once(".balign 4\n")
        .ok_or("UART end")?
        .0;
    let original = fs::read_to_string(&source)?;
    let adapted = if sse1 {
        if original.matches("movdqu").count() != 2 {
            return Err("original FP transfer sites differ; no adaptation".into());
        }
        original.replace("movdqu", "movups")
    } else {
        original
    };
    let generated = format!("{adapted}\n.text\nserial_init:\n{uart}");
    let fixture = super::cpu_performance::assemble_two_page(
        &root.join("fixtures-v2/a.iso"),
        &dir,
        &generated,
        if sse1 {
            "xbx original SSE FP continuity"
        } else {
            "xbx original FP continuity"
        },
        if sse1 { 0x58424651 } else { 0x58424650 },
    )?;
    c::write_new(
        &dir.join("build.json"),
        &c::json(
            &json!({"source":c::file_digest(&source)?,"uart_source":c::file_digest(&uart_source)?,"fixture":fixture,"original_only":true,"kernel_changed":false,"SSE1_bit_transfer_adaptation":sse1,"original_artifact_retained":true,"contract":"noninitial live x87 stack/control word, XMM0, MXCSR and RAM cookie survive the native checkpoint","commercial_acceptance":false}),
        )?,
    )?;
    Ok(())
}
fn wait(player: &mut c::desktop::Player, marker: &str, log: &mut String) -> Result<(), Error> {
    let start = Instant::now();
    let mut current = String::new();
    while start.elapsed() < Duration::from_secs(20) {
        let s = player.live.as_mut().ok_or("live")?.native.serial()?;
        current.push_str(&s);
        log.push_str(&s);
        if log.len() > 16384 {
            return Err("original FP output bound".into());
        }
        if current.contains("XBX FP_STATE FAIL")
            || current.contains("*** bug check")
            || current.contains("*** unexpected exception")
        {
            return Err("original FP state mismatch".into());
        }
        if current.contains(marker) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err("original FP response deadline".into())
}
pub(super) fn observe(repo: &Path, root: &Path, mode: &str) -> Result<(), Error> {
    let dir = root.join("fp-state-v1");
    let (app_name, returning) = match mode {
        "base" => ("base-app", false),
        "cross" => ("cross-app", true),
        "same-first" => ("same-app", false),
        "same-return" => ("same-app", true),
        _ => return Err("finite FP mode required".into()),
    };
    let built: Value = c::read_json(&dir.join("build.json"))?;
    let media = dir.join("original.iso");
    if built["source"] != c::file_digest(&repo.join("native/fixtures/fp-continuity.S"))?
        || built["fixture"] != c::file_digest(&media)?
    {
        return Err("original FP input differs".into());
    }
    let out = dir.join(mode);
    c::mkdir(&out)?;
    let app = dir.join(app_name);
    if mode == "cross" {
        let v: Value = c::read_json(&dir.join("base/result.json"))?;
        if v["passed"] != true || v["core"] != resources::APU_QUIESCENT_CORE {
            return Err("original FP baseline required".into());
        }
        let old = dir.join("base-app");
        let _lease = crate::storage::ProfileLease::acquire(&old)?;
        c::store::checked(
            Command::new("cp")
                .args(["-a", "--reflink=always"])
                .arg(old)
                .arg(&app),
        )?;
    }
    if mode == "same-return" {
        let v: Value = c::read_json(&dir.join("same-first/result.json"))?;
        if v["passed"] != true || v["core"] != super::smc::CORE {
            return Err("same-core FP cold proof required".into());
        }
    }
    let r = if mode == "base" {
        resources::Resources::apu_quiescent(&root.join("apu-quiescence-v1/package"))?
    } else if mode == "cross" {
        resources::Resources::smc_candidate(&root.join("smc-v1/package"))?
    } else {
        resources::Resources::smc_fresh(&root.join("smc-v1/package"))?
    };
    let core = r.manifest.core.clone();
    let runtime = r.identity.clone();
    let mut player = c::desktop::Player::with_resources(&app, r)?;
    let game = if returning {
        player.last_game()?.ok_or("retained FP entry")?
    } else {
        player.add(&media)?
    };
    let mut run = None;
    let mut log = String::new();
    let observed = (|| -> Result<(), Error> {
        if player.launch(&game)? != returning {
            return Err("FP cold/restore boundary differs".into());
        }
        run = Some(player.live.as_ref().ok_or("live")?.native.run.clone());
        if !returning {
            wait(&mut player, "XBX FP_STATE READY", &mut log)?;
        }
        player
            .live
            .as_mut()
            .ok_or("live")?
            .native
            .fixture_input("v")?;
        wait(&mut player, "XBX FP_STATE PASS", &mut log)
    })();
    let returned = if observed.is_ok() {
        player.depart()
    } else {
        player.stop()
    };
    let native = match &run {
        Some(p) => c::read_json::<Value>(&p.join("native-return.json"))?,
        None => Value::Null,
    };
    let passed = observed.is_ok() && returned.is_ok() && native["clean"] == true;
    c::write_new(&out.join("serial.log"), log.as_bytes())?;
    c::write_new(
        &out.join("result.json"),
        &c::json(
            &json!({"passed":passed,"mode":mode,"core":core,"runtime":runtime,"restored":returning,"fixture":built["fixture"],"native_run":run,"native_return":native,"observation_error":observed.err().map(|e|e.to_string()),"return_error":returned.err().map(|e|e.to_string()),"kernel_unchanged":true,"commercial_acceptance":false}),
        )?,
    )?;
    if !passed {
        return Err("original FP continuity failed; preserve all evidence".into());
    }
    Ok(())
}
