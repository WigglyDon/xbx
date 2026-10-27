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
    fixture(repo, root, &dir, &llvm, false)?;
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
fn fixture(repo: &Path, root: &Path, dir: &Path, llvm: &Path, values: bool) -> Result<(), Error> {
    // Drop COFF-only symbol annotations, keep all original code/data/checks.
    // Preserve separate code/data pages: stores must not invalidate executing code.
    let original = fs::read_to_string(repo.join("native/fixtures/npx-ownership.S"))?;
    let original = &original[original.find(".text\n").ok_or("original text")?..];
    if original.matches("call *_XkThunks+36").count() != 3 {
        return Err("original result call sites differ".into());
    }
    let mut adapted = original
        .replace(
            "_XkTitleEntry:\n",
            "_XkTitleEntry:\n    call npx_serial_init\n",
        )
        .replace("call *_XkThunks+36", "call npx_report")
        + &fs::read_to_string(repo.join("native/fixtures/npx-uart.S"))?;
    if values {
        let hook = "npx_report:\n    pushfl\n    pushal\n";
        if adapted.matches(hook).count() != 1 {
            return Err("ordinary result adapter hook differs".into());
        }
        adapted = adapted.replace(hook, &(hook.to_owned() + "    cmpl $0xbad00204,%ecx\n    jne npx_no_values\n    call npx_values\nnpx_no_values:\n"));
        // Non-pop store checks the original initial value before the save API.
        adapted = adapted.replace("    fldl _ParentValue\n", "    fninit\n    fld1\n    fstpl npx_unit_value\n    fldl _ParentValue\n    fstl npx_before_save\n");
        adapted = adapted.replace(
            "    call *_XkThunks+16\n",
            "    call *_XkThunks+16\n    movl %eax,npx_restore_status\n",
        );
        adapted.push_str(&fs::read_to_string(
            repo.join("native/fixtures/npx-values.S"),
        )?);
    }
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
                "-Tdata=0x12000",
                "--entry=_XkTitleEntry",
            ])
            .arg(dir.join("original.o"))
            .arg("-o")
            .arg(dir.join("original.elf")),
    )?;
    let symbols = Command::new(llvm.join("llvm-nm"))
        .args(["-n", "--defined-only"])
        .arg(dir.join("original.elf"))
        .output()?;
    if !symbols.status.success() {
        return Err("original symbol query".into());
    }
    let symbols = String::from_utf8(symbols.stdout)?;
    let offset = symbols
        .lines()
        .find(|s| s.ends_with(" D _XkThunks"))
        .and_then(|s| s.split_whitespace().next())
        .ok_or("original thunk symbol")?;
    let thunk = u32::from_str_radix(offset, 16)?;
    c::store::checked(
        Command::new(llvm.join("llvm-objcopy"))
            .args(["-O", "binary"])
            .arg(dir.join("original.elf"))
            .arg(dir.join("original.bin")),
    )?;
    let code = fs::read(dir.join("original.bin"))?;
    if code.len() <= 4096 || code.len() > 8192 || !(0x12000..0x13000).contains(&thunk) {
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
    put(&mut xbe, 0x10c, 0x3000);
    put(&mut xbe, 0x11c, 2);
    xbe[0x408..0x40e].copy_from_slice(b".data\0");
    for (index, flags, address, raw, bytes) in [
        (0usize, 6, 0x11000, 4096, &code[..4096]),
        (1, 3, 0x12000, 8192, &code[4096..]),
    ] {
        let at = 0x348 + index * 56;
        xbe[at..at + 56].fill(0);
        for (offset, value) in [
            (0, flags),
            (4, address),
            (8, 4096),
            (12, raw),
            (16, bytes.len() as u32),
            (20, 0x10400 + index as u32 * 8),
            (28, 0x10410 + index as u32 * 4),
            (32, 0x10412 + index as u32 * 4),
        ] {
            put(&mut xbe, at + offset, value);
        }
        xbe[at + 36..at + 56].copy_from_slice(&super::build::section_digest(bytes)?);
    }
    xbe.extend_from_slice(&code);
    c::write_new(&dir.join("original.xbe"), &xbe)?;
    let mut iso = vec![0; 64 * 2048];
    iso[..34 * 2048].copy_from_slice(&original[..34 * 2048]);
    put(&mut iso, 33 * 2048 + 8, xbe.len() as u32);
    iso[34 * 2048..34 * 2048 + xbe.len()].copy_from_slice(&xbe);
    c::write_new(&dir.join("original.iso"), &iso)?;
    Ok(())
}

pub(super) fn values_fixture(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("npx-owner-port-v1/values-v3");
    c::mkdir(&dir)?;
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    if c::file_digest(&repo.join("native/fixtures/npx-ownership.S"))? != ORIGINAL {
        return Err("original fixture differs".into());
    }
    fixture(repo, root, &dir, &llvm, true)?;
    c::write_new(
        &dir.join("build.identity.json"),
        &c::json(&json!({
            "original_source":ORIGINAL,"uart_adapter":c::file_digest(&repo.join("native/fixtures/npx-uart.S"))?,
            "owned_value_output_adapter":c::file_digest(&repo.join("native/fixtures/npx-values.S"))?,
            "generated_source":c::file_digest(&dir.join("original-uart.S"))?,"fixture_iso":c::file_digest(&dir.join("original.iso"))?,
            "fields":["restore_return_status","parent_observed_low","parent_observed_high","observed_xmm0_0","observed_xmm0_1","observed_xmm0_2","observed_xmm0_3","original_parent_pattern_0","original_parent_pattern_1","original_parent_pattern_2","original_parent_pattern_3","parent_initial_low","parent_initial_high","original_parent_constant_low","original_parent_constant_high","initialized_unit_low","initialized_unit_high"],
            "explicit_initialization_and_unit_control":true,"initial_value_non_pop_store":true,"only_original_guest_owned_data":true,"no_kernel_observer_or_game_memory_read":true
        }))?,
    )?;
    Ok(())
}

pub(super) fn layout_fixture(repo: &Path, root: &Path) -> Result<(), Error> {
    let parent = root.join("npx-owner-port-v1");
    let dir = parent.join("layout-v1");
    c::mkdir(&dir)?;
    if c::file_digest(&repo.join("native/fixtures/npx-ownership.S"))? != ORIGINAL {
        return Err("original regression differs".into());
    }
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    fixture(repo, root, &dir, &llvm, false)?;
    let mut identity: Value = c::read_json(&parent.join("build.identity.json"))?;
    identity["fixture_iso"] = json!(c::file_digest(&dir.join("original.iso"))?);
    identity["fixture_generated_source"] = json!(c::file_digest(&dir.join("original-uart.S"))?);
    identity["retained_firmware_identity"] =
        json!(c::file_digest(&parent.join("build.identity.json"))?);
    identity["separate_code_and_writable_data_pages"] = json!(true);
    c::write_new(&dir.join("build.identity.json"), &c::json(&identity)?)?;
    Ok(())
}

pub(super) fn outcome(text: &str) -> Option<bool> {
    if !text.contains("NPX OWNERSHIP BOOT") {
        return None;
    }
    let parent_pass = text.contains("NPX RESULT=4E505801");
    let child_pass = text.contains("NPX RESULT=600DCAFE");
    // Stage4 checks successful restore and the parent's x87/SSE values. Its
    // failure deliberately withholds ParentRestored, so the child times out.
    let parent_restore_failed =
        text.contains("NPX RESULT=BAD00204") && text.contains("NPX RESULT=BAD00301");
    let child_restore_failed =
        text.contains("NPX RESULT=BAD00302") && text.contains("NPX RESULT=BAD00205");
    if (parent_restore_failed || child_restore_failed) && !parent_pass && !child_pass {
        return Some(false);
    }
    if child_pass && parent_pass && !text.contains("NPX RESULT=BAD") {
        return Some(true);
    }
    None
}
fn profile_path(root: &Path, attempt: &str, before: bool) -> Result<std::path::PathBuf, Error> {
    if !c::valid_id(attempt) {
        return Err("invalid observation identity".into());
    }
    let path = root.join(format!(
        "np-{}-{}",
        &attempt[..12],
        if before { "b" } else { "a" }
    ));
    if path.join("qmp.sock").as_os_str().len() > 100 {
        return Err("workspace root is too long for this native QMP socket".into());
    }
    Ok(path)
}
pub(super) fn observe(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("npx-owner-port-v1");
    if dir.join("regression.qualified").exists() {
        return Err("this comparison is already qualified; retain its result".into());
    }
    let fixture_dir = if dir.join("layout-v1/build.identity.json").exists() {
        dir.join("layout-v1")
    } else {
        dir.clone()
    };
    let identity: Value = c::read_json(&fixture_dir.join("build.identity.json"))?;
    let old = root
        .parent()
        .ok_or("project")?
        .join("open-startup-v1/build-basic-v3/bios.bin");
    let new = dir.join("build-logging/bios.bin");
    let media = fixture_dir.join("original.iso");
    let r = resources::Resources::open(&root.join("package-v3"))?;
    if c::file_digest(&old)? != resources::LOGGING_OPEN
        || identity["firmware"] != c::file_digest(&new)?
        || identity["fixture_iso"] != c::file_digest(&media)?
        || identity["functional_patch"] != PATCH
    {
        return Err("qualified inputs changed".into());
    }
    // Environment refusals must not overwrite their profiles or become guest
    // compatibility failures. A subsequent admitted attempt gets fresh storage.
    let attempt_id = c::new_id()?;
    let attempt = dir.join(format!("observation-{attempt_id}"));
    c::mkdir(&attempt)?;
    let mut results = Vec::new();
    // A clean original negative control is evidence, not a reason to rerun it
    // after correcting its result classification. Its files are pinned by the
    // prepared task; failed or unmatched native attempts cannot be reused.
    let retained_before = if let Some(input) = std::env::var_os("XBX_NPX_RETAINED_BEFORE") {
        let input = std::path::PathBuf::from(input).canonicalize()?;
        if !input.starts_with(dir.canonicalize()?)
            || input.file_name().is_none_or(|n| n != "before.result.json")
        {
            return Err("retained original result is outside this qualification".into());
        }
        let mut value: Value = c::read_json(&input)?;
        if value["case"] != "before"
            || value["native_clean"] != true
            || value["supervisor_exit"] != 0
            || value["termination_record_present"] != true
            || value["firmware_sha256"] != resources::LOGGING_OPEN
            || value["same_original_fixture"] != identity["fixture_iso"]
            || value["trace_imports"] != false
        {
            return Err("retained original control bindings/return differ".into());
        }
        let profile =
            std::path::PathBuf::from(value["profile"].as_str().ok_or("retained profile")?)
                .canonicalize()?;
        if profile.parent() != Some(root.canonicalize()?.as_path())
            || profile
                .file_name()
                .is_none_or(|n| !n.to_string_lossy().starts_with("np-"))
        {
            return Err("retained original profile is outside this task".into());
        }
        let text = fs::read_to_string(profile.join("kernel.log"))?;
        let returned = fs::read_to_string(profile.join("termination.txt"))?;
        let capture = fs::read_to_string(profile.join("serial-capture.txt"))?;
        if text.len() > 4 * 1024 * 1024
            || outcome(&text) != Some(false)
            || !returned.contains("forced=false\n")
            || !returned.contains("qmp=Ok(())\n")
            || !returned.contains("storage_synced=true\n")
            || !capture.contains("truncated=false\n")
            || !capture.contains("capture_ok=true\n")
        {
            return Err("retained original explicit failure/output/return unqualified".into());
        }
        value["prior_classification"] = value["guest_passed"].clone();
        value["guest_passed"] = json!(false);
        value["retained_control"] = json!(input);
        value["retained_log_sha256"] = json!(c::file_digest(&profile.join("kernel.log"))?);
        value["failure_class"] = json!(if text.contains("NPX RESULT=BAD00204") {
            "parent_restore_or_integrity_check"
        } else {
            "displaced_child_integrity_check"
        });
        Some(value)
    } else {
        None
    };
    for (case, flash, expected) in [("before", old, false), ("after", new, true)] {
        if case == "before"
            && let Some(value) = &retained_before
        {
            c::write_new(&attempt.join("before.result.json"), &c::json(value)?)?;
            results.push(value.clone());
            continue;
        }
        let profile = profile_path(root, &attempt_id, case == "before")?;
        if profile.exists() {
            return Err("original attempt already exists; preserve it".into());
        }
        let log = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(attempt.join(format!("{case}-supervisor.log")))?;
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
        let termination = match fs::read_to_string(profile.join("termination.txt")) {
            Ok(value) => Some(value),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        let clean = status.success()
            && termination.as_ref().is_some_and(|t| {
                t.contains("forced=false\n")
                    && t.contains("qmp=Ok(())\n")
                    && t.contains("storage_synced=true\n")
            });
        let passed = outcome(&text);
        let result = json!({"case":case,"guest_passed":passed,"expected":expected,"native_clean":clean,
            "elapsed_ms":start.elapsed().as_millis(),"profile":profile,"supervisor_exit":status.code(),
            "termination_record_present":termination.is_some(),"firmware_sha256":c::file_digest(&profile.join("flash.bin")).ok(),
            "same_original_fixture":identity["fixture_iso"],"trace_imports":false,"Fable_cause_unattributed":true});
        c::write_new(
            &attempt.join(format!("{case}.result.json")),
            &c::json(&result)?,
        )?;
        println!("{result}");
        if !clean || passed != Some(expected) {
            return Err(
                "original current-release regression/return did not match; preserve first failure"
                    .into(),
            );
        }
        results.push(result);
    }
    c::write_new(
        &dir.join("comparison.json"),
        &c::json(&json!({
            "attempt":attempt,"results":results,"prior_failed_attempts_preserved":true,
            "same_firmware_and_fixture_as_built":true
        }))?,
    )?;
    c::write_new(&dir.join("regression.qualified"),b"current release red; same original regression green after accepted functional port; clean native returns; no Fable attribution\n")?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::{outcome, profile_path};
    #[test]
    fn original_observations_fit_native_socket_limit_and_keep_cases_separate() {
        let root = std::path::Path::new("/mnt/test-workspace/projects/xbx/cartridge-v1");
        let id = "0123456789abcdef0123456789abcdef";
        let before = profile_path(root, id, true).unwrap();
        let after = profile_path(root, id, false).unwrap();
        assert_ne!(before, after);
        assert!(before.join("qmp.sock").as_os_str().len() <= 100);
        assert!(
            profile_path(
                &std::path::PathBuf::from(format!("/{}", "x".repeat(100))),
                id,
                true
            )
            .is_err()
        );
    }
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
            outcome("NPX OWNERSHIP BOOT\nNPX RESULT=BAD00204\nNPX RESULT=BAD00301"),
            Some(false)
        );
        assert_eq!(outcome("NPX OWNERSHIP BOOT\nNPX RESULT=BAD00204"), None);
        assert_eq!(
            outcome(
                "NPX OWNERSHIP BOOT\nNPX RESULT=BAD00302\nNPX RESULT=BAD00205\nNPX RESULT=600DCAFE\nNPX RESULT=4E505801"
            ),
            None
        );
        assert_eq!(
            outcome(
                "NPX OWNERSHIP BOOT\nNPX RESULT=BAD00302\nNPX RESULT=600DCAFE\nNPX RESULT=4E505801"
            ),
            None
        );
    }
}
