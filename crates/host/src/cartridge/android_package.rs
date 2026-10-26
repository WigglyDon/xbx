// SPDX-License-Identifier: MIT
//! Private off-device candidate. Preserve every stage and the existing signing
//! identity; no public binary release, device operation or legacy selector edit.
use super::{
    self as c,
    android_build::{literal, output},
};
use serde_json::json;
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};
type Error = Box<dyn std::error::Error>;
fn java_files(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            java_files(&path, out)?;
        } else if path.extension().is_some_and(|s| s == "java") {
            out.push(path);
        }
    }
    Ok(())
}
pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    build_candidate(repo, root, "android-native-bridge-v1", "android-package-v2")
}
pub(super) fn build_socket(repo: &Path, root: &Path) -> Result<(), Error> {
    build_candidate(repo, root, "android-native-socket-v1", "android-package-v3")
}
pub(super) fn build_monitor(repo: &Path, root: &Path) -> Result<(), Error> {
    build_candidate(
        repo,
        root,
        "android-native-monitor-v1",
        "android-package-v4",
    )
}
pub(super) fn build_session(repo: &Path, root: &Path) -> Result<(), Error> {
    build_candidate(
        repo,
        root,
        "android-native-session-v2",
        "android-package-v5",
    )
}
fn build_candidate(
    repo: &Path,
    root: &Path,
    native_stage: &str,
    package_stage: &str,
) -> Result<(), Error> {
    let native = root.join(native_stage);
    let helper = root.join("android-helper-v4");
    let old = repo.join(".local/build-android");
    let core_hash = c::file_digest(&native.join("libxemu.so"))?;
    let helper_hash = c::file_digest(&helper.join("libxbx-disk.so"))?;
    let seed_hash = c::file_digest(&helper.join("empty.qcow2"))?;
    let native_identity: serde_json::Value = c::read_json(&native.join("build.identity.json"))?;
    if native_identity["source_sha256"] != c::file_digest(&repo.join("native/xbx_xemu_bridge.c"))?
        || native_identity["bridge_abi"] != 4
    {
        return Err("mechanical bridge source/ABI differs".into());
    }
    if native_stage == "android-native-session-v2"
        && native_identity["session_monitor_patch_sha256"]
            != c::file_digest(&repo.join("patches/xemu-android-session-monitor.patch"))?
    {
        return Err("session monitor patch identity differs".into());
    }
    let out = root.join(package_stage);
    c::mkdir(&out)?;
    let source = std::env::current_exe()?
        .parent()
        .ok_or("builder")?
        .parent()
        .ok_or("tools")?
        .join("source");
    let ndk = PathBuf::from(std::env::var("XBX_NDK")?);
    let sdk = ndk
        .parent()
        .ok_or("NDK version")?
        .parent()
        .ok_or("SDK root")?;
    let tools = sdk.join("build-tools/37.0.0");
    let jar = sdk.join("platforms/android-37.0/android.jar");
    c::store::checked(Command::new("cargo").current_dir(&source).args(["build","--locked","--offline","-j","1","--release","--target","aarch64-linux-android","-p","xbx-host","--lib"])
        .env("CARGO_TARGET_DIR",root.join("android-app-target")).env("XBX_NATIVE_LIB_DIR",&native)
        .env("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER",ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android29-clang"))
        .env("XBX_ANDROID_CORE_SHA256",&core_hash).env("XBX_ANDROID_DISK_SHA256",&helper_hash).env("XBX_ANDROID_SEED_SHA256",&seed_hash)
        .env("RUSTFLAGS",format!("-L native={} -L native={} -C link-arg=-Wl,-rpath,$ORIGIN -C link-arg=-Wl,-rpath-link,{}",literal(&native)?,literal(&old.join("_deps/sdl3-build"))?,literal(&old.join("_deps/sdl3-build"))?)))?;
    let assets = out.join("assets");
    let libs = out.join("lib/arm64-v8a");
    let classes = out.join("classes");
    let dex = out.join("dex");
    for dir in [&assets, &libs, &classes, &dex] {
        fs::create_dir_all(dir)?;
    }
    let open = root
        .parent()
        .ok_or("project")?
        .join("open-startup-v1/build-release-v3/bios.bin");
    if c::file_digest(&open)? != super::resources::OPEN {
        return Err("supported integrated open image differs".into());
    }
    fs::copy(open, assets.join("open.bin"))?;
    fs::copy(helper.join("empty.qcow2"), assets.join("empty.qcow2"))?;
    fs::copy(repo.join("THIRD_PARTY.md"), assets.join("THIRD_PARTY.md"))?;
    fs::create_dir_all(assets.join("licenses"))?;
    for entry in fs::read_dir(repo.join("licenses"))? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            fs::copy(
                entry.path(),
                assets.join("licenses").join(entry.file_name()),
            )?;
        }
    }
    let copying = output(
        Command::new("git")
            .arg("-C")
            .arg(repo.join(".local/src/xemu-android"))
            .args(["show", "4967efd3ee594cc236869a45ee4cb396e25af1ac:COPYING"])
            .env("GIT_NO_LAZY_FETCH", "1"),
    )?;
    c::write_new(
        &assets.join("licenses/XEMU-COPYING.txt"),
        copying.as_bytes(),
    )?;
    fs::copy(
        root.parent()
            .ok_or("project")?
            .join("open-startup-v1/source-v3/LICENSE"),
        assets.join("licenses/OXOS-GPL-2.0.txt"),
    )?;
    fs::copy(
        old.join("_deps/sdl3-src/LICENSE.txt"),
        assets.join("licenses/SDL-LICENSE.txt"),
    )?;
    c::write_new(
        &assets.join("runtime.json"),
        &c::json(
            &json!({"schema":1,"core":core_hash,"open_system":super::resources::OPEN,"image_helper":helper_hash,"empty_disk":seed_hash,"state_abi":1,"bridge_abi":4}),
        )?,
    )?;
    for (from, name) in [
        (native.join("libxemu.so"), "libxemu.so"),
        (helper.join("libxbx-disk.so"), "libxbx-disk.so"),
        (old.join("_deps/sdl3-build/libSDL3.so"), "libSDL3.so"),
        (
            root.join("android-app-target/aarch64-linux-android/release/libxbx.so"),
            "libxbx.so",
        ),
    ] {
        fs::copy(from, libs.join(name))?;
    }
    let mut java = Vec::new();
    java_files(
        &old.join("_deps/sdl3-src/android-project/app/src/main/java"),
        &mut java,
    )?;
    java_files(&repo.join("android/java"), &mut java)?;
    java.sort();
    let mut java_hashes = serde_json::Map::new();
    for path in &java {
        java_hashes.insert(
            path.to_string_lossy().into_owned(),
            json!(c::file_digest(path)?),
        );
    }
    c::store::checked(
        Command::new("java")
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
            .args(&java),
    )?;
    let class_jar = out.join("classes.jar");
    c::store::checked(
        Command::new("java")
            .args(["-m", "jdk.jartool/sun.tools.jar.Main", "--create", "--file"])
            .arg(&class_jar)
            .arg("-C")
            .arg(&classes)
            .arg("."),
    )?;
    c::store::checked(
        Command::new(tools.join("d8"))
            .args(["--min-api", "29", "--lib"])
            .arg(&jar)
            .arg("--output")
            .arg(&dex)
            .arg(class_jar),
    )?;
    let unsigned = out.join("unsigned.apk");
    c::store::checked(
        Command::new(tools.join("aapt2"))
            .args(["link", "-o"])
            .arg(&unsigned)
            .arg("--manifest")
            .arg(repo.join("android/AndroidManifest.xml"))
            .arg("-I")
            .arg(&jar)
            .arg("-A")
            .arg(&assets)
            .args([
                "--version-code",
                "100",
                "--version-name",
                "0.5.0-cartridge-private",
            ]),
    )?;
    fs::copy(dex.join("classes.dex"), out.join("classes.dex"))?;
    c::store::checked(
        Command::new("zip")
            .current_dir(&out)
            .arg("-q")
            .arg(&unsigned)
            .args([
                "classes.dex",
                "lib/arm64-v8a/libxemu.so",
                "lib/arm64-v8a/libSDL3.so",
                "lib/arm64-v8a/libxbx.so",
                "lib/arm64-v8a/libxbx-disk.so",
            ]),
    )?;
    let aligned = out.join("aligned.apk");
    c::store::checked(
        Command::new(tools.join("zipalign"))
            .args(["-P", "16", "-f", "4"])
            .arg(&unsigned)
            .arg(&aligned),
    )?;
    let signing = repo.join(".local/signing");
    let key = signing.join("qualification.p12");
    let password = signing.join("password");
    if !key.is_file() || !password.is_file() {
        return Err("existing XBX signing identity is unavailable; preserve candidate and do not replace it".into());
    }
    let apk = out.join("xbx-cartridge-qualification.apk");
    c::store::checked(
        Command::new(tools.join("apksigner"))
            .args(["sign", "--ks"])
            .arg(key)
            .arg("--ks-pass")
            .arg(format!("file:{}", password.display()))
            .arg("--out")
            .arg(&apk)
            .arg(&aligned),
    )?;
    let signature = output(
        Command::new(tools.join("apksigner"))
            .args(["verify", "--verbose", "--print-certs"])
            .arg(&apk),
    )?;
    c::write_new(&out.join("signature.txt"), signature.as_bytes())?;
    let inventory = output(
        Command::new(tools.join("aapt2"))
            .args(["dump", "badging"])
            .arg(&apk),
    )?;
    if !inventory.contains("package: name='com.machinerhythm.xbx'")
        || !inventory.contains("native-code: 'arm64-v8a'")
    {
        return Err("private candidate identity/ABI differs".into());
    }
    c::write_new(&out.join("package.txt"), inventory.as_bytes())?;
    c::write_new(
        &out.join("build.identity.json"),
        &c::json(
            &json!({"apk_sha256":c::file_digest(&apk)?,"rust_library_sha256":c::file_digest(&libs.join("libxbx.so"))?,"core_sha256":core_hash,"helper_sha256":helper_hash,"generated_seed_sha256":seed_hash,"open_system_sha256":super::resources::OPEN,"manifest_sha256":c::file_digest(&repo.join("android/AndroidManifest.xml"))?,"java_source_sha256":java_hashes,"rust_source":source,"signing":"existing XBX identity reused; no private key copied","version_code":100,"whole_core_rebuild":false,"device_operation":false,"lifecycle_qualified":false,"Fable_playable":false,"all_stages_retained":true}),
        )?,
    )?;
    Ok(())
}

pub(super) fn sign_retained(repo: &Path, root: &Path) -> Result<(), Error> {
    let stage = root.join("android-package-v2");
    let aligned = stage.join("aligned.apk");
    let out = root.join("android-signed-v1");
    c::mkdir(&out)?;
    let ndk = PathBuf::from(std::env::var("XBX_NDK")?);
    let sdk = ndk.parent().ok_or("NDK")?.parent().ok_or("SDK")?;
    let tools = sdk.join("build-tools/37.0.0");
    let signing = repo.join(".local/signing");
    let apk = out.join("xbx-cartridge-qualification.apk");
    c::store::checked(
        Command::new(tools.join("apksigner"))
            .args(["sign", "--ks"])
            .arg(signing.join("qualification.p12"))
            .arg("--ks-pass")
            .arg(format!("file:{}", signing.join("password").display()))
            .arg("--out")
            .arg(&apk)
            .arg(&aligned),
    )?;
    let signature = output(
        Command::new(tools.join("apksigner"))
            .args(["verify", "--verbose", "--print-certs"])
            .arg(&apk),
    )?;
    let inventory = output(
        Command::new(tools.join("aapt2"))
            .args(["dump", "badging"])
            .arg(&apk),
    )?;
    if !inventory.contains("package: name='com.machinerhythm.xbx'")
        || !inventory.contains("native-code: 'arm64-v8a'")
    {
        return Err("signed candidate package/ABI differs".into());
    }
    c::write_new(&out.join("signature.txt"), signature.as_bytes())?;
    c::write_new(&out.join("package.txt"), inventory.as_bytes())?;
    c::write_new(
        &out.join("build.identity.json"),
        &c::json(
            &json!({"apk_sha256":c::file_digest(&apk)?,"aligned_apk_sha256":c::file_digest(&aligned)?,"preserved_complete_stage":stage,"rust_source":"tools-v27/source","native_core_sha256":c::file_digest(&stage.join("lib/arm64-v8a/libxemu.so"))?,"rust_library_sha256":c::file_digest(&stage.join("lib/arm64-v8a/libxbx.so"))?,"image_helper_sha256":c::file_digest(&stage.join("lib/arm64-v8a/libxbx-disk.so"))?,"signing":"existing identity; original single password-file invocation","device_operation":false,"lifecycle_qualified":false,"Fable_playable":false}),
        )?,
    )?;
    Ok(())
}
