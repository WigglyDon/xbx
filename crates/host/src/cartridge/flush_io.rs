// SPDX-License-Identifier: MIT
//! Isolated build of the original-regression-backed retained IDE flush repair.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
fn copy(source: &Path, dest: &Path) -> Result<(), Error> {
    fs::create_dir_all(dest.parent().ok_or("source parent")?)?;
    c::write_new(dest, &fs::read(source)?)?;
    Ok(())
}
pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("flush-io-v1");
    c::mkdir(&dir)?;
    let lock: Value = c::read_json(&repo.join("cartridge-flush-io.lock.json"))?;
    let negative: Value = c::read_json(&root.join("file-cache-v2/original.result.json"))?;
    if negative["guest_passed"] != false
        || negative["data_verified"] != true
        || negative["native_clean"] != true
        || negative["seeds_unchanged"] != true
        || negative["fixture"] != lock["original_negative_fixture"]
        || negative["open_system"] != resources::NPX_LOGGING_OPEN
        || !fs::read_to_string(root.join("file-cache-v2/serial.log"))?
            .contains("FLUSH STATUS=C0000185")
    {
        return Err("exact clean original negative required".into());
    }
    let old = root.join("npx-owner-port-v1/source");
    let manifest = old.join("source.objects");
    let patch = repo.join("patches/oxos-queued-flush.patch");
    if c::file_digest(&manifest)? != lock["retained_corrected_source_objects_sha256"]
        || c::file_digest(&patch)? != lock["patch_sha256"]
    {
        return Err("retained source or functional patch differs".into());
    }
    let source = dir.join("source");
    c::mkdir(&source)?;
    let objects = fs::read_to_string(&manifest)?;
    for line in objects.lines() {
        let (metadata, name) = line.split_once('\t').ok_or("source manifest")?;
        if name.starts_with('/') || name.split('/').any(|p| p == "..") {
            return Err("source path".into());
        }
        let input = old.join(name);
        if name == "kernel/src/ke/npx.cpp" {
            if c::file_digest(&input)? != lock["accepted_npx_after"] {
                return Err("accepted NPX repair differs".into());
            }
        } else {
            let got = Command::new("git")
                .arg("hash-object")
                .arg(&input)
                .output()?;
            if !got.status.success()
                || String::from_utf8(got.stdout)?.trim()
                    != metadata.split_whitespace().nth(2).ok_or("source blob")?
            {
                return Err("retained public source bytes differ".into());
            }
        }
        copy(&input, &source.join(name))?;
    }
    copy(&manifest, &source.join("source.objects"))?;
    let archive = root
        .parent()
        .ok_or("project")?
        .join("open-startup-v1/cromwell-v3.tar");
    if c::file_digest(&archive)?
        != "280b2b1dce7d73ba40d9e4d027fb8f54624fa7db0724e2f431d813f85ef29dcb"
    {
        return Err("retained public boot archive differs".into());
    }
    fs::create_dir_all(source.join("third_party/cromwell"))?;
    c::store::checked(
        Command::new("tar")
            .args(["-x", "--no-same-owner", "-f"])
            .arg(&archive)
            .arg("-C")
            .arg(source.join("third_party/cromwell")),
    )?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .arg("apply")
            .arg(&patch),
    )?;
    let mut changed = Vec::new();
    for line in objects.lines() {
        let (_, name) = line.split_once('\t').ok_or("source manifest")?;
        if fs::read(old.join(name))? != fs::read(source.join(name))? {
            if c::file_digest(&old.join(name))? != lock["changed"][name]["before"]
                || c::file_digest(&source.join(name))? != lock["changed"][name]["after"]
            {
                return Err("functional source extent differs".into());
            }
            changed.push(name);
        }
    }
    if changed != ["kernel/src/idex/channel.cpp", "kernel/src/idex/disk.cpp"] {
        return Err("unrelated source change".into());
    }
    let out = dir.join("build-logging");
    c::mkdir(&out)?;
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let mut paths = vec![llvm.clone()];
    if let Some(path) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&path));
    }
    c::store::checked(
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
                "SERIAL=1",
            ])
            .arg(format!("BUILD={}", out.display()))
            .arg(format!("CC={}", llvm.join("clang").display()))
            .arg(format!(
                "CXX={} -Wno-error=cast-function-type-mismatch",
                llvm.join("clang++").display()
            ))
            .arg(format!("LD={}", llvm.join("lld-link").display()))
            .arg(format!("OBJCOPY={}", llvm.join("llvm-objcopy").display())),
    )?;
    for name in ["bios.bin", "xboxkrnl.exe", "xboxkrnl.map"] {
        fs::File::open(out.join(name))?.sync_all()?;
    }
    c::write_new(
        &dir.join("build.identity.json"),
        &c::json(&json!({
            "source":lock["commit"],"cromwell":lock["cromwell_commit"],"functional_patch":lock["patch_sha256"],"changed":changed,
            "firmware":c::file_digest(&out.join("bios.bin"))?,"kernel":c::file_digest(&out.join("xboxkrnl.exe"))?,"map":c::file_digest(&out.join("xboxkrnl.map"))?,
            "retained_core":resources::X87_RESTART_CORE,"compiler":c::file_digest(&llvm.join("clang"))?,"linker":c::file_digest(&llvm.join("lld-link"))?,
            "compiler_jobs":1,"serial":true,"trace_imports":false,"original_negative":negative["fixture"],"native_regression_passed":false,"Fable_cause_established":false
        }))?,
    )?;
    Ok(())
}

/// Integrate the candidate with the retained core and tools without copying or
/// rebuilding them. The supported product resource set remains separately pinned.
pub(super) fn package(root: &Path) -> Result<resources::Resources, Error> {
    let base = resources::Resources::x87_logging(&root.join("package-x87-npx-v1"))?;
    let firmware = root.join("flush-io-v1/build-logging/bios.bin");
    if c::file_digest(&firmware)? != resources::FLUSH_LOGGING_OPEN {
        return Err("queued-flush build identity changed".into());
    }
    let dir = root.join("package-flush-v1");
    if !dir.exists() {
        c::mkdir(&dir)?;
        fs::create_dir_all(dir.join("native/subprojects/slirp"))?;
        for name in [
            "native/core",
            "native/qemu-img",
            "native/subprojects/slirp/libslirp.so.0",
        ] {
            fs::hard_link(base.root.join(name), dir.join(name))?;
        }
        fs::hard_link(&firmware, dir.join("open.bin"))?;
        let mut manifest = base.manifest.clone();
        manifest.open_system = resources::FLUSH_LOGGING_OPEN.into();
        c::write_new(&dir.join("runtime.json"), &c::json(&manifest)?)?;
    }
    Ok(resources::Resources::flush_logging(&dir)?)
}

/// Admission for one changed-runtime commercial observation requires both exact
/// retained originals and actual clean returns, not merely a successful build.
pub(super) fn qualified(root: &Path) -> Result<(), Error> {
    for (name, fixture) in [
        (
            "cache-regression",
            "3e5f6805c69007b34ac8c654ace8b62648fa511f2548ab559a238d8dcee64bd0",
        ),
        (
            "disc-regression",
            "6ad5f716fd4ae18e3d59db19a676b9f1338e0942e7ea8f00a99b50a25383a15d",
        ),
    ] {
        let r: Value = c::read_json(
            &root
                .join("flush-io-v1")
                .join(name)
                .join("original.result.json"),
        )?;
        if r["guest_passed"] != true
            || r["native_clean"] != true
            || r["seeds_unchanged"] != true
            || r["fixture"] != fixture
            || r["core"] != resources::X87_RESTART_CORE
            || r["open_system"] != resources::FLUSH_LOGGING_OPEN
        {
            return Err("exact original queued-flush regressions required".into());
        }
    }
    Ok(())
}
