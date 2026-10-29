// SPDX-License-Identifier: MIT
//! Current local reconstruction uses the accepted product owners and fixtures.
//! No historical workspace, profiles or qualification markers are fabricated.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::{fs, path::Path};
type Error = Box<dyn std::error::Error>;
const DEMAND: &str = "ccc5d1c6f07024ff18c95e7e3f3d4508c92a4c24f601d14b06ca96777cef7126";

fn package(root: &Path, reporting: bool) -> Result<resources::Resources, Error> {
    let native = root.join("rebuilt-runtime-v1");
    let identity: Value = c::read_json(&native.join("identity.json"))?;
    let core = native.join("build/qemu-system-i386");
    if identity["core"] != resources::LOCAL_IO_CORE
        || c::file_digest(&core)? != resources::LOCAL_IO_CORE
        || identity["inputs"] != c::file_digest(&native.join("inputs.json"))?
    {
        return Err("current reconstructed native identity differs".into());
    }
    let firmware = root
        .join("rebuilt-io-images-v3")
        .join(if reporting { "reporting" } else { "baseline" })
        .join("bios.bin");
    let expected = if reporting {
        resources::LOCAL_IO_OPEN
    } else {
        resources::LOCAL_BASE_OPEN
    };
    if c::file_digest(&firmware)? != expected {
        return Err("current reconstructed open image differs".into());
    }
    let dir = root.join(if reporting {
        "local-io-package-reporting"
    } else {
        "local-io-package-baseline"
    });
    if !dir.exists() {
        c::mkdir(&dir)?;
        fs::create_dir_all(dir.join("native/subprojects/slirp"))?;
        fs::hard_link(core, dir.join("native/core"))?;
        fs::hard_link(firmware, dir.join("open.bin"))?;
        fs::copy("/usr/bin/qemu-img", dir.join("native/qemu-img"))?;
        fs::copy(
            native.join("build/subprojects/slirp/libslirp.so.0"),
            dir.join("native/subprojects/slirp/libslirp.so.0"),
        )?;
        let manifest = resources::Manifest {
            schema: 1,
            core: resources::LOCAL_IO_CORE.into(),
            open_system: expected.into(),
            disk_tool: c::file_digest(&dir.join("native/qemu-img"))?,
            slirp: c::file_digest(&dir.join("native/subprojects/slirp/libslirp.so.0"))?,
            state_abi: 1,
        };
        c::write_new(&dir.join("runtime.json"), &c::json(&manifest)?)?;
    }
    Ok(resources::Resources::local_io(&dir, reporting)?)
}

pub(super) fn control(
    repo: &Path,
    root: &Path,
    reporting: bool,
    demand: bool,
) -> Result<(), Error> {
    let media = root.join(if demand {
        "file-cache-demand-v3/original.iso"
    } else {
        "file-io-errors-v1/original.iso"
    });
    let pin = c::file_digest(&media)?;
    if demand {
        if pin != DEMAND {
            return Err("exact reconstructed original demand image differs".into());
        }
    } else {
        let build: Value = c::read_json(&root.join("file-io-errors-v1/build.json"))?;
        if build["fixture"] != pin
            || build["expected_public_io_error_control"] != true
            || build["source"] != c::file_digest(&repo.join("native/fixtures/file-read.S"))?
        {
            return Err("current original I/O control bindings differ".into());
        }
    }
    let r = package(root, reporting)?;
    let dir = root.join("local-io-conformance").join(if demand {
        "demand"
    } else if reporting {
        "reporting"
    } else {
        "baseline"
    });
    fs::create_dir_all(dir.parent().ok_or("control parent")?)?;
    c::mkdir(&dir)?;
    super::file_read::observe_with(repo, root, &dir, &media, demand, r)
}

fn qualified(root: &Path) -> Result<(), Error> {
    let fixture = c::file_digest(&root.join("file-io-errors-v1/original.iso"))?;
    for (name, open, media) in [
        ("baseline", resources::LOCAL_BASE_OPEN, fixture.as_str()),
        ("reporting", resources::LOCAL_IO_OPEN, fixture.as_str()),
        ("demand", resources::LOCAL_IO_OPEN, DEMAND),
    ] {
        let dir = root.join("local-io-conformance").join(name);
        let result: Value = c::read_json(&dir.join("original.result.json"))?;
        if result["core"] != resources::LOCAL_IO_CORE
            || result["open_system"] != open
            || result["fixture"] != media
            || result["guest_passed"] != true
            || result["data_verified"] != true
            || result["native_clean"] != true
            || result["seeds_unchanged"] != true
            || (name != "demand" && result["public_io_error_control_passed"] != true)
        {
            return Err("current local original conformance or return failed".into());
        }
        let serial = fs::read_to_string(dir.join("serial.log"))?.to_ascii_lowercase();
        if name == "reporting" {
            for message in [
                "xbxio: kind=0 status=c0000034",
                "xbxio: kind=1 status=c0000011",
                "xbxio: kind=2 status=c0000011",
            ] {
                if !serial.contains(message) {
                    return Err("actual public error status reporting required".into());
                }
            }
        }
        if name == "demand" && !serial.contains("demand pages pass") {
            return Err("actual original demand-commit verification required".into());
        }
    }
    Ok(())
}

pub(super) fn fable(repo: &Path, root: &Path) -> Result<(), Error> {
    qualified(root)?;
    let prefix = "fable-local-io-logging";
    if root.join(format!("{prefix}.native-started.json")).exists() {
        return Err(
            "this local reporting observation already started; reconcile instead of repeating"
                .into(),
        );
    }
    let r = package(root, true)?;
    c::write_new(
        &root.join(format!("{prefix}.started.json")),
        &c::json(&json!({
            "core":r.manifest.core,"open_system":r.manifest.open_system,"source_reconstruction":true,
            "actual_original_conformance_passed":true,"ordinary_product_policy":true,
            "memory_mib":64,"network":"none","fp":"native","graphics":"Intel OpenGL",
            "public_io_failure_diagnostics_only":true,"historical_campaign_repeated":false
        }))?,
    )?;
    super::build::observe_logging_fable(repo, root, r, prefix, true, true, true)
}
