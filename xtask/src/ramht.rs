//! One retained-core RAMHT boundary repair, original fixture and qualification.
use crate::ramht_workspace::Workspace;
use crate::reference::{git, hash};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

type Error = Box<dyn std::error::Error>;
const PIN: &str = "03b988c9d76553b33ed712ba00822ffdf49f0e7d";
const BASE: &str = "4e317283ddb368c1c80243804aa7fcfdc1e5228ac6d7ddec9ca84cb5cdd138e1";
const PFIFO: &str = "hw/xbox/nv2a/pfifo.c";
const REGS: &str = "hw/xbox/nv2a/nv2a_regs.h";

fn directory(path: &Path) -> Result<(), Error> {
    fs::DirBuilder::new().mode(0o700).create(path)?;
    Ok(())
}

fn save(path: &Path, text: &str) -> Result<(), Error> {
    let mut f = OpenOptions::new().write(true).create_new(true).open(path)?;
    f.write_all(text.as_bytes())?;
    f.sync_all()?;
    Ok(())
}

fn checked(w: &Workspace, cmd: &mut Command, log: &Path, seconds: u64) -> Result<(), Error> {
    if !w.run(cmd, log, seconds)?.success() {
        return Err(format!("operation failed; preserve {}", log.display()).into());
    }
    Ok(())
}

fn source(repo: &Path) -> PathBuf {
    repo.join(format!(".local/src/xemu-reference-{PIN}"))
}

fn tail_command(source: &str, remaining: u64) -> String {
    format!("tail -c {remaining} '{}'", source.replace('\'', "'\\''"))
}

fn media_copy(w: &Workspace, attempt: &str, resume: bool) -> Result<(), Error> {
    if std::env::var("XBX_REFERENCE_MEDIA_GROWTH_BYTES")?.as_str() != "8589934592" {
        return Err("media copy requires the separately approved 8 GiB envelope".into());
    }
    let _lease = std::env::var("CODEX_ANDROID_LEASE_FD")?.parse::<i32>()?;
    let source = std::env::var("XBX_REFERENCE_MEDIA_SOURCE")?;
    let expected = std::env::var("XBX_REFERENCE_MEDIA_EXPECTED_SHA256")?;
    if !source.starts_with("/sdcard/Download/")
        || expected.len() != 64
        || !expected.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("missing exact private input identity".into());
    }
    let root = w
        .root
        .parent()
        .ok_or("missing project root")?
        .join("reference-media-v1");
    let partial = root.join("selected-disc.iso.partial");
    let final_path = root.join("selected-disc.iso");
    let transfer = if resume {
        if std::env::var("XBX_REFERENCE_MEDIA_CONTINUE_SECONDS")? != "1200" {
            return Err("continuation requires its separately approved time amendment".into());
        }
        let retained = fs::symlink_metadata(&partial)?;
        let expected_partial =
            std::env::var("XBX_REFERENCE_MEDIA_PARTIAL_BYTES")?.parse::<u64>()?;
        if !retained.is_file()
            || retained.len() != expected_partial
            || retained.len() >= 7_825_162_240
            || final_path.exists()
            || root.join("verified.identity").exists()
        {
            return Err("preserve unexpected partial or completed media; no append".into());
        }
        let remaining = 7_825_162_240 - retained.len();
        // Stock tail seeks to the final bytes of the unchanged source. Append
        // preserves the retained prefix; final size/hash must verify before use.
        let output = OpenOptions::new().append(true).open(&partial)?;
        let status = w.run_to(
            Command::new("android-device")
                .args(["adb", "pixel4-lab", "--", "exec-out"])
                .arg(tail_command(&source, remaining)),
            &w.root.join(format!("media-resume-{attempt}.log")),
            1200,
            Some(output),
        );
        match status {
            Ok(status) if status.success() => Ok(()),
            Ok(_) => Err("stock continuation failed; preserve the partial and log".into()),
            Err(error) => Err(error),
        }
    } else {
        directory(&root)?; // Refuse an existing complete or partial attempt.
        checked(
            w,
            Command::new("android-device")
                .args(["adb", "pixel4-lab", "--", "pull"])
                .arg(&source)
                .arg(&partial),
            &w.root.join(format!("media-copy-{attempt}.log")),
            600,
        )
    };
    // Failed, short and invalid buffered writes are retained too. Keep the real
    // capacity/project leases through durability, before status/hash acceptance.
    if partial.try_exists()? {
        w.presence()?;
        File::open(&partial)?.sync_all()?;
        File::open(&root)?.sync_all()?;
    }
    transfer?;
    let metadata = fs::symlink_metadata(&partial)?;
    if !metadata.is_file()
        || metadata.len() != 7_825_162_240
        || hash(&partial)? != expected.to_ascii_lowercase()
    {
        return Err("transferred media identity differs; preserve partial without launch".into());
    }
    fs::set_permissions(&partial, fs::Permissions::from_mode(0o400))?;
    File::open(&partial)?.sync_all()?;
    if final_path.exists() {
        return Err("preserve existing media destination; no overwrite".into());
    }
    fs::rename(&partial, &final_path)?;
    save(
        &root.join("verified.identity"),
        &format!(
            "size=7825162240\nsha256={}\nphone_copy_unchanged=true\n",
            expected.to_ascii_lowercase()
        ),
    )?;
    File::open(&root)?.sync_all()?;
    w.check()?;
    println!(
        "Exact retained media copied to external storage, full hash verified and synced; phone copies preserved."
    );
    Ok(())
}

fn function(text: &str) -> Result<&str, Error> {
    let needle = "static uint32_t ramht_hash(NV2AState *d, uint32_t handle)\n{";
    let start = text.find(needle).ok_or("pinned RAMHT function missing")?;
    let end = text[start..]
        .find("\n}")
        .ok_or("RAMHT function end missing")?;
    Ok(&text[start..start + end + 2])
}

fn fixture(repo: &Path, w: &Workspace, attempt: &str) -> Result<(), Error> {
    let root = w.root.join(format!("fixture-{attempt}"));
    directory(&root)?;
    for stage in ["before", "after"] {
        let stage_root = root.join(stage);
        directory(&stage_root)?;
        fs::create_dir_all(stage_root.join("hw/xbox/nv2a"))?;
        for name in [PFIFO, REGS] {
            save(
                &stage_root.join(name),
                &git(&source(repo), &["show", &format!("{PIN}:{name}")])?,
            )?;
        }
        if stage == "after" {
            checked(
                w,
                Command::new("git")
                    .current_dir(&stage_root)
                    .arg("apply")
                    .arg(repo.join("patches/xemu-reference-ramht.patch")),
                &stage_root.join("patch.log"),
                30,
            )?;
        }
        let text = fs::read_to_string(stage_root.join(PFIFO))?;
        save(&stage_root.join("ramht-function.inc"), function(&text)?)?;
        checked(
            w,
            Command::new("cc")
                .args(["-std=c11", "-O2", "-Wall", "-Wextra", "-Werror", "-I"])
                .arg(&stage_root)
                .arg("-I")
                .arg(stage_root.join("hw/xbox/nv2a"))
                .arg("-c")
                .arg(repo.join("fixtures/ramht_shim.c"))
                .arg("-o")
                .arg(stage_root.join("shim.o")),
            &stage_root.join("cc.log"),
            30,
        )?;
        checked(
            w,
            Command::new("ar")
                .arg("rcs")
                .arg(stage_root.join("libxbx_ramht_fixture.a"))
                .arg(stage_root.join("shim.o")),
            &stage_root.join("ar.log"),
            30,
        )?;
        checked(
            w,
            Command::new("rustc")
                .args(["--edition=2024", "--test"])
                .arg(repo.join("fixtures/ramht_contract.rs"))
                .arg("-L")
                .arg(format!("native={}", stage_root.display()))
                .args(["-C", "debuginfo=0", "-o"])
                .arg(stage_root.join("contract")),
            &stage_root.join("rustc.log"),
            60,
        )?;
        let status = w.run(
            &mut Command::new(stage_root.join("contract")),
            &stage_root.join("test.log"),
            30,
        )?;
        let log = fs::read_to_string(stage_root.join("test.log"))?;
        if stage == "before" {
            if status.code() != Some(101)
                || !log.contains("synthetic handle escaped the configured table: 512")
            {
                return Err(
                    "original function did not reproduce the selected geometry failure".into(),
                );
            }
        } else if !status.success() {
            return Err("corrected function failed its original contract fixtures".into());
        }
    }
    save(
        &root.join("result.identity"),
        &format!(
            "source={PIN}\npatch_sha256={}\noriginal_function_red=true\ncorrected_function_green=true\nfirmware_or_game_input=false\nassertions_removed=false\n",
            hash(&repo.join("patches/xemu-reference-ramht.patch"))?
        ),
    )?;
    println!(
        "RAMHT: original function reproduced the bound failure; corrected function passed all original fixtures."
    );
    Ok(())
}

fn export(
    w: &Workspace,
    checkout: &Path,
    pin: &str,
    destination: &Path,
    log: &Path,
) -> Result<(), Error> {
    let mut producer = Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(["archive", pin])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut consumer = Command::new("tar");
    consumer
        .args(["-x", "--no-same-owner", "-C"])
        .arg(destination)
        .stdin(Stdio::from(
            producer
                .stdout
                .take()
                .ok_or("source archive pipe missing")?,
        ));
    let result = checked(w, &mut consumer, log, 180);
    drop(consumer);
    let status = producer.wait()?;
    result?;
    if !status.success() {
        return Err("pinned source export failed".into());
    }
    Ok(())
}

fn identity(repo: &Path) -> Result<String, Error> {
    let dsp = repo
        .join(".local/build-desktop/dsp/x86_64-unknown-linux-gnu/release/libdsp56300_emu_ffi.a");
    if hash(&repo.join("patches/xemu-reference-desktop.patch"))? != BASE
        || hash(&dsp)? != "5e78afecce79231338bd6c1838a55d300886de3be4e38f3f82229013ac23f1a7"
    {
        return Err("retained base patch or DSP library differs".into());
    }
    Ok(format!(
        "source={PIN}\nbase_patch_sha256={BASE}\nramht_patch_sha256={}\ndsp_library_sha256={}\ncompiler_jobs=1\ndownloads=disabled\n",
        hash(&repo.join("patches/xemu-reference-ramht.patch"))?,
        hash(&dsp)?
    ))
}

fn configure_environment(repo: &Path, cmd: &mut Command) -> Result<(), Error> {
    let python = repo.join(".local/tools/bin/python3");
    let path = Command::new(&python)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .args([
            "-c",
            "import sysconfig;print(sysconfig.get_paths()['purelib'])",
        ])
        .output()?;
    if !path.status.success() {
        return Err("retained Python tool unavailable".into());
    }
    cmd.env("PYTHONPATH", String::from_utf8(path.stdout)?.trim())
        .env(
            "PKG_CONFIG_PATH",
            format!(
                "{}:{}",
                repo.join(".local/host-sysroot/usr/lib64/pkgconfig")
                    .display(),
                repo.join(".local/host-sysroot/usr/share/pkgconfig")
                    .display()
            ),
        )
        .env(
            "XBX_DSP_SOURCE",
            repo.join(".local/build-android/_deps/dsp56300_src-src"),
        )
        .env(
            "XBX_DSP_LIBRARY",
            repo.join(
                ".local/build-desktop/dsp/x86_64-unknown-linux-gnu/release/libdsp56300_emu_ffi.a",
            ),
        );
    Ok(())
}

fn wrap_overlays(repo: &Path, src: &Path) -> Result<(), Error> {
    let mut receipt = format!("source={PIN}\n");
    for name in ["berkeley-softfloat-3", "berkeley-testfloat-3"] {
        for file in ["meson.build", "meson_options.txt"] {
            let public_path = format!("subprojects/packagefiles/{name}/{file}");
            let bytes = git(&source(repo), &["show", &format!("{PIN}:{public_path}")])?;
            if fs::read(src.join(&public_path))? != bytes.as_bytes() {
                return Err("pinned Meson wrapper source differs".into());
            }
            let destination = src.join("subprojects").join(name).join(file);
            if destination.exists() {
                if fs::read(&destination)? != bytes.as_bytes() {
                    return Err("preserve differing dependency wrapper; no overwrite".into());
                }
            } else {
                save(&destination, &bytes)?;
            }
            receipt.push_str(&format!(
                "subprojects/{name}/{file}={}\n",
                hash(&destination)?
            ));
        }
    }
    let path = src.join("xbx-wrap-overlays.identity");
    if path.exists() {
        if fs::read_to_string(path)? != receipt {
            return Err("wrapper receipt differs".into());
        }
    } else {
        save(&path, &receipt)?;
    }
    Ok(())
}

fn prepared_identity(repo: &Path, src: &Path) -> Result<String, Error> {
    Ok(format!(
        "{}pfifo_sha256={}\nwrap_overlays_sha256={}\n",
        identity(repo)?,
        hash(&src.join(PFIFO))?,
        hash(&src.join("xbx-wrap-overlays.identity"))?
    ))
}

fn configure(repo: &Path, w: &Workspace, attempt: &str) -> Result<(), Error> {
    let inputs = identity(repo)?;
    let fixture = fs::read_to_string(w.root.join(format!("fixture-{attempt}/result.identity")))?;
    if !fixture.contains(&format!(
        "patch_sha256={}",
        hash(&repo.join("patches/xemu-reference-ramht.patch"))?
    )) {
        return Err("fixture binding differs".into());
    }
    let src = w.root.join(format!("source-{attempt}"));
    directory(&src)?;
    let build = w.root.join(format!("build-{attempt}"));
    directory(&build)?;
    save(&build.join("build.inputs"), &inputs)?;
    export(
        w,
        &source(repo),
        PIN,
        &src,
        &build.join("source-export.log"),
    )?;
    for (i, patch) in ["xemu-reference-desktop.patch", "xemu-reference-ramht.patch"]
        .iter()
        .enumerate()
    {
        checked(
            w,
            Command::new("git")
                .current_dir(&src)
                .arg("apply")
                .arg(repo.join("patches").join(patch)),
            &build.join(format!("patch-{i}.log")),
            30,
        )?;
    }
    let original = repo.join(format!(
        ".local/build-reference-{PIN}-{BASE}-v3/dependencies.identity"
    ));
    let dependencies = fs::read_to_string(original)?;
    for line in dependencies.lines() {
        let (name, pin) = line
            .split_once('=')
            .ok_or("invalid retained dependency binding")?;
        if !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err("unsafe dependency name".into());
        }
        let retained = source(repo).join("subprojects").join(name);
        if git(&retained, &["rev-parse", "HEAD"])?.trim() != pin
            || !git(&retained, &["diff", "HEAD"])?.is_empty()
        {
            return Err("retained dependency differs".into());
        }
        let destination = src.join("subprojects").join(name);
        directory(&destination)?;
        export(
            w,
            &retained,
            pin,
            &destination,
            &build.join(format!("dependency-{name}.log")),
        )?;
    }
    // Meson may expand only these exact already-retained wrap archives locally.
    let cache = src.join("subprojects/packagecache");
    directory(&cache)?;
    for (name, expected) in [
        (
            "xxHash-0.8.3.tar.gz",
            "aae608dfe8213dfd05d909a57718ef82f30722c392344583d3f39050c7f29a80",
        ),
        (
            "xxhash_0.8.3-2_patch.zip",
            "c7f78fc2d08ec21ff1bae928d7bdcddb42713a07d9d973a885c59ea7f8cf6bc8",
        ),
    ] {
        let retained = source(repo).join("subprojects/packagecache").join(name);
        if hash(&retained)? != expected {
            return Err("retained wrap archive differs; no download fallback".into());
        }
        fs::copy(retained, cache.join(name))?;
    }
    if hash(&src.join(PFIFO))? != hash(&w.root.join(format!("fixture-{attempt}/after/{PFIFO}")))? {
        return Err("configured PFIFO source differs from the tested candidate".into());
    }
    save(&build.join("dependencies.identity"), &dependencies)?;
    configure_prepared(repo, w, attempt, &src, &build, &inputs)
}

fn configure_prepared(
    repo: &Path,
    w: &Workspace,
    source_label: &str,
    src: &Path,
    build: &Path,
    _inputs: &str,
) -> Result<(), Error> {
    wrap_overlays(repo, src)?;
    let inputs = prepared_identity(repo, src)?;
    save(&build.join("source.label"), source_label)?;
    let mut cmd = Command::new(src.join("configure"));
    cmd.current_dir(build)
        .arg(format!(
            "--python={}",
            repo.join(".local/tools/bin/python3").display()
        ))
        .args([
            "--target-list=i386-softmmu",
            "--extra-cflags=-DXBOX=1",
            "--disable-werror",
            "--disable-docs",
            "--disable-tools",
            "--disable-guest-agent",
            "--disable-rust",
            "--disable-download",
        ]);
    configure_environment(repo, &mut cmd)?;
    checked(w, &mut cmd, &build.join("configure.log"), 900)?;
    save(&build.join("configured.identity"), &inputs)?;
    println!(
        "Configured separate RAMHT candidate, one compiler, retained dependencies, downloads disabled."
    );
    Ok(())
}

pub fn command(repo: &Path, action: &str, attempt: &str) -> Result<(), Error> {
    if attempt.is_empty()
        || attempt.len() > 16
        || !attempt
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("RAMHT attempt must be a short simple label".into());
    }
    let w = Workspace::acquire(repo, action)?;
    let result = match action {
        "ramht-media-copy" | "ramht-media-resume" => {
            media_copy(&w, attempt, action == "ramht-media-resume")
        }
        "ramht-check" => fixture(repo, &w, attempt),
        "ramht-configure" => configure(repo, &w, attempt),
        "ramht-reconfigure" => {
            let (source_label, build_label) = if let Some((base, _)) = attempt.split_once("-retry")
            {
                (base, attempt.to_owned())
            } else {
                (attempt, format!("{attempt}-retry1"))
            };
            if source_label.is_empty() {
                return Err("missing retained source label".into());
            }
            let old = w.root.join(format!("build-{source_label}"));
            let src = w.root.join(format!("source-{source_label}"));
            let inputs = identity(repo)?;
            if fs::read_to_string(old.join("build.inputs"))? != inputs
                || old.join("configured.identity").exists()
                || hash(&src.join(PFIFO))?
                    != hash(&w.root.join(format!("fixture-{source_label}/after/{PFIFO}")))?
            {
                return Err("retained incomplete configuration/source binding differs".into());
            }
            let build = w.root.join(format!("build-{build_label}"));
            directory(&build)?;
            save(&build.join("build.inputs"), &inputs)?;
            save(
                &build.join("dependencies.identity"),
                &fs::read_to_string(old.join("dependencies.identity"))?,
            )?;
            configure_prepared(repo, &w, source_label, &src, &build, &inputs)
        }
        "ramht-build" => {
            let build = w.root.join(format!("build-{attempt}"));
            let label = fs::read_to_string(build.join("source.label"))?;
            if !label
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                || hash(&w.root.join(format!("source-{label}/{PFIFO}")))?
                    != hash(&w.root.join(format!("fixture-{label}/after/{PFIFO}")))?
            {
                return Err("native PFIFO source differs from the tested candidate".into());
            }
            let inputs = prepared_identity(repo, &w.root.join(format!("source-{label}")))?;
            if fs::read_to_string(build.join("configured.identity"))? != inputs
                || build.join("build.identity").exists()
            {
                return Err("candidate differs or is already complete".into());
            }
            let mut cmd = Command::new("ninja");
            cmd.arg("-C").arg(&build).args(["-j1", "qemu-system-i386"]);
            configure_environment(repo, &mut cmd)?;
            checked(&w, &mut cmd, &build.join("native-build.log"), 3600)?;
            save(
                &build.join("build.identity"),
                &format!(
                    "{inputs}core_sha256={}\n",
                    hash(&build.join("qemu-system-i386"))?
                ),
            )?;
            println!(
                "Built separate RAMHT compatibility candidate; native behavior remains unqualified."
            );
            Ok(())
        }
        "ramht-observe" | "ramht-title" => {
            let build = w.root.join(format!("build-{attempt}"));
            let core = build.join("qemu-system-i386");
            if !fs::read_to_string(build.join("build.identity"))?
                .contains(&format!("core_sha256={}", hash(&core)?))
            {
                return Err("candidate executable differs".into());
            }
            let supervisor = PathBuf::from(std::env::var("XBX_DESKTOP_SUPERVISOR")?);
            let profile = w.root.parent().ok_or("missing project root")?.join(format!(
                "rh-{}64-{attempt}",
                if action == "ramht-title" {
                    "title"
                } else {
                    "dash"
                }
            ));
            let private = w.root.parent().unwrap().join("reference-input-v1");
            let mut cmd = if let Some(driver) = std::env::var_os("XBX_OBSERVATION_DRIVER") {
                let mut command = Command::new(driver);
                command.arg(&supervisor);
                command
            } else {
                Command::new(&supervisor)
            };
            cmd.arg(if action == "ramht-title" {
                "run-title"
            } else {
                "run-dashboard"
            })
            .arg(&core)
            .arg(&profile)
            .arg(private.join("flash.bin"))
            .arg(repo.join(".local/dashboard-v20260516-0955/xbox_hdd.qcow2"))
            .arg(repo.join(".local/quick-reboot-d87cfc33-run1/eeprom.bin"));
            if action == "ramht-title" {
                cmd.arg(PathBuf::from(std::env::var("XBX_REFERENCE_MEDIA")?));
            }
            cmd.arg("300")
                .arg(private.join("mcpx.bin"))
                .env("XBX_DESKTOP_MEMORY_MIB", "64")
                .env("XBX_DESKTOP_VIDEO_DRIVER", "x11")
                .env("XBX_DESKTOP_CONTROLLER", "keyboard")
                .env("XBX_DESKTOP_FP", "native");
            checked(
                &w,
                &mut cmd,
                &w.root.join(format!(
                    "{}-{attempt}.log",
                    if action == "ramht-title" {
                        "title"
                    } else {
                        "observe"
                    }
                )),
                390,
            )
        }
        _ => Err("unknown RAMHT action".into()),
    };
    if result.is_ok() {
        w.completed();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_tail_continuation_preserves_prefix_and_quotes_source() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("xbx-tail-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let source = root.join("image (test)'s.iso");
        let partial = root.join("partial");
        let bytes: Vec<u8> = (0..=255).cycle().take(1024).collect();
        fs::write(&source, &bytes).unwrap();
        fs::write(&partial, &bytes[..655]).unwrap();
        let output = OpenOptions::new().append(true).open(&partial).unwrap();
        let status = Command::new("sh")
            .arg("-c")
            .arg(tail_command(source.to_str().unwrap(), 369))
            .stdout(Stdio::from(output))
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(fs::read(&partial).unwrap(), bytes);
        fs::remove_file(source).unwrap();
        fs::remove_file(partial).unwrap();
        fs::remove_dir(root).unwrap();
    }
}
