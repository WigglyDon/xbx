// SPDX-License-Identifier: MIT
//! Original value-coherence controls for the retained Vulkan binding cache.
use super::{
    self as c,
    android_build::{compile, literal, output},
};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const PIN: &str = "4967efd3ee594cc236869a45ee4cb396e25af1ac";
const UNIT: &str = "hw/xbox/nv2a/pgraph/vk/vertex.c";
const VALUE_UNIT: &str = "hw/xbox/nv2a/pgraph/vertex.c";
const PATCH: &str = "patches/xemu-vk-vertex-cache-values.patch";

pub(super) fn pinned(repo: &Path, unit: &str) -> Result<String, Error> {
    output(
        Command::new("git")
            .arg("-C")
            .arg(repo.join(".local/src/xemu-android"))
            .args(["show", &format!("{PIN}:{unit}")])
            .env("GIT_NO_LAZY_FETCH", "1"),
    )
}

pub(super) fn formats(repo: &Path, root: &Path) -> Result<(), Error> {
    let out = root.join("android-vertex-formats-v1");
    c::mkdir(&out)?;
    let compiler = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android29-clang");
    let source = repo.join("native/fixtures/vulkan-vertex-formats.c");
    let binary = out.join("xbx-vertex-formats");
    c::store::checked(
        Command::new(&compiler)
            .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-O2"])
            .arg(&source)
            .arg("-lvulkan")
            .arg("-o")
            .arg(&binary),
    )?;
    let elf = fs::read(&binary)?;
    if elf.get(..6) != Some(b"\x7fELF\x02\x01") || elf.get(18..20) != Some(&[183, 0]) {
        return Err("original format check ABI differs".into());
    }
    c::write_new(
        &out.join("identity.json"),
        &c::json(&json!({
            "source":c::file_digest(&source)?,"binary":c::digest(&elf),"compiler":c::file_digest(&compiler)?,
            "device_execution":false,"queries":"21 public vertex-buffer format support bits and two geometry feature booleans",
            "device_identifiers_or_content_collected":false,"settings_changed":false,"Fable_cause_established":false
        }))?,
    )?;
    Ok(())
}

pub(super) fn function(source: &str, marker: &str) -> Result<String, Error> {
    let start = source.find(marker).ok_or("exact public function missing")?;
    let open = start + source[start..].find('{').ok_or("function opening")?;
    let mut depth = 0;
    for (offset, byte) in source.as_bytes()[open..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(source[start..open + offset + 1].into());
                }
            }
            _ => (),
        }
    }
    Err("incomplete public function".into())
}

fn subject(source: &str, values: &str) -> Result<String, Error> {
    let mut text = String::from("/* SPDX-License-Identifier: LGPL-2.0-or-later */\n");
    text += &function(values, "void pgraph_update_inline_value(")?;
    text += "\n";
    text += &function(source, "static void update_memory_buffer(")?;
    for name in [
        "float_to_count",
        "ub_to_count",
        "s1_to_count",
        "s32k_to_count",
        "s32k_int_to_count",
    ] {
        let start = source
            .find(&format!("static const VkFormat {name}[]"))
            .ok_or("format table")?;
        let end = start + source[start..].find("};").ok_or("format table end")? + 2;
        text += &source[start..end];
        text += "\n";
    }
    text += &function(source, "void pgraph_vk_bind_vertex_attributes(")?;
    Ok(text)
}

pub(super) fn contract(repo: &Path, root: &Path) -> Result<(), Error> {
    let out = root.join("vertex-cache-values-contract-v1");
    c::mkdir(&out)?;
    let source = out.join("source");
    fs::create_dir_all(source.join("hw/xbox/nv2a/pgraph/vk"))?;
    let before = pinned(repo, UNIT)?;
    let values = pinned(repo, VALUE_UNIT)?;
    let regs = pinned(repo, "hw/xbox/nv2a/nv2a_regs.h")?;
    c::write_new(&source.join(UNIT), before.as_bytes())?;
    c::write_new(&out.join("nv2a_regs.h"), regs.as_bytes())?;
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
    for (version, code) in [("before", &before), ("after", &after)] {
        let include = out.join(format!("{version}.inc"));
        c::write_new(&include, subject(code, &values)?.as_bytes())?;
        let binary = out.join(version);
        c::store::checked(
            Command::new("cc")
                .args(["-std=gnu11", "-Wall", "-Wextra", "-Werror"])
                .arg("-I")
                .arg(&out)
                .arg(format!("-DSUBJECT=\"{}\"", include.display()))
                .arg(repo.join("native/fixtures/vertex-cache-values.c"))
                .arg("-o")
                .arg(&binary),
        )?;
        for case in ["control", "uniform", "provoking", "disabled"] {
            let run = Command::new(&binary).arg(case).output()?;
            c::write_new(&out.join(format!("{version}-{case}.stdout")), &run.stdout)?;
            c::write_new(&out.join(format!("{version}-{case}.stderr")), &run.stderr)?;
            let expected =
                i32::from(version == "before" && matches!(case, "uniform" | "provoking"));
            let passed = run.status.code() == Some(expected);
            results.push(json!({"version":version,"case":case,"exit":run.status.code(),"expected":expected,"passed":passed}));
            if !passed {
                return Err("original vertex-value regression failed; preserve outputs".into());
            }
        }
    }
    c::write_new(
        &out.join("result.json"),
        &c::json(&json!({
            "source_pin":PIN,"unit":UNIT,"before":c::digest(before.as_bytes()),"after":c::digest(after.as_bytes()),
            "value_function":c::digest(values.as_bytes()),"register_definitions":c::digest(regs.as_bytes()),
            "patch":c::file_digest(&repo.join(PATCH))?,"fixture":c::file_digest(&repo.join("native/fixtures/vertex-cache-values.c"))?,
            "exact_function_bodies":true,"results":results,"passed":true,"controlled_original_RAM":true,
            "actual_Vulkan_submission":false,"game_bytes_used":false,"Fable_cause_established":false
        }))?,
    )?;
    Ok(())
}

pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let proof: serde_json::Value =
        c::read_json(&root.join("vertex-cache-values-contract-v1/result.json"))?;
    if proof["passed"] != true || proof["patch"] != c::file_digest(&repo.join(PATCH))? {
        return Err("exact function-body original red/green required".into());
    }
    let android = root.join("android-demo-v1");
    let prior = android.join("android-fastmem-alias-v1");
    let previous: serde_json::Value = c::read_json(&prior.join("build.identity.json"))?;
    if c::file_digest(&prior.join("libxemu.so"))?
        != "5042323792e178072d2351e5d4043540706c7bcbd8ec2f321a787cbe117d9a15"
    {
        return Err("qualified fastmem parent differs".into());
    }
    let new = android.join("android-vertex-cache-values-v1");
    c::mkdir(&new)?;
    let source = new.join("source");
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(prior.join("source"))
            .arg(&source),
    )?;
    let path = source.join(UNIT);
    if path.exists() {
        return Err("unexpected existing vertex subject; preserve it".into());
    }
    let before = pinned(repo, UNIT)?;
    if proof["before"] != c::digest(before.as_bytes()) {
        return Err("public vertex source differs".into());
    }
    c::write_new(&path, before.as_bytes())?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &new)
            .arg("apply")
            .arg(repo.join(PATCH)),
    )?;
    if proof["after"] != c::file_digest(&path)? {
        return Err("native vertex subject differs from original proof".into());
    }
    let old = repo.join(".local/build-android");
    let base = repo.join(".local/src/xemu-android");
    let target = format!(
        "CMakeFiles/xemu_core.dir/{}/{UNIT}.o",
        literal(&base)?.trim_start_matches('/')
    );
    let object = compile(
        &old,
        &source,
        &base,
        &new,
        UNIT,
        &target,
        "xbx-vk-vertex-values",
    )?;
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let ar = llvm.join("llvm-ar");
    let mut expected = super::gdb_features::objects(&fs::read(prior.join("libxemu_core.a"))?)?;
    let count = expected
        .iter()
        .filter(|(name, _)| name == "vertex.c.o")
        .count();
    let mut found = None;
    for nth in 1..=count {
        let dir = new.join(format!("vertex-member-{nth}"));
        c::mkdir(&dir)?;
        c::store::checked(
            Command::new(&ar)
                .current_dir(&dir)
                .args(["xN", &nth.to_string()])
                .arg(prior.join("libxemu_core.a"))
                .arg("vertex.c.o"),
        )?;
        let symbols = output(
            Command::new(llvm.join("llvm-nm"))
                .arg("--defined-only")
                .arg(dir.join("vertex.c.o")),
        )?;
        if symbols
            .lines()
            .any(|l| l.ends_with(" pgraph_vk_bind_vertex_attributes"))
        {
            if found.is_some() {
                return Err("ambiguous Vulkan binding member".into());
            }
            found = Some((nth, c::file_digest(&dir.join("vertex.c.o"))?));
        }
    }
    let (nth, removed) = found.ok_or("Vulkan binding object absent")?;
    let archive = new.join("libxemu_core.a");
    c::store::checked(
        Command::new("cp")
            .arg("--reflink=always")
            .arg(prior.join("libxemu_core.a"))
            .arg(&archive),
    )?;
    c::store::checked(
        Command::new(&ar)
            .args(["dN", &nth.to_string()])
            .arg(&archive)
            .arg("vertex.c.o"),
    )?;
    c::store::checked(Command::new(&ar).arg("rD").arg(&archive).arg(&object))?;
    let at = expected
        .iter()
        .position(|row| row == &("vertex.c.o".into(), removed.clone()))
        .ok_or("prior object identity")?;
    expected.remove(at);
    let unchanged = expected.len();
    expected.push(("xbx-vk-vertex-values.c.o".into(), c::file_digest(&object)?));
    expected.sort();
    let mut after = super::gdb_features::objects(&fs::read(&archive)?)?;
    after.sort();
    if expected != after {
        return Err("unrelated archive member changed".into());
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
            "source_pin":PIN,"bridge_abi":4,"source_sha256":c::file_digest(&repo.join("native/xbx_xemu_bridge.c"))?,
            "library_sha256":c::file_digest(&lib)?,"prior_library_sha256":previous["library_sha256"],
            "open_image_native_parent":previous["open_image_native_parent"],"inherited_repairs":previous,
            "original_contract":proof,"recompiled_units":[UNIT],"removed_object":removed,
            "all_other_archive_objects_unchanged":true,"unchanged_core_object_count":unchanged,
            "whole_core_rebuild":false,"state_layout_changed":false,"old_checkpoints_retagged":false,
            "Android_execution":false,"Fable_cause_established":false
        }))?,
    )?;
    Ok(())
}
