use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use crate::run;

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
    let stage = root.join(".local/apk");
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
    run(Command::new(tools.join("aapt2"))
        .args(["link", "-o"])
        .arg(&unsigned)
        .arg("--manifest")
        .arg(root.join("android/AndroidManifest.xml"))
        .arg("-I")
        .arg(&jar)
        .args(["--version-code", "1", "--version-name", "0.1.0"]))?;
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
    println!(
        "Local qualification artifact: {} (no installation or public binary release)",
        signed.display()
    );
    Ok(())
}
