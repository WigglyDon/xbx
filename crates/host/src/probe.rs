//! Original synthetic reset-vector probe: proves guest CPU execution, not a game/kernel.
use crate::storage::ProfileLease;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

struct OwnedChild(std::process::Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

fn response(reader: &mut BufReader<UnixStream>) -> std::io::Result<String> {
    for _ in 0..16 {
        let mut line = String::new();
        reader.by_ref().take(8192).read_line(&mut line)?;
        if line.contains("\"return\"") {
            return Ok(line);
        }
        if line.is_empty() || line.contains("\"error\"") {
            break;
        }
    }
    Err(std::io::Error::other("bounded QMP response failed"))
}

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
    // CLI; MOV EAX,"XBX!"; MOV EBX,"RST!"; HLT; loop to HLT.
    // Markers are read from the emulated CPU through QMP, never injected by it.
    let code = [
        0xfa, 0x66, 0xb8, 0x21, 0x58, 0x42, 0x58, 0x66, 0xbb, 0x21, 0x54, 0x53, 0x52, 0xf4, 0xeb,
        0xfd,
    ];
    // Xbox maps flash high, not into PC's low F0000 segment. A near jump
    // preserves reset CS's hidden FFFF0000 base; FFF3 - 0x1f3 = FE00.
    rom[0xffe00..0xffe00 + code.len()].copy_from_slice(&code);
    rom[0xffff0..0xffff3].copy_from_slice(&[0xe9, 0x0d, 0xfe]);
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
    let log = File::create(root.join("native.log"))?;
    let mut command = Command::new(engine);
    command
        .current_dir(&root)
        .env("SDL_VIDEO_DRIVER", "offscreen")
        .env("SDL_AUDIO_DRIVER", "dummy")
        .env("LIBGL_ALWAYS_SOFTWARE", "1")
        .env("XDG_DATA_HOME", root.join("data"))
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_CACHE_HOME", root.join("cache"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("MESA_SHADER_CACHE_DIR", root.join("mesa-shader-cache"))
        .args(["-config_path"])
        .arg(root.join("xemu.toml"))
        .args(["-accel", "tcg", "-qmp"])
        .arg(format!("unix:{},server=on,wait=off", qmp.display()))
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log);
    lease.begin_writes()?;
    lease.inherit_for(&mut command);
    let mut owned = OwnedChild(command.spawn()?);
    let child = &mut owned.0;
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut booted = false;
    while !qmp.exists() && Instant::now() < deadline {
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
        let mut greeting = String::new();
        reader.by_ref().take(8192).read_line(&mut greeting)?;
        if !greeting.contains("QMP") {
            return Err("invalid QMP greeting".into());
        }
        socket.write_all(b"{\"execute\":\"qmp_capabilities\"}\n")?;
        response(&mut reader)?;
        while Instant::now() < deadline {
            socket.write_all(b"{\"execute\":\"human-monitor-command\",\"arguments\":{\"command-line\":\"info registers\"}}\n")?;
            let registers = response(&mut reader)?;
            if registers.contains("EAX=58425821") && registers.contains("EBX=52535421") {
                booted = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        socket.write_all(b"{\"execute\":\"quit\"}\n")?;
        // Keep the transport alive until QEMU has accepted the request. Closing
        // immediately after write can discard a command still queued at EOF.
        response(&mut reader)?;
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
