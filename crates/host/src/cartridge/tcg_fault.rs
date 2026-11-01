// SPDX-License-Identifier: MIT
//! Retained Android tier-1 condition-code state across potentially faulting IR.
use super::{
    self as c,
    android_build::{compile, literal, output},
};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const PIN: &str = "4967efd3ee594cc236869a45ee4cb396e25af1ac";
const PATCH: &str = "patches/xemu-tier1-fault-state.patch";

pub(super) fn contract(repo: &Path, root: &Path) -> Result<(), Error> {
    let out = root.join("android-tcg-fault-contract-v1");
    c::mkdir(&out)?;
    let source = out.join("source");
    fs::create_dir_all(source.join("tcg"))?;
    let before = output(
        Command::new("git")
            .arg("-C")
            .arg(repo.join(".local/src/xemu-android"))
            .args(["show", &format!("{PIN}:tcg/tier1-opt.c")])
            .env("GIT_NO_LAZY_FETCH", "1"),
    )?;
    c::write_new(&source.join("tcg/tier1-opt.c"), before.as_bytes())?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &out)
            .arg("apply")
            .arg(repo.join(PATCH)),
    )?;
    let after = fs::read_to_string(source.join("tcg/tier1-opt.c"))?;
    let mut results = Vec::new();
    for (label, subject) in [("before", &before), ("after", &after)] {
        // Keep every function body and original notice verbatim. Only replace
        // the surrounding includes/declarations with the controlled IR fixture.
        let subject = subject
            .lines()
            .filter(|l| !l.starts_with("#include"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let inc = out.join(format!("{label}.inc"));
        c::write_new(&inc, subject.as_bytes())?;
        let binary = out.join(label);
        c::store::checked(
            Command::new("cc")
                .args([
                    "-std=gnu11",
                    "-Wall",
                    "-Werror",
                    "-Wno-unused-but-set-variable",
                ])
                .arg(format!(
                    "-I{}",
                    literal(&root.join("android-serial-v1/source/include"))?
                ))
                .arg(format!("-DSUBJECT=\"{}\"", literal(&inc)?))
                .arg(repo.join("native/fixtures/tier1-fault-state.c"))
                .arg("-o")
                .arg(&binary),
        )?;
        for case in ["local-safe", "local-fault", "cross-safe", "cross-fault"] {
            let r = Command::new(&binary).arg(case).output()?;
            c::write_new(&out.join(format!("{label}-{case}.stdout")), &r.stdout)?;
            c::write_new(&out.join(format!("{label}-{case}.stderr")), &r.stderr)?;
            let expected = if label == "before" && case.ends_with("fault") {
                1
            } else {
                0
            };
            let passed = r.status.code() == Some(expected);
            results.push(json!({"version":label,"case":case,"exit":r.status.code(),"expected_exit":expected,"passed":passed}));
            if !passed {
                return Err("actual optimizer IR contract failed; preserve outputs".into());
            }
        }
    }
    c::write_new(
        &out.join("result.json"),
        &c::json(&json!({
            "source_pin":PIN,"subject_after":c::file_digest(&source.join("tcg/tier1-opt.c"))?,
            "patch":c::file_digest(&repo.join(PATCH))?,"fixture":c::file_digest(&repo.join("native/fixtures/tier1-fault-state.c"))?,
            "results":results,"passed":true,"unchanged_function_bodies":true,
            "controlled_IR_and_fault_observation":true,"actual_vCPU_or_Android_exception_delivery":false,
            "game_causality":false
        }))?,
    )?;
    Ok(())
}

pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    build_candidate(repo, root, false)
}
pub(super) fn baseline(repo: &Path, root: &Path) -> Result<(), Error> {
    build_candidate(repo, root, true)
}
fn build_candidate(repo: &Path, root: &Path, baseline: bool) -> Result<(), Error> {
    let proof: serde_json::Value =
        c::read_json(&root.join("android-tcg-fault-contract-v1/result.json"))?;
    if proof["passed"] != true || proof["patch"] != c::file_digest(&repo.join(PATCH))? {
        return Err("qualified actual-source IR contract required".into());
    }
    let prior = root.join(if baseline {
        "android-tcg-fault-v1"
    } else {
        "android-serial-v1"
    });
    let patch = if baseline {
        "patches/xemu-tier1-baseline.patch"
    } else {
        PATCH
    };
    let old = repo.join(".local/build-android");
    let base = repo.join(".local/src/xemu-android");
    let previous: serde_json::Value = c::read_json(&prior.join("build.identity.json"))?;
    if previous["source_pin"] != PIN
        || previous["library_sha256"] != c::file_digest(&prior.join("libxemu.so"))?
    {
        return Err("retained serial-corrected core differs".into());
    }
    let new = root.join(if baseline {
        "android-tcg-baseline-v1"
    } else {
        "android-tcg-fault-v1"
    });
    c::mkdir(&new)?;
    let source = new.join("source");
    c::store::checked(
        Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(prior.join("source"))
            .arg(&source),
    )?;
    let unit = if baseline {
        "accel/tcg/cpu-exec.c"
    } else {
        "tcg/tier1-opt.c"
    };
    fs::create_dir_all(source.join(unit).parent().ok_or("unit parent")?)?;
    let original = output(
        Command::new("git")
            .arg("-C")
            .arg(&base)
            .args(["show", &format!("{PIN}:{unit}")])
            .env("GIT_NO_LAZY_FETCH", "1"),
    )?;
    c::write_new(&source.join(unit), original.as_bytes())?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &new)
            .arg("apply")
            .arg(repo.join(patch)),
    )?;
    if !baseline && proof["subject_after"] != c::file_digest(&source.join(unit))? {
        return Err("built optimizer must equal qualified subject".into());
    }
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
    let stem = if baseline { "cpu-exec" } else { "tier1-opt" };
    let member = format!("{stem}.c.o");
    if output(Command::new(&ar).arg("t").arg(&core))?
        .lines()
        .filter(|v| *v == member)
        .count()
        != 1
    {
        return Err("retained optimizer archive member ambiguous".into());
    }
    let target = format!(
        "CMakeFiles/xemu_core.dir/{}/{unit}.o",
        literal(&base)?.trim_start_matches('/')
    );
    let object = compile(&old, &source, &base, &new, unit, &target, stem)?;
    c::store::checked(Command::new(&ar).arg("rD").arg(&core).arg(object))?;
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
    let open_parent = if baseline {
        previous["prior_library_sha256"].clone()
    } else {
        previous["library_sha256"].clone()
    };
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(&json!({
            "source_pin":PIN,"bridge_abi":4,"source_sha256":c::file_digest(&repo.join("native/xbx_xemu_bridge.c"))?,
            "library_sha256":c::file_digest(&lib)?,"prior_library_sha256":previous["library_sha256"],
            "patch":c::file_digest(&repo.join(patch))?,"subject":c::file_digest(&source.join(unit))?,"contract":proof,
            "extra_tier_promotion_disabled":baseline,"open_image_native_parent":open_parent,
            "inherited_repairs":previous,"recompiled_units":[unit],"whole_core_rebuild":false,
            "state_layout_changed":false,"old_checkpoints_retagged":false,"game_causality":false
        }))?,
    )?;
    Ok(())
}
