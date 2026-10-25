use std::path::{Path, PathBuf};
use std::process::Command;

mod apk;
mod capacity;
mod cartridge;
mod desktop;
mod kernel;
mod loader_fixture;
mod open_startup;
mod profile;
mod ramht;
mod ramht_workspace;
mod reference;
mod roswell;
mod source;

fn run(command: &mut Command) -> Result<(), Box<dyn std::error::Error>> {
    let status = command.status()?;
    if !status.success() {
        return Err(format!("command failed: {status}").into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root)?;
    let source = root.join(".local/src/xemu-android");
    let build = root.join(".local/build-android");
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|a| a.starts_with("cartridge-")) {
        return cartridge::command(
            args.get(1).map(Path::new).unwrap_or(root),
            args.first().unwrap(),
            args.get(2).map(String::as_str).unwrap_or("v1"),
        );
    }
    if args.first().is_some_and(|a| a.starts_with("open-startup-")) {
        return open_startup::command(
            args.get(1).map(Path::new).unwrap_or(root),
            args.first().unwrap(),
            args.get(2).map(String::as_str).unwrap_or("v1"),
        );
    }
    if let Some(
        action @ ("ramht-check"
        | "ramht-configure"
        | "ramht-reconfigure"
        | "ramht-build"
        | "ramht-observe"
        | "ramht-title"
        | "ramht-title-host"
        | "ramht-title-host-extended"
        | "ramht-open-dashboard-host"
        | "ramht-open-title-host"
        | "ramht-media-copy"
        | "ramht-media-resume"),
    ) = args.first().map(String::as_str)
    {
        return ramht::command(
            args.get(1).map(Path::new).unwrap_or(root),
            action,
            args.get(2).map(String::as_str).unwrap_or("v1"),
        );
    }
    if args.first().map(String::as_str) == Some("roswell-observe") && matches!(args.len(), 5 | 6) {
        return roswell::observe(
            Path::new(&args[1]),
            &args[2],
            &args[3],
            args[4].parse()?,
            args.get(5).map(Path::new),
        );
    }
    if let Some(action @ ("roswell-configure" | "roswell-build")) = args.first().map(String::as_str)
    {
        return roswell::command(
            args.get(2).map(Path::new).unwrap_or(root),
            action,
            args.get(1).map(String::as_str).unwrap_or("v1"),
        );
    }
    if args.first().map(String::as_str) == Some("startup-totals-fixture-build") {
        return kernel::abi_title(
            root,
            "startup-totals.S",
            "build-startup-totals-fixture-v1",
            "xbx startup totals contract",
            "0x58425808",
            "startup-totals.iso",
        );
    }
    if args.first().map(String::as_str) == Some("observer-fixture-build") {
        return kernel::abi_title(
            root,
            "observer.S",
            "build-observer-fixture-v2",
            "xbx bounded observer contract",
            "0x58425807",
            "observer.iso",
        );
    }
    if let Some(action @ ("reference-configure" | "reference-build")) =
        args.first().map(String::as_str)
    {
        return reference::command(
            root,
            action,
            args.get(1).map(String::as_str).unwrap_or("v1"),
        );
    }
    if args.first().map(String::as_str) == Some("journal-fixture-build") {
        return kernel::abi_title(
            root,
            "api-journal.S",
            "build-api-journal-fixture-v2",
            "xbx public API journal",
            "0x58425804",
            "journal.iso",
        );
    }
    if args.first().map(String::as_str) == Some("thread-fixture-build") {
        return kernel::abi_title(
            root,
            "thread-startup.S",
            "build-thread-startup-fixture-v1",
            "xbx thread startup contract",
            "0x58425805",
            "thread.iso",
        );
    }
    if args.first().map(String::as_str) == Some("npx-fixture-build") {
        return kernel::abi_title(
            root,
            "npx-ownership.S",
            "build-npx-ownership-fixture-v1",
            "xbx floating point ownership",
            "0x58425806",
            "npx.iso",
        );
    }
    if args.first().map(String::as_str) == Some("loader-fixture-build") {
        return loader_fixture::build(root);
    }
    if args.first().map(String::as_str) == Some("vm-import-build") {
        return kernel::vm_import_title(root);
    }
    if args.first().map(String::as_str) == Some("nullcall-build") {
        return kernel::nullcall_title(root);
    }
    if args.first().map(String::as_str) == Some("relaunch-build") {
        return kernel::title(root);
    }
    if args.first().map(String::as_str) == Some("profile-assets") {
        return profile::prepare(root, args.get(1).map(String::as_str) == Some("trace"));
    }
    if args.first().map(String::as_str) == Some("kernel-build") {
        return kernel::build(root, args.get(1).map(String::as_str).unwrap_or("selftest"));
    }
    if let Some(action @ ("desktop-dsp" | "desktop-configure" | "desktop-build")) =
        args.first().map(String::as_str)
    {
        return desktop::command(root, action);
    }
    if args.first().map(String::as_str) == Some("capacity") {
        return capacity::check(
            root,
            args.get(1).map(String::as_str).unwrap_or("6").parse()?,
        );
    }
    if args.first().map(String::as_str) == Some("source-check") {
        source::verify(root)?;
        println!("Android xemu HEAD and complete working patch match the recorded candidate.");
        return Ok(());
    }
    if args.first().map(String::as_str) == Some("source-prepare") {
        return source::prepare(root);
    }
    if args.first().map(String::as_str) == Some("android-configure") {
        let _lock = capacity::writer(root, 6)?;
        source::verify(root)?;
        let ndk = PathBuf::from(
            std::env::var_os("XBX_NDK").ok_or("set XBX_NDK to a read-only installed NDK")?,
        );
        let meson = root.join(".local/tools/bin/meson");
        return run(Command::new("cmake")
            .arg("-S")
            .arg(source.join("android/app/src/main/cpp"))
            .arg("-B")
            .arg(build)
            .args(["-G", "Ninja"])
            .arg(format!(
                "-DCMAKE_TOOLCHAIN_FILE={}",
                ndk.join("build/cmake/android.toolchain.cmake").display()
            ))
            .args([
                "-DANDROID_ABI=arm64-v8a",
                "-DANDROID_PLATFORM=android-29",
                "-DCMAKE_BUILD_TYPE=Release",
                "-DXEMU_ENABLE_XISO_CONVERTER=OFF",
                "-DXEMU_ENABLE_PERF_LOG=OFF",
            ])
            .arg(format!("-DMESON_EXECUTABLE={}", meson.display()))
            .arg(format!(
                "-DXBX_BRIDGE_SOURCE={}",
                root.join("native/xbx_xemu_bridge.c").display()
            ))
            .arg(format!(
                "-DXEMU_CARGO_HOME={}",
                root.join(".local/cargo-home").display()
            )));
    }
    if args.first().map(String::as_str) == Some("android-build") {
        let _lock = capacity::writer(root, 6)?;
        source::verify(root)?;
        // One outer job; also bound Cargo, whose parallelism Ninja does not control.
        return run(Command::new("cmake")
            .env("CARGO_BUILD_JOBS", "1")
            .arg("--build")
            .arg(build)
            .args(["--parallel", "1", "--target", "xemu"]));
    }
    if args.first().map(String::as_str) == Some("android-host") {
        let _lock = capacity::writer(root, 1)?;
        let ndk = PathBuf::from(std::env::var_os("XBX_NDK").ok_or("set XBX_NDK")?);
        return run(Command::new("cargo")
            .env("XBX_NATIVE_LIB_DIR", &build)
            .env(
                "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER",
                ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android29-clang"),
            )
            .args([
                "build",
                "--locked",
                "--offline",
                "-j",
                "1",
                "-p",
                "xbx-host",
                "--lib",
                "--release",
                "--target",
                "aarch64-linux-android",
            ]));
    }
    if args.first().map(String::as_str) == Some("android-apk") {
        let _lock = capacity::writer(root, 1)?;
        let sdk = PathBuf::from(
            std::env::var_os("XBX_SDK")
                .ok_or("set XBX_SDK to a read-only installed Android SDK")?,
        );
        return apk::build(root, &sdk);
    }
    Err("usage: cargo xtask source-check | android-configure | android-build; run host capacity preflight before growth".into())
}
