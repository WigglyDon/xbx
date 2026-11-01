// SPDX-License-Identifier: MIT
//! Narrow public reboot/launch disposition on the retained Android candidate.
//! No caller address, launch path or launch payload is collected.
use super::{self as c, buffer_probe, resources};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;

pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let retained = root
        .parent()
        .ok_or("workspace")?
        .join("rebuilt-io-images-v3");
    if c::file_digest(&retained.join("reporting/bios.bin"))? != resources::LOCAL_IO_OPEN {
        return Err("retained qualified reporting image differs".into());
    }
    let dir = root.join("android-launch-v2");
    c::mkdir(&dir)?;
    let source = dir.join("source");
    let out = dir.join("build");
    for (from, to) in [
        (retained.join("source"), &source),
        (retained.join("reporting"), &out),
    ] {
        c::store::checked(
            Command::new("cp")
                .args(["-a", "--reflink=always"])
                .arg(from)
                .arg(to),
        )?;
    }
    let patch = repo.join("patches/oxos-launch-disposition.patch");
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
        c::read_json(&root.join("android-serial-v1/build.identity.json"))?;
    c::write_new(
        &dir.join("identity.json"),
        &c::json(&json!({
            "open_system":c::file_digest(&out.join("bios.bin"))?,
            "parent_open_system":resources::LOCAL_IO_OPEN,
            "kernel":c::file_digest(&out.join("xboxkrnl.exe"))?,
            "patch":c::file_digest(&patch)?,
            "source_files":{
                "kernel/src/hal/smbus.cpp":c::file_digest(&source.join("kernel/src/hal/smbus.cpp"))?,
                "kernel/src/xe/loader.cpp":c::file_digest(&source.join("kernel/src/xe/loader.cpp"))?
            },
            "compiler":c::file_digest(&llvm.join("clang++"))?,
            "native_core":native["library_sha256"],
            "source_date_epoch":"1792632338","jobs":1,
            "public_launch_header_fields_only":true,"private_game_code_inspection":false,
            "prior_sources_and_objects_retained":true,"ordinary_default_changed":false,
            "original_or_Fable_qualified":false
        }))?,
    )?;
    fs::File::open(dir.join("identity.json"))?.sync_all()?;
    Ok(())
}
