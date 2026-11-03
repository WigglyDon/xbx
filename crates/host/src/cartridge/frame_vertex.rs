// SPDX-License-Identifier: MIT
//! Original separate-buffer regression for retained frame RAM propagation.
use super::{
    self as c,
    android_build::{compile, literal, output},
    vertex_cache::{function, pinned},
};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const VERTEX: &str = "hw/xbox/nv2a/pgraph/vk/vertex.c";
const DRAW: &str = "hw/xbox/nv2a/pgraph/vk/draw.c";
const PATCH: &str = "patches/xemu-vk-frame-vertex-coherence.patch";

fn subject(vertex: &str, draw: &str) -> Result<String, Error> {
    let mut text = String::from("/* SPDX-License-Identifier: LGPL-2.0-or-later */\n");
    text += &function(vertex, "void pgraph_vk_update_vertex_ram_buffer(")?;
    let start = draw
        .find("FrameStagingState *cur_fs =")
        .ok_or("frame copy block start")?;
    let marker = "vertex_ram_propagate_max = 0;";
    let end = start + draw[start..].find(marker).ok_or("frame copy block end")? + marker.len();
    text += "\nstatic void subject_rotate(PGRAPHState *pg, int next_frame) {\nPGRAPHVkState *r=pg->vk_renderer_state;\n";
    text += &draw[start..end];
    text += "\nr->current_frame=next_frame;\n}\n";
    Ok(text)
}

pub(super) fn contract(repo: &Path, root: &Path) -> Result<(), Error> {
    let out = root.join("frame-vertex-coherence-contract-v1");
    c::mkdir(&out)?;
    let source = out.join("source");
    fs::create_dir_all(source.join("hw/xbox/nv2a/pgraph/vk"))?;
    let vertex = pinned(repo, VERTEX)?;
    let draw = pinned(repo, DRAW)?;
    c::write_new(&source.join(VERTEX), vertex.as_bytes())?;
    c::write_new(&source.join(DRAW), draw.as_bytes())?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &out)
            .arg("apply")
            .arg(repo.join(PATCH)),
    )?;
    let new_vertex = fs::read_to_string(source.join(VERTEX))?;
    let new_draw = fs::read_to_string(source.join(DRAW))?;
    let mut results = Vec::new();
    for (version, v, d) in [
        ("before", &vertex, &draw),
        ("after", &new_vertex, &new_draw),
    ] {
        let include = out.join(format!("{version}.inc"));
        c::write_new(&include, subject(v, d)?.as_bytes())?;
        let binary = out.join(version);
        c::store::checked(
            Command::new("cc")
                .args(["-std=gnu11", "-Wall", "-Wextra", "-Werror"])
                .arg(format!("-DSUBJECT=\"{}\"", include.display()))
                .arg(repo.join("native/fixtures/frame-vertex-coherence.c"))
                .arg("-o")
                .arg(&binary),
        )?;
        for case in ["one", "two", "three", "idle", "overlap"] {
            let run = Command::new(&binary).arg(case).output()?;
            c::write_new(&out.join(format!("{version}-{case}.stdout")), &run.stdout)?;
            c::write_new(&out.join(format!("{version}-{case}.stderr")), &run.stderr)?;
            let expected = i32::from(version == "before" && matches!(case, "three" | "idle"));
            let passed = run.status.code() == Some(expected);
            results.push(json!({"version":version,"case":case,"exit":run.status.code(),"expected":expected,"passed":passed}));
            if !passed {
                return Err("original frame coherence regression failed; preserve output".into());
            }
        }
    }
    c::write_new(
        &out.join("result.json"),
        &c::json(&json!({
            "source_pin":"4967efd3ee594cc236869a45ee4cb396e25af1ac",
            "exact_public_update_function_and_frame_copy_block":true,"controlled_separate_RAM_buffers":true,
            "before_vertex":c::digest(vertex.as_bytes()),"after_vertex":c::digest(new_vertex.as_bytes()),
            "before_draw":c::digest(draw.as_bytes()),"after_draw":c::digest(new_draw.as_bytes()),
            "patch":c::file_digest(&repo.join(PATCH))?,"fixture":c::file_digest(&repo.join("native/fixtures/frame-vertex-coherence.c"))?,
            "results":results,"passed":true,"other_inflight_buffers_not_written":true,
            "idle_does_not_copy_historical_updates_forever":true,"actual_Vulkan_submission":false,
            "game_bytes_used":false,"Fable_cause_established":false
        }))?,
    )?;
    Ok(())
}

pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let contract_dir = root.join("frame-vertex-coherence-contract-v1");
    let proof: serde_json::Value = c::read_json(&contract_dir.join("result.json"))?;
    if proof["passed"] != true || proof["patch"] != c::file_digest(&repo.join(PATCH))? {
        return Err("original frame coherence proof required".into());
    }
    let android = root.join("android-demo-v1");
    let prior = android.join("android-vertex-cache-values-v1");
    let previous: serde_json::Value = c::read_json(&prior.join("build.identity.json"))?;
    if c::file_digest(&prior.join("libxemu.so"))?
        != "3472db2f195df618aa2465f34d74937b8810641a416892eaa0116b685c769b30"
    {
        return Err("original-qualified vertex cache parent differs".into());
    }
    let new = android.join("android-frame-vertex-v1");
    c::mkdir(&new)?;
    let source = new.join("source");
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(prior.join("source"))
            .arg(&source),
    )?;
    if c::file_digest(&source.join(VERTEX))? != previous["original_contract"]["after"] {
        return Err("accepted vertex cache source differs".into());
    }
    if source.join(DRAW).exists() {
        return Err("unexpected draw overlay; preserve it".into());
    }
    let draw = pinned(repo, DRAW)?;
    if proof["before_draw"] != c::digest(draw.as_bytes()) {
        return Err("public draw source differs".into());
    }
    c::write_new(&source.join(DRAW), draw.as_bytes())?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &new)
            .arg("apply")
            .arg(repo.join(PATCH)),
    )?;
    let qualified_vertex = fs::read_to_string(contract_dir.join("source").join(VERTEX))?;
    if proof["after_vertex"] != c::digest(qualified_vertex.as_bytes())
        || function(
            &fs::read_to_string(source.join(VERTEX))?,
            "void pgraph_vk_update_vertex_ram_buffer(",
        )? != function(
            &qualified_vertex,
            "void pgraph_vk_update_vertex_ram_buffer(",
        )?
        || proof["after_draw"] != c::file_digest(&source.join(DRAW))?
    {
        return Err("native frame policy differs from exact-body proof".into());
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
    for (unit, member, name, symbol) in [
        (
            VERTEX,
            "xbx-vk-vertex-values.c.o",
            "xbx-frame-vertex",
            " pgraph_vk_bind_vertex_attributes",
        ),
        (
            DRAW,
            "draw.c.o",
            "xbx-frame-draw",
            " pgraph_vk_flush_all_frames",
        ),
    ] {
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
                    return Err("ambiguous native frame member".into());
                }
                selected = Some((nth, c::file_digest(&dir.join(member))?));
            }
        }
        let (nth, hash) = selected.ok_or("native frame member absent")?;
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
        return Err("unrelated frame archive object changed".into());
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
            "all_other_archive_objects_unchanged":true,"unchanged_core_object_count":original_count-2,
            "recompiled_units":[VERTEX,DRAW],"whole_core_rebuild":false,"state_layout_changed":false,
            "old_checkpoints_retagged":false,"Android_execution":false,"Fable_cause_established":false
        }))?,
    )?;
    Ok(())
}
