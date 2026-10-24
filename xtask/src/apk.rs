use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use crate::run;

fn retire_stage(root: &Path, stage: &Path, signed: &Path) -> std::io::Result<()> {
    let local = root.join(".local");
    let name = stage.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let id = name.strip_prefix("apk-player-").unwrap_or("");
    let refused =
        || std::io::Error::other("APK staging ownership/output not verified; preserve it");
    if stage.parent() != Some(local.as_path())
        || id.is_empty()
        || !id.bytes().all(|b| b.is_ascii_digit())
        || signed != local.join("xbx-qualification.apk")
        || !fs::symlink_metadata(stage)?.file_type().is_dir()
        || !fs::symlink_metadata(signed)?.file_type().is_file()
        || fs::metadata(signed)?.len() == 0
        || fs::read_to_string(stage.join(".xbx-stage-owner"))? != id
    {
        return Err(refused());
    }
    // Called only after signing, signature verification and package inspection.
    // Failed builds never reach this point and retain their complete staging.
    fs::remove_dir_all(stage)
}

fn java_sources(directory: &Path, output: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            java_sources(&path, output)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("java") {
            output.push(path);
        }
    }
    Ok(())
}

pub fn build(root: &Path, sdk: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let tools = sdk.join("build-tools/37.0.0");
    let jar = sdk.join("platforms/android-37.0/android.jar");
    let build_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let stage = root.join(format!(".local/apk-player-{build_id}"));
    // Exclusive creation prevents reusing another build's failed/retained stage.
    fs::create_dir(&stage)?;
    fs::write(stage.join(".xbx-stage-owner"), build_id.to_string())?;
    let classes = stage.join("classes");
    let dex = stage.join("dex");
    let libs = stage.join("lib/arm64-v8a");
    for directory in [&classes, &dex, &libs] {
        fs::create_dir_all(directory)?;
    }
    let mut sources = Vec::new();
    java_sources(
        &root.join(".local/build-android/_deps/sdl3-src/android-project/app/src/main/java"),
        &mut sources,
    )?;
    java_sources(&root.join("android/java"), &mut sources)?;
    sources.sort();
    run(Command::new("java")
        .args([
            "-m",
            "jdk.compiler/com.sun.tools.javac.Main",
            "-source",
            "8",
            "-target",
            "8",
            "-bootclasspath",
        ])
        .arg(&jar)
        .arg("-d")
        .arg(&classes)
        .args(sources))?;
    let class_jar = stage.join("classes.jar");
    run(Command::new("java")
        .args(["-m", "jdk.jartool/sun.tools.jar.Main", "--create", "--file"])
        .arg(&class_jar)
        .arg("-C")
        .arg(&classes)
        .arg("."))?;
    run(Command::new(tools.join("d8"))
        .args(["--min-api", "29", "--lib"])
        .arg(&jar)
        .arg("--output")
        .arg(&dex)
        .arg(class_jar))?;
    for (from, name) in [
        (".local/build-android/libxemu.so", "libxemu.so"),
        (
            ".local/build-android/_deps/sdl3-build/libSDL3.so",
            "libSDL3.so",
        ),
        (
            "target/aarch64-linux-android/release/libxbx.so",
            "libxbx.so",
        ),
    ] {
        fs::copy(root.join(from), libs.join(name))?;
    }
    let unsigned = stage.join("unsigned.apk");
    let assets = fs::read_to_string(root.join(".local/selected-profile-assets"))?;
    let assets = Path::new(assets.trim());
    let mut entries: Vec<_> = fs::read_dir(assets)?
        .map(|v| v.map(|e| e.file_name()))
        .collect::<Result<_, _>>()?;
    entries.sort();
    if entries != ["flash.bin", "hdd.qcow2", "profile.identity"].map(std::ffi::OsString::from) {
        return Err("profile asset inventory differs from the three reviewed files".into());
    }
    run(Command::new(tools.join("aapt2"))
        .args(["link", "-o"])
        .arg(&unsigned)
        .arg("--manifest")
        .arg(root.join("android/AndroidManifest.xml"))
        .arg("-I")
        .arg(&jar)
        .arg("-A")
        .arg(assets)
        .args([
            "--version-code",
            "9",
            "--version-name",
            "0.4.2-nullcall-caller",
        ]))?;
    fs::copy(dex.join("classes.dex"), stage.join("classes.dex"))?;
    run(Command::new("zip")
        .current_dir(&stage)
        .arg("-q")
        .arg(&unsigned)
        .args([
            "classes.dex",
            "lib/arm64-v8a/libxemu.so",
            "lib/arm64-v8a/libSDL3.so",
            "lib/arm64-v8a/libxbx.so",
        ]))?;
    let aligned = stage.join("aligned.apk");
    run(Command::new(tools.join("zipalign"))
        .args(["-P", "16", "-f", "4"])
        .arg(unsigned)
        .arg(&aligned))?;

    let signing = root.join(".local/signing");
    fs::create_dir_all(&signing)?;
    fs::set_permissions(&signing, fs::Permissions::from_mode(0o700))?;
    let password = signing.join("password");
    let key = signing.join("qualification.p12");
    if !key.exists() {
        // Never replace a partial key/password pair; retain failed evidence for recovery.
        let mut random = [0; 32];
        fs::File::open("/dev/urandom")?.read_exact(&mut random)?;
        let text: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&password)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        run(Command::new("java")
            .args([
                "-m",
                "java.base/sun.security.tools.keytool.Main",
                "-genkeypair",
                "-storetype",
                "PKCS12",
                "-alias",
                "xbx-qualification",
                "-dname",
                "CN=xbx qualification",
                "-keyalg",
                "RSA",
                "-keysize",
                "2048",
                "-validity",
                "3650",
                "-keystore",
            ])
            .arg(&key)
            .arg("-storepass:file")
            .arg(&password)
            .arg("-keypass:file")
            .arg(&password))?;
        fs::set_permissions(&key, fs::Permissions::from_mode(0o600))?;
    }
    let signed = root.join(".local/xbx-qualification.apk");
    run(Command::new(tools.join("apksigner"))
        .arg("sign")
        .arg("--ks")
        .arg(key)
        .arg("--ks-pass")
        .arg(format!("file:{}", password.display()))
        .arg("--out")
        .arg(&signed)
        .arg(aligned))?;
    run(Command::new(tools.join("apksigner"))
        .arg("verify")
        .arg(&signed))?;
    run(Command::new(tools.join("aapt"))
        .args(["dump", "badging"])
        .arg(&signed))?;
    retire_stage(root, &stage, &signed)?;
    println!(
        "Local qualification artifact: {} (no installation or public binary release)",
        signed.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture() -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "xbx-apk-retention-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let stage = root.join(".local/apk-player-123");
        let signed = root.join(".local/xbx-qualification.apk");
        fs::create_dir_all(&stage).unwrap();
        fs::write(stage.join(".xbx-stage-owner"), "123").unwrap();
        fs::write(stage.join("intermediate"), b"synthetic rebuildable bytes").unwrap();
        (root, stage, signed)
    }

    #[test]
    fn completed_owned_stage_retires_without_touching_output_or_other_data() {
        let (root, stage, signed) = fixture();
        fs::write(&signed, b"synthetic verified-output boundary").unwrap();
        let failed = root.join(".local/apk-player-456");
        fs::create_dir(&failed).unwrap();
        fs::write(failed.join("evidence"), b"preserve failure").unwrap();
        retire_stage(&root, &stage, &signed).unwrap();
        assert!(!stage.exists());
        assert_eq!(
            fs::read(signed).unwrap(),
            b"synthetic verified-output boundary"
        );
        assert_eq!(
            fs::read(failed.join("evidence")).unwrap(),
            b"preserve failure"
        );
    }

    #[test]
    fn missing_output_or_stage_owner_preserves_all_intermediates() {
        let (root, stage, signed) = fixture();
        assert!(retire_stage(&root, &stage, &signed).is_err());
        fs::write(&signed, []).unwrap();
        assert!(retire_stage(&root, &stage, &signed).is_err());
        fs::write(&signed, b"synthetic output").unwrap();
        fs::remove_file(stage.join(".xbx-stage-owner")).unwrap();
        assert!(retire_stage(&root, &stage, &signed).is_err());
        assert_eq!(
            fs::read(stage.join("intermediate")).unwrap(),
            b"synthetic rebuildable bytes"
        );
    }

    #[test]
    fn symlink_stage_or_output_never_authorizes_retirement() {
        use std::os::unix::fs::symlink;
        let (root, stage, signed) = fixture();
        let original = root.join("retained-original");
        fs::write(&original, b"keep original").unwrap();
        symlink(&original, &signed).unwrap();
        assert!(retire_stage(&root, &stage, &signed).is_err());
        fs::remove_file(&signed).unwrap();
        fs::write(&signed, b"synthetic output").unwrap();
        let outside = root.join("retained-stage");
        fs::rename(&stage, &outside).unwrap();
        symlink(&outside, &stage).unwrap();
        assert!(retire_stage(&root, &stage, &signed).is_err());
        assert_eq!(fs::read(original).unwrap(), b"keep original");
        assert!(outside.join("intermediate").exists());
    }
}
