// SPDX-License-Identifier: MIT
//! Narrow public reboot/launch disposition on the retained Android candidate.
//! No caller address, launch path or launch payload is collected.
use super::{self as c, buffer_probe, resources};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;

pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    build_selected(repo, root, false)
}
pub(super) fn handoff(repo: &Path, root: &Path) -> Result<(), Error> {
    build_selected(repo, root, true)
}
fn build_selected(repo: &Path, root: &Path, handoff: bool) -> Result<(), Error> {
    let retained = root
        .parent()
        .ok_or("workspace")?
        .join("rebuilt-io-images-v3");
    if c::file_digest(&retained.join("reporting/bios.bin"))? != resources::LOCAL_IO_OPEN {
        return Err("retained qualified reporting image differs".into());
    }
    let prior = root.join("android-launch-v2");
    let parent_image = if handoff {
        c::file_digest(&prior.join("build/bios.bin"))?
    } else {
        resources::LOCAL_IO_OPEN.to_owned()
    };
    let dir = root.join(if handoff {
        "android-handoff-v1"
    } else {
        "android-launch-v2"
    });
    c::mkdir(&dir)?;
    let source = dir.join("source");
    let out = dir.join("build");
    for (from, to) in [
        (
            if handoff {
                prior.join("source")
            } else {
                retained.join("source")
            },
            &source,
        ),
        (
            if handoff {
                prior.join("build")
            } else {
                retained.join("reporting")
            },
            &out,
        ),
    ] {
        c::store::checked(
            Command::new("cp")
                .args(["-a", "--reflink=always"])
                .arg(from)
                .arg(to),
        )?;
    }
    let patch = repo.join(if handoff {
        "patches/oxos-startup-handoff.patch"
    } else {
        "patches/oxos-launch-disposition.patch"
    });
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
    let native: serde_json::Value = c::read_json(&root.join(if handoff {
        "android-tcg-baseline-v1/build.identity.json"
    } else {
        "android-serial-v1/build.identity.json"
    }))?;
    let units = if handoff {
        ["kernel/src/ps/thread.cpp", "kernel/src/ke/except.cpp"]
    } else {
        ["kernel/src/hal/smbus.cpp", "kernel/src/xe/loader.cpp"]
    };
    let source_files: serde_json::Map<String, serde_json::Value> = units
        .into_iter()
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
            "public_launch_header_fields_only":!handoff,
            "bounded_public_thread_and_exception_fields":handoff,
            "private_game_code_inspection":false,
            "prior_sources_and_objects_retained":true,"ordinary_default_changed":false,
            "original_or_Fable_qualified":false
        }))?,
    )?;
    fs::File::open(dir.join("identity.json"))?.sync_all()?;
    Ok(())
}
