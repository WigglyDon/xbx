use crate::run;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn command(root: &Path, action: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _lease = crate::capacity::writer(root, 3)?;
    crate::source::verify(root)?;
    let build = root.join(".local/build-desktop");
    let source = root.join(".local/src/xemu-android");
    let dsp = root.join(".local/build-android/_deps/dsp56300_src-src");
    let dsp_target = build.join("dsp");
    fs::create_dir_all(&build)?;
    if action == "desktop-dsp" {
        return run(Command::new("cargo")
            .env("RUSTUP_TOOLCHAIN", "stable")
            .env("CARGO_HOME", root.join(".local/cargo-home"))
            .env("CARGO_TARGET_DIR", &dsp_target)
            .args([
                "build",
                "--locked",
                "--offline",
                "--release",
                "-j",
                "1",
                "--target",
                "x86_64-unknown-linux-gnu",
                "--manifest-path",
            ])
            .arg(dsp.join("crates/emu-ffi/Cargo.toml")));
    }
    if action == "desktop-configure" {
        for (name, cached) in [
            ("glslang", "glslang"),
            ("volk", "volk"),
            ("SPIRV-Reflect", "spirv_reflect"),
            ("VulkanMemoryAllocator", "vma"),
            ("nv2a_vsh_cpu", "nv2a_vsh_cpu"),
            ("tomlplusplus", "tomlplusplus"),
            ("slirp", "libslirp"),
        ] {
            let from = root.join(format!(".local/build-android/_deps/{cached}-src"));
            let to = source.join("subprojects").join(name);
            if to.exists() {
                if to.canonicalize()? != from.canonicalize()? {
                    return Err(
                        "preserving a different existing subproject; inspect it before reuse"
                            .into(),
                    );
                }
            } else {
                std::os::unix::fs::symlink(from, to)?;
            }
        }
        let pc = root.join(".local/host-sysroot/usr/lib64/pkgconfig");
        let pc_share = root.join(".local/host-sysroot/usr/share/pkgconfig");
        return run(Command::new(source.join("configure"))
            .current_dir(&build)
            .env(
                "PKG_CONFIG_PATH",
                format!("{}:{}", pc.display(), pc_share.display()),
            )
            .env("XBX_DSP_SOURCE", dsp)
            .env(
                "XBX_DSP_LIBRARY",
                dsp_target.join("x86_64-unknown-linux-gnu/release/libdsp56300_emu_ffi.a"),
            )
            .arg(format!(
                "--python={}",
                root.join(".local/tools/bin/python3").display()
            ))
            .args([
                "--target-list=i386-softmmu",
                "--extra-cflags=-DXBOX=1",
                "--disable-werror",
                "--disable-docs",
                "--disable-tools",
                "--disable-guest-agent",
                "--disable-rust",
            ]));
    }
    if action == "desktop-build" {
        let python_paths = Command::new(root.join(".local/tools/bin/python3"))
            .args([
                "-c",
                "import sysconfig; print(sysconfig.get_paths()['purelib'])",
            ])
            .output()?;
        if !python_paths.status.success() {
            return Err("cannot resolve task-local Python tool path".into());
        }
        let python_paths = String::from_utf8(python_paths.stdout)?;
        return run(Command::new("ninja")
            .env("PYTHONPATH", python_paths.trim())
            .arg("-C")
            .arg(&build)
            .args(["-j", "1", "qemu-system-i386"]));
    }
    Err("unknown desktop action".into())
}
