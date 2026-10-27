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
    let dir = root.join(if cache {
        "file-cache-v1"
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
    let generated = format!(
        ".set XBX_CACHE_TEST, {}\n{}\n.text\nserial_init:\n{}",
        u8::from(cache),
        fs::read_to_string(&source)?,
        uart
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
            "cache_file_control":cache,"cache_extent_bytes":if cache { Some(0x1200004u32) } else { None },
            "code_data_pages_separate":true,"original_generated_content_only":true
        }))?,
    )?;
    Ok(())
}
pub(super) fn observe(repo: &Path, root: &Path, cache: bool) -> Result<(), Error> {
    let dir = root.join(if cache {
        "file-cache-v1"
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
    let result = dir.join("original.result.json");
    if result.exists() || dir.join("serial.log").exists() {
        return Err("preserve prior original observation".into());
    }
    let r = resources::Resources::x87_logging(&root.join("package-x87-npx-v1"))?;
    let app = root.join(format!("file-read-{}", c::new_id()?));
    let mut player = c::desktop::Player::with_resources(&app, r)?;
    let game = player.add(&media)?;
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
    let observed = (|| -> Result<bool, Error> {
        let mut output = String::new();
        loop {
            let text = player.live.as_mut().ok_or("live")?.native.serial()?;
            log.write_all(text.as_bytes())?;
            output.push_str(&text);
            if output.len() > 65536 {
                return Err("original output limit".into());
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
        &c::json(&json!({"original":true,"fixture":identity["fixture"],
        "core":resources::X87_RESTART_CORE,"open_system":resources::NPX_LOGGING_OPEN,
        "cache_file_control":cache,"app":app,"native_run":run,"guest_passed":pass,"observation_error":observed.err().map(|e|e.to_string()),
        "native_clean":clean,"native_return":native,"return_error":returned.err().map(|e|e.to_string()),
        "seeds_unchanged":unchanged,"elapsed_ms":started.elapsed().as_millis(),"commercial_acceptance":false}))?,
    )?;
    if !clean || !pass {
        return Err("original file-read contract or return failed; preserve evidence".into());
    }
    Ok(())
}
