// SPDX-License-Identifier: MIT
//! Actual retained mapping functions, original owned-memory red/green control.
use super::{
    self as c,
    android_build::{compile, literal, output},
};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const PIN: &str = "4967efd3ee594cc236869a45ee4cb396e25af1ac";
const UNIT: &str = "accel/tcg/xbox-fastmem.c";
const PATCH: &str = "patches/xemu-fastmem-stale-alias.patch";

fn bodies(source: &str) -> Result<String, Error> {
    let mut result = String::from("/* SPDX-License-Identifier: GPL-2.0-or-later */\n");
    for name in ["xbox_fastmem_unmap_range", "xbox_fastmem_reset_dirty"] {
        let start = source
            .find(&format!("void {name}("))
            .ok_or("public mapping function")?;
        let open = start + source[start..].find('{').ok_or("function body")?;
        let mut depth = 0;
        let mut end = None;
        for (offset, byte) in source.as_bytes()[open..].iter().enumerate() {
            match byte {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(open + offset + 1);
                        break;
                    }
                }
                _ => (),
            }
        }
        result.push_str(&source[start..end.ok_or("complete function")?]);
        result.push('\n');
    }
    Ok(result)
}

pub(super) fn contract(repo: &Path, root: &Path) -> Result<(), Error> {
    let out = root.join("fastmem-alias-contract-v1");
    c::mkdir(&out)?;
    let source = out.join("source");
    fs::create_dir_all(source.join("accel/tcg"))?;
    let before = output(
        Command::new("git")
            .arg("-C")
            .arg(repo.join(".local/src/xemu-android"))
            .args(["show", &format!("{PIN}:{UNIT}")])
            .env("GIT_NO_LAZY_FETCH", "1"),
    )?;
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
    for (version, subject) in [("before", &before), ("after", &after)] {
        let inc = out.join(format!("{version}.inc"));
        c::write_new(&inc, bodies(subject)?.as_bytes())?;
        let binary = out.join(version);
        c::store::checked(
            Command::new("cc")
                .args(["-std=gnu11", "-Wall", "-Wextra", "-Werror"])
                .arg(format!("-DSUBJECT=\"{}\"", inc.display()))
                .arg(repo.join("native/fixtures/fastmem-stale-alias.c"))
                .arg("-o")
                .arg(&binary),
        )?;
        for case in ["live", "stale"] {
            let run = Command::new(&binary).arg(case).output()?;
            c::write_new(&out.join(format!("{version}-{case}.stdout")), &run.stdout)?;
            c::write_new(&out.join(format!("{version}-{case}.stderr")), &run.stderr)?;
            let expected = i32::from(version == "before" && case == "stale");
            let passed = run.status.code() == Some(expected);
            results.push(json!({"version":version,"case":case,"exit":run.status.code(),"expected":expected,"passed":passed}));
            if !passed {
                return Err("actual mapping-function regression failed; preserve outputs".into());
            }
        }
    }
    c::write_new(
        &out.join("result.json"),
        &c::json(&json!({
            "source_pin":PIN,"unit":UNIT,"before":c::digest(before.as_bytes()),
            "after":c::digest(after.as_bytes()),"patch":c::file_digest(&repo.join(PATCH))?,
            "fixture":c::file_digest(&repo.join("native/fixtures/fastmem-stale-alias.c"))?,
            "unchanged_function_bodies":true,"controlled_environment":"owned memfd and host window; actual mmap/mprotect; child read/write permission observations; no core dumps",
            "results":results,"passed":true,"actual_Android_core_execution":false,
            "game_bytes_used":false,"Fable_cause_established":false
        }))?,
    )?;
    Ok(())
}

pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let proof: serde_json::Value =
        c::read_json(&root.join("fastmem-alias-contract-v1/result.json"))?;
    if proof["passed"] != true || proof["patch"] != c::file_digest(&repo.join(PATCH))? {
        return Err("actual mapping red/green required before native correction".into());
    }
    let android = root.join("android-demo-v1");
    let prior = android.join("android-tcg-baseline-v1");
    let previous: serde_json::Value = c::read_json(&prior.join("build.identity.json"))?;
    if c::file_digest(&prior.join("libxemu.so"))?
        != "c2386913418acbb3d318ee266d6d618c533dd166b96a6e813a9a4bec63854832"
        || previous["source_pin"] != PIN
    {
        return Err("qualified retained Android baseline differs".into());
    }
    let new = android.join("android-fastmem-alias-v1");
    c::mkdir(&new)?;
    let source = new.join("source");
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(prior.join("source"))
            .arg(&source),
    )?;
    let subject = source.join(UNIT);
    if subject.exists() {
        return Err("unexpected existing unqualified fastmem subject; preserve it".into());
    }
    let before = output(
        Command::new("git")
            .arg("-C")
            .arg(repo.join(".local/src/xemu-android"))
            .args(["show", &format!("{PIN}:{UNIT}")])
            .env("GIT_NO_LAZY_FETCH", "1"),
    )?;
    if proof["before"] != c::digest(before.as_bytes()) {
        return Err("public subject changed".into());
    }
    c::write_new(&subject, before.as_bytes())?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &new)
            .arg("apply")
            .arg(repo.join(PATCH)),
    )?;
    if proof["after"] != c::file_digest(&subject)? {
        return Err("native subject differs from host-qualified body".into());
    }
    let core = new.join("libxemu_core.a");
    c::store::checked(
        Command::new("cp")
            .arg("--reflink=always")
            .arg(prior.join("libxemu_core.a"))
            .arg(&core),
    )?;
    let inventory = super::gdb_features::objects(&fs::read(&core)?)?;
    let member = "xbox-fastmem.c.o";
    if inventory.iter().filter(|(n, _)| n == member).count() != 1 {
        return Err("ambiguous native fastmem object".into());
    }
    let old = repo.join(".local/build-android");
    let base = repo.join(".local/src/xemu-android");
    let target = format!(
        "CMakeFiles/xemu_core.dir/{}/{UNIT}.o",
        literal(&base)?.trim_start_matches('/')
    );
    let object = compile(&old, &source, &base, &new, UNIT, &target, "xbox-fastmem")?;
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    c::store::checked(
        Command::new(llvm.join("llvm-ar"))
            .arg("rD")
            .arg(&core)
            .arg(object),
    )?;
    let mut unchanged: Vec<_> = inventory
        .iter()
        .filter(|(n, _)| n != member)
        .cloned()
        .collect();
    let after_inventory = super::gdb_features::objects(&fs::read(&core)?)?;
    let mut after: Vec<_> = after_inventory
        .iter()
        .filter(|(n, _)| n != member)
        .cloned()
        .collect();
    unchanged.sort();
    after.sort();
    if unchanged != after || after_inventory.len() != inventory.len() {
        return Err("unrelated core object changed".into());
    }
    let lib = new.join("libxemu.so");
    let link = fs::read_to_string(prior.join("link.command"))?
        .replace(&literal(&prior.join("libxemu.so"))?, &literal(&lib)?)
        .replace(&literal(&prior.join("libxemu_core.a"))?, &literal(&core)?)
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
            "patch":proof["patch"],"original_contract":proof,"recompiled_units":[UNIT],
            "all_other_archive_objects_unchanged":true,"unchanged_core_object_count":unchanged.len(),
            "whole_core_rebuild":false,"state_layout_changed":false,"old_checkpoints_retagged":false,
            "Android_execution":false,"Fable_cause_established":false
        }))?,
    )?;
    Ok(())
}
