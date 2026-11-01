// SPDX-License-Identifier: MIT
//! Reconstruct only the retained Android integration and accepted repairs.
//! No dependency fetch, donor replacement, old workspace or whole-core rebuild.
use super::{
    self as c,
    android_build::{compile, literal, output},
};
use serde_json::json;
use std::{fs, path::Path, process::Command};

type Error = Box<dyn std::error::Error>;
const PIN: &str = "4967efd3ee594cc236869a45ee4cb396e25af1ac";

fn retained(base: &Path, relative: &str) -> Result<Vec<u8>, Error> {
    let result = Command::new("git")
        .arg("-C")
        .arg(base)
        .args(["show", &format!("{PIN}:{relative}")])
        .env("GIT_NO_LAZY_FETCH", "1")
        .output()?;
    if !result.status.success() || result.stdout.len() > 8 * 1024 * 1024 {
        return Err("required retained source unavailable; no automatic fetch".into());
    }
    Ok(result.stdout)
}

pub(super) fn native(repo: &Path, root: &Path) -> Result<(), Error> {
    let base = repo.join(".local/src/xemu-android");
    let old = repo.join(".local/build-android");
    if output(
        Command::new("git")
            .arg("-C")
            .arg(&base)
            .args(["rev-parse", "HEAD"]),
    )?
    .trim()
        != PIN
    {
        return Err("retained Android source pin differs".into());
    }
    let new = root.join("android-demo-native-v2");
    c::mkdir(&new)?;
    let source = new.join("source");
    c::mkdir(&source)?;
    let tree = output(
        Command::new("git")
            .arg("-C")
            .arg(&base)
            .args(["ls-tree", "-rl", PIN]),
    )?;
    let mut headers = Vec::new();
    let mut bytes = 0u64;
    for line in tree.lines() {
        let (meta, path) = line.split_once('\t').ok_or("retained tree format")?;
        if path.ends_with(".h") || path.ends_with(".inc") {
            bytes += meta
                .split_whitespace()
                .nth(3)
                .ok_or("header size")?
                .parse::<u64>()?;
            headers.push(path);
        }
    }
    if headers.len() > 5000 || bytes > 64 * 1024 * 1024 {
        return Err("bounded retained header reconstruction exceeded".into());
    }
    let archive = new.join("headers.tar");
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&archive)?;
    let status = Command::new("git")
        .arg("-C")
        .arg(&base)
        .args(["archive", PIN, "--"])
        .args(&headers)
        .env("GIT_NO_LAZY_FETCH", "1")
        .stdout(file)
        .status()?;
    if !status.success() {
        return Err("retained headers incomplete; preserve partial reconstruction".into());
    }
    c::store::checked(
        Command::new("tar")
            .args(["--no-same-owner", "-xf"])
            .arg(&archive)
            .arg("-C")
            .arg(&source),
    )?;
    for path in &headers {
        if base.join(path).is_file() {
            fs::copy(base.join(path), source.join(path))?;
        }
    }
    let adapters = [
        "xemu_android.cpp",
        "xemu_lan.cpp",
        "android_crash_handler.cpp",
        "vmstate_if_android.c",
        "xemu_fatx_import.c",
        "xemu_hdd_tools_jni.c",
    ];
    let units = [
        ("migration/migration-hmp-cmds.c", "migration-hmp-cmds.c.o"),
        ("hw/xbox/nv2a/pfifo.c", "pfifo.c.o"),
        ("hw/xbox/nv2a/nv2a.c", "nv2a.c.o"),
        ("target/i386/tcg/translate.c", "translate.c.o"),
    ];
    let monitor = "android/app/src/main/cpp/monitor_stub.c";
    let mut sources: Vec<String> = units.iter().map(|(path, _)| (*path).into()).collect();
    sources.extend(
        adapters
            .iter()
            .map(|p| format!("android/app/src/main/cpp/{p}")),
    );
    sources.push(monitor.into());
    let mut identities = serde_json::Map::new();
    for path in &sources {
        let to = source.join(path);
        fs::create_dir_all(to.parent().ok_or("source parent")?)?;
        // Retain applied mechanical Android changes where an actual source
        // survives; otherwise consume the exact locally retained public blob.
        let data = if base.join(path).is_file() {
            fs::read(base.join(path))?
        } else {
            retained(&base, path)?
        };
        c::write_new(&to, &data)?;
        identities.insert(path.clone(), json!(c::file_digest(&to)?));
    }
    c::write_new(
        &new.join("inherited-source.patch"),
        output(
            Command::new("git")
                .arg("-C")
                .arg(&base)
                .args(["diff", "--binary"]),
        )?
        .as_bytes(),
    )?;
    let patches = [
        "patches/xemu-paused-savevm.patch",
        "patches/xemu-prelaunch-savevm.patch",
        "patches/xemu-reference-ramht.patch",
        "patches/xemu-android-image-reader.patch",
        "patches/xemu-android-session-monitor.patch",
        "patches/xemu-x87-instruction-boundary.patch",
        "patches/xemu-android-vertex-state.patch",
    ];
    let mut patch_ids = serde_json::Map::new();
    for patch in patches {
        c::store::checked(
            Command::new("git")
                .arg("-C")
                .arg(&source)
                .env("GIT_CEILING_DIRECTORIES", &new)
                .arg("apply")
                .arg(repo.join(patch)),
        )?;
        patch_ids.insert(patch.into(), json!(c::file_digest(&repo.join(patch))?));
    }
    let mut patched_sources = serde_json::Map::new();
    for path in &sources {
        patched_sources.insert(path.clone(), json!(c::file_digest(&source.join(path))?));
    }
    let core = new.join("libxemu_core.a");
    fs::copy(old.join("libxemu_core.a"), &core)?;
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let ar = llvm.join("llvm-ar");
    let members = output(Command::new(&ar).arg("t").arg(&core))?;
    for (path, member) in units {
        if members.lines().filter(|line| *line == member).count() != 1 {
            return Err("retained replacement member ambiguous".into());
        }
        let target = format!(
            "CMakeFiles/xemu_core.dir/{}/{path}.o",
            literal(&base)?.trim_start_matches('/')
        );
        let name = member.strip_suffix(".c.o").ok_or("member name")?;
        let object = compile(&old, &source, &base, &new, path, &target, name)?;
        c::store::checked(Command::new(&ar).arg("rD").arg(&core).arg(object))?;
    }
    let monitor_obj = compile(
        &old,
        &source,
        &base,
        &new,
        monitor,
        "CMakeFiles/xemu_core.dir/monitor_stub.c.o",
        "monitor_stub",
    )?;
    c::store::checked(Command::new(&ar).arg("rD").arg(&core).arg(monitor_obj))?;
    let lib = new.join("libxemu.so");
    let mut link = output(Command::new("ninja").current_dir(&old).args([
        "-t",
        "commands",
        "-s",
        "libxemu.so",
    ]))?;
    if link.lines().count() != 1 || !link.contains(" -o libxemu.so ") {
        return Err("retained native link owner differs".into());
    }
    link = link
        .replace(" -o libxemu.so ", &format!(" -o {} ", literal(&lib)?))
        .replace("CMakeFiles/xemu.dir/link.d", &literal(&new.join("link.d"))?)
        .replace("libxemu_core.a", &literal(&core)?);
    for name in adapters {
        let path = format!("android/app/src/main/cpp/{name}");
        let target = format!("CMakeFiles/xemu.dir/{name}.o");
        let stem = name
            .strip_suffix(".cpp")
            .or_else(|| name.strip_suffix(".c"))
            .ok_or("adapter extension")?;
        let object = compile(&old, &source, &base, &new, &path, &target, stem)?;
        link = link.replace(&target, &literal(&object)?);
    }
    let relative = "native/xbx_xemu_bridge.c";
    fs::create_dir_all(source.join("native"))?;
    c::write_new(&source.join(relative), &fs::read(repo.join(relative))?)?;
    let target = "CMakeFiles/xemu.dir/home/don/xbx/native/xbx_xemu_bridge.c.o";
    let bridge = compile(
        &old,
        &source,
        &base,
        &new,
        relative,
        target,
        "xbx_xemu_bridge",
    )?;
    link = link.replace(target, &literal(&bridge)?);
    // Keep the exact previously accepted constructor registrations. These
    // members otherwise disappear from the Android static archive link.
    for member in ["char-socket.c.o", "event-loop-base.c.o"] {
        if members.lines().filter(|line| *line == member).count() != 1 {
            return Err("retained registration member ambiguous".into());
        }
        let result = Command::new(&ar).arg("p").arg(&core).arg(member).output()?;
        if !result.status.success() || result.stdout.len() > 1024 * 1024 {
            return Err("bounded constructor extraction failed".into());
        }
        let object = new.join(member);
        c::write_new(&object, &result.stdout)?;
        link = link.replace(
            &literal(&core)?,
            &format!("{} {}", literal(&object)?, literal(&core)?),
        );
    }
    c::write_new(&new.join("link.command"), link.as_bytes())?;
    c::store::checked(Command::new("sh").current_dir(&old).arg("-c").arg(link))?;
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(&json!({
            "source_pin":PIN,"bridge_abi":4,
            "source_sha256":c::file_digest(&repo.join(relative))?,
            "library_sha256":c::file_digest(&lib)?,
            "retained_archive_sha256":c::file_digest(&old.join("libxemu_core.a"))?,
        "sources_before_patch":identities,"sources_after_patch":patched_sources,"patches":patch_ids,
            "native_units_compiled":12,"whole_core_rebuild":false,
            "Android_NV2A_version":4,"minimum_version":4,
            "retained_Android_timing_unchanged":true,
            "device_qualification":false,"performance_qualification":false
        }))?,
    )?;
    Ok(())
}
