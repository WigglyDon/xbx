// SPDX-License-Identifier: MIT
//! Reuse the accepted displaced-owner repair on the current open runtime.
//! The original regression changes only its output transport, not its checks.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
const PATCH: &str = "86463b1ada31746ed652d1db88bede4e61c5b1acc5157979fe616669aa20291f";
const ORIGINAL: &str = "70d6e0a45c60dcc9b2b0b524c3dd6f8a07d5558c78cc31074db90e00bf278591";
const NPX_BEFORE: &str = "4caab333b579653d16a95c502c9221dc4579b2af981f352d0db78f2635e18fd9";
fn put(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn copied(source: &Path, dest: &Path) -> Result<(), Error> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    c::write_new(dest, &fs::read(source)?)?;
    Ok(())
}
pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("npx-owner-port-v1");
    c::mkdir(&dir)?;
    let project = root.parent().ok_or("project")?;
    let old = project.join("open-startup-v1/source-v3");
    if c::file_digest(&old.join("kernel/src/ke/npx.cpp"))? != NPX_BEFORE
        || c::file_digest(&repo.join("patches/oxos-npx-owner-functional.patch"))? != PATCH
        || c::file_digest(&repo.join("native/fixtures/npx-ownership.S"))? != ORIGINAL
    {
        return Err("retained public source/accepted repair/original regression differs".into());
    }
    let source = dir.join("source");
    c::mkdir(&source)?;
    let manifest = fs::read_to_string(old.join("source.objects"))?;
    for line in manifest.lines() {
        let (metadata, relative) = line.split_once('\t').ok_or("source manifest")?;
        if relative.starts_with('/') || relative.split('/').any(|p| p == "..") {
            return Err("source path".into());
        }
        let expected = metadata.split_whitespace().nth(2).ok_or("source blob")?;
        let input = old.join(relative);
        let got = Command::new("git")
            .arg("hash-object")
            .arg(&input)
            .output()?;
        if !got.status.success() || String::from_utf8(got.stdout)?.trim() != expected {
            return Err("retained corrected public source bytes differ".into());
        }
        copied(&input, &source.join(relative))?;
    }
    for name in [
        "source.identity",
        "source.objects",
        "phase.identity",
        "required.identity",
    ] {
        copied(&old.join(name), &source.join(name))?;
    }
    let archive = project.join("open-startup-v1/cromwell-v3.tar");
    if c::file_digest(&archive)?
        != "280b2b1dce7d73ba40d9e4d027fb8f54624fa7db0724e2f431d813f85ef29dcb"
    {
        return Err("retained pinned public boot source archive differs".into());
    }
    fs::create_dir_all(source.join("third_party/cromwell"))?;
    c::store::checked(
        Command::new("tar")
            .args(["-x", "--no-same-owner", "-f"])
            .arg(archive)
            .arg("-C")
            .arg(source.join("third_party/cromwell")),
    )?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .arg("apply")
            .arg(repo.join("patches/oxos-npx-owner-functional.patch")),
    )?;
    let mut changed = Vec::new();
    for line in manifest.lines() {
        let (_, relative) = line.split_once('\t').ok_or("manifest")?;
        if fs::read(old.join(relative))? != fs::read(source.join(relative))? {
            changed.push(relative);
        }
    }
    if changed != ["kernel/src/ke/npx.cpp"] {
        return Err("functional port changed unrelated source".into());
    }
    c::write_new(
        &dir.join("source.identity.json"),
        &c::json(&json!({
            "source":"9d8cd58ca19f49f5ecb4b7b55bfa194f4e4a5a96",
            "cromwell":"86f547387184d1001d377cce97d3756c8acf91cc",
            "retained_corrected_source_objects":c::file_digest(&old.join("source.objects"))?,
            "accepted_repair":"fcbaeab8f544f6ab4b78c027c032ba429549bd49",
            "functional_patch":PATCH,"changed":changed,"before":NPX_BEFORE,
            "after":c::file_digest(&source.join("kernel/src/ke/npx.cpp"))?,
            "paused_instrumentation":false
        }))?,
    )?;
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    // Drop COFF-only symbol annotations, keep all original code/data/checks.
    // Flat ELF layout permits reuse of the already-qualified original XBE template.
    let original = fs::read_to_string(repo.join("native/fixtures/npx-ownership.S"))?;
    let original = &original[original.find(".text\n").ok_or("original text")?..];
    if original.matches("call *_XkThunks+36").count() != 3 {
        return Err("original result call sites differ".into());
    }
    let adapted = original
        .replace(
            "_XkTitleEntry:\n",
            "_XkTitleEntry:\n    call npx_serial_init\n",
        )
        .replace("call *_XkThunks+36", "call npx_report")
        .replace("\n.data\n", "\n.text\n")
        + &fs::read_to_string(repo.join("native/fixtures/npx-uart.S"))?;
    c::write_new(&dir.join("original-uart.S"), adapted.as_bytes())?;
    c::store::checked(
        Command::new(llvm.join("clang"))
            .args(["--target=i386-none-elf", "-march=pentium3", "-c"])
            .arg(dir.join("original-uart.S"))
            .arg("-o")
            .arg(dir.join("original.o")),
    )?;
    c::store::checked(
        Command::new(llvm.join("ld.lld"))
            .args([
                "-m",
                "elf_i386",
                "-Ttext=0x11000",
                "--oformat=binary",
                "--entry=_XkTitleEntry",
            ])
            .arg(dir.join("original.o"))
            .arg("-o")
            .arg(dir.join("original.bin")),
    )?;
    let symbols = Command::new(llvm.join("llvm-nm"))
        .args(["-n", "--defined-only"])
        .arg(dir.join("original.o"))
        .output()?;
    if !symbols.status.success() {
        return Err("original symbol query".into());
    }
    let symbols = String::from_utf8(symbols.stdout)?;
    let offset = symbols
        .lines()
        .find(|s| s.ends_with(" T _XkThunks"))
        .and_then(|s| s.split_whitespace().next())
        .ok_or("original thunk symbol")?;
    let thunk = 0x11000 + u32::from_str_radix(offset, 16)?;
    let code = fs::read(dir.join("original.bin"))?;
    if code.is_empty() || code.len() > 4096 {
        return Err("original extent".into());
    }
    let template = root.join("fixtures-v2/a.iso");
    if c::file_digest(&template)? != resources::ORIGINAL_FIXTURES[0] {
        return Err("fixture template".into());
    }
    let original = fs::read(template)?;
    let mut xbe = original[34 * 2048..34 * 2048 + 4096].to_vec();
    put(&mut xbe, 0x158, thunk ^ 0x5b6d40b6);
    put(&mut xbe, 0x180, 0x58424e50);
    xbe[0x184..0x1d4].fill(0);
    for (i, ch) in "xbx original NPX owner".encode_utf16().enumerate() {
        xbe[0x184 + i * 2..0x186 + i * 2].copy_from_slice(&ch.to_le_bytes());
    }
    put(&mut xbe, 0x348 + 16, code.len() as u32);
    xbe[0x348 + 36..0x348 + 56].copy_from_slice(&super::build::section_digest(&code)?);
    xbe.extend_from_slice(&code);
    c::write_new(&dir.join("original.xbe"), &xbe)?;
    let mut iso = vec![0; 64 * 2048];
    iso[..34 * 2048].copy_from_slice(&original[..34 * 2048]);
    put(&mut iso, 33 * 2048 + 8, xbe.len() as u32);
    iso[34 * 2048..34 * 2048 + xbe.len()].copy_from_slice(&xbe);
    c::write_new(&dir.join("original.iso"), &iso)?;
    let out = dir.join("build-logging");
    c::mkdir(&out)?;
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
            "firmware":c::file_digest(&out.join("bios.bin"))?,
            "kernel":c::file_digest(&out.join("xboxkrnl.exe"))?,
            "map":c::file_digest(&out.join("xboxkrnl.map"))?,
            "fixture_original_source":ORIGINAL,"fixture_uart_adapter":c::file_digest(&repo.join("native/fixtures/npx-uart.S"))?,
            "fixture_generated_source":c::file_digest(&dir.join("original-uart.S"))?,
            "fixture_iso":c::file_digest(&dir.join("original.iso"))?,"functional_patch":PATCH,
            "serial":true,"trace_imports":false,"core":resources::CORE,
            "clang":c::file_digest(&llvm.join("clang"))?,"linker":c::file_digest(&llvm.join("lld-link"))?
        }))?,
    )?;
    Ok(())
}
fn outcome(text: &str) -> Option<bool> {
    if !text.contains("NPX OWNERSHIP BOOT") {
        return None;
    }
    if text.contains("NPX RESULT=BAD00302") && text.contains("NPX RESULT=BAD00205") {
        return Some(false);
    }
    if text.contains("NPX RESULT=600DCAFE")
        && text.contains("NPX RESULT=4E505801")
        && !text.contains("NPX RESULT=BAD")
    {
        return Some(true);
    }
    None
}
pub(super) fn observe(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("npx-owner-port-v1");
    let identity: Value = c::read_json(&dir.join("build.identity.json"))?;
    let old = root
        .parent()
        .ok_or("project")?
        .join("open-startup-v1/build-basic-v3/bios.bin");
    let new = dir.join("build-logging/bios.bin");
    let media = dir.join("original.iso");
    let r = resources::Resources::open(&root.join("package-v3"))?;
    if c::file_digest(&old)? != resources::LOGGING_OPEN
        || identity["firmware"] != c::file_digest(&new)?
        || identity["fixture_iso"] != c::file_digest(&media)?
        || identity["functional_patch"] != PATCH
    {
        return Err("qualified inputs changed".into());
    }
    for (case, flash, expected) in [("before", old, false), ("after", new, true)] {
        let profile = dir.join(format!("profile-{case}"));
        if profile.exists() {
            return Err("original attempt already exists; preserve it".into());
        }
        let log = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(format!("{case}-supervisor.log")))?;
        let mut command = Command::new(std::env::current_exe()?.with_file_name("xbx-host"));
        command
            .arg("run-title")
            .arg(r.core())
            .arg(&profile)
            .arg(flash)
            .arg(repo.join(".local/dashboard-v20260516-0955/xbox_hdd.qcow2"))
            .arg(repo.join(".local/quick-reboot-d87cfc33-run1/eeprom.bin"))
            .arg(&media)
            .arg("90")
            .env("XBX_DESKTOP_MEMORY_MIB", "64")
            .env("XBX_DESKTOP_VIDEO_DRIVER", "x11")
            .env("XBX_DESKTOP_GRAPHICS", "host")
            .env("XBX_DESKTOP_CONTROLLER", "keyboard")
            .env("XBX_DESKTOP_FP", "native")
            .env("XBX_DESKTOP_BASIC_SERIAL", "1")
            .stdout(log.try_clone()?)
            .stderr(log)
            .stdin(Stdio::null());
        let mut owned = crate::probe::OwnedChild(command.spawn()?);
        let start = Instant::now();
        let mut text = String::new();
        let mut stopped = false;
        let status = loop {
            if let Ok(t) = fs::read_to_string(profile.join("kernel.log")) {
                if t.len() > 4 * 1024 * 1024 {
                    return Err("original output bound".into());
                }
                text = t;
            }
            if !stopped && outcome(&text).is_some() {
                c::write_new(
                    &profile.join("observation.complete"),
                    b"original NPX final response; require native return\n",
                )?;
                stopped = true;
            }
            if let Some(s) = owned.0.try_wait()? {
                break s;
            }
            if start.elapsed() > Duration::from_secs(110) {
                return Err("original supervisor bound".into());
            }
            std::thread::sleep(Duration::from_millis(100));
        };
        let termination = fs::read_to_string(profile.join("termination.txt"))?;
        let clean = status.success()
            && termination.contains("forced=false\n")
            && termination.contains("qmp=Ok(())\n")
            && termination.contains("storage_synced=true\n");
        let passed = outcome(&text);
        let result = json!({"case":case,"guest_passed":passed,"expected":expected,"native_clean":clean,
            "elapsed_ms":start.elapsed().as_millis(),"firmware_sha256":c::file_digest(&profile.join("flash.bin")).ok(),
            "same_original_fixture":identity["fixture_iso"],"trace_imports":false,"Fable_cause_unattributed":true});
        c::write_new(&dir.join(format!("{case}.result.json")), &c::json(&result)?)?;
        println!("{result}");
        if !clean || passed != Some(expected) {
            return Err(
                "original current-release regression/return did not match; preserve first failure"
                    .into(),
            );
        }
    }
    c::write_new(&dir.join("regression.qualified"),b"current release red; same original regression green after accepted functional port; clean native returns; no Fable attribution\n")?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::outcome;
    #[test]
    fn final_parent_and_child_results_are_both_required() {
        assert_eq!(outcome("NPX OWNERSHIP BOOT\nNPX RESULT=600DCAFE"), None);
        assert_eq!(outcome("NPX RESULT=600DCAFE\nNPX RESULT=4E505801"), None);
        assert_eq!(
            outcome("NPX OWNERSHIP BOOT\nNPX RESULT=600DCAFE\nNPX RESULT=4E505801"),
            Some(true)
        );
        assert_eq!(
            outcome("NPX OWNERSHIP BOOT\nNPX RESULT=BAD00302\nNPX RESULT=BAD00205"),
            Some(false)
        );
        assert_eq!(
            outcome(
                "NPX OWNERSHIP BOOT\nNPX RESULT=BAD00302\nNPX RESULT=600DCAFE\nNPX RESULT=4E505801"
            ),
            None
        );
    }
}
