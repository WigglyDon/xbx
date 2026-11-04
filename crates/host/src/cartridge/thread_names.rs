// SPDX-License-Identifier: MIT
//! Original pthread-role boundary regression against the retained thread entry.
use super::{
    self as c,
    vertex_cache::{function, pinned},
};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const UNIT: &str = "util/qemu-thread-posix.c";
const PATCH: &str = "patches/xemu-native-thread-names.patch";

pub(super) fn contract(repo: &Path, root: &Path) -> Result<(), Error> {
    let out = root.join("native-thread-names-contract-v1");
    c::mkdir(&out)?;
    let source = out.join("source");
    fs::create_dir_all(source.join("util"))?;
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
        c::write_new(
            &include,
            format!(
                "/* SPDX-License-Identifier: GPL-2.0-or-later */\n{}\n",
                function(text, "static void *qemu_thread_start(")?
            )
            .as_bytes(),
        )?;
        let binary = out.join(version);
        c::store::checked(
            Command::new("cc")
                .args([
                    "-std=gnu11",
                    "-O2",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-pthread",
                ])
                .arg(format!("-DSUBJECT=\"{}\"", include.display()))
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
            let expected = i32::from(version == "before" && enabled && label.len() >= 16);
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
            "patch":c::file_digest(&repo.join(PATCH))?,"fixture":c::file_digest(&repo.join("native/fixtures/thread-names.c"))?,
            "exact_retained_thread_entry_function":true,"real_host_pthread_name_and_join":true,
            "default_disabled_and_null_name_preserved":true,"guest_state_or_execution_changed":false,
            "Android_execution":false,"performance_cause_or_speedup_claimed":false,
            "results":results,"passed":true
        }))?,
    )?;
    Ok(())
}
