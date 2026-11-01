// SPDX-License-Identifier: MIT
//! Targeted Android UART source lifetime repair; retain every prior artifact.
use super::{
    self as c,
    android_build::{compile, literal, output},
};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;

fn function<'a>(source: &'a str, marker: &str) -> Result<&'a str, Error> {
    let start = source.find(marker).ok_or("retained function absent")?;
    let end = source[start..]
        .find("\n}\n")
        .ok_or("function boundary absent")?;
    Ok(&source[start..start + end + 3])
}

/// Real GLib contexts/socket backpressure and unchanged native function bodies;
/// controlled declarations qualify this boundary, never an Android ABI or game.
pub(super) fn contract(repo: &Path, root: &Path) -> Result<(), Error> {
    use std::os::unix::process::ExitStatusExt;
    let base = repo.join(".local/src/xemu-android");
    let changed = root.join("android-serial-v1/source");
    let out = root.join("android-serial-contract-v1");
    c::mkdir(&out)?;
    let flags = output(Command::new("pkg-config").args(["--cflags", "--libs", "glib-2.0"]))?;
    let mut results = Vec::new();
    for label in ["before", "after"] {
        let read = |path: &str| -> Result<String, Error> {
            if label == "after" {
                return Ok(fs::read_to_string(changed.join(path))?);
            }
            output(
                Command::new("git")
                    .arg("-C")
                    .arg(&base)
                    .args([
                        "show",
                        &format!("4967efd3ee594cc236869a45ee4cb396e25af1ac:{path}"),
                    ])
                    .env("GIT_NO_LAZY_FETCH", "1"),
            )
        };
        let serial = read("hw/char/serial.c")?;
        let frontend = read("chardev/char-fe.c")?;
        let mut subject = format!(
            "{}\n{}\n",
            serial.split("#include").next().ok_or("serial notice")?,
            frontend.split("#include").next().ok_or("frontend notice")?
        );
        for line in serial
            .lines()
            .filter(|l| l.starts_with("#define UART_") || l.starts_with("#define MAX_XMIT_RETRY"))
        {
            subject.push_str(line);
            subject.push('\n');
        }
        if label == "after" {
            subject.push_str(function(
                &frontend,
                "GSource *qemu_chr_fe_add_watch_source(",
            )?);
        }
        subject.push_str(function(&frontend, "guint qemu_chr_fe_add_watch(")?);
        if label == "after" {
            subject.push_str(function(&serial, "static void serial_remove_watch(")?);
        }
        for marker in [
            "static gboolean serial_watch_cb(",
            "static void serial_xmit(SerialState *s)\n{",
            "static void serial_reset(",
        ] {
            subject.push_str(function(&serial, marker)?);
        }
        let include = out.join(format!("{label}.inc"));
        c::write_new(&include, subject.as_bytes())?;
        let binary = out.join(label);
        c::store::checked(
            Command::new("cc")
                .args([
                    "-std=gnu11",
                    "-Wall",
                    "-Werror",
                    "-Wno-cast-function-type",
                    "-Wno-unused-function",
                ])
                .arg(format!("-DSUBJECT=\"{}\"", literal(&include)?))
                .arg(repo.join("native/fixtures/serial-watch-context.c"))
                .arg("-o")
                .arg(&binary)
                .args(flags.split_whitespace()),
        )?;
        for case in ["default-reset", "retry", "custom-reset"] {
            let result = Command::new(&binary).arg(case).output()?;
            c::write_new(&out.join(format!("{label}-{case}.stdout")), &result.stdout)?;
            c::write_new(&out.join(format!("{label}-{case}.stderr")), &result.stderr)?;
            let red = label == "before" && case == "custom-reset";
            let error = String::from_utf8_lossy(&result.stderr);
            let pass = if red {
                result.status.signal() == Some(6)
                    && error.contains("serial_xmit")
                    && error.contains("UART_LSR_TEMT")
            } else {
                result.status.success()
            };
            results.push(
                json!({"version":label,"case":case,"exit":result.status.code(),
                "signal":result.status.signal(),"expected_result":pass}),
            );
            if !pass {
                return Err(
                    "actual serial-context regression/control failed; preserve outputs".into(),
                );
            }
        }
    }
    c::write_new(
        &out.join("result.json"),
        &c::json(&json!({
            "source_pin":"4967efd3ee594cc236869a45ee4cb396e25af1ac","cases":results,
            "real_GLib_and_socket_backpressure":true,"unchanged_function_bodies":true,
            "controlled_declarations_not_Android_ABI":true,"game_executed":false
        }))?,
    )?;
    Ok(())
}

pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let prior = root.join("android-demo-native-v2");
    let old = repo.join(".local/build-android");
    let base = repo.join(".local/src/xemu-android");
    let previous: serde_json::Value = c::read_json(&prior.join("build.identity.json"))?;
    if previous["library_sha256"] != c::file_digest(&prior.join("libxemu.so"))?
        || previous["source_pin"] != "4967efd3ee594cc236869a45ee4cb396e25af1ac"
    {
        return Err("retained Android integration differs".into());
    }
    let new = root.join("android-serial-v1");
    c::mkdir(&new)?;
    let source = new.join("source");
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(prior.join("source"))
            .arg(&source),
    )?;
    // SerialState embeds a host-only source pointer. Recompile every retained
    // serial device embedding, not only the UART implementation itself.
    let units = [
        "chardev/char-fe.c",
        "hw/char/serial.c",
        "hw/char/serial-isa.c",
        "hw/char/serial-mm.c",
        "hw/char/serial-pci.c",
        "hw/char/serial-pci-multi.c",
        "hw/xbox/lpc47m157.c",
    ];
    for path in units {
        let data = if base.join(path).is_file() {
            fs::read(base.join(path))?
        } else {
            output(
                Command::new("git")
                    .arg("-C")
                    .arg(&base)
                    .args(["show", &format!("HEAD:{path}")])
                    .env("GIT_NO_LAZY_FETCH", "1"),
            )?
            .into_bytes()
        };
        let to = source.join(path);
        fs::create_dir_all(to.parent().ok_or("source parent")?)?;
        c::write_new(&to, &data)?;
    }
    let patch = repo.join("patches/xemu-serial-watch-source.patch");
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &new)
            .arg("apply")
            .arg(&patch),
    )?;
    let core = new.join("libxemu_core.a");
    c::store::checked(
        Command::new("cp")
            .arg("--reflink=always")
            .arg(prior.join("libxemu_core.a"))
            .arg(&core),
    )?;
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    let ar = llvm.join("llvm-ar");
    let members = output(Command::new(&ar).arg("t").arg(&core))?;
    let mut identities = serde_json::Map::new();
    for path in units {
        let member = format!(
            "{}.o",
            Path::new(path)
                .file_name()
                .ok_or("unit name")?
                .to_str()
                .ok_or("unit encoding")?
        );
        if members.lines().filter(|line| *line == member).count() != 1 {
            return Err("retained serial member ambiguous".into());
        }
        let target = format!(
            "CMakeFiles/xemu_core.dir/{}/{path}.o",
            literal(&base)?.trim_start_matches('/')
        );
        let stem = Path::new(path)
            .file_stem()
            .ok_or("unit stem")?
            .to_str()
            .ok_or("stem encoding")?;
        let object = compile(&old, &source, &base, &new, path, &target, stem)?;
        c::store::checked(Command::new(&ar).arg("rD").arg(&core).arg(&object))?;
        identities.insert(path.into(), json!(c::file_digest(&source.join(path))?));
    }
    for path in ["include/hw/char/serial.h", "include/chardev/char-fe.h"] {
        identities.insert(path.into(), json!(c::file_digest(&source.join(path))?));
    }
    let lib = new.join("libxemu.so");
    let link = fs::read_to_string(prior.join("link.command"))?
        .replace(&literal(&prior.join("libxemu.so"))?, &literal(&lib)?)
        .replace(
            &literal(&prior.join("link.d"))?,
            &literal(&new.join("link.d"))?,
        )
        .replace(&literal(&prior.join("libxemu_core.a"))?, &literal(&core)?);
    c::write_new(&new.join("link.command"), link.as_bytes())?;
    c::store::checked(Command::new("sh").current_dir(&old).arg("-c").arg(link))?;
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(&json!({
            "source_pin":previous["source_pin"],"bridge_abi":4,
            "source_sha256":c::file_digest(&repo.join("native/xbx_xemu_bridge.c"))?,
            "library_sha256":c::file_digest(&lib)?,
            "prior_library_sha256":previous["library_sha256"],
            "serial_watch_patch_sha256":c::file_digest(&patch)?,
            "recompiled_units":units,"source_files_after_patch":identities,
            "inherited_accepted_repairs":previous,"whole_core_rebuild":false,
            "serialized_UART_fields_changed":false,"new_host_source_pointer_serialized":false,
            "old_checkpoints_retagged":false,"device_qualified":false
        }))?,
    )?;
    Ok(())
}
