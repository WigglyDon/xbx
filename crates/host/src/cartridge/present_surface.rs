// SPDX-License-Identifier: MIT
//! Retained Android presentation lifetime regression and bounded object rebuild.
use super::{
    self as c,
    android_build::{compile, literal, output},
    vertex_cache::{function, pinned},
};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const UNIT: &str = "hw/xbox/nv2a/pgraph/vk/display.c";
const PATCH: &str = "patches/xemu-android-present-surface.patch";

fn between<'a>(s: &'a str, start: &str, end: &str) -> Result<&'a str, Error> {
    let a = s.find(start).ok_or("presentation branch start missing")?;
    let b = a + s[a..].find(end).ok_or("presentation branch end missing")?;
    Ok(&s[a..b])
}
fn subject(s: &str) -> Result<String, Error> {
    let mut out = String::from("/* SPDX-License-Identifier: LGPL-2.0-or-later */\n");
    if s.contains("static void reset_android_present_surface(") {
        out += &function(s, "static void reset_android_present_surface(")?;
    }
    let create = &s[s
        .find("static bool create_android_swapchain(")
        .ok_or("creation subject")?..];
    out += between(
        create,
        "static bool create_android_swapchain(",
        "    VkSurfaceCapabilitiesKHR caps;",
    )?;
    out += "(void)d;(void)width;(void)height;return true;}\n";
    out += "static bool subject_caps(PGRAPHState *pg,VkResult result){(void)pg;\n";
    out += between(
        create,
        "    if (result != VK_SUCCESS)",
        "    uint32_t format_count = 0;",
    )?;
    out += "return true;}\n";
    let render = &s[s
        .find("static void render_display(")
        .ok_or("render subject")?..];
    for (name, start, end) in [
        (
            "acquire",
            "        if (acquire_result ==",
            "        disp->render_idx = acquired_index;",
        ),
        (
            "present",
            "        if (present_result ==",
            "        disp->present_frame++;",
        ),
    ] {
        out += &format!(
            "static void subject_{name}(PGRAPHState *pg,VkResult {name}_result){{(void)pg;\n"
        );
        out += between(render, start, end)?;
        out += "}\n";
    }
    Ok(out)
}
pub(super) fn contract(repo: &Path, root: &Path) -> Result<(), Error> {
    let out = root.join("present-surface-contract-v1");
    c::mkdir(&out)?;
    let source = out.join("source");
    fs::create_dir_all(source.join("hw/xbox/nv2a/pgraph/vk"))?;
    let before = pinned(repo, UNIT)?;
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
    let mut results = Vec::new();
    for (version, text) in [("before", &before), ("after", &after)] {
        let include = out.join(format!("{version}.inc"));
        c::write_new(&include, subject(text)?.as_bytes())?;
        let binary = out.join(version);
        c::store::checked(
            Command::new("cc")
                .args(["-std=gnu11", "-Wall", "-Wextra", "-Werror"])
                .arg(format!("-DSUBJECT=\"{}\"", include.display()))
                .arg(repo.join("native/fixtures/present-surface.c"))
                .arg("-o")
                .arg(&binary),
        )?;
        for case in [
            "acquire-success",
            "acquire-suboptimal",
            "acquire-outdated",
            "acquire-device-lost",
            "acquire-lost",
            "present-success",
            "present-outdated",
            "present-device-lost",
            "present-lost",
            "caps-success",
            "caps-device-lost",
            "caps-lost",
            "retry",
        ] {
            let run = Command::new(&binary).arg(case).output()?;
            c::write_new(&out.join(format!("{version}-{case}.stdout")), &run.stdout)?;
            c::write_new(&out.join(format!("{version}-{case}.stderr")), &run.stderr)?;
            let expected = i32::from(
                version == "before"
                    && matches!(
                        case,
                        "acquire-lost" | "present-lost" | "caps-lost" | "retry"
                    ),
            );
            let passed = run.status.code() == Some(expected);
            results.push(json!({"version":version,"case":case,"exit":run.status.code(),"expected":expected,"passed":passed}));
            if !passed {
                return Err("original presentation lifetime control failed; retain output".into());
            }
        }
    }
    c::write_new(
        &out.join("result.json"),
        &c::json(
            &json!({"source_pin":"4967efd3ee594cc236869a45ee4cb396e25af1ac","before":c::digest(before.as_bytes()),"after":c::digest(after.as_bytes()),"patch":c::file_digest(&repo.join(PATCH))?,"fixture":c::file_digest(&repo.join("native/fixtures/present-surface.c"))?,"exact_public_creation_prefix_and_error_branches":true,"mocked_handles_and_existing_swapchain_retirement":true,"surface_released_only_after_swapchain":true,"unavailable_window_retried_without_duplicate_release":true,"actual_GPU_submission":false,"game_bytes_used":false,"Fable_Home_cause_established":false,"results":results,"passed":true}),
        )?,
    )?;
    Ok(())
}

pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let contract_dir = root.join("present-surface-contract-v1");
    let proof: serde_json::Value = c::read_json(&contract_dir.join("result.json"))?;
    if proof["passed"] != true || proof["patch"] != c::file_digest(&repo.join(PATCH))? {
        return Err("original presentation lifetime proof required".into());
    }
    let android = root.join("android-demo-v1");
    let prior = android.join("android-frame-vertex-v1");
    let previous: serde_json::Value = c::read_json(&prior.join("build.identity.json"))?;
    if c::file_digest(&prior.join("libxemu.so"))?
        != "773cff5d5a8e1530c7adceb269a88807b596499be8ccf81794e491dc1ed1a1f7"
    {
        return Err("original-qualified coherent frame parent differs".into());
    }
    let new = android.join("android-present-surface-v1");
    c::mkdir(&new)?;
    let source = new.join("source");
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(prior.join("source"))
            .arg(&source),
    )?;
    if source.join(UNIT).exists() {
        return Err("unexpected presentation overlay; retain prior work".into());
    }
    c::write_new(&source.join(UNIT), pinned(repo, UNIT)?.as_bytes())?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &new)
            .arg("apply")
            .arg(repo.join(PATCH)),
    )?;
    if c::file_digest(&source.join(UNIT))? != proof["after"] {
        return Err("native presentation source differs from original proof".into());
    }
    let old = repo.join(".local/build-android");
    let base = repo.join(".local/src/xemu-android");
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let ar = llvm.join("llvm-ar");
    let archive = new.join("libxemu_core.a");
    c::store::checked(
        Command::new("cp")
            .arg("--reflink=always")
            .arg(prior.join("libxemu_core.a"))
            .arg(&archive),
    )?;
    let mut expected = super::gdb_features::objects(&fs::read(&archive)?)?;
    let original_count = expected.len();
    let (unit, member, name, symbol) = (
        UNIT,
        "display.c.o",
        "xbx-present-surface",
        " pgraph_vk_render_display",
    );
    {
        let target = format!(
            "CMakeFiles/xemu_core.dir/{}/{unit}.o",
            literal(&base)?.trim_start_matches('/')
        );
        let object = compile(&old, &source, &base, &new, unit, &target, name)?;
        let count = expected.iter().filter(|(n, _)| n == member).count();
        let mut selected = None;
        for nth in 1..=count {
            let dir = new.join(format!("{name}-member-{nth}"));
            c::mkdir(&dir)?;
            c::store::checked(
                Command::new(&ar)
                    .current_dir(&dir)
                    .args(["xN", &nth.to_string()])
                    .arg(&archive)
                    .arg(member),
            )?;
            let symbols = output(
                Command::new(llvm.join("llvm-nm"))
                    .arg("--defined-only")
                    .arg(dir.join(member)),
            )?;
            if symbols.lines().any(|l| l.ends_with(symbol)) {
                if selected.is_some() {
                    return Err("ambiguous native presentation member".into());
                }
                selected = Some((nth, c::file_digest(&dir.join(member))?));
            }
        }
        let (nth, hash) = selected.ok_or("native presentation member absent")?;
        c::store::checked(
            Command::new(&ar)
                .args(["dN", &nth.to_string()])
                .arg(&archive)
                .arg(member),
        )?;
        c::store::checked(Command::new(&ar).arg("rD").arg(&archive).arg(&object))?;
        let at = expected
            .iter()
            .position(|r| r == &(member.to_string(), hash.clone()))
            .ok_or("original object identity")?;
        expected.remove(at);
        expected.push((format!("{name}.c.o"), c::file_digest(&object)?));
    }
    expected.sort();
    let mut actual = super::gdb_features::objects(&fs::read(&archive)?)?;
    actual.sort();
    if expected != actual {
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
    c::store::checked(Command::new("sh").current_dir(&old).arg("-c").arg(link))?;
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(&json!({
            "source_pin":proof["source_pin"],"bridge_abi":4,"source_sha256":c::file_digest(&repo.join("native/xbx_xemu_bridge.c"))?,
            "library_sha256":c::file_digest(&lib)?,"prior_library_sha256":previous["library_sha256"],
            "open_image_native_parent":previous["open_image_native_parent"],"inherited_repairs":previous,"original_contract":proof,
            "all_other_archive_objects_unchanged":true,"unchanged_core_object_count":original_count-1,
            "recompiled_units":[UNIT],"whole_core_rebuild":false,"state_layout_changed":false,
            "old_checkpoints_retagged":false,"Android_execution":false,"Fable_Home_cause_established":false
        }))?,
    )?;
    Ok(())
}
