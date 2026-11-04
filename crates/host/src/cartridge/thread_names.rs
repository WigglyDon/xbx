// SPDX-License-Identifier: MIT
//! Original pthread-role boundary regression against the retained thread entry.
use super::{
    self as c,
    android_build::{compile, literal},
    vertex_cache::{function, pinned},
};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const UNIT: &str = "util/qemu-thread-posix.c";
const PATCH: &str = "patches/xemu-native-thread-names.patch";
const HOST_CONFIG: &str = "android/app/src/main/cpp/config-host.h";

pub(super) fn contract(repo: &Path, root: &Path) -> Result<(), Error> {
    let out = root.join("native-thread-names-contract-v2");
    c::mkdir(&out)?;
    let source = out.join("source");
    fs::create_dir_all(source.join("util"))?;
    let before = pinned(repo, UNIT)?;
    c::write_new(&source.join(UNIT), before.as_bytes())?;
    let before_config = pinned(repo, HOST_CONFIG)?;
    let before_config_path = out.join("before-config.h");
    c::write_new(&before_config_path, before_config.as_bytes())?;
    fs::create_dir_all(source.join("android/app/src/main/cpp"))?;
    c::write_new(&source.join(HOST_CONFIG), before_config.as_bytes())?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &out)
            .arg("apply")
            .arg(repo.join(PATCH)),
    )?;
    let after = fs::read_to_string(source.join(UNIT))?;
    let after_config_path = source.join(HOST_CONFIG);
    let mut results = Vec::new();
    for (version, text, config) in [
        ("before-posix", &before, None),
        ("before-android", &before, Some(&before_config_path)),
        ("after-posix", &after, None),
        ("after-android", &after, Some(&after_config_path)),
    ] {
        let include = out.join(format!("{version}.inc"));
        c::write_new(
            &include,
            format!(
                "/* SPDX-License-Identifier: GPL-2.0-or-later */\n{}\n",
                function(text, "static void *qemu_thread_start(")?
            )
            .as_bytes(),
        )?;
        let binary = out.join(version);
        let mut cc = Command::new("cc");
        cc.args([
            "-std=gnu11",
            "-O2",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-pthread",
        ]);
        if let Some(config) = config {
            cc.arg(format!("-DHOST_CONFIG=\"{}\"", config.display()));
        } else {
            cc.arg("-DCONFIG_PTHREAD_SETNAME_NP_W_TID=1");
        }
        c::store::checked(
            cc.arg(format!("-DSUBJECT=\"{}\"", include.display()))
                .arg(repo.join("native/fixtures/thread-names.c"))
                .arg("-o")
                .arg(&binary),
        )?;
        for (enabled, label) in [
            (true, "pgraph.vk.render"),
            (true, "pgraph.vk.compile"),
            (true, "pgraph.vk.submit"),
            (true, "nv2a.pfifo_thread"),
            (true, "mcpx.apu_thread"),
            (true, "CPU 0/TCG"),
            (false, "pgraph.vk.render"),
            (true, "null-name"),
        ] {
            let run = Command::new(&binary)
                .args([if enabled { "on" } else { "off" }, label])
                .output()?;
            let expected = i32::from(
                enabled
                    && label != "null-name"
                    && (version == "before-android"
                        || (version == "before-posix" && label.len() >= 16)),
            );
            let passed = run.status.code() == Some(expected);
            results.push(json!({"version":version,"enabled":enabled,"role":label,"exit":run.status.code(),"expected":expected,"passed":passed}));
            let label_file = label.replace('/', "_");
            c::write_new(
                &out.join(format!("{version}-{enabled}-{label_file}.stdout")),
                &run.stdout,
            )?;
            c::write_new(
                &out.join(format!("{version}-{enabled}-{label_file}.stderr")),
                &run.stderr,
            )?;
            if !passed {
                return Err("original native role-name control failed; retain evidence".into());
            }
        }
    }
    c::write_new(
        &out.join("result.json"),
        &c::json(&json!({
            "source_pin":"4967efd3ee594cc236869a45ee4cb396e25af1ac",
            "before":c::digest(before.as_bytes()),"after":c::digest(after.as_bytes()),
            "before_Android_config":c::digest(before_config.as_bytes()),
            "after_Android_config":c::file_digest(&after_config_path)?,
            "patch":c::file_digest(&repo.join(PATCH))?,"fixture":c::file_digest(&repo.join("native/fixtures/thread-names.c"))?,
            "exact_retained_thread_entry_function":true,"real_host_pthread_name_and_join":true,
            "default_disabled_and_null_name_preserved":true,"guest_state_or_execution_changed":false,
            "Android_execution":false,"performance_cause_or_speedup_claimed":false,
            "results":results,"passed":true
        }))?,
    )?;
    Ok(())
}

pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let proof: serde_json::Value =
        c::read_json(&root.join("native-thread-names-contract-v2/result.json"))?;
    if proof["passed"] != true || proof["patch"] != c::file_digest(&repo.join(PATCH))? {
        return Err("original pthread-name boundary proof required".into());
    }
    let android = root.join("android-demo-v1");
    let prior = android.join("android-present-surface-v1");
    let previous: serde_json::Value = c::read_json(&prior.join("build.identity.json"))?;
    if c::file_digest(&prior.join("libxemu.so"))?
        != "410e1493c2dc027a883e53ea6f88ed7cc3741f955b7647cdd8ee8ddf908c30f0"
    {
        return Err("qualified presentation core differs; preserve candidates".into());
    }
    let new = android.join("android-thread-names-v1");
    c::mkdir(&new)?;
    let source = new.join("source");
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(prior.join("source"))
            .arg(&source),
    )?;
    fs::create_dir_all(source.join("util"))?;
    if source.join(UNIT).exists() {
        return Err("unexpected existing native-name overlay".into());
    }
    c::write_new(&source.join(UNIT), pinned(repo, UNIT)?.as_bytes())?;
    // Restore only this unit's local header from retained Git objects, never
    // repopulate the retired source expansion or alter the donor checkout.
    let common = "util/qemu-thread-common.h";
    if !source.join(common).exists() {
        c::write_new(&source.join(common), pinned(repo, common)?.as_bytes())?;
    }
    if c::file_digest(&source.join(HOST_CONFIG))? != proof["before_Android_config"] {
        return Err("retained Android pthread configuration differs".into());
    }
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &new)
            .arg("apply")
            .arg(repo.join(PATCH)),
    )?;
    if c::file_digest(&source.join(UNIT))? != proof["after"]
        || c::file_digest(&source.join(HOST_CONFIG))? != proof["after_Android_config"]
    {
        return Err("native source differs from pthread boundary proof".into());
    }
    let old = repo.join(".local/build-android");
    let base = repo.join(".local/src/xemu-android");
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let archive = new.join("libxemu_core.a");
    c::store::checked(
        Command::new("cp")
            .arg("--reflink=always")
            .arg(prior.join("libxemu_core.a"))
            .arg(&archive),
    )?;
    let mut expected = super::gdb_features::objects(&fs::read(&archive)?)?;
    let original_count = expected.len();
    let member = "qemu-thread-posix.c.o";
    if expected.iter().filter(|(name, _)| name == member).count() != 1 {
        return Err("native thread object is absent or ambiguous".into());
    }
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
        "xbx-thread-names",
    )?;
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
    expected.retain(|(name, _)| name != member);
    expected.push(("xbx-thread-names.c.o".into(), c::file_digest(&object)?));
    expected.sort();
    let mut actual = super::gdb_features::objects(&fs::read(&archive)?)?;
    actual.sort();
    if actual != expected {
        return Err("unrelated native archive object changed".into());
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
            "source_pin":proof["source_pin"],"bridge_abi":4,
            "source_sha256":c::file_digest(&repo.join("native/xbx_xemu_bridge.c"))?,
            "library_sha256":c::file_digest(&lib)?,"prior_library_sha256":previous["library_sha256"],
            "open_image_native_parent":previous["open_image_native_parent"],
            "original_contract":proof,"inherited_repairs":previous,
            "all_other_archive_objects_unchanged":true,"unchanged_core_object_count":original_count-1,
            "recompiled_units":[UNIT],"whole_core_rebuild":false,"state_layout_changed":false,
            "old_checkpoints_retagged":false,"Android_execution":false,"performance_cause_established":false
        }))?,
    )?;
    Ok(())
}
