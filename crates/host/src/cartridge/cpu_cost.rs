// SPDX-License-Identifier: MIT
//! Developer-only aggregate native phase timing after stock sampling refusal.
use super::{
    self as c,
    android_build::{compile, literal},
    vertex_cache::pinned,
};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const PATCH: &str = "patches/xemu-cpu-phase-cost.patch";
const HEADER: &str = "native/xbx_cpu_cost.h";
const CPU: &str = "accel/tcg/cpu-exec.c";
const STATS: &str = "accel/tcg/tcg-stats.c";
const PARENT: &str = "6ec2ac7c4fc60de09d46b445a41349ddf40a13ad1d47b9aaceea2b995408ebe2";

fn without_hooks(text: &str) -> Result<String, Error> {
    let mut inside = false;
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        match line.trim() {
            "/* XBX cost begin */" if !inside => inside = true,
            "/* XBX cost end */" if inside => inside = false,
            "/* XBX cost begin */" | "/* XBX cost end */" => {
                return Err("invalid timing markers".into());
            }
            _ if !inside => out.push_str(line),
            _ => {}
        }
    }
    if inside {
        return Err("unfinished timing hook".into());
    }
    Ok(out)
}
pub(super) fn contract(repo: &Path, root: &Path) -> Result<(), Error> {
    let parent = root.join("android-demo-v1/android-thread-names-v1");
    if c::file_digest(&parent.join("libxemu.so"))? != PARENT {
        return Err("qualified timing parent differs".into());
    }
    let out = root.join("native-cpu-phase-contract-v1");
    c::mkdir(&out)?;
    let source = out.join("source");
    fs::create_dir_all(source.join("accel/tcg"))?;
    let mut before = Vec::new();
    for unit in [CPU, STATS] {
        let path = parent.join("source").join(unit);
        let text = if path.exists() {
            fs::read_to_string(path)?
        } else {
            pinned(repo, unit)?
        };
        c::write_new(&source.join(unit), text.as_bytes())?;
        before.push((unit, text));
    }
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &out)
            .arg("apply")
            .arg(repo.join(PATCH)),
    )?;
    let mut identities = Vec::new();
    for (unit, text) in &before {
        let after = fs::read_to_string(source.join(unit))?;
        if without_hooks(&after)? != *text {
            return Err("timing patch changes preexisting execution statements".into());
        }
        identities.push(json!({"unit":unit,"before":c::digest(text.as_bytes()),"after":c::digest(after.as_bytes()),"all_preexisting_bytes_and_order_preserved":true}));
    }
    let header = source.join("accel/tcg/xbx-cpu-cost.h");
    c::write_new(&header, &fs::read(repo.join(HEADER))?)?;
    let flags = Command::new("pkg-config")
        .args(["--cflags", "--libs", "glib-2.0"])
        .output()?;
    if !flags.status.success() {
        return Err("existing host GLib development interface unavailable".into());
    }
    let binary = out.join("original");
    c::store::checked(
        Command::new("cc")
            .args(["-std=gnu11", "-O2", "-Wall", "-Wextra", "-Werror"])
            .arg(format!("-DSUBJECT=\"{}\"", header.display()))
            .arg(repo.join("native/fixtures/cpu-cost.c"))
            .args(String::from_utf8(flags.stdout)?.split_whitespace())
            .arg("-o")
            .arg(&binary),
    )?;
    c::store::checked(&mut Command::new(&binary))?;
    c::write_new(
        &out.join("result.json"),
        &c::json(
            &json!({"passed":true,"parent_core":PARENT,"patch":c::file_digest(&repo.join(PATCH))?,"header":c::file_digest(&repo.join(HEADER))?,"fixture":c::file_digest(&repo.join("native/fixtures/cpu-cost.c"))?,"units":identities,"controls":["1024 ordinary iterations preserve exact work; four selected samples and24 clock reads","abandoned stack-local sample cannot contaminate later sample","backwards clock contributes no partial durations","exact named aggregate serialization"],"clock":"thread CPU time; selected completed iterations only; no claim to unbiased whole-worker cost","sampler_stride":256,"guest_state_PC_or_bytes_collected":false,"Android_execution":false}),
        )?,
    )?;
    Ok(())
}
pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    build_candidate(repo, root, false)
}
pub(super) fn build_present(repo: &Path, root: &Path) -> Result<(), Error> {
    build_candidate(repo, root, true)
}
fn build_candidate(repo: &Path, root: &Path, present: bool) -> Result<(), Error> {
    let proof: Value = c::read_json(&root.join("native-cpu-phase-contract-v1/result.json"))?;
    if proof["passed"] != true
        || proof["patch"] != c::file_digest(&repo.join(PATCH))?
        || proof["header"] != c::file_digest(&repo.join(HEADER))?
    {
        return Err("original timing boundary proof required".into());
    }
    let android = root.join("android-demo-v1");
    let (parent_stage, new_stage, parent_digest) = if present {
        (
            "android-present-semaphore-v1",
            "android-cpu-present-cost-v1",
            "c5dff2b5205a467fcc85f46e7f3458b748ecd89a403df5eea649ccc7080e4652",
        )
    } else {
        (
            "android-thread-names-v1",
            "android-cpu-phase-cost-v1",
            PARENT,
        )
    };
    let prior = android.join(parent_stage).canonicalize()?;
    if c::file_digest(&prior.join("libxemu.so"))? != parent_digest {
        return Err("timing parent changed".into());
    }
    let previous: Value = c::read_json(&prior.join("build.identity.json"))?;
    // Reuse the accepted phase controls only when both original source units
    // match their proven pre-hook bytes. The presentation repair changes
    // neither execution unit; it remains included in the new parent archive.
    for unit in [CPU, STATS] {
        let path = prior.join("source").join(unit);
        let text = if path.exists() {
            fs::read_to_string(path)?
        } else {
            pinned(repo, unit)?
        };
        let control = proof["units"]
            .as_array()
            .and_then(|units| units.iter().find(|entry| entry["unit"] == unit))
            .ok_or("timing source control missing")?;
        if control["before"] != c::digest(text.as_bytes()) {
            return Err("timing parent execution source differs from accepted controls".into());
        }
    }
    let new = android.join(new_stage);
    c::mkdir(&new)?;
    let source = new.join("source");
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=auto"])
            .arg(prior.join("source"))
            .arg(&source),
    )?;
    // Only the two proven units are replaced in this separate source overlay.
    let proven = root.join("native-cpu-phase-contract-v1/source");
    for unit in [CPU, STATS, "accel/tcg/xbx-cpu-cost.h"] {
        fs::copy(proven.join(unit), source.join(unit))?;
    }
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
    for (unit, member, name) in [
        (CPU, "cpu-exec.c.o", "xbx-cpu-phase-exec"),
        (STATS, "tcg-stats.c.o", "xbx-cpu-phase-stats"),
    ] {
        if expected.iter().filter(|(n, _)| n == member).count() != 1 {
            return Err("ambiguous timing object".into());
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
        return Err("unrelated timing archive object changed".into());
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
            &json!({"source_pin":"4967efd3ee594cc236869a45ee4cb396e25af1ac","bridge_abi":4,"source_sha256":c::file_digest(&repo.join("native/xbx_xemu_bridge.c"))?,"library_sha256":c::file_digest(&lib)?,"prior_library_sha256":parent_digest,"original_contract":proof,"inherited_repairs":previous,"presentation_semaphore_repair_retained":present,"all_other_archive_objects_unchanged":true,"unchanged_core_object_count":original_count-2,"recompiled_units":[CPU,STATS],"whole_core_rebuild":false,"state_layout_changed":false,"old_checkpoints_retagged":false,"Android_execution":false,"performance_cause_established":false}),
        )?,
    )?;
    Ok(())
}
