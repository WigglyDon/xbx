// SPDX-License-Identifier: MIT
//! Execute the retained overlay decoder on original paired register banks.
use super::{self as c, present_semaphore, present_tag};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const UNIT: &str = "hw/xbox/nv2a/pgraph/vk/display.c";
const PATCH: &str = "patches/xemu-pvideo-buffers.patch";
const MARKER: &str = "patches/xemu-present-tag-optional.patch";
const PARENT: &str = "c5dff2b5205a467fcc85f46e7f3458b748ecd89a403df5eea649ccc7080e4652";
pub(super) fn contract(repo: &Path, root: &Path) -> Result<(), Error> {
    let parent = root.join("android-demo-v1/android-present-semaphore-v1");
    if c::file_digest(&parent.join("libxemu.so"))? != PARENT {
        return Err("qualified overlay parent differs".into());
    }
    let out = root.join("native-pvideo-banks-contract-v1");
    c::mkdir(&out)?;
    let source = out.join("source");
    fs::create_dir_all(source.join("hw/xbox/nv2a/pgraph/vk"))?;
    let before = fs::read_to_string(parent.join("source").join(UNIT))?;
    c::write_new(&source.join(UNIT), before.as_bytes())?;
    for patch in [PATCH, MARKER] {
        c::store::checked(
            Command::new("git")
                .arg("-C")
                .arg(&source)
                .env("GIT_CEILING_DIRECTORIES", &out)
                .arg("apply")
                .arg(repo.join(patch)),
        )?;
    }
    let after = fs::read_to_string(source.join(UNIT))?;
    if present_semaphore::subject(&before)? != present_semaphore::subject(&after)? {
        return Err("overlay patch alters submission/presentation block".into());
    }
    let headers = repo.join(".local/src/xemu-android/hw/xbox/nv2a");
    let renderer = fs::read_to_string(headers.join("pgraph/vk/renderer.h"))?;
    let state = present_tag::declaration(
        &renderer,
        "typedef struct PvideoState {",
        "\ntypedef struct DisplayImage",
    )?;
    let mut cases = Vec::new();
    let mut failed_before = 0;
    for (version, text) in [("before", &before), ("after", &after)] {
        let helper = present_tag::declaration(
            text,
            "static float pvideo_calculate_scale(",
            "\nstatic void destroy_pvideo_image",
        )?;
        let end = if version == "before" {
            "\nstatic void update_uniforms"
        } else {
            "\n#ifdef __ANDROID__\n/* Tokens"
        };
        let decoder = present_tag::declaration(text, "static PvideoState get_pvideo_state(", end)?;
        let include = out.join(format!("{version}.inc"));
        c::write_new(
            &include,
            format!(
                "/* SPDX-License-Identifier: LGPL-2.0-or-later */\n{state}\n{helper}\n{decoder}\n"
            )
            .as_bytes(),
        )?;
        let binary = out.join(version);
        let mut compile = Command::new("cc");
        compile
            .args(["-std=gnu11", "-O2", "-Wall", "-Wextra", "-Werror"])
            .arg(format!("-DSUBJECT=\"{}\"", include.display()))
            .arg(format!(
                "-DREGS=\"{}\"",
                headers.join("nv2a_regs.h").display()
            ));
        if version == "after" {
            compile.arg("-DAFTER=1");
        }
        compile
            .arg(repo.join("native/fixtures/pvideo-buffers.c"))
            .args(["-lm", "-o"])
            .arg(&binary);
        c::store::checked(&mut compile)?;
        for case in 0..9 {
            let run = Command::new(&binary).arg(case.to_string()).output()?;
            let expected = if version == "before" && matches!(case, 1 | 3) {
                1
            } else {
                0
            };
            if run.status.code() != Some(expected) {
                return Err(format!(
                    "unexpected overlay {version} case{case} outcome: {:?}",
                    run.status.code()
                )
                .into());
            }
            if expected == 1 {
                failed_before += 1;
            }
            cases.push(json!({"version":version,"case":case,"exit":run.status.code()}));
        }
    }
    let shader = present_tag::compile_shader(repo, &out, &after)?;
    c::write_new(
        &out.join("result.json"),
        &c::json(&json!({
            "passed":true,"parent_core":PARENT,"patch":c::file_digest(&repo.join(PATCH))?,
            "marker_patch":c::file_digest(&repo.join(MARKER))?,"fixture":c::file_digest(&repo.join("native/fixtures/pvideo-buffers.c"))?,
            "register_header":c::file_digest(&headers.join("nv2a_regs.h"))?,"state_type_header":c::file_digest(&headers.join("pgraph/vk/renderer.h"))?,
            "before_source":c::digest(before.as_bytes()),"after_source":c::digest(after.as_bytes()),
            "before_failed":failed_before,"after_passed":9,"cases":cases,
            "controls":["bank0 all fields","bank1 all fields","both retain bank0 priority","invalid unused bank cannot hide selected bank","STOP/none","selected pitch/VRAM/format guards"],
            "submission_presentation_block_bytes_unchanged":true,"shader":shader,
            "diagnostic_marker_Rust_owned_cosmetic_only_same_core_for_ordinary_APK":true,
            "primary_reference":"https://github.com/torvalds/linux/blob/master/drivers/gpu/drm/nouveau/dispnv04/overlay.c",
            "actual_GPU_execution":false,"Fable_cause_or_performance_qualified":false
        }))?,
    )?;
    Ok(())
}
