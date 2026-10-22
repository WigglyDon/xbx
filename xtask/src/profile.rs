use crate::run;
use std::{fs, path::Path, process::Command};

pub fn prepare(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let _lock = crate::capacity::writer(root, 1)?;
    let flash = root.join(".local/build-oxos-play/bios.bin");
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
                "xbx-qualification-profile-v1\nfirmware=oxos:c0c887a1263ebf89ba765f34a88ca8f132901e25\nsha256={sha}\nselftest=0\nserial=1\nram_mib=64\ndisk=retail-fatx-8g-v2\n"
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
