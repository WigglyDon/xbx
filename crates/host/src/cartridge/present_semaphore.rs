// SPDX-License-Identifier: MIT
//! Exact retained presentation block against delayed presentation consumption.
use super::{
    self as c,
    android_build::{compile, literal},
};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const UNIT: &str = "hw/xbox/nv2a/pgraph/vk/display.c";
const PATCH: &str = "patches/xemu-present-semaphore.patch";
const PARENT: &str = "6ec2ac7c4fc60de09d46b445a41349ddf40a13ad1d47b9aaceea2b995408ebe2";
fn subject(text: &str) -> Result<String, Error> {
    let start = text
        .find("    VkSubmitInfo submit_info = {")
        .ok_or("submit block absent")?;
    let end = start
        + text[start..]
            .find("VkResult present_result = vkQueuePresentKHR(r->queue, &present_info);")
            .ok_or("present block absent")?
        + "VkResult present_result = vkQueuePresentKHR(r->queue, &present_info);".len();
    Ok(format!(
        "/* SPDX-License-Identifier: LGPL-2.0-or-later */\nstatic void subject(PGRAPHVkState *r,unsigned present_sync_index){{PGRAPHVkDisplayState *disp=&r->display;int cmd=77;\n{}\n(void)present_result;\n}}\n#endif\n}}\n",
        &text[start..end]
    ))
}
pub(super) fn contract(repo: &Path, root: &Path) -> Result<(), Error> {
    let parent = root.join("android-demo-v1/android-thread-names-v1");
    if c::file_digest(&parent.join("libxemu.so"))? != PARENT {
        return Err("qualified presentation parent changed".into());
    }
    let out = root.join("native-present-semaphore-contract-v1");
    c::mkdir(&out)?;
    let source = out.join("source");
    fs::create_dir_all(source.join("hw/xbox/nv2a/pgraph/vk"))?;
    let before = fs::read_to_string(parent.join("source").join(UNIT))?;
    c::write_new(&source.join(UNIT), before.as_bytes())?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &out)
            .arg("apply")
            .arg(repo.join(PATCH)),
    )?;
    let after = fs::read_to_string(source.join(UNIT))?;
    if before
        .matches("&disp->render_finished[present_sync_index]")
        .count()
        != 2
        || after
            != before.replace(
                "&disp->render_finished[present_sync_index]",
                "&disp->render_finished[disp->render_idx]",
            )
    {
        return Err("unexpected presentation correction scope".into());
    }
    let mut results = Vec::new();
    let mut failed_before = 0;
    for (version, text) in [("before", &before), ("after", &after)] {
        let include = out.join(format!("{version}.inc"));
        c::write_new(&include, subject(text)?.as_bytes())?;
        let bin = out.join(version);
        c::store::checked(
            Command::new("cc")
                .args([
                    "-std=gnu11",
                    "-O2",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-D__ANDROID__=1",
                ])
                .arg(format!("-DSUBJECT=\"{}\"", include.display()))
                .arg(repo.join("native/fixtures/present-semaphore.c"))
                .arg("-o")
                .arg(&bin),
        )?;
        for pattern in 0..81 {
            let run = Command::new(&bin).arg(pattern.to_string()).output()?;
            if version == "before" && run.status.code() == Some(1) {
                failed_before += 1;
            }
            if (version == "after" && !run.status.success())
                || !matches!(run.status.code(), Some(0 | 1))
            {
                return Err("presentation lifetime control failed unexpectedly".into());
            }
            results.push(json!({"version":version,"pattern":pattern,"exit":run.status.code()}));
        }
    }
    if failed_before == 0 {
        return Err("presentation defect not reproduced".into());
    }
    c::write_new(
        &out.join("result.json"),
        &c::json(
            &json!({"passed":true,"parent_core":PARENT,"patch":c::file_digest(&repo.join(PATCH))?,"fixture":c::file_digest(&repo.join("native/fixtures/present-semaphore.c"))?,"before_source":c::digest(before.as_bytes()),"after_source":c::digest(after.as_bytes()),"failed_before":failed_before,"after_passed":81,"cases":results,"contract":"Rendering fence completion does not retire a presentation wait; acquired-image wait orders safe reuse of that image semaphore","reference":"https://docs.vulkan.org/guide/latest/swapchain_semaphore_reuse.html","actual_Vulkan_GPU_execution":false,"Fable_cause_established":false}),
        )?,
    )?;
    Ok(())
}

pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let proof: Value =
        c::read_json(&root.join("native-present-semaphore-contract-v1/result.json"))?;
    if proof["passed"] != true || proof["patch"] != c::file_digest(&repo.join(PATCH))? {
        return Err("original presentation boundary proof required".into());
    }
    let android = root.join("android-demo-v1");
    let prior = android.join("android-thread-names-v1").canonicalize()?;
    if c::file_digest(&prior.join("libxemu.so"))? != PARENT {
        return Err("presentation parent changed".into());
    }
    let previous: Value = c::read_json(&prior.join("build.identity.json"))?;
    let new = android.join("android-present-semaphore-v1");
    c::mkdir(&new)?;
    let source = new.join("source");
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=auto"])
            .arg(prior.join("source"))
            .arg(&source),
    )?;
    // Only the two proven units are replaced in this separate source overlay.
    let proven = root.join("native-present-semaphore-contract-v1/source");
    fs::copy(proven.join(UNIT), source.join(UNIT))?;
    let archive = new.join("libxemu_core.a");
    c::store::checked(
        Command::new("cp")
            .arg("--reflink=auto")
            .arg(prior.join("libxemu_core.a"))
            .arg(&archive),
    )?;
    let mut expected = super::gdb_features::objects(&fs::read(&archive)?)?;
    let original_count = expected.len();
    let old = repo.join(".local/build-android");
    let base = repo.join(".local/src/xemu-android");
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    {
        let (unit, member, name) = (UNIT, "xbx-present-surface.c.o", "xbx-present-semaphore");
        if expected.iter().filter(|(n, _)| n == member).count() != 1 {
            return Err("ambiguous presentation object".into());
        }
        let target = format!(
            "CMakeFiles/xemu_core.dir/{}/{unit}.o",
            literal(&base)?.trim_start_matches('/')
        );
        let object = compile(&old, &source, &base, &new, unit, &target, name)?;
        c::store::checked(
            Command::new(llvm.join("llvm-ar"))
                .arg("d")
                .arg(&archive)
                .arg(member),
        )?;
        c::store::checked(
            Command::new(llvm.join("llvm-ar"))
                .arg("rD")
                .arg(&archive)
                .arg(&object),
        )?;
        expected.retain(|(n, _)| n != member);
        expected.push((format!("{name}.c.o"), c::file_digest(&object)?));
    }
    expected.sort();
    let mut actual = super::gdb_features::objects(&fs::read(&archive)?)?;
    actual.sort();
    if actual != expected {
        return Err("unrelated presentation archive object changed".into());
    }
    let lib = new.join("libxemu.so");
    let link = fs::read_to_string(prior.join("link.command"))?
        .replace(&literal(&prior.join("libxemu.so"))?, &literal(&lib)?)
        .replace(
            &literal(&prior.join("libxemu_core.a"))?,
            &literal(&archive)?,
        )
        .replace(
            &literal(&prior.join("link.d"))?,
            &literal(&new.join("link.d"))?,
        );
    c::write_new(&new.join("link.command"), link.as_bytes())?;
    c::store::checked(Command::new("sh").current_dir(old).arg("-c").arg(link))?;
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(
            &json!({"source_pin":"4967efd3ee594cc236869a45ee4cb396e25af1ac","bridge_abi":4,"source_sha256":c::file_digest(&repo.join("native/xbx_xemu_bridge.c"))?,"library_sha256":c::file_digest(&lib)?,"prior_library_sha256":PARENT,"original_contract":proof,"inherited_repairs":previous,"all_other_archive_objects_unchanged":true,"unchanged_core_object_count":original_count-1,"recompiled_units":[UNIT],"whole_core_rebuild":false,"state_layout_changed":false,"old_checkpoints_retagged":false,"Android_execution":false,"performance_cause_established":false}),
        )?,
    )?;
    Ok(())
}
