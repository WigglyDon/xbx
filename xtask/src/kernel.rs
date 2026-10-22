//! Bounded source qualification only. Never select the donor's comparison-image route.
use crate::run;
use std::path::{Path, PathBuf};
use std::process::Command;

const OXOS: &str = "c0c887a1263ebf89ba765f34a88ca8f132901e25";
const CROMWELL: &str = "86f547387184d1001d377cce97d3756c8acf91cc";

fn verify(path: &Path, pin: &str, patch: Option<&[u8]>) -> Result<(), Box<dyn std::error::Error>> {
    let head = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "HEAD"])
        .output()?;
    let diff = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["diff", "--binary", "HEAD"])
        .output()?;
    let untracked = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["ls-files", "--others", "--exclude-standard"])
        .output()?;
    if !head.status.success()
        || String::from_utf8(head.stdout)?.trim() != pin
        || !diff.status.success()
        || diff.stdout != patch.unwrap_or_default()
        || !untracked.status.success()
        || !untracked.stdout.is_empty()
    {
        return Err(
            "open-kernel source differs from its qualification pin; preserve and inspect it".into(),
        );
    }
    Ok(())
}

pub fn build(root: &Path, mode: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _lease = crate::capacity::writer(root, 1)?;
    let source = root.join(".local/src/oxos");
    let trace = mode == "trace";
    let play = mode == "play" || trace;
    let patch = if trace {
        Some(std::fs::read(root.join("patches/oxos-launch-trace.patch"))?)
    } else {
        None
    };
    verify(&source, OXOS, patch.as_deref())?;
    verify(&source.join("third_party/cromwell"), CROMWELL, None)?;
    let trace_hash = if trace {
        let out = Command::new("sha256sum")
            .arg(root.join("patches/oxos-launch-trace.patch"))
            .output()?;
        if !out.status.success() {
            return Err("trace patch hash failed".into());
        }
        let sha = String::from_utf8(out.stdout)?
            .split_whitespace()
            .next()
            .ok_or("missing patch digest")?
            .to_owned();
        let lock = std::fs::read_to_string(root.join("kernel-trace.lock.json"))?;
        if !lock.contains(&format!("\"patch_sha256\": \"{sha}\"")) {
            return Err("trace patch differs from recorded identity".into());
        }
        Some(sha)
    } else {
        None
    };
    let ndk = PathBuf::from(
        std::env::var_os("XBX_NDK").ok_or("set XBX_NDK to the read-only LLVM toolchain")?,
    );
    let llvm = ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let mut paths = vec![llvm.clone(), root.join(".local/tools/bin")];
    if let Some(path) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&path));
    }
    let build = if let Some(ref sha) = trace_hash {
        root.join(format!(".local/build-oxos-launch-{}", &sha[..12]))
    } else {
        root.join(if play {
            ".local/build-oxos-play"
        } else {
            ".local/build-oxos"
        })
    };
    run(Command::new("make")
        .arg("-C")
        .arg(source.join("kernel"))
        .env("PATH", std::env::join_paths(paths)?)
        .args([
            "-j1",
            "bios",
            "IMAGE=open",
            "SERIAL=1",
            "TRACE_IMPORTS=0",
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
            "CXX={} -Wno-error=cast-function-type-mismatch {}",
            llvm.join("clang++").display(),
            if trace { "-DXK_XBX_LAUNCH_TRACE=1" } else { "" }
        ))
        .arg(format!("LD={}", llvm.join("lld-link").display()))
        .arg(format!("OBJCOPY={}", llvm.join("llvm-objcopy").display())))?;
    if let Some(ref sha) = trace_hash {
        let out = Command::new("sha256sum")
            .arg(build.join("bios.bin"))
            .output()?;
        if !out.status.success() {
            return Err("image hash failed".into());
        }
        let fw = String::from_utf8(out.stdout)?
            .split_whitespace()
            .next()
            .ok_or("missing firmware hash")?
            .to_owned();
        std::fs::write(
            build.join("build.identity"),
            format!(
                "source={OXOS}\npatch_sha256={sha}\nfirmware_sha256={fw}\nselftest=0\nserial=1\ntrace_imports=0\nram_mib=64\n"
            ),
        )?;
        std::fs::write(
            root.join(".local/selected-launch-build"),
            build.to_str().ok_or("invalid launch build path")?,
        )?;
    }
    println!(
        "Local open-image build complete. Provenance, boot, game/save compatibility and shipping default remain unqualified."
    );
    Ok(())
}

/// Build only the retained title; no upstream test campaign or HDD download.
pub fn title(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let _lease = crate::capacity::writer(root, 1)?;
    let source = root.join(".local/src/oxos");
    let patch = std::fs::read(root.join("patches/oxos-launch-trace.patch"))?;
    verify(&source, OXOS, Some(&patch))?;
    let ndk = PathBuf::from(std::env::var_os("XBX_NDK").ok_or("set XBX_NDK")?);
    let llvm = ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let build = root.join(".local/build-relaunch-title");
    run(Command::new("make")
        .arg("-C")
        .arg(source.join("kernel"))
        .arg("-j1")
        .arg(format!("BUILD={}", build.display()))
        .arg(format!("CXX={}", llvm.join("clang++").display()))
        .arg(format!("LD={}", llvm.join("lld-link").display()))
        .arg(build.join("kernel-test-title.xbe")))?;
    run(Command::new("python3").args(["-I","-c","import sys;sys.path.insert(0,sys.argv[1]);import mkxiso;mkxiso.build(sys.argv[2],open(sys.argv[3],'rb').read(),True)"])
        .arg(source.join("tests")).arg(build.join("relaunch.iso")).arg(build.join("kernel-test-title.xbe")))?;
    Ok(())
}

/// Original mechanical guest ABI shim; Rust owns the build and acceptance policy.
pub fn nullcall_title(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    abi_title(
        root,
        "nullcall.S",
        "build-nullcall-fixture-v2",
        "xbx handled null call",
        "0x58425801",
        "nullcall.iso",
    )
}

/// One bounded loader-resolved reserve/commit/free contract, with synthetic RAM.
pub fn vm_import_title(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    abi_title(
        root,
        "vm-imports.S",
        "build-vm-import-fixture-v1",
        "xbx VM import contract",
        "0x58425802",
        "vm-imports.iso",
    )
}

fn abi_title(
    root: &Path,
    assembly: &str,
    directory: &str,
    name: &str,
    title_id: &str,
    disc: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let _lease = crate::capacity::writer(root, 1)?;
    let source = root.join(".local/src/oxos");
    let patch = std::fs::read(root.join("patches/oxos-launch-trace.patch"))?;
    verify(&source, OXOS, Some(&patch))?;
    let build = root.join(".local").join(directory);
    // Never replace a previously built fixture or failure evidence.
    std::fs::create_dir(&build)?;
    let ndk = PathBuf::from(std::env::var_os("XBX_NDK").ok_or("set XBX_NDK")?);
    let llvm = ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    run(Command::new(llvm.join("clang"))
        .args(["--target=i386-pc-win32", "-march=pentium3", "-c"])
        .arg(root.join("native/fixtures").join(assembly))
        .arg("-o")
        .arg(build.join("title.obj")))?;
    run(Command::new(llvm.join("lld-link"))
        .args([
            "/NODEFAULTLIB",
            "/ENTRY:XkTitleEntry",
            "/SUBSYSTEM:NATIVE",
            "/BASE:0x10000",
            "/FIXED",
            "/Brepro",
            "/DEBUG:NONE",
        ])
        .arg(format!("/MAP:{}", build.join("title.map").display()))
        .arg(format!("/OUT:{}", build.join("title.exe").display()))
        .arg(build.join("title.obj")))?;
    run(Command::new("python3")
        .arg("-I")
        .arg(source.join("tools/mkxbe.py"))
        .arg("--pe")
        .arg(build.join("title.exe"))
        .arg("--map")
        .arg(build.join("title.map"))
        .args([
            "--thunks",
            "_XkThunks",
            "--name",
            name,
            "--title-id",
            title_id,
            "--out",
        ])
        .arg(build.join("title.xbe")))?;
    run(Command::new("python3").args(["-I", "-c", "import sys;sys.path.insert(0,sys.argv[1]);import mkxiso;mkxiso.build(sys.argv[2],open(sys.argv[3],'rb').read(),False)"])
        .arg(source.join("tests")).arg(build.join(disc)).arg(build.join("title.xbe")))?;
    Ok(())
}
