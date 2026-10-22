use crate::run;
use std::{fs, path::Path, process::Command};

pub fn prepare(root: &Path, trace: bool) -> Result<(), Box<dyn std::error::Error>> {
    let _lock = crate::capacity::writer(root, 1)?;
    let trace_build = if trace {
        Some(std::path::PathBuf::from(fs::read_to_string(
            root.join(".local/selected-launch-build"),
        )?))
    } else {
        None
    };
    let flash = if let Some(ref build) = trace_build {
        build.join("bios.bin")
    } else {
        root.join(".local/build-oxos-play/bios.bin")
    };
    let output = Command::new("sha256sum").arg(&flash).output()?;
    if !output.status.success() {
        return Err("build the separate SELFTEST=0 image first".into());
    }
    let output = String::from_utf8(output.stdout)?;
    let sha = output
        .split_whitespace()
        .next()
        .ok_or("missing flash digest")?;
    if sha.len() != 64 || !sha.bytes().all(|v| v.is_ascii_hexdigit()) {
        return Err("invalid flash digest".into());
    }
    let trace_fields = if let Some(ref build) = trace_build {
        let receipt = fs::read_to_string(build.join("build.identity"))?;
        let output = Command::new("sha256sum")
            .arg(root.join("patches/oxos-launch-trace.patch"))
            .output()?;
        if !output.status.success() {
            return Err("patch digest failed".into());
        }
        let patch = String::from_utf8(output.stdout)?
            .split_whitespace()
            .next()
            .ok_or("missing trace digest")?
            .to_owned();
        if !receipt
            .lines()
            .any(|l| l == format!("firmware_sha256={sha}"))
            || !receipt
                .lines()
                .any(|l| l == format!("patch_sha256={patch}"))
        {
            return Err("trace image/build identity mismatch".into());
        }
        format!(
            "mode=launch-diagnostic-v2\npatch_sha256={patch}\ntrace_imports=0\nparent_firmware_sha256=668d08af6de204ce07a1479c8854d8f688af312ed6104e8ab56391dffc301fa6\n"
        )
    } else {
        String::new()
    };
    let build = root.join(format!(".local/profile-build-{sha}-fatx-v2"));
    let assets = build.join("assets");
    if !assets.exists() {
        fs::create_dir_all(&assets)?;
        fs::copy(&flash, assets.join("flash.bin"))?;
        run(Command::new("cargo")
            .current_dir(root)
            .args(["run", "-p", "xbx-host", "--offline", "--", "empty-disk"])
            .arg(build.join("empty.raw")))?;
        run(Command::new("qemu-img")
            .args(["convert", "-f", "raw", "-O", "qcow2", "-S", "4096"])
            .arg(build.join("empty.raw"))
            .arg(assets.join("hdd.qcow2")))?;
        run(Command::new("qemu-img")
            .arg("check")
            .arg(assets.join("hdd.qcow2")))?;
        fs::write(
            assets.join("profile.identity"),
            format!(
                "xbx-qualification-profile-v1\nfirmware=oxos:c0c887a1263ebf89ba765f34a88ca8f132901e25\nsha256={sha}\nselftest=0\nserial=1\nram_mib=64\ndisk=retail-fatx-8g-v2\n{trace_fields}"
            ),
        )?;
    }
    if !assets.join("profile.identity").is_file() {
        return Err("incomplete profile seed preserved; inspect before reuse".into());
    }
    fs::write(
        root.join(".local/selected-profile-assets"),
        assets.to_str().ok_or("invalid assets path")?,
    )?;
    println!(
        "Selected private normal-play seed at {}; no EEPROM/game/owner save bundled",
        assets.display()
    );
    Ok(())
}
