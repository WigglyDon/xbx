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
    Yield,
    Word,
    Frame,
    FrameWord,
    Dispatch,
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
pub(super) fn serial_yield(repo: &Path, root: &Path) -> Result<(), Error> {
    build_selected(repo, root, Image::Yield)
}
pub(super) fn fault_word(repo: &Path, root: &Path) -> Result<(), Error> {
    build_selected(repo, root, Image::Word)
}
pub(super) fn frame(repo: &Path, root: &Path) -> Result<(), Error> {
    build_selected(repo, root, Image::Frame)
}
pub(super) fn frame_word(repo: &Path, root: &Path) -> Result<(), Error> {
    build_selected(repo, root, Image::FrameWord)
}
pub(super) fn dispatch(repo: &Path, root: &Path) -> Result<(), Error> {
    build_selected(repo, root, Image::Dispatch)
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
        Image::Dispatch => (
            root.parent()
                .ok_or("workspace")?
                .join("io-read-status-v1/source"),
            root.parent()
                .ok_or("workspace")?
                .join("io-read-status-v1/build"),
            "android-dispatch-stop-v2",
            "patches/oxos-dispatch-stop.patch",
            "android-tcg-baseline-v1",
            &["kernel/src/ke/except.cpp"],
        ),
        Image::FrameWord => (
            root.join("android-frame-classification-v1/source"),
            root.join("android-frame-classification-v1/build"),
            "android-frame-word-v1",
            "patches/oxos-frame-private-word.patch",
            "android-tcg-baseline-v1",
            &["kernel/src/ke/except.cpp"],
        ),
        Image::Frame => (
            root.parent()
                .ok_or("workspace")?
                .join("io-read-status-v1/source"),
            root.parent()
                .ok_or("workspace")?
                .join("io-read-status-v1/build"),
            "android-frame-classification-v1",
            "patches/oxos-null-frame-classification.patch",
            "android-tcg-baseline-v1",
            &["kernel/src/ke/except.cpp"],
        ),
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
        Image::Yield => (
            root.join("android-fault-v1/source"),
            root.join("android-fault-v1/build"),
            "android-fault-yield-v1",
            "patches/oxos-serial-poll-yield.patch",
            "android-tcg-baseline-v1",
            &["kernel/src/hal/serial.cpp"],
        ),
        Image::Word => (
            root.join("android-fault-yield-v1/source"),
            root.join("android-fault-yield-v1/build"),
            "android-fault-word-v1",
            "patches/oxos-android-fault-word.patch",
            "android-tcg-baseline-v1",
            &["kernel/src/ke/except.cpp"],
        ),
    };
    let parent_image = c::file_digest(&parent_build.join("bios.bin"))?;
    if matches!(image, Image::Frame | Image::Dispatch)
        && parent_image != "7ce5164c85a31e2e6a037bfb5690a3b1c710392e6da64f109d9d6ab8d4ee8563"
    {
        return Err("qualified current read-status image differs".into());
    }
    if matches!(image, Image::FrameWord)
        && parent_image != "6e41ef97adc7890b604950508abf68a66f5cda8541e52777d1f9368e36ac4afa"
    {
        return Err("qualified frame-classification parent differs".into());
    }
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
            "private_single_exception_context":matches!(image, Image::Fault | Image::Yield | Image::Word | Image::FrameWord),
            "serial_spin_pause_hint":matches!(image, Image::Yield | Image::Word),
            "public_kernel_return_classification_only":matches!(image, Image::Frame),
            "private_guarded_frame_word":matches!(image, Image::FrameWord),
            "private_dispatch_stop_without_title_memory_reads":matches!(image, Image::Dispatch),
            "maximum_guarded_stack_words":if matches!(image, Image::Word | Image::Frame | Image::FrameWord) { 1 } else { 0 },
            "private_game_code_inspection":false,
            "prior_sources_and_objects_retained":true,"ordinary_default_changed":false,
            "original_or_Fable_qualified":false
        }))?,
    )?;
    fs::File::open(dir.join("identity.json"))?.sync_all()?;
    Ok(())
}
