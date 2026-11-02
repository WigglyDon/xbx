// SPDX-License-Identifier: MIT
//! Retained Android developer images; no ordinary firmware selector.
//! Public handoff fields and a separately bound private minimal fault record.
use super::{self as c, buffer_probe, resources};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
#[derive(Clone, Copy)]
enum Image {
    Launch,
    Handoff,
    Fault,
}

pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    build_selected(repo, root, Image::Launch)
}
pub(super) fn handoff(repo: &Path, root: &Path) -> Result<(), Error> {
    build_selected(repo, root, Image::Handoff)
}
pub(super) fn fault(repo: &Path, root: &Path) -> Result<(), Error> {
    build_selected(repo, root, Image::Fault)
}
fn build_selected(repo: &Path, root: &Path, image: Image) -> Result<(), Error> {
    let retained = root
        .parent()
        .ok_or("workspace")?
        .join("rebuilt-io-images-v3");
    if c::file_digest(&retained.join("reporting/bios.bin"))? != resources::LOCAL_IO_OPEN {
        return Err("retained qualified reporting image differs".into());
    }
    let (parent_source, parent_build, stage, patch_name, native_stage, units): (
        _,
        _,
        _,
        _,
        _,
        &[&str],
    ) = match image {
        Image::Launch => (
            retained.join("source"),
            retained.join("reporting"),
            "android-launch-v2",
            "patches/oxos-launch-disposition.patch",
            "android-serial-v1",
            &["kernel/src/hal/smbus.cpp", "kernel/src/xe/loader.cpp"],
        ),
        Image::Handoff => (
            root.join("android-launch-v2/source"),
            root.join("android-launch-v2/build"),
            "android-handoff-v1",
            "patches/oxos-startup-handoff.patch",
            "android-tcg-baseline-v1",
            &["kernel/src/ps/thread.cpp", "kernel/src/ke/except.cpp"],
        ),
        Image::Fault => (
            root.join("android-handoff-v1/source"),
            root.join("android-handoff-v1/build"),
            "android-fault-v1",
            "patches/oxos-android-fault-record.patch",
            "android-tcg-baseline-v1",
            &["kernel/src/ke/except.cpp"],
        ),
    };
    let parent_image = c::file_digest(&parent_build.join("bios.bin"))?;
    let dir = root.join(stage);
    c::mkdir(&dir)?;
    let source = dir.join("source");
    let out = dir.join("build");
    for (from, to) in [(parent_source, &source), (parent_build, &out)] {
        c::store::checked(
            Command::new("cp")
                .args(["-a", "--reflink=always"])
                .arg(from)
                .arg(to),
        )?;
    }
    let patch = repo.join(patch_name);
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &dir)
            .arg("apply")
            .arg(&patch),
    )?;
    // Retain unchanged objects; only the changed units and link outputs rebuild.
    let llvm = buffer_probe::compile_image(&source, &out, Some("1792632338"))?;
    let native: serde_json::Value =
        c::read_json(&root.join(native_stage).join("build.identity.json"))?;
    let source_files: serde_json::Map<String, serde_json::Value> = units
        .iter()
        .copied()
        .map(|name| Ok((name.to_owned(), json!(c::file_digest(&source.join(name))?))))
        .collect::<Result<_, Error>>()?;
    c::write_new(
        &dir.join("identity.json"),
        &c::json(&json!({
            "open_system":c::file_digest(&out.join("bios.bin"))?,
            "parent_open_system":parent_image,
            "kernel":c::file_digest(&out.join("xboxkrnl.exe"))?,
            "patch":c::file_digest(&patch)?,
            "source_files":source_files,
            "compiler":c::file_digest(&llvm.join("clang++"))?,
            "native_core":native["library_sha256"],
            "source_date_epoch":"1792632338","jobs":1,
            "public_launch_header_fields_only":matches!(image, Image::Launch),
            "bounded_public_thread_and_exception_fields":!matches!(image, Image::Launch),
            "private_single_exception_context":matches!(image, Image::Fault),
            "private_game_code_inspection":false,
            "prior_sources_and_objects_retained":true,"ordinary_default_changed":false,
            "original_or_Fable_qualified":false
        }))?,
    )?;
    fs::File::open(dir.join("identity.json"))?.sync_all()?;
    Ok(())
}
