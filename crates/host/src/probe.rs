//! Original synthetic reset-vector probe: proves guest CPU execution, not a game/kernel.
use crate::storage::ProfileLease;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const BANNER: &[u8] = b"XBX:SYNTHETIC-X86-BOOT\n";

fn quote(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let text = path.to_str().ok_or("non-UTF8 diagnostic path")?;
    if text.chars().any(char::is_control) {
        return Err("control character in path".into());
    }
    Ok(format!(
        "\"{}\"",
        text.replace('\\', "\\\\").replace('"', "\\\"")
    ))
}

pub fn run(engine: &Path, new_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let engine = engine.canonicalize()?;
    fs::DirBuilder::new().mode(0o700).create(new_root)?;
    let root = new_root.canonicalize()?;
    let lease = ProfileLease::acquire(&root)?;
    let mut rom = vec![0xff; 1024 * 1024];
    let mut code = vec![0xfa, 0xba, 0xe9, 0x00]; // cli; mov dx, 0xe9
    for byte in BANNER {
        code.extend_from_slice(&[0xb0, *byte, 0xee]);
    } // mov al, byte; out dx, al
    code.extend_from_slice(&[0xf4, 0xeb, 0xfd]); // hlt; loop to hlt
    rom[0xf0000..0xf0000 + code.len()].copy_from_slice(&code);
    rom[0xffff0..0xffff5].copy_from_slice(&[0xea, 0x00, 0x00, 0x00, 0xf0]); // far jmp F000:0000
    fs::write(root.join("flash.bin"), rom)?;
    let config = format!(
        "[general]\nshow_welcome=false\nskip_boot_anim=false\n[general.updates]\ncheck=false\n[display]\nrenderer=\"OPENGL\"\n[sys]\nmem_limit=\"64\"\n[sys.files]\nflashrom_path={}\nbootrom_path=\"\"\nhdd_path=\"\"\ndvd_path=\"\"\neeprom_path={}\n",
        quote(&root.join("flash.bin"))?,
        quote(&root.join("eeprom.bin"))?
    );
    fs::write(root.join("xemu.toml"), config)?;
    let qmp = root.join("qmp.sock");
    // Unix socket paths are kernel-bounded. Refuse before launching a writer.
    if qmp.as_os_str().len() > 100 {
        return Err("use a shorter new scratch directory for QMP".into());
    }
    let console = root.join("guest.txt");
    let log = File::create(root.join("native.log"))?;
    let mut command = Command::new(engine);
    command
        .current_dir(&root)
        .env("SDL_VIDEODRIVER", "offscreen")
        .env("SDL_AUDIODRIVER", "dummy")
        .args(["-config_path"])
        .arg(root.join("xemu.toml"))
        .args(["-accel", "tcg", "-debugcon"])
        .arg(format!("file:{}", console.display()))
        .args(["-global", "isa-debugcon.iobase=0xe9", "-qmp"])
        .arg(format!("unix:{},server=on,wait=off", qmp.display()))
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log);
    lease.begin_writes()?;
    lease.inherit_for(&mut command);
    let mut child = command.spawn()?;
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut booted = false;
    while Instant::now() < deadline {
        if fs::read(&console).is_ok_and(|bytes| bytes == BANNER) {
            booted = true;
            break;
        }
        if child.try_wait()?.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let shutdown = (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut socket = UnixStream::connect(&qmp)?;
        socket.set_read_timeout(Some(Duration::from_secs(2)))?;
        socket.set_write_timeout(Some(Duration::from_secs(2)))?;
        let mut reader = BufReader::new(socket.try_clone()?);
        let mut response = String::new();
        reader.by_ref().take(8192).read_line(&mut response)?;
        if !response.contains("QMP") {
            return Err("invalid QMP greeting".into());
        }
        socket.write_all(b"{\"execute\":\"qmp_capabilities\"}\n")?;
        response.clear();
        reader.by_ref().take(8192).read_line(&mut response)?;
        if !response.contains("return") {
            return Err("QMP capabilities refused".into());
        }
        socket.write_all(b"{\"execute\":\"quit\"}\n")?;
        Ok(())
    })();
    let stop_deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if shutdown.is_err() || Instant::now() >= stop_deadline {
            child.kill()?;
            break child.wait()?;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    lease.sync_storage()?;
    println!(
        "synthetic reset-vector executed: {booted}; normal QMP shutdown: {}",
        shutdown.is_ok() && status.success()
    );
    if !booted || shutdown.is_err() || !status.success() {
        return Err(
            "native probe failed; retained scratch/native.log and all generated inputs".into(),
        );
    }
    println!(
        "CPU/device diagnostic only; gameplay, graphics, audio, in-game saves and full resume are untested."
    );
    Ok(())
}
