//! Pinned basic-logging open firmware; no import tracer or paused probes.
use crate::ramht_workspace::Workspace;
use crate::reference::{git, hash};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;
type Error = Box<dyn std::error::Error>;
const PIN: &str = "9d8cd58ca19f49f5ecb4b7b55bfa194f4e4a5a96";
const CROMWELL: &str = "86f547387184d1001d377cce97d3756c8acf91cc";
const CORE: &str = "eaefdb4f61d01133557374585383e531438e0341d11579c11e8575283a5ccb9b";
fn directory(p: &Path) -> Result<(), Error> {
    fs::DirBuilder::new().mode(0o700).create(p)?;
    Ok(())
}
fn save(p: &Path, value: &str) -> Result<(), Error> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(p)?;
    f.write_all(value.as_bytes())?;
    f.sync_all()?;
    Ok(())
}
fn checked(w: &Workspace, c: &mut Command, name: &str, seconds: u64) -> Result<(), Error> {
    if !w.run(c, &w.root.join(name), seconds)?.success() {
        return Err(format!("operation failed; retained {name}").into());
    }
    Ok(())
}
fn export(
    w: &Workspace,
    source: &Path,
    pin: &str,
    dest: &Path,
    label: &str,
    paths: &[&str],
) -> Result<(), Error> {
    directory(dest)?;
    let archive = w.root.join(format!("{label}.tar"));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&archive)?;
    let mut c = Command::new("git");
    c.arg("-C")
        .arg(source)
        .args(["archive", "--format=tar", pin])
        .env("GIT_NO_LAZY_FETCH", "1");
    if !paths.is_empty() {
        c.arg("--").args(paths);
    }
    if !w
        .run_to(
            &mut c,
            &w.root.join(format!("{label}-archive.log")),
            180,
            Some(file),
        )?
        .success()
    {
        return Err("pinned archive refused; preserve partial".into());
    }
    File::open(&archive)?.sync_all()?;
    checked(
        w,
        Command::new("tar")
            .args(["-x", "--no-same-owner", "-f"])
            .arg(&archive)
            .arg("-C")
            .arg(dest),
        &format!("{label}-extract.log"),
        60,
    )
}
fn prepare(repo: &Path, w: &Workspace, label: &str) -> Result<(), Error> {
    let depot = w.root.join(format!("depot-{label}"));
    directory(&depot)?;
    checked(
        w,
        Command::new("git")
            .args(["init", "--bare", "-q"])
            .arg(&depot),
        &format!("init-{label}.log"),
        30,
    )?;
    // Reuse retained objects without fetching into or changing their repository.
    let retained = if label != "v1" && w.root.join("depot-v1").is_dir() {
        w.root.join("depot-v1")
    } else {
        repo.join(".local/src/oxos")
    };
    let objects = git(
        &retained,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "objects",
        ],
    )?;
    save(
        &depot.join("objects/info/alternates"),
        &format!("{}\n", objects.trim()),
    )?;
    let listing = git(
        &depot,
        &[
            "ls-tree",
            "-r",
            PIN,
            "--",
            "kernel",
            "tools",
            "LICENSE",
            "README.md",
        ],
    )?;
    let ids = listing
        .lines()
        .map(|line| {
            line.split_whitespace()
                .nth(2)
                .ok_or("missing source object")
        })
        .collect::<Result<Vec<_>, _>>()?;
    if ids.is_empty() || ids.len() > 256 {
        return Err("unexpected pinned source extent".into());
    }
    let input = w.root.join(format!("objects-{label}.txt"));
    save(&input, &(ids.join("\n") + "\n"))?;
    checked(
        w,
        Command::new("git")
            .arg("-C")
            .arg(&depot)
            .arg("cat-file")
            .arg("--batch-check")
            .env("GIT_NO_LAZY_FETCH", "1")
            .stdin(File::open(&input)?),
        &format!("objects-{label}.log"),
        30,
    )?;
    let check = fs::read_to_string(w.root.join(format!("objects-{label}.log")))?;
    let missing: Vec<_> = check
        .lines()
        .filter(|s| s.ends_with(" missing"))
        .filter_map(|s| s.split_whitespace().next())
        .collect();
    if !missing.is_empty() {
        // Exact missing public blobs only, all new objects on the admitted volume.
        checked(
            w,
            Command::new("git")
                .arg("-C")
                .arg(&depot)
                .args([
                    "fetch",
                    "--no-tags",
                    "--no-write-fetch-head",
                    "https://github.com/kooscode/oxos.git",
                ])
                .args(&missing),
            &format!("missing-source-{label}.log"),
            300,
        )?;
    }
    let source = w.root.join(format!("source-{label}"));
    export(
        w,
        &depot,
        PIN,
        &source,
        &format!("source-{label}"),
        &["kernel", "tools", "LICENSE", "README.md"],
    )?;
    directory(&source.join("third_party"))?;
    let cromwell =
        repo.join(".local/src/oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2/third_party/cromwell");
    export(
        w,
        &cromwell,
        CROMWELL,
        &source.join("third_party/cromwell"),
        &format!("cromwell-{label}"),
        &[],
    )?;
    save(
        &source.join("source.identity"),
        &format!(
            "source={PIN}\ncromwell={CROMWELL}\nsource_archive_sha256={}\ncromwell_archive_sha256={}\nsource_objects={}\nmissing_public_blobs_fetched={}\npaused_patches=none\n",
            hash(&w.root.join(format!("source-{label}.tar")))?,
            hash(&w.root.join(format!("cromwell-{label}.tar")))?,
            ids.len(),
            missing.len()
        ),
    )?;
    save(&source.join("source.objects"), &listing)?;
    Ok(())
}
fn verify_source(source: &Path) -> Result<(), Error> {
    for line in fs::read_to_string(source.join("source.objects"))?.lines() {
        let (metadata, path) = line.split_once('\t').ok_or("bad source manifest")?;
        let expected = metadata.split_whitespace().nth(2).ok_or("missing blob")?;
        let out = Command::new("git")
            .arg("hash-object")
            .arg(source.join(path))
            .output()?;
        if !out.status.success() || String::from_utf8(out.stdout)?.trim() != expected {
            return Err("pinned source bytes differ before build".into());
        }
    }
    Ok(())
}
fn apply_loader_patch(
    repo: &Path,
    w: &Workspace,
    label: &str,
    name: &str,
    expected: &str,
    kind: &str,
) -> Result<(), Error> {
    let source = w.root.join(format!("source-{label}"));
    verify_source(&source)?;
    let patch = repo.join(name);
    let digest = hash(&patch)?;
    if digest != expected {
        return Err("pinned public loader patch differs".into());
    }
    checked(
        w,
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .arg("apply")
            .arg(&patch),
        &format!("{kind}-{label}.log"),
        30,
    )?;
    let original = fs::read_to_string(source.join("source.objects"))?;
    save(&source.join(format!("before-{kind}.objects")), &original)?;
    let path = "kernel/src/xe/loader.cpp";
    let out = Command::new("git")
        .arg("hash-object")
        .arg(source.join(path))
        .output()?;
    if !out.status.success() {
        return Err("patched source hash failed".into());
    }
    let changed = String::from_utf8(out.stdout)?;
    let updated = original
        .lines()
        .map(|line| {
            if line.ends_with(&format!("\t{path}")) {
                format!("100644 blob {}\t{path}", changed.trim())
            } else {
                line.into()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(source.join("source.objects"), updated)?;
    File::open(source.join("source.objects"))?.sync_all()?;
    save(
        &source.join(format!("{kind}.identity")),
        &format!("{kind}_patch_sha256={digest}\n"),
    )
}
fn phases(repo: &Path, w: &Workspace, label: &str) -> Result<(), Error> {
    apply_loader_patch(
        repo,
        w,
        label,
        "patches/oxos-basic-startup-phases.patch",
        "f8614fb02be334a96fe5140230c5d057b9031611f649a5e634a1cc3b56e3e8a5",
        "phase",
    )
}
fn extracted<'a>(source: &'a str, name: &str) -> Result<&'a str, Error> {
    let start = source
        .find(&format!("static NTSTATUS {name}("))
        .ok_or("actual loader function absent")?;
    let end = source[start..]
        .find("\n}")
        .ok_or("actual loader function end absent")?;
    Ok(&source[start..start + end + 2])
}
fn read_contract(repo: &Path, w: &Workspace, label: &str) -> Result<(), Error> {
    let root = w.root.join(format!("read-contract-{label}"));
    directory(&root)?;
    for stage in ["before", "after"] {
        let source = w.root.join(if stage == "before" {
            "source-v1".into()
        } else {
            format!("source-{label}")
        });
        verify_source(&source)?;
        let code = fs::read_to_string(source.join("kernel/src/xe/loader.cpp"))?;
        let dst = root.join(stage);
        directory(&dst)?;
        let mut functions = extracted(&code, "XeReadFile")?.to_owned();
        functions.push('\n');
        if stage == "after" {
            functions.push_str(extracted(&code, "XepReadRequired")?);
            functions.push('\n');
        }
        functions.push_str(extracted(&code, "XepLoadSection")?);
        functions.push('\n');
        save(&dst.join("required-functions.inc"), &functions)?;
        checked(
            w,
            Command::new("c++")
                .args(["-std=c++20", "-O2", "-Wall", "-Wextra", "-Werror", "-I"])
                .arg(&dst)
                .arg("-c")
                .arg(repo.join("fixtures/required_reads_shim.cpp"))
                .arg("-o")
                .arg(dst.join("shim.o")),
            &format!("read-{label}-{stage}-cc.log"),
            60,
        )?;
        checked(
            w,
            Command::new("ar")
                .arg("rcs")
                .arg(dst.join("librequired_reads.a"))
                .arg(dst.join("shim.o")),
            &format!("read-{label}-{stage}-ar.log"),
            30,
        )?;
        checked(
            w,
            Command::new("rustc")
                .args(["--edition=2024", "--test"])
                .arg(repo.join("fixtures/required_reads_contract.rs"))
                .arg("-L")
                .arg(format!("native={}", dst.display()))
                .args(["-C", "debuginfo=0", "-o"])
                .arg(dst.join("contract")),
            &format!("read-{label}-{stage}-rustc.log"),
            60,
        )?;
        let log = w.root.join(format!("read-{label}-{stage}-test.log"));
        let status = w.run(
            Command::new(dst.join("contract")).args(["--test-threads=1"]),
            &log,
            30,
        )?;
        let text = fs::read_to_string(log)?;
        if stage == "before" {
            if status.code() != Some(101)
                || !text.contains("short successful section read was accepted")
                || !text.contains("2 passed; 1 failed")
            {
                return Err(
                    "actual original function did not reproduce selected short-read failure".into(),
                );
            }
        } else if !status.success() || !text.contains("3 passed; 0 failed") {
            return Err("corrected actual function failed focused contract".into());
        }
    }
    save(
        &root.join("result.identity"),
        "original_function_red=true\ncorrected_function_green=true\nsynthetic_IO_only=true\nbyte_counts_32bit=true\nhost_pointer_adapter_not_i386_ABI_test=true\nFable_cause_unattributed=true\n",
    )
}
fn build(repo: &Path, w: &Workspace, label: &str, serial: bool) -> Result<(), Error> {
    let source = w.root.join(format!("source-{label}"));
    let mut identity = fs::read_to_string(source.join("source.identity"))?;
    for kind in ["phase", "required"] {
        if source.join(format!("{kind}.identity")).exists() {
            identity.push_str(&fs::read_to_string(
                source.join(format!("{kind}.identity")),
            )?);
        }
    }
    if !identity.contains(&format!("source={PIN}\n")) || !identity.contains("paused_patches=none\n")
    {
        return Err("source identity differs".into());
    }
    verify_source(&source)?;
    if !serial
        && !fs::read_to_string(
            w.root
                .join(format!("read-contract-{label}/result.identity")),
        )?
        .contains("corrected_function_green=true\n")
    {
        return Err("serial-off validation requires independently evidenced correction".into());
    }
    let kind = if serial { "basic" } else { "release" };
    let out = w.root.join(format!("build-{kind}-{label}"));
    directory(&out)?;
    let ndk = PathBuf::from(std::env::var_os("XBX_NDK").ok_or("set retained read-only NDK")?);
    let llvm = ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let mut paths = vec![llvm.clone()];
    if let Some(path) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&path));
    }
    save(
        &out.join("build.inputs"),
        &format!(
            "{identity}image=open\nselftest=0\ntrace_imports=0\ndebug=0\nserial={}\noptimization=upstream_release_O2_g0\nsplash=none\ncompiler_jobs=1\nclang_sha256={}\nlld_link_sha256={}\nllvm_objcopy_sha256={}\nmakefile_sha256={}\n",
            u8::from(serial),
            hash(&llvm.join("clang"))?,
            hash(&llvm.join("lld-link"))?,
            hash(&llvm.join("llvm-objcopy"))?,
            hash(&source.join("kernel/Makefile"))?
        ),
    )?;
    checked(
        w,
        Command::new("make")
            .arg("-C")
            .arg(source.join("kernel"))
            .env("PATH", std::env::join_paths(paths)?)
            .args([
                "-j1",
                "bios",
                "IMAGE=open",
                "SELFTEST=0",
                "TRACE_IMPORTS=0",
                "DEBUG=0",
                "SPLASH=",
                "MCPX16=0",
                "BIOS_BASE=",
                "MCPX_ROM=",
                "STOCK_MCPX=",
            ])
            .arg(format!("SERIAL={}", u8::from(serial)))
            .arg(format!("BUILD={}", out.display()))
            .arg(format!("CC={}", llvm.join("clang").display()))
            .arg(format!(
                "CXX={} -Wno-error=cast-function-type-mismatch",
                llvm.join("clang++").display()
            ))
            .arg(format!("LD={}", llvm.join("lld-link").display()))
            .arg(format!("OBJCOPY={}", llvm.join("llvm-objcopy").display())),
        &format!("build-{kind}-{label}.log"),
        300,
    )?;
    for name in ["bios.bin", "xboxkrnl.exe", "xboxkrnl.map"] {
        File::open(out.join(name))?.sync_all()?;
    }
    let image = out.join("bios.bin");
    fs::set_permissions(&image, fs::Permissions::from_mode(0o400))?;
    save(
        &out.join("build.identity"),
        &format!(
            "bios_sha256={}\nkernel_sha256={}\nmap_sha256={}\nserial={}\n",
            hash(&image)?,
            hash(&out.join("xboxkrnl.exe"))?,
            hash(&out.join("xboxkrnl.map"))?,
            u8::from(serial)
        ),
    )?;
    File::open(&out)?.sync_all()?;
    let _ = repo;
    Ok(())
}
fn observe(
    repo: &Path,
    w: &Workspace,
    label: &str,
    fixture: bool,
    serial: bool,
) -> Result<(), Error> {
    let project = w.root.parent().ok_or("missing project")?;
    let core = project.join("ramht-v1/build-v2-retry2/qemu-system-i386");
    if hash(&core)? != CORE {
        return Err("corrected core differs".into());
    }
    let build = w.root.join(format!(
        "build-{}-{label}",
        if serial { "basic" } else { "release" }
    ));
    let flash = build.join("bios.bin");
    if !fs::read_to_string(build.join("build.identity"))?
        .contains(&format!("bios_sha256={}\n", hash(&flash)?))
    {
        return Err("logging image differs".into());
    }
    let firmware = hash(&flash)?;
    let qualified_image = if serial {
        firmware.clone()
    } else {
        hash(&w.root.join(format!("build-basic-{label}/bios.bin")))?
    };
    if !fixture
        && !fs::read_to_string(w.root.join(format!("fixture-{label}.qualified")))?
            .contains(&format!("bios_sha256={qualified_image}\n"))
    {
        return Err("this logging image needs its retained fixture qualification".into());
    }
    let media = if fixture {
        repo.join(".local/build-relaunch-title/relaunch.iso")
    } else {
        PathBuf::from(std::env::var("XBX_REFERENCE_MEDIA")?)
    };
    if fixture
        && hash(&media)? != "8efe94b6f8a26f63247f44bc42da9ed437e431a5ed03180ba73044140cb88491"
    {
        return Err("retained fixture differs".into());
    }
    let seconds = if fixture { 90 } else { 600 };
    let profile = w.root.join(format!(
        "p-{}-{label}",
        if fixture {
            "fixture"
        } else if serial {
            "fable"
        } else {
            "release-fable"
        }
    ));
    let supervisor = PathBuf::from(std::env::var("XBX_DESKTOP_SUPERVISOR")?);
    let mut cmd = if let Some(driver) = std::env::var_os("XBX_OBSERVATION_DRIVER") {
        let mut c = Command::new(driver);
        c.arg(&supervisor);
        c
    } else {
        Command::new(&supervisor)
    };
    cmd.arg("run-title")
        .arg(core)
        .arg(profile)
        .arg(flash)
        .arg(repo.join(".local/dashboard-v20260516-0955/xbox_hdd.qcow2"))
        .arg(repo.join(".local/quick-reboot-d87cfc33-run1/eeprom.bin"))
        .arg(media)
        .arg(seconds.to_string())
        .env("XBX_DESKTOP_MEMORY_MIB", "64")
        .env("XBX_DESKTOP_VIDEO_DRIVER", "x11")
        .env("XBX_DESKTOP_GRAPHICS", "host")
        .env("XBX_DESKTOP_CONTROLLER", "keyboard")
        .env("XBX_DESKTOP_FP", "native");
    if serial {
        cmd.env("XBX_DESKTOP_BASIC_SERIAL", "1");
    } else {
        cmd.env_remove("XBX_DESKTOP_BASIC_SERIAL");
    }
    if !fixture {
        cmd.env("XBX_DESKTOP_REFERENCE_INTERVAL_SECONDS", "600");
    }
    checked(
        w,
        &mut cmd,
        &format!(
            "{}-{label}.log",
            if fixture {
                "fixture"
            } else if serial {
                "fable"
            } else {
                "release-fable"
            }
        ),
        seconds + 90,
    )?;
    if fixture {
        let profile = w.root.join(format!("p-fixture-{label}"));
        let capture = fs::read_to_string(profile.join("serial-capture.txt"))?;
        if !capture.contains("truncated=false\n")
            || !capture.contains("capture_ok=true\n")
            || !fixture_completed(&fs::read_to_string(profile.join("kernel.log"))?)
        {
            return Err(
                "known fixture completion/log coverage not demonstrated; retain evidence".into(),
            );
        }
        save(
            &w.root.join(format!("fixture-{label}.qualified")),
            &format!(
                "bios_sha256={firmware}\nknown_guest_first_stage_relaunch_completion=true\nsecond_stage_execution=unqualified\ntitle_memory_reads=none\n"
            ),
        )?;
    }
    Ok(())
}

fn fixture_completed(log: &str) -> bool {
    log.contains(": cold boot")
        && log.contains(": quick boot")
        && log.contains("load \\Device\\CdRom0\\default.xbe: 0x0")
        && log.contains("load \\Device\\CdRom0\\media\\second.xbe: 0x0")
}

pub fn command(repo: &Path, action: &str, label: &str) -> Result<(), Error> {
    if !matches!(
        action,
        "open-startup-prepare"
            | "open-startup-phases"
            | "open-startup-repair"
            | "open-startup-release-fable"
            | "open-startup-build"
            | "open-startup-release"
            | "open-startup-fixture"
            | "open-startup-fable"
    ) {
        return Err("unknown basic startup action".into());
    }
    if label.is_empty()
        || label.len() > 12
        || !label
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("short simple startup label required".into());
    }
    let w = Workspace::acquire_startup(repo, action)?;
    match action {
        "open-startup-prepare" => prepare(repo, &w, label)?,
        "open-startup-phases" => {
            prepare(repo, &w, label)?;
            phases(repo, &w, label)?;
            build(repo, &w, label, true)?;
        }
        "open-startup-build" => build(repo, &w, label, true)?,
        "open-startup-release" => build(repo, &w, label, false)?,
        "open-startup-repair" => {
            prepare(repo, &w, label)?;
            phases(repo, &w, label)?;
            apply_loader_patch(
                repo,
                &w,
                label,
                "patches/oxos-required-reads.patch",
                "a111bee046f078cbe740ae58ac0e26abfd08ab13fe2bde4a74588bc95ce64d43",
                "required",
            )?;
            read_contract(repo, &w, label)?;
            build(repo, &w, label, true)?;
        }
        "open-startup-fixture" => observe(repo, &w, label, true, true)?,
        "open-startup-fable" => observe(repo, &w, label, false, true)?,
        "open-startup-release-fable" => observe(repo, &w, label, false, false)?,
        _ => return Err("unknown basic startup action".into()),
    }
    w.check()?;
    w.completed();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::fixture_completed;
    #[test]
    fn loader_return_alone_is_not_fixture_completion() {
        let loading = "test: cold boot\nload \\Device\\CdRom0\\default.xbe: 0x0\n";
        assert!(!fixture_completed(loading));
        let second =
            format!("{loading}test: quick boot\nload \\Device\\CdRom0\\media\\second.xbe: 0x0\n");
        assert!(fixture_completed(&second));
        assert!(fixture_completed(&format!(
            "{second}test: quick boot\nload \\Device\\CdRom0\\media\\missing.xbe: 0xc0000034\n"
        )));
    }
}
