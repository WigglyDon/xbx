// SPDX-License-Identifier: MIT
//! Targeted retained-object Android integration, never a fresh core/SDK build.
use super::{self as c};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
type Error = Box<dyn std::error::Error>;
pub(super) fn output(command: &mut Command) -> Result<String, Error> {
    let value = command.output()?;
    if !value.status.success() {
        return Err(format!(
            "retained build metadata/source operation failed: {}",
            String::from_utf8_lossy(&value.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(value.stdout)?)
}
pub(super) fn literal(path: &Path) -> Result<String, Error> {
    let s = path.to_str().ok_or("build path encoding")?;
    if !s
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"/_-.".contains(&b))
    {
        return Err("unsafe retained compiler path".into());
    }
    Ok(s.into())
}
fn compile(
    old: &Path,
    source: &Path,
    base: &Path,
    new: &Path,
    relative: &str,
    target: &str,
    name: &str,
) -> Result<PathBuf, Error> {
    let original = output(
        Command::new("ninja")
            .current_dir(old)
            .args(["-t", "commands", "-s", target]),
    )?;
    if original.lines().count() != 1 || !original.contains(" -c ") {
        return Err("expected one retained compile command".into());
    }
    let object = new.join(format!("{name}.c.o"));
    let input = original
        .rsplit_once(" -c ")
        .ok_or("retained input argument")?
        .1
        .trim();
    let command = original
        .replace(target, &literal(&object)?)
        .replace(&literal(base)?, &literal(source)?);
    let command = command.replace(input, &literal(&source.join(relative))?);
    if !command.contains(&format!(" -c {}", literal(&source.join(relative))?))
        || command.contains(&format!(" -o {target}"))
    {
        return Err("retained compile mapping incomplete".into());
    }
    c::write_new(&new.join(format!("{name}.command")), command.as_bytes())?;
    c::store::checked(Command::new("sh").current_dir(old).arg("-c").arg(command))?;
    Ok(object)
}

pub(super) fn link_adapters(repo: &Path, root: &Path) -> Result<(), Error> {
    let old = repo.join(".local/build-android");
    let base = repo.join(".local/src/xemu-android");
    let previous = root.join("android-native-v1");
    let source = previous.join("source");
    let new = root.join("android-native-link-v1");
    c::mkdir(&new)?;
    let mut replacements = Vec::new();
    let mut sources = serde_json::Map::new();
    for (name, relative, target) in [
        (
            "xemu_android",
            "android/app/src/main/cpp/xemu_android.cpp",
            "CMakeFiles/xemu.dir/xemu_android.cpp.o",
        ),
        (
            "xemu_lan",
            "android/app/src/main/cpp/xemu_lan.cpp",
            "CMakeFiles/xemu.dir/xemu_lan.cpp.o",
        ),
        (
            "android_crash_handler",
            "android/app/src/main/cpp/android_crash_handler.cpp",
            "CMakeFiles/xemu.dir/android_crash_handler.cpp.o",
        ),
        (
            "vmstate_if_android",
            "android/app/src/main/cpp/vmstate_if_android.c",
            "CMakeFiles/xemu.dir/vmstate_if_android.c.o",
        ),
        (
            "xemu_fatx_import",
            "android/app/src/main/cpp/xemu_fatx_import.c",
            "CMakeFiles/xemu.dir/xemu_fatx_import.c.o",
        ),
        (
            "xbx_xemu_bridge",
            "native/xbx_xemu_bridge.c",
            "CMakeFiles/xemu.dir/home/don/xbx/native/xbx_xemu_bridge.c.o",
        ),
    ] {
        let from = if relative.starts_with("native/") {
            repo.join(relative)
        } else {
            base.join(relative)
        };
        let to = source.join(relative);
        fs::create_dir_all(to.parent().ok_or("source parent")?)?;
        c::write_new(&to, &fs::read(&from)?)?;
        sources.insert(relative.into(), json!(c::file_digest(&to)?));
        let object = compile(&old, &source, &base, &new, relative, target, name)?;
        replacements.push((target, object));
    }
    let link = output(Command::new("ninja").current_dir(&old).args([
        "-t",
        "commands",
        "-s",
        "libxemu.so",
    ]))?;
    if link.lines().count() != 1 || !link.contains(" -o libxemu.so ") {
        return Err("retained link owner differs".into());
    }
    let lib = new.join("libxemu.so");
    let core = previous.join("libxemu_core.a");
    let mut command = link
        .replace(" -o libxemu.so ", &format!(" -o {} ", literal(&lib)?))
        .replace("CMakeFiles/xemu.dir/link.d", &literal(&new.join("link.d"))?)
        .replace(
            "CMakeFiles/xemu.dir/xemu_hdd_tools_jni.c.o",
            &literal(&previous.join("xemu_hdd_tools_jni.c.o"))?,
        )
        .replace("libxemu_core.a", &literal(&core)?);
    for (target, object) in replacements {
        command = command.replace(target, &literal(&object)?);
    }
    c::write_new(&new.join("link.command"), command.as_bytes())?;
    c::store::checked(Command::new("sh").current_dir(&old).arg("-c").arg(command))?;
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(&json!({
            "source_pin":"4967efd3ee594cc236869a45ee4cb396e25af1ac",
            "retained_library_sha256":c::file_digest(&old.join("libxemu.so"))?,
            "retained_archive_sha256":c::file_digest(&old.join("libxemu_core.a"))?,
            "reused_prepared_archive_sha256":c::file_digest(&core)?,
            "reused_completed_targeted_units":["migration-hmp-cmds.c","pfifo.c","xemu_hdd_tools_jni.c"],
            "six_previously_missing_adapter_objects_source_sha256":sources,
            "inherited_source_patch_sha256":c::file_digest(&previous.join("inherited-source.patch"))?,
            "library_sha256":c::file_digest(&lib)?,"new_SDK":false,"whole_core_rebuild":false,
            "earlier_source_and_builds_preserved":true,"device_operation":false,"lifecycle_qualification":false
        }))?,
    )?;
    Ok(())
}
pub(super) fn native(repo: &Path, root: &Path) -> Result<(), Error> {
    let old = repo.join(".local/build-android");
    let base = repo.join(".local/src/xemu-android");
    let pin = output(
        Command::new("git")
            .arg("-C")
            .arg(&base)
            .args(["rev-parse", "HEAD"]),
    )?;
    if pin.trim() != "4967efd3ee594cc236869a45ee4cb396e25af1ac" {
        return Err("retained Android pin differs".into());
    }
    let new = root.join("android-native-v1");
    c::mkdir(&new)?;
    let source = new.join("source");
    c::mkdir(&source)?;
    let tree = output(
        Command::new("git")
            .arg("-C")
            .arg(&base)
            .args(["ls-tree", "-rl", "HEAD"]),
    )?;
    let mut files = Vec::new();
    let mut size = 0u64;
    for line in tree.lines() {
        let (meta, path) = line.split_once('\t').ok_or("retained tree entry")?;
        let fields: Vec<_> = meta.split_whitespace().collect();
        if fields.get(1) == Some(&"blob") && (path.ends_with(".h") || path.ends_with(".inc")) {
            size += fields.get(3).ok_or("header size")?.parse::<u64>()?;
            files.push(path.to_owned());
        }
    }
    if files.len() > 5000 || size > 64 * 1024 * 1024 {
        return Err("selected public headers exceed bounded source restoration".into());
    }
    for p in [
        "android/app/src/main/cpp/xemu_hdd_tools_jni.c",
        "migration/migration-hmp-cmds.c",
        "hw/xbox/nv2a/pfifo.c",
    ] {
        files.push(p.into());
    }
    let archive = new.join("retained-public-headers.tar");
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&archive)?;
    let status = Command::new("git")
        .arg("-C")
        .arg(&base)
        .args(["archive", "HEAD", "--"])
        .args(&files)
        .env("GIT_NO_LAZY_FETCH", "1")
        .stdout(Stdio::from(file))
        .status()?;
    if !status.success() {
        return Err(
            "retained public source archive incomplete; no new dependency fetch or core rebuild"
                .into(),
        );
    }
    c::store::checked(
        Command::new("tar")
            .args(["--no-same-owner", "-xf"])
            .arg(&archive)
            .arg("-C")
            .arg(&source),
    )?;
    // Preserve accepted modified headers and bind all inherited local changes;
    // the old source, native archive and every previous artifact stay untouched.
    let diff = output(
        Command::new("git")
            .arg("-C")
            .arg(&base)
            .args(["diff", "--binary"]),
    )?;
    c::write_new(&new.join("inherited-source.patch"), diff.as_bytes())?;
    for path in &files {
        let retained = base.join(path);
        if (path.ends_with(".h") || path.ends_with(".inc")) && retained.is_file() {
            fs::copy(retained, source.join(path))?;
        }
    }
    for patch in [
        "patches/xemu-paused-savevm.patch",
        "patches/xemu-reference-ramht.patch",
        "patches/xemu-android-image-reader.patch",
    ] {
        c::store::checked(
            Command::new("git")
                .arg("-C")
                .arg(&source)
                .arg("apply")
                .arg(repo.join(patch)),
        )?;
    }
    let hmp = compile(
        &old,
        &source,
        &base,
        &new,
        "migration/migration-hmp-cmds.c",
        "CMakeFiles/xemu_core.dir/home/don/xbx/.local/src/xemu-android/migration/migration-hmp-cmds.c.o",
        "migration-hmp-cmds",
    )?;
    let pfifo = compile(
        &old,
        &source,
        &base,
        &new,
        "hw/xbox/nv2a/pfifo.c",
        "CMakeFiles/xemu_core.dir/home/don/xbx/.local/src/xemu-android/hw/xbox/nv2a/pfifo.c.o",
        "pfifo",
    )?;
    let disk_target = "CMakeFiles/xemu.dir/xemu_hdd_tools_jni.c.o";
    let disk = compile(
        &old,
        &source,
        &base,
        &new,
        "android/app/src/main/cpp/xemu_hdd_tools_jni.c",
        disk_target,
        "xemu_hdd_tools_jni",
    )?;
    let core = new.join("libxemu_core.a");
    fs::copy(old.join("libxemu_core.a"), &core)?;
    let ar = PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar");
    let members = output(Command::new(&ar).arg("t").arg(&core))?;
    for name in ["migration-hmp-cmds.c.o", "pfifo.c.o"] {
        if members.lines().filter(|s| *s == name).count() != 1 {
            return Err("archive member identity ambiguous; preserve outputs".into());
        }
    }
    c::store::checked(Command::new(ar).arg("rD").arg(&core).arg(hmp).arg(pfifo))?;
    let link = output(Command::new("ninja").current_dir(&old).args([
        "-t",
        "commands",
        "-s",
        "libxemu.so",
    ]))?;
    if link.lines().count() != 1 || !link.contains(" -o libxemu.so ") {
        return Err("retained link owner differs".into());
    }
    let lib = new.join("libxemu.so");
    let command = link
        .replace(" -o libxemu.so ", &format!(" -o {} ", literal(&lib)?))
        .replace("CMakeFiles/xemu.dir/link.d", &literal(&new.join("link.d"))?)
        .replace(disk_target, &literal(&disk)?)
        .replace("libxemu_core.a", &literal(&core)?);
    c::write_new(&new.join("link.command"), command.as_bytes())?;
    c::store::checked(Command::new("sh").current_dir(&old).arg("-c").arg(command))?;
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(&json!({
            "source_pin":pin.trim(),"retained_library_sha256":c::file_digest(&old.join("libxemu.so"))?,
            "retained_archive_sha256":c::file_digest(&old.join("libxemu_core.a"))?,
            "inherited_source_patch_sha256":c::file_digest(&new.join("inherited-source.patch"))?,
            "restored_header_count":files.len()-3,"restored_header_logical_bytes":size,
            "recompiled_units":["migration/migration-hmp-cmds.c","hw/xbox/nv2a/pfifo.c","android/app/src/main/cpp/xemu_hdd_tools_jni.c"],
            "source_archive_sha256":c::file_digest(&archive)?,"library_sha256":c::file_digest(&lib)?,
            "new_SDK":false,"whole_core_rebuild":false,"earlier_source_and_builds_preserved":true,
            "device_operation":false,"lifecycle_qualification":false
        }))?,
    )?;
    Ok(())
}

pub(super) fn image_helper(repo: &Path, root: &Path) -> Result<(), Error> {
    let native = root.join("android-native-bridge-v1");
    let old = repo.join(".local/build-android");
    let out = root.join("android-helper-v4");
    c::mkdir(&out)?;
    let source = std::env::current_exe()?
        .parent()
        .ok_or("builder")?
        .parent()
        .ok_or("tools")?
        .join("source");
    let ndk = PathBuf::from(std::env::var("XBX_NDK")?);
    let linker =
        ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android29-clang");
    c::store::checked(Command::new("cargo").current_dir(&source)
        .args(["build","--locked","--offline","-j","1","--release","--target","aarch64-linux-android","-p","xbx-host","--bin","xbx-disk"])
        .env("CARGO_TARGET_DIR",root.join("android-rust-target"))
        .env("XBX_NATIVE_LIB_DIR",&native)
        .env("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER",linker)
        .env("RUSTFLAGS",format!("-L native={} -L native={} -C link-arg=-Wl,-rpath,$ORIGIN -C link-arg=-Wl,-rpath-link,{}",literal(&native)?,literal(&old.join("_deps/sdl3-build"))?,literal(&old.join("_deps/sdl3-build"))?)))?;
    fs::copy(
        root.join("android-rust-target/aarch64-linux-android/release/xbx-disk"),
        out.join("libxbx-disk.so"),
    )?;
    // Generated by XBX, using no dashboard, owner saves or firmware contents.
    crate::disk::create_empty_raw(&out.join("empty.raw"))?;
    c::store::checked(
        Command::new("/usr/bin/qemu-img")
            .args(["convert", "-f", "raw", "-O", "qcow2", "-S", "4096"])
            .arg(out.join("empty.raw"))
            .arg(out.join("empty.qcow2")),
    )?;
    c::write_new(
        &out.join("build.identity.json"),
        &c::json(&json!({
            "rust_source":source,"native_library_sha256":c::file_digest(&native.join("libxemu.so"))?,
            "image_helper_sha256":c::file_digest(&out.join("libxbx-disk.so"))?,
            "generated_empty_seed_sha256":c::file_digest(&out.join("empty.qcow2"))?,
            "disk_tool_semantics":"same native qcow2 backend; read-only canonical nonzero64KiB logical-block digest in separate helper process",
            "helper_logical_size_bound":crate::disk::DISK_BYTES,"helper_read_deadline_seconds":90,
            "no_private_input":true,"device_execution":false
        }))?,
    )?;
    Ok(())
}

pub(super) fn bridge(repo: &Path, root: &Path) -> Result<(), Error> {
    let old = repo.join(".local/build-android");
    let base = repo.join(".local/src/xemu-android");
    let previous = root.join("android-native-link-v1");
    let headers = root.join("android-native-v1/source");
    let new = root.join("android-native-bridge-v1");
    c::mkdir(&new)?;
    let source = new.join("source");
    c::mkdir(&source)?;
    for name in ["include", "ui", "android"] {
        std::os::unix::fs::symlink(headers.join(name), source.join(name))?;
    }
    c::mkdir(&source.join("native"))?;
    let shim = repo.join("native/xbx_xemu_bridge.c");
    c::write_new(&source.join("native/xbx_xemu_bridge.c"), &fs::read(&shim)?)?;
    let object = compile(
        &old,
        &source,
        &base,
        &new,
        "native/xbx_xemu_bridge.c",
        "CMakeFiles/xemu.dir/home/don/xbx/native/xbx_xemu_bridge.c.o",
        "xbx_xemu_bridge",
    )?;
    let lib = new.join("libxemu.so");
    let command = fs::read_to_string(previous.join("link.command"))?
        .replace(
            &format!(" -o {} ", literal(&previous.join("libxemu.so"))?),
            &format!(" -o {} ", literal(&lib)?),
        )
        .replace(
            &literal(&previous.join("link.d"))?,
            &literal(&new.join("link.d"))?,
        )
        .replace(
            &literal(&previous.join("xbx_xemu_bridge.c.o"))?,
            &literal(&object)?,
        );
    c::write_new(&new.join("link.command"), command.as_bytes())?;
    c::store::checked(Command::new("sh").current_dir(&old).arg("-c").arg(command))?;
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(
            &json!({"bridge_abi":4,"source_sha256":c::file_digest(&shim)?,"library_sha256":c::file_digest(&lib)?,"prior_library_sha256":c::file_digest(&previous.join("libxemu.so"))?,"recompiled_only":"native/xbx_xemu_bridge.c","reused_prepared_core_and_adapters":true,"device_operation":false}),
        )?,
    )?;
    Ok(())
}

/// QMP and the original cartridge UART both require the socket chardev. Its
/// registration lives in a constructor-only archive member, so the static
/// Android link otherwise drops it. Retain that exact member as a direct input;
/// do not rebuild or force-load the entire core.
pub(super) fn socket_driver(repo: &Path, root: &Path) -> Result<(), Error> {
    retain_registration(
        repo,
        root,
        Registration {
            previous: "android-native-bridge-v1",
            next: "android-native-socket-v1",
            expected: "9c8b5e1dcf9654d1ec907bc6c70ba97421183a99efa5e2dc8681fd9097cc8c7a",
            member: "char-socket.c.o",
            symbols: &[
                "qmp_chardev_open_socket",
                "char_socket_class_init",
                "tcp_chr_connect",
            ],
        },
    )
}

/// The donor's fallback event-loop base has no completion interface. QMP's
/// IOThread therefore exists with a null context unless the real base type's
/// constructor is retained. Keep the original implementation and its ordering.
pub(super) fn monitor_base(repo: &Path, root: &Path) -> Result<(), Error> {
    retain_registration(
        repo,
        root,
        Registration {
            previous: "android-native-socket-v1",
            next: "android-native-monitor-v1",
            expected: "0a79043449af4dae36c56b2ab255f176098e96e31b2f582eec2cb473aa86ec6d",
            member: "event-loop-base.c.o",
            symbols: &[
                "event_loop_base_complete",
                "event_loop_base_class_init",
                "qmp_chardev_open_socket",
            ],
        },
    )
}

struct Registration {
    previous: &'static str,
    next: &'static str,
    expected: &'static str,
    member: &'static str,
    symbols: &'static [&'static str],
}
fn retain_registration(repo: &Path, root: &Path, registration: Registration) -> Result<(), Error> {
    let previous = root.join(registration.previous);
    let archive = root.join("android-native-v1/libxemu_core.a");
    if c::file_digest(&previous.join("libxemu.so"))? != registration.expected {
        return Err("registration repair requires its retained ABI4 candidate".into());
    }
    let new = root.join(registration.next);
    c::mkdir(&new)?;
    let llvm =
        PathBuf::from(std::env::var("XBX_NDK")?).join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let members = output(Command::new(llvm.join("llvm-ar")).arg("t").arg(&archive))?;
    if members
        .lines()
        .filter(|s| *s == registration.member)
        .count()
        != 1
    {
        return Err("retained registration archive member is absent or ambiguous".into());
    }
    let member = Command::new(llvm.join("llvm-ar"))
        .arg("p")
        .arg(&archive)
        .arg(registration.member)
        .output()?;
    if !member.status.success() || member.stdout.is_empty() || member.stdout.len() > 1024 * 1024 {
        return Err("bounded retained registration member extraction failed".into());
    }
    let object = new.join(registration.member);
    c::write_new(&object, &member.stdout)?;
    let lib = link_registration(repo, root, &previous, &new, &object, registration.symbols)?;
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(&json!({
            "bridge_abi":4,"source_sha256":c::file_digest(&repo.join("native/xbx_xemu_bridge.c"))?,
            "library_sha256":c::file_digest(&lib)?,"prior_library_sha256":c::file_digest(&previous.join("libxemu.so"))?,
            "retained_archive_sha256":c::file_digest(&archive)?,"direct_member":registration.member,
            "direct_member_sha256":c::file_digest(&object)?,"source_pin":"4967efd3ee594cc236869a45ee4cb396e25af1ac",
            "recompiled_units":[],"whole_core_rebuild":false,"all_prior_artifacts_preserved":true,
            "evidence":"attributed original-cartridge QMP startup; retained constructor-only registration",
            "device_operation":false,"lifecycle_qualification":false
        }))?,
    )?;
    Ok(())
}

fn link_registration(
    repo: &Path,
    root: &Path,
    previous: &Path,
    new: &Path,
    object: &Path,
    required_symbols: &[&str],
) -> Result<PathBuf, Error> {
    let archive = root.join("android-native-v1/libxemu_core.a");
    let llvm =
        PathBuf::from(std::env::var("XBX_NDK")?).join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let lib = new.join("libxemu.so");
    let retained_link = fs::read_to_string(previous.join("link.command"))?;
    let archive_argument = format!(" {} ", literal(&archive)?);
    if retained_link.matches(&archive_argument).count() != 1 {
        return Err("expected one retained core archive input".into());
    }
    let command = retained_link
        .replace(
            &format!(" -o {} ", literal(&previous.join("libxemu.so"))?),
            &format!(" -o {} ", literal(&lib)?),
        )
        .replace(
            &literal(&previous.join("link.d"))?,
            &literal(&new.join("link.d"))?,
        )
        .replace(
            &archive_argument,
            &format!(" {} {} ", literal(object)?, literal(&archive)?),
        );
    c::write_new(&new.join("link.command"), command.as_bytes())?;
    c::store::checked(
        Command::new("sh")
            .current_dir(repo.join(".local/build-android"))
            .arg("-c")
            .arg(command),
    )?;
    let symbols = output(
        Command::new(llvm.join("llvm-nm"))
            .arg("--defined-only")
            .arg(&lib),
    )?;
    for symbol in required_symbols {
        if !symbols
            .lines()
            .any(|line| line.split_whitespace().last() == Some(*symbol))
        {
            return Err("linked required registration/backend is missing".into());
        }
    }
    c::write_new(&new.join("registration-symbols.txt"), symbols.as_bytes())?;
    Ok(lib)
}

pub(super) fn session_monitor(repo: &Path, root: &Path) -> Result<(), Error> {
    let previous = root.join("android-native-monitor-v1");
    if c::file_digest(&previous.join("libxemu.so"))?
        != "3e77e3f5c2b4ee156ad413db94b5001e05abc50c4dfc4456413f8799aa2e97ba"
    {
        return Err("session monitor requires the retained corrected QMP core".into());
    }
    let new = root.join("android-native-session-v2");
    c::mkdir(&new)?;
    let source = new.join("source");
    c::mkdir(&source)?;
    let headers = root.join("android-native-v1/source");
    for name in ["include", "monitor", "ui"] {
        std::os::unix::fs::symlink(headers.join(name), source.join(name))?;
    }
    let relative = "android/app/src/main/cpp/monitor_stub.c";
    let retained = output(
        Command::new("git")
            .arg("-C")
            .arg(repo.join(".local/src/xemu-android"))
            .args([
                "show",
                "4967efd3ee594cc236869a45ee4cb396e25af1ac:android/app/src/main/cpp/monitor_stub.c",
            ])
            .env("GIT_NO_LAZY_FETCH", "1"),
    )?;
    fs::create_dir_all(
        source
            .join(relative)
            .parent()
            .ok_or("monitor source parent")?,
    )?;
    // Android's pinned config-host/config-target headers live beside this
    // adapter, rather than in the generic CMake output directory. Reuse them
    // read-only in the separate source; never alter the retained header tree.
    for entry in fs::read_dir(headers.join("android/app/src/main/cpp"))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| matches!(ext.to_str(), Some("h" | "hpp" | "inc")))
        {
            std::os::unix::fs::symlink(
                path,
                source
                    .join("android/app/src/main/cpp")
                    .join(entry.file_name()),
            )?;
        }
    }
    c::write_new(&source.join(relative), retained.as_bytes())?;
    let patch = repo.join("patches/xemu-android-session-monitor.patch");
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .arg("apply")
            .arg(&patch),
    )?;
    let object = compile(
        &repo.join(".local/build-android"),
        &source,
        &repo.join(".local/src/xemu-android"),
        &new,
        relative,
        "CMakeFiles/xemu_core.dir/monitor_stub.c.o",
        "monitor_stub",
    )?;
    let lib = link_registration(
        repo,
        root,
        &previous,
        &new,
        &object,
        &[
            "hmp_savevm",
            "hmp_loadvm",
            "hmp_info_snapshots",
            "hmp_compare_cmd",
            "event_loop_base_complete",
            "qmp_chardev_open_socket",
        ],
    )?;
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(&json!({
            "bridge_abi":4,"source_sha256":c::file_digest(&repo.join("native/xbx_xemu_bridge.c"))?,
            "library_sha256":c::file_digest(&lib)?,"prior_library_sha256":c::file_digest(&previous.join("libxemu.so"))?,
            "source_pin":"4967efd3ee594cc236869a45ee4cb396e25af1ac","session_monitor_patch_sha256":c::file_digest(&patch)?,
            "retained_config_host_sha256":c::file_digest(&headers.join("android/app/src/main/cpp/config-host.h"))?,
            "retained_config_target_sha256":c::file_digest(&headers.join("android/app/src/main/cpp/config-target.h"))?,
            "recompiled_units":[relative],"monitor_source_sha256":c::file_digest(&source.join(relative))?,
            "reuses_accepted_paused_capture":true,"whole_core_rebuild":false,"device_operation":false,
            "all_prior_artifacts_preserved":true,"lifecycle_qualification":false
        }))?,
    )?;
    Ok(())
}

/// Execute the actual table/comparator with controlled handler callbacks. This
/// protects command routing; the Android VM still owns all snapshot acceptance.
pub(super) fn monitor_contract(repo: &Path, root: &Path) -> Result<(), Error> {
    let native = root.join("android-native-session-v2");
    let identity: serde_json::Value = c::read_json(&native.join("build.identity.json"))?;
    let subject = native.join("source/android/app/src/main/cpp/monitor_stub.c");
    if identity["session_monitor_patch_sha256"]
        != c::file_digest(&repo.join("patches/xemu-android-session-monitor.patch"))?
        || identity["monitor_source_sha256"] != c::file_digest(&subject)?
    {
        return Err("monitor contract requires the exact built source/patch".into());
    }
    let retained = output(
        Command::new("git")
            .arg("-C")
            .arg(repo.join(".local/src/xemu-android"))
            .args([
                "show",
                "4967efd3ee594cc236869a45ee4cb396e25af1ac:android/app/src/main/cpp/monitor_stub.c",
            ])
            .env("GIT_NO_LAZY_FETCH", "1"),
    )?;
    let changed = fs::read_to_string(&subject)?;
    let attempt = std::env::var("XBX_MONITOR_CONTRACT_ATTEMPT").unwrap_or_else(|_| "v1".into());
    if attempt.is_empty()
        || attempt.len() > 24
        || !attempt
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("invalid isolated monitor contract attempt".into());
    }
    let out = root.join(format!("android-monitor-contract-{attempt}"));
    c::mkdir(&out)?;
    let fixture = repo.join("native/fixtures/android-hmp-session.c");
    let mut exits = serde_json::Map::new();
    for (label, source, expected) in [("before", retained, 31), ("after", changed, 0)] {
        let include = out.join(format!("{label}.inc"));
        // The fixture supplies controlled host declarations, not an emulated
        // Android ABI. Function bodies and command entries are unchanged.
        let source = source
            .lines()
            .filter(|line| !line.trim_start().starts_with("#include"))
            .collect::<Vec<_>>()
            .join("\n");
        c::write_new(&include, source.as_bytes())?;
        let binary = out.join(label);
        c::store::checked(
            Command::new("cc")
                .args(["-std=gnu11", "-Wall", "-Werror"])
                .arg(format!("-DSUBJECT=\"{}\"", literal(&include)?))
                .arg(&fixture)
                .arg("-o")
                .arg(&binary),
        )?;
        let result = Command::new(&binary).output()?;
        c::write_new(&out.join(format!("{label}.stdout")), &result.stdout)?;
        c::write_new(&out.join(format!("{label}.stderr")), &result.stderr)?;
        exits.insert(label.into(), json!(result.status.code()));
        if result.status.code() != Some(expected) {
            return Err("monitor routing red/green contract failed; evidence retained".into());
        }
    }
    c::write_new(
        &out.join("result.json"),
        &c::json(&json!({
            "source_pin":"4967efd3ee594cc236869a45ee4cb396e25af1ac",
            "fixture_sha256":c::file_digest(&fixture)?,"native_monitor_source_sha256":c::file_digest(&subject)?,
            "session_monitor_patch_sha256":identity["session_monitor_patch_sha256"],"exits":exits,
            "qualifies":"actual comparator matching aliases/rejecting prefixes and separate save/load/list routing",
            "does_not_qualify":"Android ABI execution, native snapshot contents, guest execution or restoration",
            "passed":true,"device_operation":false
        }))?,
    )?;
    Ok(())
}
