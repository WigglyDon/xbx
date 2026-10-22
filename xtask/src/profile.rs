use crate::run;
use std::{fs, path::Path, process::Command};

// These are accepted historical Android seeds, not the current desktop kernel
// qualification pin. A new kernel requires its own explicit profile decision.
const ANDROID_OXOS: &str = "c0c887a1263ebf89ba765f34a88ca8f132901e25";
const BASELINE: &str = "668d08af6de204ce07a1479c8854d8f688af312ed6104e8ab56391dffc301fa6";
const V4: &str = "0cdedf6d81dabdbeafc1cff78039cf78f19cb7923078f201724e346ca94ee8e8";
const V4_PATCH: &str = "7eb48fb10ca5784a887d93db23f02c07ec83dc7b105930e1a71204724e0a0dc5";

fn identity(sha: &str, receipt: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let trace_fields = if let Some(receipt) = receipt {
        let required = [
            format!("source={ANDROID_OXOS}"),
            format!("firmware_sha256={V4}"),
            format!("patch_sha256={V4_PATCH}"),
            "selftest=0".into(),
            "serial=1".into(),
            "trace_imports=0".into(),
            "ram_mib=64".into(),
        ];
        // Exactly one matching field; duplicate/conflicting identity fields refuse.
        if sha != V4
            || !required.iter().all(|field| {
                let key = field.split_once('=').unwrap().0;
                let values: Vec<_> = receipt
                    .lines()
                    .filter(|line| line.starts_with(&format!("{key}=")))
                    .collect();
                values == [field.as_str()]
            })
        {
            return Err("only the retained accepted Android v4 seed is allowed; desktop candidates require a separate profile decision".into());
        }
        format!(
            "mode=launch-diagnostic-v4\npatch_sha256={V4_PATCH}\ntrace_imports=0\nparent_firmware_sha256={BASELINE}\n"
        )
    } else {
        if sha != BASELINE {
            return Err("only the retained accepted Android baseline seed is allowed".into());
        }
        String::new()
    };
    Ok(format!(
        "xbx-qualification-profile-v1\nfirmware=oxos:{ANDROID_OXOS}\nsha256={sha}\nselftest=0\nserial=1\nram_mib=64\ndisk=retail-fatx-8g-v2\n{trace_fields}"
    ))
}

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
    let receipt = if let Some(ref build) = trace_build {
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
        if patch != V4_PATCH {
            return Err("Android v4 patch identity changed; preserve selected assets".into());
        }
        Some(fs::read_to_string(build.join("build.identity"))?)
    } else {
        None
    };
    let expected_identity = identity(sha, receipt.as_deref())?;
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
        fs::write(assets.join("profile.identity"), &expected_identity)?;
    }
    if fs::read_to_string(assets.join("profile.identity"))? != expected_identity {
        return Err("incomplete/mismatched profile seed preserved; inspect before reuse".into());
    }
    if fs::read(assets.join("flash.bin"))? != fs::read(&flash)? {
        return Err("existing asset firmware mismatch; preserve and inspect seed".into());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_candidate_cannot_be_labeled_android_baseline_or_v4() {
        let receipt = format!(
            "source={ANDROID_OXOS}\nfirmware_sha256={V4}\npatch_sha256={V4_PATCH}\nselftest=0\nserial=1\ntrace_imports=0\nram_mib=64\n"
        );
        assert!(identity(BASELINE, None).is_ok());
        assert!(identity(V4, Some(&receipt)).is_ok());
        assert!(identity("new-candidate-firmware", None).is_err());
        assert!(identity("new-candidate-firmware", Some(&receipt)).is_err());
        // Even spoofing the accepted digest does not permit a new source pin,
        // conflicting metadata, or the baseline image under the v4 mode.
        assert!(
            identity(
                V4,
                Some(&receipt.replace(ANDROID_OXOS, "d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2"))
            )
            .is_err()
        );
        assert!(identity(V4, Some(&format!("{receipt}source=other\n"))).is_err());
        assert!(identity(V4, Some(&receipt.replace("selftest=0", "selftest=1"))).is_err());
        assert!(identity(BASELINE, Some(&receipt)).is_err());
    }
}
