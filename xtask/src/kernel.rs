//! Bounded source qualification only. Never select the donor's comparison-image route.
use crate::run;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) const OXOS: &str = "d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2";
const CROMWELL: &str = "86f547387184d1001d377cce97d3756c8acf91cc";

fn source(root: &Path) -> PathBuf {
    root.join(format!(".local/src/oxos-{OXOS}"))
}

// Full pins/digest and mode separate firmware variants. Exclusive
// creation also preserves interrupted attempts and date/time-dependent images.
fn reserve_build(root: &Path, pin: &str, patch: &str, mode: &str) -> std::io::Result<PathBuf> {
    let build = root.join(format!(".local/build-oxos-{pin}-{CROMWELL}-{mode}-{patch}"));
    std::fs::create_dir(&build)?;
    std::fs::write(
        build.join("build.inputs"),
        format!(
            "source={pin}\ncromwell={CROMWELL}\nmode={mode}\npatch_sha256={patch}\nimage=open\nselftest={}\nserial=1\ntrace_imports={}\nlaunch_trace={}\nram_mib=64\nsplash=none\nproprietary_inputs=none\n",
            u8::from(mode == "selftest"),
            u8::from(matches!(mode, "journal" | "npx" | "observer" | "context")),
            u8::from(matches!(
                mode,
                "trace" | "loader" | "journal" | "npx" | "observer" | "context"
            ))
        ),
    )?;
    Ok(build)
}

pub(crate) fn verify(
    path: &Path,
    pin: &str,
    patch: Option<&[u8]>,
) -> Result<(), Box<dyn std::error::Error>> {
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
    if !matches!(
        mode,
        "trace" | "loader" | "journal" | "npx" | "observer" | "context" | "play" | "selftest"
    ) {
        return Err(
            "kernel mode must be trace, loader, journal, npx, observer, context, play or selftest"
                .into(),
        );
    }
    let _lease = crate::capacity::writer(root, 1)?;
    let source = if mode == "context" {
        root.join(format!(".local/src/oxos-{OXOS}-context-v1"))
    } else if mode == "observer" {
        root.join(format!(".local/src/oxos-{OXOS}-observer-v1"))
    } else if mode == "npx" {
        root.join(format!(".local/src/oxos-{OXOS}-npx-owner-v2"))
    } else if mode == "journal" {
        root.join(format!(".local/src/oxos-{OXOS}-api-journal-v2"))
    } else if mode == "loader" {
        root.join(format!(".local/src/oxos-{OXOS}-loader-complete-v1"))
    } else {
        source(root)
    };
    let journal = matches!(mode, "journal" | "npx" | "observer" | "context");
    let trace = matches!(
        mode,
        "trace" | "loader" | "journal" | "npx" | "observer" | "context"
    );
    let play = mode == "play" || trace;
    let patch_path = root.join(if mode == "context" {
        "patches/oxos-context.patch"
    } else if mode == "observer" {
        "patches/oxos-observer.patch"
    } else if mode == "npx" {
        "patches/oxos-npx-owner.patch"
    } else if mode == "journal" {
        "patches/oxos-api-journal.patch"
    } else if mode == "loader" {
        "patches/oxos-loader-complete.patch"
    } else {
        "patches/oxos-launch-trace.patch"
    });
    let lock_path = root.join(if mode == "context" {
        "kernel-context.lock.json"
    } else if mode == "observer" {
        "kernel-observer.lock.json"
    } else if mode == "npx" {
        "kernel-npx.lock.json"
    } else if mode == "journal" {
        "kernel-journal.lock.json"
    } else if mode == "loader" {
        "kernel-loader.lock.json"
    } else {
        "kernel-trace.lock.json"
    });
    let patch = if trace {
        Some(std::fs::read(&patch_path)?)
    } else {
        None
    };
    verify(&source, OXOS, patch.as_deref())?;
    verify(&source.join("third_party/cromwell"), CROMWELL, None)?;
    let trace_hash = if trace {
        let out = Command::new("sha256sum").arg(&patch_path).output()?;
        if !out.status.success() {
            return Err("trace patch hash failed".into());
        }
        let sha = String::from_utf8(out.stdout)?
            .split_whitespace()
            .next()
            .ok_or("missing patch digest")?
            .to_owned();
        let lock = std::fs::read_to_string(&lock_path)?;
        if !lock.contains(&format!("\"patch_sha256\": \"{sha}\""))
            || !lock.contains(&format!("\"commit\": \"{OXOS}\""))
            || !lock.contains(&format!("\"cromwell_commit\": \"{CROMWELL}\""))
        {
            return Err("trace patch differs from recorded identity".into());
        }
        if matches!(mode, "loader" | "journal" | "npx" | "observer" | "context") {
            let base = Command::new("sha256sum")
                .arg(root.join("patches/oxos-launch-trace.patch"))
                .output()?;
            let hash = String::from_utf8(base.stdout)?
                .split_whitespace()
                .next()
                .ok_or("missing base digest")?
                .to_owned();
            if !base.status.success()
                || !lock.contains(&format!("\"base_trace_patch_sha256\": \"{hash}\""))
            {
                return Err("loader candidate base trace identity changed".into());
            }
            if journal {
                let base = Command::new("sha256sum")
                    .arg(root.join("patches/oxos-loader-complete.patch"))
                    .output()?;
                let hash = String::from_utf8(base.stdout)?;
                let hash = hash
                    .split_whitespace()
                    .next()
                    .ok_or("missing loader digest")?;
                if !base.status.success()
                    || !lock.contains(&format!("\"base_loader_patch_sha256\": \"{hash}\""))
                {
                    return Err("API journal base loader identity changed".into());
                }
            }
            if matches!(mode, "npx" | "observer" | "context") {
                let out = Command::new("sha256sum")
                    .arg(root.join("patches/oxos-api-journal.patch"))
                    .output()?;
                let output = String::from_utf8(out.stdout)?;
                let hash = output
                    .split_whitespace()
                    .next()
                    .ok_or("missing journal digest")?;
                if !out.status.success()
                    || !lock.contains(&format!("\"base_journal_patch_sha256\": \"{hash}\""))
                {
                    return Err("NPX correction base journal identity changed".into());
                }
            }
            if matches!(mode, "observer" | "context") {
                let out = Command::new("sha256sum")
                    .arg(root.join("patches/oxos-npx-owner.patch"))
                    .output()?;
                let output = String::from_utf8(out.stdout)?;
                let hash = output
                    .split_whitespace()
                    .next()
                    .ok_or("missing NPX digest")?;
                if !out.status.success()
                    || !lock.contains(&format!("\"base_npx_patch_sha256\": \"{hash}\""))
                {
                    return Err("observer base NPX identity changed".into());
                }
            }
        }
        if mode == "context" {
            let out = Command::new("sha256sum")
                .arg(root.join("patches/oxos-observer.patch"))
                .output()?;
            let output = String::from_utf8(out.stdout)?;
            let hash = output
                .split_whitespace()
                .next()
                .ok_or("missing observer digest")?;
            let lock = std::fs::read_to_string(&lock_path)?;
            if !out.status.success()
                || !lock.contains(&format!("\"base_observer_patch_sha256\": \"{hash}\""))
            {
                return Err("context base observer identity changed".into());
            }
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
    let build = reserve_build(root, OXOS, trace_hash.as_deref().unwrap_or("none"), mode)?;
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
        .arg(if journal {
            "TRACE_IMPORTS=1"
        } else {
            "TRACE_IMPORTS=0"
        })
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
    {
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
                "{}firmware_sha256={fw}\nmap_sha256={}\n",
                std::fs::read_to_string(build.join("build.inputs"))?,
                {
                    let out = Command::new("sha256sum")
                        .arg(build.join("xboxkrnl.map"))
                        .output()?;
                    if !out.status.success() {
                        return Err("kernel map hash failed".into());
                    }
                    String::from_utf8(out.stdout)?
                        .split_whitespace()
                        .next()
                        .ok_or("missing map digest")?
                        .to_owned()
                }
            ),
        )?;
    }
    println!(
        "Local open-image candidate: {}. No Android selector changed; boot/game/save compatibility and shipping default remain unqualified.",
        build.display()
    );
    Ok(())
}

/// Build only the retained title; no upstream test campaign or HDD download.
pub fn title(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let _lease = crate::capacity::writer(root, 1)?;
    let source = source(root);
    let patch = std::fs::read(root.join("patches/oxos-launch-trace.patch"))?;
    verify(&source, OXOS, Some(&patch))?;
    let ndk = PathBuf::from(std::env::var_os("XBX_NDK").ok_or("set XBX_NDK")?);
    let llvm = ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let build = root.join(format!(".local/build-relaunch-title-{OXOS}"));
    std::fs::create_dir(&build)?;
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

pub(crate) fn abi_title(
    root: &Path,
    assembly: &str,
    directory: &str,
    name: &str,
    title_id: &str,
    disc: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let _lease = crate::capacity::writer(root, 1)?;
    let source = source(root);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firmware_attempts_preserve_prior_images_and_inputs() {
        let root = std::env::temp_dir().join(format!(
            "xbx-build-policy-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join(".local")).unwrap();
        let prior = reserve_build(&root, "prior-source", "prior-patch", "trace").unwrap();
        std::fs::write(
            prior.join("bios.bin"),
            b"preserve interrupted or accepted image",
        )
        .unwrap();
        let receipt = std::fs::read(prior.join("build.inputs")).unwrap();
        assert_eq!(
            reserve_build(&root, "prior-source", "prior-patch", "trace")
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::AlreadyExists
        );
        for (pin, patch, mode) in [
            ("new-source", "prior-patch", "trace"),
            ("prior-source", "new-patch", "trace"),
            ("prior-source", "prior-patch", "play"),
            ("prior-source", "prior-patch", "selftest"),
        ] {
            assert_ne!(reserve_build(&root, pin, patch, mode).unwrap(), prior);
        }
        assert_eq!(
            std::fs::read(prior.join("bios.bin")).unwrap(),
            b"preserve interrupted or accepted image"
        );
        assert_eq!(std::fs::read(prior.join("build.inputs")).unwrap(), receipt);
        std::fs::remove_dir_all(root).unwrap();
    }
}
