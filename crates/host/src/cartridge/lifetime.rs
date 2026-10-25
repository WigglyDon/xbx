// SPDX-License-Identifier: MIT
//! Master6399143636's two original startup-lifetime cases, after continuity.
//! Private firmware is a separate developer execution control, never selectable
//! by the ordinary packaged player or used as implementation source.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
fn put(b: &mut [u8], at: usize, value: u32) {
    b[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("startup-lifetime-v1");
    c::mkdir(&dir)?;
    let template = root.join("fixtures-v2/a.iso");
    if c::file_digest(&template)? != resources::ORIGINAL_FIXTURES[0] {
        return Err("original generated XBE/XISO template differs".into());
    }
    let original = fs::read(template)?;
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let mut cases = Vec::new();
    for (name, exit) in [("control", 0), ("exit", 1)] {
        let object = dir.join(format!("{name}.o"));
        let bin = dir.join(format!("{name}.bin"));
        c::store::checked(
            Command::new(llvm.join("clang"))
                .args(["--target=i386-none-elf", "-march=pentium3", "-c"])
                .arg(format!("-DLIFETIME_EXIT={exit}"))
                .arg(repo.join("native/fixtures/startup-lifetime.S"))
                .arg("-o")
                .arg(&object),
        )?;
        c::store::checked(
            Command::new(llvm.join("ld.lld"))
                .args([
                    "-m",
                    "elf_i386",
                    "-Ttext=0x11000",
                    "--oformat=binary",
                    "--entry=_start",
                ])
                .arg(&object)
                .arg("-o")
                .arg(&bin),
        )?;
        let symbols = Command::new(llvm.join("llvm-nm"))
            .args(["-n", "--defined-only"])
            .arg(&object)
            .output()?;
        if !symbols.status.success() {
            return Err("original symbol table unavailable".into());
        }
        let symbols = String::from_utf8(symbols.stdout)?;
        let thunk = symbols
            .lines()
            .find(|s| s.ends_with(" T thunks"))
            .and_then(|s| s.split_whitespace().next())
            .ok_or("original thunk symbol")?;
        let thunk = 0x11000 + u32::from_str_radix(thunk, 16)?;
        let code = fs::read(bin)?;
        if code.is_empty() || code.len() > 4096 {
            return Err("original fixture extent".into());
        }
        let mut xbe = original[34 * 2048..34 * 2048 + 4096].to_vec();
        put(&mut xbe, 0x158, thunk ^ 0x5b6d40b6);
        put(&mut xbe, 0x180, 0x58424c00 + exit);
        xbe[0x184..0x1d4].fill(0);
        for (i, ch) in format!("xbx lifetime {name}").encode_utf16().enumerate() {
            xbe[0x184 + i * 2..0x186 + i * 2].copy_from_slice(&ch.to_le_bytes());
        }
        put(&mut xbe, 0x348 + 16, code.len() as u32);
        xbe[0x348 + 36..0x348 + 56].copy_from_slice(&super::build::section_digest(&code)?);
        xbe.extend_from_slice(&code);
        c::write_new(&dir.join(format!("{name}.xbe")), &xbe)?;
        let mut iso = vec![0; 64 * 2048];
        iso[..34 * 2048].copy_from_slice(&original[..34 * 2048]);
        put(&mut iso, 33 * 2048 + 8, xbe.len() as u32);
        iso[34 * 2048..34 * 2048 + xbe.len()].copy_from_slice(&xbe);
        let image = dir.join(format!("{name}.iso"));
        c::write_new(&image, &iso)?;
        cases.push(json!({"case":name,"sha256":c::file_digest(&image)?,"thunk_virtual_address":thunk,"bytes":iso.len()}));
    }
    c::write_new(
        &dir.join("build.identity.json"),
        &c::json(&json!({
            "source_sha256":c::file_digest(&repo.join("native/fixtures/startup-lifetime.S"))?,
            "public_ABI":"XboxDev/nxdk a6c31ee90540710a3df26a0712b61492f2ea03dd",
            "entry_and_callback_preserve_callee_saved_registers":true,
            "context":"persistent_original_image_data_not_parent_stack",
            "after_parent_proof":"independently_opened_parent_thread_handle_and_finite_NtWaitForSingleObject",
            "output":"ordinary_guest_COM1_with_explicit_public_UART_activation",
            "cases":cases
        }))?,
    )?;
    Ok(())
}

pub(super) fn observe(repo: &Path, root: &Path) -> Result<(), Error> {
    let project = root.parent().ok_or("project")?;
    let r = resources::Resources::open(&root.join("package-v3"))?;
    let fixtures = root.join("startup-lifetime-v1");
    let identity: Value = c::read_json(&fixtures.join("build.identity.json"))?;
    if identity["source_sha256"]
        != c::file_digest(&repo.join("native/fixtures/startup-lifetime.S"))?
    {
        return Err("built original source differs".into());
    }
    let private = project.join("reference-input-v1");
    let selection: Value = c::read_json(&private.join("selection.json"))?;
    for name in ["flash.bin", "mcpx.bin"] {
        let path = private.join(name);
        let selected = selection["members"]
            .as_array()
            .ok_or("private provenance")?
            .iter()
            .find(|v| v["output"].as_str() == path.to_str())
            .ok_or("retained private input selection")?;
        if selected["sha256"] != c::file_digest(&path)?
            || selected["bytes"] != fs::metadata(&path)?.len()
        {
            return Err("retained private control input differs".into());
        }
    }
    let open = project.join("open-startup-v1/build-basic-v3/bios.bin");
    if c::file_digest(&open)? != resources::LOGGING_OPEN {
        return Err("corrected open input differs".into());
    }
    let mut results = Vec::new();
    for (firmware, prefix) in [("reference", "r"), ("open", "o")] {
        for (case, suffix) in [("control", "c"), ("exit", "e")] {
            let media = fixtures.join(format!("{case}.iso"));
            let built = identity["cases"]
                .as_array()
                .ok_or("cases")?
                .iter()
                .find(|v| v["case"] == case)
                .ok_or("case")?;
            if built["sha256"] != c::file_digest(&media)? {
                return Err("original fixture changed".into());
            }
            let profile = root.join(format!("l-{prefix}{suffix}-v1"));
            if profile.exists() {
                return Err("original case already attempted; preserve and reconcile it".into());
            }
            let flash = if firmware == "reference" {
                private.join("flash.bin")
            } else {
                open.clone()
            };
            let supervisor = std::env::current_exe()?.with_file_name("xbx-host");
            let log = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(fixtures.join(format!("{prefix}{suffix}-supervisor.log")))?;
            let mut command = Command::new(supervisor);
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
            if firmware == "reference" {
                command.arg(private.join("mcpx.bin"));
            }
            let mut owned = crate::probe::OwnedChild(command.spawn()?);
            let start = Instant::now();
            let mut text = String::new();
            let expected = if case == "control" {
                "LIFETIME CONTROL COMPLETE"
            } else {
                "LIFETIME EXIT AFTER_PARENT_TERMINATION COMPLETE"
            };
            let mut stop_requested = false;
            let status = loop {
                if let Ok(s) = fs::read_to_string(profile.join("kernel.log")) {
                    if s.len() > 4 * 1024 * 1024 {
                        return Err("original output bound".into());
                    }
                    text = s;
                }
                if !stop_requested && (text.contains(expected) || text.contains("LIFETIME FAIL")) {
                    c::write_new(
                        &profile.join("observation.complete"),
                        b"original guest output observed; native return required\n",
                    )?;
                    stop_requested = true;
                }
                if let Some(s) = owned.0.try_wait()? {
                    break s;
                }
                if start.elapsed() > Duration::from_secs(110) {
                    return Err("original finite supervisor bound; preserve failed return".into());
                }
                std::thread::sleep(Duration::from_millis(100));
            };
            let termination = fs::read_to_string(profile.join("termination.txt"))?;
            let clean = status.success()
                && termination.contains("forced=false\n")
                && termination.contains("qmp=Ok(())\n")
                && termination.contains("storage_synced=true\n");
            let boot = text.contains(&format!("LIFETIME {} BOOT", case.to_uppercase()));
            let complete = text.contains(expected)
                && !text.contains("LIFETIME FAIL")
                && (case != "control" || text.contains("LIFETIME CONTROL CHILD PASS"));
            let result = json!({"firmware":firmware,"case":case,"ordinary_output_verified":boot,"guest_completed":complete,"native_return_clean":clean,"supervisor_exit":status.code(),"elapsed_ms":start.elapsed().as_millis(),"profile":profile,"fixture_sha256":c::file_digest(&media)?});
            c::write_new(
                &fixtures.join(format!("{prefix}{suffix}.result.json")),
                &c::json(&result)?,
            )?;
            println!("{result}");
            results.push(result);
            if !boot || !clean {
                return Err("original output channel or native return unqualified; do not attribute a platform failure".into());
            }
        }
    }
    c::write_new(
        &fixtures.join("comparison.json"),
        &c::json(
            &json!({"results":results,"no_title_memory_or_instruction_inspection":true,"no_paused_diagnostic_chain":true}),
        )?,
    )?;
    Ok(())
}
