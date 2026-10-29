// SPDX-License-Identifier: MIT
//! Reconstruct only the unfinished I/O slice from retained public Git objects.
//! Historical native qualification is never recreated as a current receipt.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const PIN: &str = "9d8cd58ca19f49f5ecb4b7b55bfa194f4e4a5a96";
const BOOT: &str = "86f547387184d1001d377cce97d3756c8acf91cc";

/// Owner-authorized read-only retrieval under the installed device lease.
/// A failed or mismatched copy stays partial; no source removal or game launch.
pub(super) fn media(_repo: &Path, root: &Path) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;
    let lease: i32 = std::env::var("CODEX_ANDROID_LEASE_FD")?.parse()?;
    if lease < 3 {
        return Err("installed device custody required".into());
    }
    let input = std::env::var("XBX_REFERENCE_MEDIA_SOURCE")?;
    let expected = std::env::var("XBX_REFERENCE_MEDIA_EXPECTED_SHA256")?;
    if !input.starts_with("/sdcard/Download/")
        || input.contains(['\n', '\r'])
        || expected.len() != 64
        || !expected.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("exact retained private media identity required".into());
    }
    let dir = root.join("media");
    c::mkdir(&dir)?;
    let partial = dir.join("selected-disc.iso.partial");
    let status = Command::new("android-device")
        .args(["adb", "pixel4-lab", "--", "pull"])
        .arg(input)
        .arg(&partial)
        .status()?;
    if partial.exists() {
        fs::File::open(&partial)?.sync_all()?;
    }
    if !status.success() {
        return Err("stock read-only copy failed; preserve partial".into());
    }
    let metadata = fs::symlink_metadata(&partial)?;
    if !metadata.is_file()
        || metadata.len() != 7_825_162_240
        || c::file_digest(&partial)? != expected.to_ascii_lowercase()
    {
        return Err("copied media identity differs; preserve without launch".into());
    }
    fs::set_permissions(&partial, fs::Permissions::from_mode(0o400))?;
    fs::File::open(&partial)?.sync_all()?;
    // The enclosing build owner holds the project writer lease and this
    // operation exclusively created the private staging directory. Keep one
    // logical image throughout publication, including output-quota sampling.
    let final_path = dir.join("selected-disc.iso");
    if final_path.try_exists()? {
        return Err("preserve existing media destination; no overwrite".into());
    }
    fs::rename(&partial, &final_path)?;
    c::write_new(
        &dir.join("verified.identity.json"),
        &c::json(&json!({
            "bytes":metadata.len(),"sha256":expected.to_ascii_lowercase(),
            "phone_input_modified":false,"full_hash_verified":true,"game_launched":false
        }))?,
    )?;
    fs::File::open(&dir)?.sync_all()?;
    Ok(())
}

fn export(depot: &Path, pin: &str, dest: &Path, paths: &[&str]) -> Result<String, Error> {
    c::mkdir(dest)?;
    let archive = dest.with_extension("tar");
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&archive)?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(depot)
            .args(["archive", "--format=tar", pin])
            .args(paths)
            .env("GIT_NO_LAZY_FETCH", "1")
            .stdout(file),
    )?;
    c::store::checked(
        Command::new("tar")
            .args(["-x", "--no-same-owner", "-f"])
            .arg(&archive)
            .arg("-C")
            .arg(dest),
    )?;
    Ok(c::file_digest(&archive)?)
}

fn patch(repo: &Path, source: &Path, name: &str, hash: &str) -> Result<(), Error> {
    let input = repo.join(name);
    if c::file_digest(&input)? != hash {
        return Err("reconstruction patch identity differs".into());
    }
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(source)
            // The output can live inside an ignored directory of the product
            // checkout. Apply as a standalone tree, not its enclosing Git repo.
            .env(
                "GIT_CEILING_DIRECTORIES",
                source.parent().ok_or("source parent")?.canonicalize()?,
            )
            .arg("apply")
            .arg(input),
    )?;
    Ok(())
}

pub(super) fn io_images(repo: &Path, root: &Path) -> Result<(), Error> {
    let depot = std::path::PathBuf::from(std::env::var("XBX_OPEN_SOURCE")?);
    let boot = std::path::PathBuf::from(std::env::var("XBX_BOOT_SOURCE")?);
    let time = Command::new("git")
        .arg("-C")
        .arg(&depot)
        .args(["show", "-s", "--format=%ct", PIN])
        .env("GIT_NO_LAZY_FETCH", "1")
        .output()?;
    if !time.status.success() {
        return Err("public source date unavailable".into());
    }
    let epoch = String::from_utf8(time.stdout)?;
    let epoch = epoch.trim();
    let _: u64 = epoch.parse()?;
    let lock: Value = c::read_json(&repo.join("cartridge-io-errors.lock.json"))?;
    let startup: Value = c::read_json(&repo.join("open-startup.lock.json"))?;
    let npx: Value = c::read_json(&repo.join("cartridge-npx-owner.lock.json"))?;
    let flush: Value = c::read_json(&repo.join("cartridge-flush-io.lock.json"))?;
    let probe: Value = c::read_json(&repo.join("cartridge-buffer-probe.lock.json"))?;
    let attempt = std::env::var("XBX_REBUILD_ATTEMPT").unwrap_or_else(|_| "v1".into());
    if attempt.is_empty()
        || !attempt
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("invalid reconstruction attempt label".into());
    }
    let dir = root.join(format!("rebuilt-io-images-{attempt}"));
    c::mkdir(&dir)?;
    let source = dir.join("source");
    let archive = export(
        &depot,
        PIN,
        &source,
        &["kernel", "tools", "LICENSE", "README.md"],
    )?;
    fs::create_dir_all(source.join("third_party"))?;
    let boot_archive = export(&boot, BOOT, &source.join("third_party/cromwell"), &[])?;
    if boot_archive != "280b2b1dce7d73ba40d9e4d027fb8f54624fa7db0724e2f431d813f85ef29dcb" {
        return Err("reconstructed public boot archive differs".into());
    }
    for (name, hash) in [
        (
            "optional_fixed_phase_patch",
            "optional_fixed_phase_patch_sha256",
        ),
        ("required_reads_patch", "required_reads_patch_sha256"),
    ] {
        patch(
            repo,
            &source,
            startup[name].as_str().ok_or("patch name")?,
            startup[hash].as_str().ok_or("patch hash")?,
        )?;
    }
    // The accepted corrected manifest changed only the loader blob. Verify it
    // before composing later functional repairs, without inventing old outputs.
    let listing = Command::new("git")
        .arg("-C")
        .arg(&depot)
        .args([
            "ls-tree",
            "-r",
            PIN,
            "--",
            "kernel",
            "tools",
            "LICENSE",
            "README.md",
        ])
        .env("GIT_NO_LAZY_FETCH", "1")
        .output()?;
    if !listing.status.success() {
        return Err("public source listing failed".into());
    }
    let loader = Command::new("git")
        .arg("hash-object")
        .arg(source.join("kernel/src/xe/loader.cpp"))
        .output()?;
    if !loader.status.success() {
        return Err("corrected loader identity failed".into());
    }
    let blob = String::from_utf8(loader.stdout)?;
    let manifest = String::from_utf8(listing.stdout)?
        .lines()
        .map(|line| {
            if line.ends_with("\tkernel/src/xe/loader.cpp") {
                format!("100644 blob {}\tkernel/src/xe/loader.cpp", blob.trim())
            } else {
                line.into()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
        + "\n";
    if c::digest(manifest.as_bytes()) != lock["retained_corrected_source_objects_sha256"] {
        return Err("accepted corrected source manifest differs".into());
    }
    for (name, hash) in [
        (&npx["functional_patch"], &npx["functional_patch_sha256"]),
        (&flush["patch"], &flush["patch_sha256"]),
        (&probe["patch"], &probe["patch_sha256"]),
    ] {
        patch(
            repo,
            &source,
            name.as_str().ok_or("functional patch")?,
            hash.as_str().ok_or("functional hash")?,
        )?;
    }
    for (name, expected) in lock["accepted_source"]
        .as_object()
        .ok_or("accepted source")?
    {
        if c::file_digest(&source.join(name))? != *expected {
            return Err("reconstructed accepted functional source differs".into());
        }
    }
    let mut images = Vec::new();
    for (name, expected) in [
        ("baseline", resources::BUFFER_PROBE_LOGGING_OPEN),
        ("reporting", resources::IO_ERRORS_LOGGING_OPEN),
    ] {
        if name == "reporting" {
            patch(
                repo,
                &source,
                lock["patch"].as_str().ok_or("reporting patch")?,
                lock["patch_sha256"].as_str().ok_or("reporting hash")?,
            )?;
            for (name, hashes) in lock["changed"].as_object().ok_or("reporting source")? {
                if c::file_digest(&source.join(name))? != hashes["after"] {
                    return Err("reconstructed reporting source differs".into());
                }
            }
        }
        let out = dir.join(name);
        c::mkdir(&out)?;
        // startup.cpp includes __DATE__/__TIME__. Bind them to public source
        // time rather than claiming a wall-clock rebuild is the old binary.
        let llvm = super::buffer_probe::compile_image(&source, &out, Some(epoch))?;
        let actual = c::file_digest(&out.join("bios.bin"))?;
        let identity = json!({"firmware":actual,"expected_historical_firmware":expected,
            "historical_firmware_equal":actual==expected,"kernel":c::file_digest(&out.join("xboxkrnl.exe"))?,
            "map":c::file_digest(&out.join("xboxkrnl.map"))?,"compiler":c::file_digest(&llvm.join("clang"))?,
            "linker":c::file_digest(&llvm.join("lld-link"))?,"source_date_epoch":epoch,"native_execution":false});
        c::write_new(&out.join("identity.json"), &c::json(&identity)?)?;
        images.push(identity);
    }
    c::write_new(
        &dir.join("result.json"),
        &c::json(&json!({
            "source":PIN,"cromwell":BOOT,"source_archive":archive,"boot_archive":boot_archive,
        "images":images,"compiler_jobs":1,"serial":true,"trace_imports":false,"source_date_epoch":epoch,
        "new_candidates_require_native_qualification":true,"ordinary_runtime_default_changed":false,
            "historical_experiments_restarted":false,"native_conformance_passed":false,"Fable_observed":false
        }))?,
    )?;
    Ok(())
}

fn full_native(repo: &Path, root: &Path, old: &Path, retained: &Path) -> Result<(), Error> {
    let dir = root.join("rebuilt-runtime-v1");
    c::mkdir(&dir)?;
    let source = dir.join("source");
    let archive = export(
        retained,
        "03b988c9d76553b33ed712ba00822ffdf49f0e7d",
        &source,
        &[],
    )?;
    let reference: Value = c::read_json(&repo.join("reference-core.lock.json"))?;
    let ramht: Value = c::read_json(&repo.join("ramht-core.lock.json"))?;
    let runtime: Value = c::read_json(&repo.join("cartridge-runtime.lock.json"))?;
    let engine = &runtime["engine"];
    for (name, hash) in [
        (&ramht["base_patch"], &ramht["base_patch_sha256"]),
        (
            &ramht["compatibility_patch"],
            &ramht["compatibility_patch_sha256"],
        ),
        (&engine["headless_patch"], &engine["headless_patch_sha256"]),
        (
            &engine["paused_capture_patch"],
            &engine["paused_capture_patch_sha256"],
        ),
    ] {
        patch(
            repo,
            &source,
            name.as_str().ok_or("native patch")?,
            hash.as_str().ok_or("native patch pin")?,
        )?;
    }
    let x87 = repo.join("patches/xemu-x87-instruction-boundary.patch");
    patch(
        repo,
        &source,
        "patches/xemu-x87-instruction-boundary.patch",
        &c::file_digest(&x87)?,
    )?;
    let inputs = fs::read_to_string(old.join("dependencies.identity"))?;
    let mut dependencies = Vec::new();
    for line in inputs.lines() {
        let (name, pin) = line.split_once('=').ok_or("dependency receipt")?;
        if name.is_empty()
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
            || pin.len() != 40
            || !pin.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("dependency receipt path or pin differs".into());
        }
        let original = retained.join("subprojects").join(name);
        let target = source.join("subprojects").join(name);
        let hash = export(&original, pin, &target, &[])?;
        dependencies.push(json!({"name":name,"pin":pin,"archive":hash}));
    }
    stage_packages(retained, &source)?;
    let product =
        std::path::PathBuf::from(std::env::var("XBX_RETAINED_PRODUCT")?).canonicalize()?;
    let dsp = product
        .join(".local/build-desktop/dsp/x86_64-unknown-linux-gnu/release/libdsp56300_emu_ffi.a");
    if c::file_digest(&dsp)? != "5e78afecce79231338bd6c1838a55d300886de3be4e38f3f82229013ac23f1a7" {
        return Err("retained DSP library differs".into());
    }
    let build = dir.join("build");
    c::mkdir(&build)?;
    let python = product.join(".local/tools/bin/python3");
    let pc = product.join(".local/host-sysroot/usr/lib64/pkgconfig");
    let share = product.join(".local/host-sysroot/usr/share/pkgconfig");
    c::write_new(
        &dir.join("inputs.json"),
        &c::json(&json!({
            "source":"03b988c9d76553b33ed712ba00822ffdf49f0e7d","archive":archive,
            "reference_lock":reference,"dependencies":dependencies,"dsp":c::file_digest(&dsp)?,
            "compiler_jobs":1,"old_build_mutated":false,"retired_object_bytes":315813888,
            "reconstruction_only":true,"native_qualified":false
        }))?,
    )?;
    // Selected dependencies come from retained Git objects. Missing sources
    // refuse; configure cannot download an unbudgeted dependency graph.
    let status = Command::new(source.join("configure"))
        .current_dir(&build)
        .env("GIT_CEILING_DIRECTORIES", &dir)
        .env(
            "PKG_CONFIG_PATH",
            format!("{}:{}", pc.display(), share.display()),
        )
        .env("PIP_CACHE_DIR", dir.join("pip-cache"))
        .env(
            "XBX_DSP_SOURCE",
            product.join(".local/build-android/_deps/dsp56300_src-src"),
        )
        .env("XBX_DSP_LIBRARY", &dsp)
        .arg(format!("--python={}", python.display()))
        .args([
            "--target-list=i386-softmmu",
            "--extra-cflags=-DXBOX=1",
            "--disable-werror",
            "--disable-docs",
            "--disable-tools",
            "--disable-guest-agent",
            "--disable-rust",
            "--disable-download",
        ])
        .status()?;
    if !status.success() {
        return Err("current-runtime configure failed; preserve source and logs".into());
    }
    let site = Command::new(&python)
        .args([
            "-c",
            "import sysconfig;print(sysconfig.get_paths()['purelib'])",
        ])
        .output()?;
    if !site.status.success() {
        return Err("retained Python tool path unavailable".into());
    }
    let status = Command::new("ninja")
        .arg("-C")
        .arg(&build)
        .args(["-j1", "qemu-system-i386"])
        .env("GIT_CEILING_DIRECTORIES", &dir)
        .env("PYTHONPATH", String::from_utf8(site.stdout)?.trim())
        .status()?;
    if !status.success() {
        return Err("current-runtime compilation failed; preserve intermediates".into());
    }
    fs::File::open(build.join("qemu-system-i386"))?.sync_all()?;
    c::write_new(
        &dir.join("identity.json"),
        &c::json(&json!({
            "core":c::file_digest(&build.join("qemu-system-i386"))?,
            "inputs":c::file_digest(&dir.join("inputs.json"))?,"compiler_jobs":1,
            "accepted_repairs_reused":true,"native_qualified":false,"ordinary_default_changed":false
        }))?,
    )?;
    Ok(())
}

fn stage_packages(retained: &Path, source: &Path) -> Result<(), Error> {
    fn copy_tree(from: &Path, to: &Path) -> Result<(), Error> {
        fs::create_dir_all(to)?;
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let target = to.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                copy_tree(&entry.path(), &target)?;
            } else if entry.file_type()?.is_file() {
                let content = fs::read(entry.path())?;
                if target.exists() {
                    if fs::read(&target)? != content {
                        return Err("preserve differing package overlay".into());
                    }
                } else {
                    c::write_new(&target, &content)?;
                }
            } else {
                return Err("package overlay is not a regular source file".into());
            }
        }
        Ok(())
    }
    for entry in fs::read_dir(source.join("subprojects"))? {
        let path = entry?.path();
        if path.extension().is_none_or(|s| s != "wrap") {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        let fields: std::collections::BTreeMap<_, _> = text
            .lines()
            .filter_map(|s| s.split_once('='))
            .map(|(k, v)| (k.trim(), v.trim()))
            .collect();
        if let Some(name) = fields.get("patch_directory") {
            let target = source
                .join("subprojects")
                .join(path.file_stem().ok_or("wrap name")?);
            if target.is_dir() {
                copy_tree(&source.join("subprojects/packagefiles").join(name), &target)?;
            }
        }
        for kind in ["source", "patch"] {
            if let (Some(name), Some(hash)) = (
                fields.get(format!("{kind}_filename").as_str()),
                fields.get(format!("{kind}_hash").as_str()),
            ) {
                if name.contains('/') || name.contains("..") {
                    return Err("package cache filename".into());
                }
                let from = retained.join("subprojects/packagecache").join(name);
                if !from.is_file() {
                    continue;
                }
                if c::file_digest(&from)? != *hash {
                    return Err("retained package checksum differs".into());
                }
                let to = source.join("subprojects/packagecache").join(name);
                fs::create_dir_all(to.parent().ok_or("cache parent")?)?;
                if to.exists() {
                    if c::file_digest(&to)? != *hash {
                        return Err("preserve differing package cache".into());
                    }
                } else {
                    c::write_new(&to, &fs::read(from)?)?;
                }
            }
        }
    }
    Ok(())
}

/// Reuse the surviving reference objects; rebuild only the four accepted
/// repairs lost with the external runtime. Never write into the donor build.
pub(super) fn native(repo: &Path, root: &Path) -> Result<(), Error> {
    let old = std::path::PathBuf::from(std::env::var("XBX_REFERENCE_BUILD")?).canonicalize()?;
    let retained =
        std::path::PathBuf::from(std::env::var("XBX_REFERENCE_SOURCE")?).canonicalize()?;
    for path in [repo, root, &old, &retained] {
        if !path
            .as_os_str()
            .as_encoded_bytes()
            .iter()
            .all(|v| v.is_ascii_alphanumeric() || b"/_-.".contains(v))
        {
            return Err("unsupported native reconstruction path".into());
        }
    }
    let ramht: Value = c::read_json(&repo.join("ramht-core.lock.json"))?;
    if c::file_digest(&old.join("qemu-system-i386"))? != ramht["retained_baseline_core_sha256"] {
        return Err("surviving reference build differs".into());
    }
    // Cache retirement preserved the executable but removed compiler objects.
    // Reconstruct only the current corrected runtime in a new directory.
    if !old.join("libsystem.a.p/ui_xemu.c.o").is_file() {
        return full_native(repo, root, &old, &retained);
    }
    let db = old.join("compile_commands.json");
    if fs::metadata(&db)?.len() > 16 * 1024 * 1024 {
        return Err("compiler database too large".into());
    }
    let commands: Vec<Value> = serde_json::from_slice(&fs::read(&db)?)?;
    let dir = root.join("rebuilt-native-v1");
    c::mkdir(&dir)?;
    let source = dir.join("source");
    c::mkdir(&source)?;
    let subjects = [
        (
            "ui/xemu.c",
            "libsystem.a.p/ui_xemu.c.o",
            "patches/xemu-headless-session.patch",
        ),
        (
            "migration/migration-hmp-cmds.c",
            "libsystem.a.p/migration_migration-hmp-cmds.c.o",
            "patches/xemu-paused-savevm.patch",
        ),
        (
            "target/i386/tcg/translate.c",
            "libqemu-i386-softmmu.a.p/target_i386_tcg_translate.c.o",
            "patches/xemu-x87-instruction-boundary.patch",
        ),
        (
            "hw/xbox/nv2a/pfifo.c",
            "libqemu-i386-softmmu.a.p/hw_xbox_nv2a_pfifo.c.o",
            "patches/xemu-reference-ramht.patch",
        ),
    ];
    let mut identities = Vec::new();
    let mut replacements = Vec::new();
    let mut rules = String::new();
    for (i, (name, subject, repair)) in subjects.iter().enumerate() {
        let original = retained.join(name);
        let content = fs::read(&original)?;
        let public = Command::new("git")
            .arg("-C")
            .arg(&retained)
            .args([
                "show",
                &format!("03b988c9d76553b33ed712ba00822ffdf49f0e7d:{name}"),
            ])
            .env("GIT_NO_LAZY_FETCH", "1")
            .output()?;
        if !public.status.success() || public.stdout != content {
            return Err("retained native unit differs from its public pin".into());
        }
        let target = source.join(name);
        fs::create_dir_all(target.parent().ok_or("native source parent")?)?;
        c::write_new(&target, &content)?;
        let hash = c::file_digest(&repo.join(repair))?;
        patch(repo, &source, repair, &hash)?;
        let cc = commands
            .iter()
            .find(|v| v["output"] == *subject)
            .ok_or("retained compile command")?;
        let file = cc["file"].as_str().ok_or("compile source")?;
        let command = cc["command"].as_str().ok_or("compile command")?;
        if command.matches(file).count() != 1 || cc["directory"].as_str() != old.to_str() {
            return Err("retained compile path differs".into());
        }
        let object = dir.join(format!("unit-{i}.o"));
        let command = command
            .replace(&format!("{subject}.d"), &format!("{}.d", object.display()))
            .replace(subject, object.to_str().ok_or("native object")?)
            .replace(file, target.to_str().ok_or("native source")?);
        let command = format!(
            "{command} -I{}",
            original.parent().ok_or("include parent")?.display()
        );
        rules.push_str(&format!(
            "rule compile_{i}\n command = {command}\nbuild {}: compile_{i} {}\n",
            object.display(),
            target.display()
        ));
        replacements.push((*subject, object));
        identities.push(json!({"file":name,"public_before":c::digest(&content),"after":c::file_digest(&target)?,"patch":hash}));
    }
    let ninja = fs::read_to_string(old.join("build.ninja"))?;
    let edge = ninja
        .split("\n\n")
        .find(|s| s.starts_with("build qemu-system-i386:"))
        .ok_or("retained link edge")?;
    let mut inputs = edge
        .lines()
        .next()
        .and_then(|s| s.strip_prefix("build qemu-system-i386: cpp_LINKER_RSP "))
        .and_then(|s| s.split(" | ").next())
        .ok_or("retained link inputs")?
        .to_owned();
    for (subject, object) in replacements {
        if inputs.split_whitespace().filter(|p| *p == subject).count() != 1 {
            return Err("retained link subject differs".into());
        }
        inputs = inputs.replace(subject, object.to_str().ok_or("object path")?);
    }
    let link = edge
        .lines()
        .find_map(|s| s.strip_prefix(" LINK_ARGS = "))
        .ok_or("retained link flags")?
        .replace(
            &format!(":{}", repo.join(".local/host-sysroot/usr/lib64").display()),
            "",
        );
    rules.push_str(&format!("rule link_core\n command = c++ -m64 @$out.rsp\n rspfile = $out.rsp\n rspfile_content = -o $out $in {link}\nbuild {}: link_core {inputs}\n",dir.join("core").display()));
    c::write_new(&dir.join("build.ninja"), rules.as_bytes())?;
    c::store::checked(
        Command::new("ninja")
            .current_dir(&old)
            .arg("-f")
            .arg(dir.join("build.ninja"))
            .args(["-j", "1"])
            .arg(dir.join("core")),
    )?;
    fs::File::open(dir.join("core"))?.sync_all()?;
    c::write_new(
        &dir.join("identity.json"),
        &c::json(&json!({
            "public_source":"03b988c9d76553b33ed712ba00822ffdf49f0e7d","retained_reference":ramht["retained_baseline_core_sha256"],
            "core":c::file_digest(&dir.join("core"))?,"recompiled_units":identities,
            "retained_commands":c::file_digest(&db)?,"retained_ninja":c::file_digest(&old.join("build.ninja"))?,
            "compiler_jobs":1,"prior_build_modified":false,"native_qualified":false,"ordinary_default_changed":false
        }))?,
    )?;
    Ok(())
}
