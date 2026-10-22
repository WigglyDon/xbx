//! Bounded source qualification only. Never select the donor's comparison-image route.
use crate::run;
use std::path::{Path, PathBuf};
use std::process::Command;

const OXOS: &str = "c0c887a1263ebf89ba765f34a88ca8f132901e25";
const CROMWELL: &str = "86f547387184d1001d377cce97d3756c8acf91cc";

fn verify(path: &Path, pin: &str) -> Result<(), Box<dyn std::error::Error>> {
    let head = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "HEAD"])
        .output()?;
    let diff = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["diff", "--exit-code", "HEAD"])
        .output()?;
    let untracked = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["ls-files", "--others", "--exclude-standard"])
        .output()?;
    if !head.status.success()
        || String::from_utf8(head.stdout)?.trim() != pin
        || !diff.status.success()
        || !untracked.status.success()
        || !untracked.stdout.is_empty()
    {
        return Err(
            "open-kernel source differs from its qualification pin; preserve and inspect it".into(),
        );
    }
    Ok(())
}

pub fn build(root: &Path, play: bool) -> Result<(), Box<dyn std::error::Error>> {
    let _lease = crate::capacity::writer(root, 1)?;
    let source = root.join(".local/src/oxos");
    verify(&source, OXOS)?;
    verify(&source.join("third_party/cromwell"), CROMWELL)?;
    let ndk = PathBuf::from(
        std::env::var_os("XBX_NDK").ok_or("set XBX_NDK to the read-only LLVM toolchain")?,
    );
    let llvm = ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let mut paths = vec![llvm.clone(), root.join(".local/tools/bin")];
    if let Some(path) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&path));
    }
    let build = root.join(if play {
        ".local/build-oxos-play"
    } else {
        ".local/build-oxos"
    });
    run(Command::new("make")
        .arg("-C")
        .arg(source.join("kernel"))
        .env("PATH", std::env::join_paths(paths)?)
        .args([
            "-j1",
            "bios",
            "IMAGE=open",
            "SERIAL=1",
            "SPLASH=",
            "MCPX16=0",
            "BIOS_BASE=",
            "MCPX_ROM=",
            "STOCK_MCPX=",
        ])
        .arg(if play { "SELFTEST=0" } else { "SELFTEST=1" })
        .arg(format!("BUILD={}", build.display()))
        .arg(format!("CC={}", llvm.join("clang").display()))
        // The NT timer APC adapter intentionally casts equal-width i386 argument
        // slots. New LLVM diagnoses this retained donor cast; keep the warning
        // visible and all other -Werror checks rather than changing its ABI.
        .arg(format!(
            "CXX={} -Wno-error=cast-function-type-mismatch",
            llvm.join("clang++").display()
        ))
        .arg(format!("LD={}", llvm.join("lld-link").display()))
        .arg(format!("OBJCOPY={}", llvm.join("llvm-objcopy").display())))?;
    println!(
        "Local open-image build complete. Provenance, boot, game/save compatibility and shipping default remain unqualified."
    );
    Ok(())
}
