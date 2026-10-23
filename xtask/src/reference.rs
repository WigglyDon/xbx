//! One separately bound upstream xemu desktop comparison; never selects Android assets.
use crate::{kernel, run};
use std::fs;
use std::path::Path;
use std::process::Command;

const PIN: &str = "03b988c9d76553b33ed712ba00822ffdf49f0e7d";
fn hash(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let out = Command::new("sha256sum").arg(path).output()?;
    if !out.status.success() {
        return Err("reference identity hash failed".into());
    }
    Ok(String::from_utf8(out.stdout)?
        .split_whitespace()
        .next()
        .ok_or("missing digest")?
        .to_owned())
}
fn git(path: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()?;
    if !out.status.success() {
        return Err("reference git identity failed".into());
    }
    Ok(String::from_utf8(out.stdout)?)
}

pub fn command(root: &Path, action: &str, attempt: &str) -> Result<(), Box<dyn std::error::Error>> {
    if attempt.is_empty()
        || attempt.len() > 16
        || !attempt
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("reference attempt must be a short simple label".into());
    }
    let _lease = crate::capacity::writer(root, 8)?;
    let source = root.join(format!(".local/src/xemu-reference-{PIN}"));
    let patch = fs::read(root.join("patches/xemu-reference-desktop.patch"))?;
    kernel::verify(&source, PIN, Some(&patch))?;
    let digest = hash(&root.join("patches/xemu-reference-desktop.patch"))?;
    let lock = fs::read_to_string(root.join("reference-core.lock.json"))?;
    if !lock.contains(&format!("\"commit\": \"{PIN}\""))
        || !lock.contains(&format!("\"patch_sha256\": \"{digest}\""))
    {
        return Err("reference source lock differs".into());
    }
    let dsp = root.join(".local/build-android/_deps/dsp56300_src-src");
    if git(&dsp, &["rev-parse", "HEAD"])?.trim() != "bde7c233a447d32d558c48cb4438e1425fe0bbcc"
        || !git(&dsp, &["diff", "HEAD"])?.is_empty()
    {
        return Err("preserve and inspect changed DSP source".into());
    }
    let dsp_library = root
        .join(".local/build-desktop/dsp/x86_64-unknown-linux-gnu/release/libdsp56300_emu_ffi.a");
    let inputs = format!(
        "source={PIN}\npatch_sha256={digest}\ndsp_source=bde7c233a447d32d558c48cb4438e1425fe0bbcc\ndsp_library_sha256={}\ncompiler_jobs=1\nram_mib=64\nandroid_selection=unchanged\n",
        hash(&dsp_library)?
    );
    let build = root.join(format!(".local/build-reference-{PIN}-{digest}-{attempt}"));
    if action == "reference-configure" {
        fs::create_dir(&build)?;
        fs::write(build.join("build.inputs"), &inputs)?;
        // Reuse only exact, clean public Git dependencies already retained in
        // xbx. Other required sources are fetched at upstream's pinned wraps.
        for entry in fs::read_dir(source.join("subprojects"))? {
            let path = entry?.path();
            if path.extension().is_none_or(|e| e != "wrap") {
                continue;
            }
            let spec = fs::read_to_string(&path)?;
            if !spec.starts_with("[wrap-git]") {
                continue;
            }
            let revision = spec
                .lines()
                .filter_map(|line| line.split_once('='))
                .find_map(|(key, value)| (key.trim() == "revision").then_some(value.trim()));
            let Some(revision) = revision else { continue };
            let name = path.file_stem().ok_or("invalid wrap filename")?;
            let retained = root.join(".local/src/xemu-android/subprojects").join(name);
            let destination = source.join("subprojects").join(name);
            if !destination.exists()
                && retained.exists()
                && git(&retained, &["rev-parse", "HEAD"]).is_ok_and(|head| head.trim() == revision)
                && git(&retained, &["diff", "HEAD"]).is_ok_and(|diff| diff.is_empty())
            {
                std::os::unix::fs::symlink(retained.canonicalize()?, destination)?;
            }
        }
        let pc = root.join(".local/host-sysroot/usr/lib64/pkgconfig");
        let pc_share = root.join(".local/host-sysroot/usr/share/pkgconfig");
        run(Command::new(source.join("configure"))
            .current_dir(&build)
            .env(
                "PKG_CONFIG_PATH",
                format!("{}:{}", pc.display(), pc_share.display()),
            )
            .env("PIP_CACHE_DIR", root.join(".local/pip-cache"))
            .env("XBX_DSP_SOURCE", &dsp)
            .env("XBX_DSP_LIBRARY", &dsp_library)
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
            ]))?;
        let mut dependencies = String::new();
        for entry in fs::read_dir(source.join("subprojects"))? {
            let path = entry?.path();
            if path.is_dir()
                && path.join(".git").exists()
                && let Ok(head) = git(&path, &["rev-parse", "HEAD"])
            {
                dependencies.push_str(&format!(
                    "{}={}\n",
                    path.file_name().unwrap().to_string_lossy(),
                    head.trim()
                ));
            }
        }
        fs::write(build.join("dependencies.identity"), dependencies)?;
        println!(
            "Configured separate reference candidate: {}",
            build.display()
        );
        return Ok(());
    }
    if action != "reference-build"
        || fs::read_to_string(build.join("build.inputs"))? != inputs
        || build.join("build.identity").exists()
    {
        return Err(
            "reference build identity differs or completed output must be preserved".into(),
        );
    }
    let python = Command::new(root.join(".local/tools/bin/python3"))
        .args([
            "-c",
            "import sysconfig;print(sysconfig.get_paths()['purelib'])",
        ])
        .output()?;
    if !python.status.success() {
        return Err("reference Python tool path unavailable".into());
    }
    run(Command::new("ninja")
        .env("PYTHONPATH", String::from_utf8(python.stdout)?.trim())
        .arg("-C")
        .arg(&build)
        .args(["-j1", "qemu-system-i386"]))?;
    fs::write(
        build.join("build.identity"),
        format!(
            "{inputs}core_sha256={}\ndependencies_sha256={}\n",
            hash(&build.join("qemu-system-i386"))?,
            hash(&build.join("dependencies.identity"))?
        ),
    )?;
    println!("Built separate reference candidate: {}", build.display());
    Ok(())
}
