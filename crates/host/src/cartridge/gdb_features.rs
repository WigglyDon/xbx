// SPDX-License-Identifier: MIT
//! Restore pinned upstream register metadata; no guest execution code changes.
use super::{
    self as c,
    android_build::{compile, literal, output},
};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const PIN: &str = "4967efd3ee594cc236869a45ee4cb396e25af1ac";
fn objects(bytes: &[u8]) -> Result<Vec<(String, String)>, Error> {
    if !bytes.starts_with(b"!<arch>\n") {
        return Err("archive signature".into());
    }
    let mut at = 8;
    let mut names = &[][..];
    let mut result = Vec::new();
    while at < bytes.len() {
        let h = bytes.get(at..at + 60).ok_or("archive header")?;
        if &h[58..] != b"`\n" {
            return Err("archive header terminator".into());
        }
        let size: usize = std::str::from_utf8(&h[48..58])?.trim().parse()?;
        let data = bytes.get(at + 60..at + 60 + size).ok_or("archive body")?;
        let raw = std::str::from_utf8(&h[..16])?.trim();
        if raw == "//" {
            names = data
        } else if raw != "/" && raw != "/SYM64/" {
            let name = if let Some(index) = raw.strip_prefix('/') {
                let start: usize = index.parse()?;
                let tail = names.get(start..).ok_or("archive long name")?;
                let end = tail
                    .iter()
                    .position(|&b| b == b'\n')
                    .ok_or("long name terminator")?;
                std::str::from_utf8(&tail[..end])?
                    .trim_end_matches('/')
                    .to_string()
            } else {
                raw.trim_end_matches('/').to_string()
            };
            result.push((name, c::digest(data)));
        }
        at += 60 + size + (size & 1);
    }
    Ok(result)
}
fn counts(rows: impl IntoIterator<Item = (String, String)>) -> BTreeMap<(String, String), usize> {
    let mut out = BTreeMap::new();
    for row in rows {
        *out.entry(row).or_default() += 1
    }
    out
}
pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let prior = root.join("android-tcg-baseline-v1");
    let old = repo.join(".local/build-android");
    let base = repo.join(".local/src/xemu-android");
    let previous: serde_json::Value = c::read_json(&prior.join("build.identity.json"))?;
    if c::file_digest(&prior.join("libxemu.so"))?
        != "c2386913418acbb3d318ee266d6d618c533dd166b96a6e813a9a4bec63854832"
    {
        return Err("qualified parent core changed".into());
    }
    let new = root.join("android-gdb-features-v1");
    c::mkdir(&new)?;
    let source = new.join("source");
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(prior.join("source"))
            .arg(&source),
    )?;
    let mut pins = serde_json::Map::new();
    for name in [
        "scripts/feature_to_c.py",
        "gdb-xml/i386-32bit.xml",
        "stubs/gdbstub.c",
    ] {
        let bytes = output(
            Command::new("git")
                .arg("-C")
                .arg(&base)
                .args(["show", &format!("{PIN}:{name}")])
                .env("GIT_NO_LAZY_FETCH", "1"),
        )?;
        let target = new.join("upstream").join(name);
        fs::create_dir_all(target.parent().ok_or("source parent")?)?;
        c::write_new(&target, bytes.as_bytes())?;
        pins.insert(name.into(), json!(c::digest(bytes.as_bytes())));
    }
    let generated = output(
        Command::new("python3")
            .arg("-I")
            .arg(new.join("upstream/scripts/feature_to_c.py"))
            .arg(new.join("upstream/gdb-xml/i386-32bit.xml")),
    )?;
    // Reuse the actual pinned metadata type in the original native contract.
    let header = fs::read_to_string(source.join("include/exec/gdbstub.h"))?;
    let start = header
        .find("typedef struct GDBFeature {")
        .ok_or("metadata type")?;
    let end = start
        + header[start..]
            .find("} GDBFeature;")
            .ok_or("metadata end")?
        + "} GDBFeature;".len();
    let test = new.join("contract");
    fs::create_dir_all(test.join("qemu"))?;
    fs::create_dir_all(test.join("exec"))?;
    c::write_new(&test.join("qemu/osdep.h"), b"#include <stddef.h>\n")?;
    c::write_new(&test.join("exec/gdbstub.h"), &header.as_bytes()[start..end])?;
    let mut results = Vec::new();
    for (label, body, expected) in [
        (
            "before",
            fs::read_to_string(new.join("upstream/stubs/gdbstub.c"))?,
            1,
        ),
        ("after", generated.clone(), 0),
    ] {
        let input = test.join(format!("{label}.c"));
        let exe = test.join(label);
        c::write_new(&input, body.as_bytes())?;
        c::store::checked(
            Command::new("cc")
                .args(["-std=gnu11", "-Wall", "-Werror"])
                .arg(format!("-I{}", literal(&test)?))
                .arg(&input)
                .arg(repo.join("native/fixtures/gdb-feature-table.c"))
                .arg("-o")
                .arg(&exe),
        )?;
        let status = Command::new(&exe).status()?;
        let pass = status.code() == Some(expected);
        results
            .push(json!({"version":label,"exit":status.code(),"expected":expected,"passed":pass}));
        if !pass {
            return Err("original register metadata contract failed".into());
        }
    }
    fs::create_dir_all(source.join("stubs"))?;
    let subject = source.join("stubs/gdbstub.c");
    if subject.try_exists()? {
        fs::rename(&subject, source.join("stubs/gdbstub-preserved.c"))?
    }
    c::write_new(&subject, generated.as_bytes())?;
    let target = format!(
        "CMakeFiles/xemu_core.dir/{}/stubs/gdbstub.c.o",
        literal(&base)?.trim_start_matches('/')
    );
    let object = compile(
        &old,
        &source,
        &base,
        &new,
        "stubs/gdbstub.c",
        &target,
        "gdb-features",
    )?;
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let ar = llvm.join("llvm-ar");
    let original = fs::read(prior.join("libxemu_core.a"))?;
    let before = objects(&original)?;
    let duplicates = before.iter().filter(|(n, _)| n == "gdbstub.c.o").count();
    let mut found = None;
    for nth in 1..=duplicates {
        let dir = new.join(format!("member-{nth}"));
        c::mkdir(&dir)?;
        c::store::checked(
            Command::new(&ar)
                .current_dir(&dir)
                .args(["xN", &nth.to_string()])
                .arg(prior.join("libxemu_core.a"))
                .arg("gdbstub.c.o"),
        )?;
        let symbols = output(
            Command::new(llvm.join("llvm-nm"))
                .arg("--defined-only")
                .arg(dir.join("gdbstub.c.o")),
        )?;
        if symbols.lines().any(|l| l.ends_with(" gdb_static_features")) {
            if found.is_some() {
                return Err("ambiguous metadata object".into());
            }
            found = Some((nth, c::file_digest(&dir.join("gdbstub.c.o"))?));
        }
    }
    let (nth, removed) = found.ok_or("metadata object missing")?;
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
            .arg("gdbstub.c.o"),
    )?;
    c::store::checked(Command::new(&ar).arg("rD").arg(&archive).arg(&object))?;
    let after = objects(&fs::read(&archive)?)?;
    let mut expected = counts(before.clone());
    let key = ("gdbstub.c.o".into(), removed.clone());
    let count = expected.get_mut(&key).ok_or("removed object identity")?;
    *count -= 1;
    if *count == 0 {
        expected.remove(&key);
    }
    expected.insert(("gdb-features.c.o".into(), c::file_digest(&object)?), 1);
    if expected != counts(after) {
        return Err("unrelated archive object changed".into());
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
        &c::json(
            &json!({"source_pin":PIN,"bridge_abi":4,"source_sha256":c::file_digest(&repo.join("native/xbx_xemu_bridge.c"))?,"library_sha256":c::file_digest(&lib)?,"prior_library_sha256":previous["library_sha256"],"open_image_native_parent":previous["open_image_native_parent"],"inherited_repairs":previous,"metadata_pins":pins,"generated_metadata":c::digest(generated.as_bytes()),"original_contract":results,"removed_empty_feature_object":removed,"all_other_archive_objects_unchanged":true,"archive_member_count":before.len(),"recompiled_units":["stubs/gdbstub.c"],"whole_core_rebuild":false,"guest_execution_code_changed":false,"state_layout_changed":false,"old_checkpoints_retagged":false,"game_causality":false}),
        )?,
    )?;
    Ok(())
}
