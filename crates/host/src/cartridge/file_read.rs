// SPDX-License-Identifier: MIT
//! Original public file-read contract through the existing cartridge owners.
use super::{self as c, resources};
use serde_json::json;
use std::{
    fs,
    io::Write,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
fn put(b: &mut [u8], at: usize, value: u32) {
    b[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
pub(super) fn build(repo: &Path, root: &Path, cache: bool) -> Result<(), Error> {
    build_variant(repo, root, cache, false, false, false)
}
pub(super) fn build_high_va(repo: &Path, root: &Path) -> Result<(), Error> {
    build_variant(repo, root, true, true, false, false)
}
pub(super) fn build_demand(repo: &Path, root: &Path) -> Result<(), Error> {
    build_variant(repo, root, true, true, true, false)
}
pub(super) fn build_io_errors(repo: &Path, root: &Path) -> Result<(), Error> {
    build_variant(repo, root, false, true, false, true)
}
fn build_variant(
    repo: &Path,
    root: &Path,
    cache: bool,
    high_va: bool,
    demand: bool,
    io_errors: bool,
) -> Result<(), Error> {
    let dir = root.join(if io_errors {
        "file-io-errors-v1"
    } else if demand {
        "file-cache-demand-v3"
    } else if high_va {
        "file-cache-highva-v1"
    } else if cache {
        "file-cache-v2"
    } else {
        "file-read-v1"
    });
    c::mkdir(&dir)?;
    let template = root.join("fixtures-v2/a.iso");
    if c::file_digest(&template)? != resources::ORIGINAL_FIXTURES[0] {
        return Err("original fixture template changed".into());
    }
    let source = repo.join("native/fixtures/file-read.S");
    let uart_source = repo.join("native/fixtures/startup-lifetime.S");
    let uart = fs::read_to_string(&uart_source)?;
    let uart = uart
        .split_once("serial_init:\n")
        .ok_or("original UART start")?
        .1;
    let uart = uart.split_once(".balign 4\n").ok_or("original UART end")?.0;
    let handler = repo.join("native/fixtures/file-demand-handler.S");
    let handler_source = if demand {
        fs::read_to_string(&handler)?
    } else {
        String::new()
    };
    let generated = format!(
        ".set XBX_HIGH_VA, {}\n.set XBX_CACHE_TEST, {}\n.set XBX_DEMAND_BUFFER, {}\n.set XBX_IO_ERRORS_CONTROL, {}\n{}\n.text\nserial_init:\n{}\n{}",
        u8::from(high_va),
        u8::from(cache),
        u8::from(demand),
        u8::from(io_errors),
        fs::read_to_string(&source)?,
        uart,
        handler_source
    );
    c::write_new(&dir.join("original.S"), generated.as_bytes())?;
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    c::store::checked(
        Command::new(llvm.join("clang"))
            .args(["--target=i386-none-elf", "-march=pentium3", "-c"])
            .arg(dir.join("original.S"))
            .arg("-o")
            .arg(dir.join("original.o")),
    )?;
    c::store::checked(
        Command::new(llvm.join("ld.lld"))
            .args([
                "-m",
                "elf_i386",
                "-Ttext=0x11000",
                "-Tdata=0x12000",
                "--entry=_start",
            ])
            .arg(dir.join("original.o"))
            .arg("-o")
            .arg(dir.join("original.elf")),
    )?;
    let symbols = Command::new(llvm.join("llvm-nm"))
        .args(["-n", "--defined-only"])
        .arg(dir.join("original.elf"))
        .output()?;
    if !symbols.status.success() {
        return Err("original symbols unavailable".into());
    }
    let symbols = String::from_utf8(symbols.stdout)?;
    let thunk = symbols
        .lines()
        .find(|s| s.ends_with(" D thunks"))
        .and_then(|s| s.split_whitespace().next())
        .ok_or("original thunk address")?;
    let thunk = u32::from_str_radix(thunk, 16)?;
    c::store::checked(
        Command::new(llvm.join("llvm-objcopy"))
            .args(["-O", "binary"])
            .arg(dir.join("original.elf"))
            .arg(dir.join("original.bin")),
    )?;
    let bytes = fs::read(dir.join("original.bin"))?;
    if bytes.len() <= 4096 || bytes.len() > 24576 || !(0x12000..0x13000).contains(&thunk) {
        return Err("original code/data extent".into());
    }
    let export_abi = if demand || io_errors {
        // Check the assembled imports against an independent pinned declaration,
        // including function/data kind and stdcall argument bytes.
        let exports = root.join("npx-owner-port-v1/source/kernel/exports.tsv");
        let hash = c::file_digest(&exports)?;
        if hash != "aa5c07f0bb9a0c1053dc9aadd4d4271ba53d3a8b69c49356d8479faeb66d7e7b" {
            return Err("original demand-control ABI declaration changed".into());
        }
        let table = fs::read_to_string(exports)?;
        let mut imports = Vec::new();
        if demand {
            imports.extend([
                (36, 184u32, "NtAllocateVirtualMemory", 20),
                (40, 174, "MmIsAddressValid", 4),
                (44, 103, "KeGetCurrentIrql", 0),
            ]);
        }
        if io_errors {
            imports.extend([
                (0, 219u32, "NtReadFile", 32),
                (8, 190, "NtCreateFile", 36),
                (36, 184, "NtAllocateVirtualMemory", 20),
            ]);
        }
        for (offset, ordinal, name, arguments) in imports {
            let at = (thunk - 0x11000) as usize + offset;
            let value = u32::from_le_bytes(
                bytes
                    .get(at..at + 4)
                    .ok_or("original import extent")?
                    .try_into()?,
            );
            let row = format!("{ordinal}\t{name}\tstdcall\t{arguments}\tfunc");
            if value != 0x80000000 | ordinal || !table.lines().any(|line| line == row) {
                return Err("assembled original demand-control import ABI mismatch".into());
            }
        }
        Some(hash)
    } else {
        None
    };
    let original = fs::read(template)?;
    let mut xbe = original[34 * 2048..34 * 2048 + 4096].to_vec();
    let data_pages = (bytes.len() - 4096).div_ceil(4096) as u32 * 4096;
    put(&mut xbe, 0x10c, 0x2000 + data_pages);
    put(&mut xbe, 0x11c, 2);
    put(&mut xbe, 0x158, thunk ^ 0x5b6d40b6);
    put(&mut xbe, 0x180, 0x58424652);
    xbe[0x184..0x1d4].fill(0);
    for (i, ch) in "xbx original file read".encode_utf16().enumerate() {
        xbe[0x184 + i * 2..0x186 + i * 2].copy_from_slice(&ch.to_le_bytes());
    }
    xbe[0x408..0x40e].copy_from_slice(b".data\0");
    for (i, flags, va, raw, size, part) in [
        (0usize, 6, 0x11000, 4096, 4096, &bytes[..4096]),
        (1, 3, 0x12000, 8192, data_pages, &bytes[4096..]),
    ] {
        let at = 0x348 + i * 56;
        xbe[at..at + 56].fill(0);
        for (off, value) in [
            (0, flags),
            (4, va),
            (8, size),
            (12, raw),
            (16, part.len() as u32),
            (20, 0x10400 + i as u32 * 8),
            (28, 0x10410 + i as u32 * 4),
            (32, 0x10412 + i as u32 * 4),
        ] {
            put(&mut xbe, at + off, value);
        }
        xbe[at + 36..at + 56].copy_from_slice(&super::build::section_digest(part)?);
    }
    xbe.extend_from_slice(&bytes);
    c::write_new(&dir.join("original.xbe"), &xbe)?;
    let mut payload = Vec::with_capacity(32772);
    for i in 0..8193u32 {
        payload.extend_from_slice(&(i.wrapping_mul(0x01020305) ^ 0x6e000000).to_le_bytes());
    }
    let sector = 34 + xbe.len().div_ceil(2048);
    let mut iso = vec![0; 80 * 2048];
    iso[..34 * 2048].copy_from_slice(&original[..34 * 2048]);
    iso[33 * 2048 + 2..33 * 2048 + 4].copy_from_slice(&8u16.to_le_bytes());
    put(&mut iso, 33 * 2048 + 8, xbe.len() as u32);
    let node = 33 * 2048 + 32;
    put(&mut iso, node + 4, sector as u32);
    put(&mut iso, node + 8, payload.len() as u32);
    iso[node + 12] = 0x20;
    iso[node + 13] = 11;
    iso[node + 14..node + 25].copy_from_slice(b"payload.bin");
    iso[34 * 2048..34 * 2048 + xbe.len()].copy_from_slice(&xbe);
    iso[sector * 2048..sector * 2048 + payload.len()].copy_from_slice(&payload);
    c::write_new(&dir.join("original.iso"), &iso)?;
    c::write_new(
        &dir.join("build.json"),
        &c::json(&json!({
            "source":c::file_digest(&source)?,"uart_source":c::file_digest(&uart_source)?,
            "fixture":c::file_digest(&dir.join("original.iso"))?,"payload":c::digest(&payload),
            "payload_bytes":payload.len(),"public_abi":"XboxDev/nxdk a6c31ee90540710a3df26a0712b61492f2ea03dd",
            "heap_buffer_control":high_va,"heap_buffer_bytes":if high_va {Some(16384)} else {None},"cache_file_control":cache,"cache_extent_bytes":if cache { Some(0x1200004u32) } else { None },
            "demand_committed_buffer_control":demand,"demand_header_page_control":demand,"handler_source":if demand {Some(c::file_digest(&handler)?)} else {None},
            "export_abi_sha256":export_abi,
            "expected_public_io_error_control":io_errors,
            "code_data_pages_separate":true,"original_generated_content_only":true
        }))?,
    )?;
    Ok(())
}
pub(super) fn observe(repo: &Path, root: &Path, cache: bool) -> Result<(), Error> {
    let dir = root.join(if cache {
        "file-cache-v2"
    } else {
        "file-read-v1"
    });
    let identity: serde_json::Value = c::read_json(&dir.join("build.json"))?;
    let media = dir.join("original.iso");
    if identity["cache_file_control"] != cache
        || identity["fixture"] != c::file_digest(&media)?
        || identity["source"] != c::file_digest(&repo.join("native/fixtures/file-read.S"))?
    {
        return Err("original fixture bindings changed".into());
    }
    let r = resources::Resources::x87_logging(&root.join("package-x87-npx-v1"))?;
    observe_with(repo, root, &dir, &media, cache, r)
}
/// Reuse the exact original negative and accepted disc fixture. No rebuild or
/// reinterpretation of prior receipts when the runtime changes.
pub(super) fn flush_regression(repo: &Path, root: &Path, cache: bool) -> Result<(), Error> {
    let (fixture, pin) = if cache {
        (
            "file-cache-v2",
            "3e5f6805c69007b34ac8c654ace8b62648fa511f2548ab559a238d8dcee64bd0",
        )
    } else {
        (
            "file-read-v1",
            "6ad5f716fd4ae18e3d59db19a676b9f1338e0942e7ea8f00a99b50a25383a15d",
        )
    };
    let media = root.join(fixture).join("original.iso");
    if c::file_digest(&media)? != pin {
        return Err("retained original image changed".into());
    }
    let r = super::flush_io::package(root)?;
    let dir = root.join("flush-io-v1").join(if cache {
        "cache-regression"
    } else {
        "disc-regression"
    });
    c::mkdir(&dir)?;
    observe_with(repo, root, &dir, &media, cache, r)
}
fn observe_with(
    repo: &Path,
    root: &Path,
    dir: &Path,
    media: &Path,
    cache: bool,
    r: resources::Resources,
) -> Result<(), Error> {
    let fixture = c::file_digest(media)?;
    let core = r.manifest.core.clone();
    let open = r.manifest.open_system.clone();
    let result = dir.join("original.result.json");
    if result.exists() || dir.join("serial.log").exists() {
        return Err("preserve prior original observation".into());
    }
    let app = root.join(format!("file-read-{}", c::new_id()?));
    let mut player = c::desktop::Player::with_resources(&app, r)?;
    let game = player.add(media)?;
    let entry = c::library::Library::read(&app)?.resolve(&game)?;
    let disk = repo.join(".local/dashboard-v20260516-0955/xbox_hdd.qcow2");
    let eeprom = repo.join(".local/quick-reboot-d87cfc33-run1/eeprom.bin");
    let seeds = (c::file_digest(&disk)?, c::file_digest(&eeprom)?);
    if seeds.0 != "00d7df7a2bc235f8801764f00b7f40e194d1e392f7a9619d6b2396c89770f6dd" {
        return Err("seed changed".into());
    }
    {
        let _profile =
            c::store::Profile::open_seeded(&app, &entry, &player.resources, &disk, &eeprom)?;
    }
    player.launch(&game)?;
    let run = player.live.as_ref().ok_or("live")?.native.run.clone();
    let started = Instant::now();
    let mut log = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("serial.log"))?;
    let mut output = String::new();
    let observed = (|| -> Result<bool, Error> {
        loop {
            let text = player.live.as_mut().ok_or("live")?.native.serial()?;
            log.write_all(text.as_bytes())?;
            output.push_str(&text);
            if output.len() > 65536 {
                return Err("original output limit".into());
            }
            if output.contains("*** bug check") {
                return Ok(false);
            }
            if output.contains("XBX FILE READ BOOT") {
                if output.contains("XBX FILE READ FAIL") {
                    return Ok(false);
                }
                if output.contains("XBX FILE READ PASS") {
                    return Ok(true);
                }
            }
            if started.elapsed() > Duration::from_secs(if cache { 180 } else { 90 }) {
                return Err("original guest deadline".into());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    })();
    let returned = player.stop();
    log.sync_all()?;
    let native: serde_json::Value = c::read_json(&run.join("native-return.json"))?;
    let unchanged = seeds == (c::file_digest(&disk)?, c::file_digest(&eeprom)?);
    let pass = observed.as_ref().is_ok_and(|p| *p);
    let clean = returned.is_ok() && native["clean"] == true && unchanged;
    c::write_new(
        &result,
        &c::json(&json!({"original":true,"fixture":fixture,
        "core":core,"open_system":open,
        "cache_file_control":cache,"data_verified":output.contains("XBX FILE READ DATA PASS"),"public_io_error_control_passed":output.contains("XBX FILE READ EXPECTED ERRORS PASS"),"app":app,"native_run":run,"guest_passed":pass,"observation_error":observed.err().map(|e|e.to_string()),
        "native_clean":clean,"native_return":native,"return_error":returned.err().map(|e|e.to_string()),
        "seeds_unchanged":unchanged,"elapsed_ms":started.elapsed().as_millis(),"commercial_acceptance":false}))?,
    )?;
    if !clean || !pass {
        return Err("original file-read contract or return failed; preserve evidence".into());
    }
    Ok(())
}

/// Reuse the preserved negative and allocated-buffer image without rebuilding
/// their game-independent data or reinterpreting earlier results.
pub(super) fn probe_regression(repo: &Path, root: &Path, demand: bool) -> Result<(), Error> {
    let (source, name, pin) = if demand {
        (
            "file-cache-demand-v3",
            "demand-regression",
            "ccc5d1c6f07024ff18c95e7e3f3d4508c92a4c24f601d14b06ca96777cef7126",
        )
    } else {
        (
            "file-cache-highva-v1",
            "allocated-regression",
            "da22e0e3ef9444f110a5961c2ae43ba795ba22c513823f230f02c3ed2091d951",
        )
    };
    let media = root.join(source).join("original.iso");
    if c::file_digest(&media)? != pin {
        return Err("retained original buffer image changed".into());
    }
    let r = super::buffer_probe::package(root)?;
    let dir = root.join("buffer-probe-v1").join(name);
    c::mkdir(&dir)?;
    observe_with(repo, root, &dir, &media, true, r)
}

pub(super) fn io_error_control(repo: &Path, root: &Path, candidate: bool) -> Result<(), Error> {
    let dir = root.join("file-io-errors-v1");
    let identity: serde_json::Value = c::read_json(&dir.join("build.json"))?;
    let media = dir.join("original.iso");
    if identity["fixture"] != c::file_digest(&media)?
        || identity["expected_public_io_error_control"] != true
        || identity["source"] != c::file_digest(&repo.join("native/fixtures/file-read.S"))?
    {
        return Err("public I/O error control bindings differ".into());
    }
    let r = if candidate {
        super::buffer_probe::io_package(root)?
    } else {
        super::buffer_probe::package(root)?
    };
    let dest = root.join("io-errors-v1").join(if candidate {
        "diagnostic-control"
    } else {
        "baseline-control"
    });
    c::mkdir(&dest)?;
    observe_with(repo, root, &dest, &media, false, r)
}
pub(super) fn io_demand_regression(repo: &Path, root: &Path) -> Result<(), Error> {
    let media = root.join("file-cache-demand-v3/original.iso");
    if c::file_digest(&media)? != "ccc5d1c6f07024ff18c95e7e3f3d4508c92a4c24f601d14b06ca96777cef7126"
    {
        return Err("retained original demand image differs".into());
    }
    let r = super::buffer_probe::io_package(root)?;
    let dir = root.join("io-errors-v1/demand-regression");
    c::mkdir(&dir)?;
    observe_with(repo, root, &dir, &media, true, r)
}

/// Same generated cached-file data at a normally allocated user VA above the
/// console's physical RAM extent. This is a file-read buffer control, not a
/// replacement allocator or a commercial compatibility claim.
pub(super) fn observe_high_va(repo: &Path, root: &Path) -> Result<(), Error> {
    super::flush_io::qualified(root)?;
    let dir = root.join("file-cache-highva-v1");
    let identity: serde_json::Value = c::read_json(&dir.join("build.json"))?;
    let media = dir.join("original.iso");
    if identity["heap_buffer_control"] != true
        || identity["cache_file_control"] != true
        || identity["fixture"] != c::file_digest(&media)?
        || identity["source"] != c::file_digest(&repo.join("native/fixtures/file-read.S"))?
    {
        return Err("original heap-buffer control bindings changed".into());
    }
    observe_with(
        repo,
        root,
        &dir,
        &media,
        true,
        super::flush_io::package(root)?,
    )
}

/// This is an expected negative qualification, judged from the actual receipt.
/// A false success or dirty native return cannot admit a functional correction.
pub(super) fn demand_negative(repo: &Path, root: &Path) -> Result<(), Error> {
    super::flush_io::qualified(root)?;
    let dir = root.join("file-cache-demand-v3");
    let identity: serde_json::Value = c::read_json(&dir.join("build.json"))?;
    let media = dir.join("original.iso");
    if identity["demand_committed_buffer_control"] != true
        || identity["demand_header_page_control"] != true
        || identity["fixture"] != c::file_digest(&media)?
        || identity["source"] != c::file_digest(&repo.join("native/fixtures/file-read.S"))?
        || identity["handler_source"]
            != c::file_digest(&repo.join("native/fixtures/file-demand-handler.S"))?
    {
        return Err("original demand-commit control bindings changed".into());
    }
    let observed = observe_with(
        repo,
        root,
        &dir,
        &media,
        true,
        super::flush_io::package(root)?,
    );
    let result: serde_json::Value = c::read_json(&dir.join("original.result.json"))?;
    let serial = fs::read_to_string(dir.join("serial.log"))?;
    if observed.is_ok()
        || result["guest_passed"] != false
        || result["native_clean"] != true
        || result["seeds_unchanged"] != true
        || !serial.contains("XBX FILE READ DEMAND HEADER PASS")
        || !serial.contains(
            "FAIL stage=00000011 status=00000000 count=00004000 first=00000000 faults=00000000",
        )
    {
        return Err("expected clean original unmapped-buffer negative not reproduced".into());
    }
    Ok(())
}
