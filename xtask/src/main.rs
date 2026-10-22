use std::path::{Path, PathBuf};
use std::process::Command;

mod apk;
mod capacity;
mod desktop;
mod kernel;
mod profile;
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
