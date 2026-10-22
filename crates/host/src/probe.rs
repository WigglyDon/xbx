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
    run_inner(engine, new_root, None, None)
}

pub fn kernel(
    engine: &Path,
    new_root: &Path,
    flash: &Path,
    disk: &Path,
    map: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    run_inner(engine, new_root, Some((flash, disk, map)), None)
}

/// Two-stage retained homebrew fixture, not selected-game compatibility.
pub fn relaunch(
    engine: &Path,
    new_root: &Path,
    flash: &Path,
    disk: &Path,
    disc: &Path,
    title_map: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    run_inner(engine, new_root, Some((flash, disk, title_map)), Some(disc))
}

fn hmp_words(reply: &str) -> Vec<u32> {
    let mut words = Vec::new();
    for line in reply.split("\\n") {
        if let Some((_, tail)) = line.rsplit_once(':') {
            for token in tail.split_ascii_whitespace() {
                if let Some(hex) = token.strip_prefix("0x") {
                    let hex: String = hex.chars().take_while(char::is_ascii_hexdigit).collect();
                    if let Ok(value) = u32::from_str_radix(&hex, 16) {
                        words.push(value);
                    }
                }
            }
        }
    }
    words
}

fn run_inner(
    engine: &Path,
    new_root: &Path,
    kernel: Option<(&Path, &Path, &Path)>,
    relaunch: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
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
    let mut symbols = Vec::new();
    if let Some((flash, disk, map)) = kernel {
        fs::copy(flash, root.join("flash.bin"))?;
        fs::copy(disk, root.join("hdd.qcow2"))?;
        let map = fs::read_to_string(map)?;
        if relaunch.is_some() {
            let line = map
                .lines()
                .find(|l| l.split_whitespace().nth(1) == Some("_XkTitleResults"))
                .ok_or("title results symbol missing")?;
            symbols.push((
                "XkTitleResults",
                u64::from_str_radix(
                    line.split_whitespace().nth(2).ok_or("invalid title map")?,
                    16,
                )?,
            ));
        } else {
            for name in [
                "KiBootStage",
                "KiBugCheckCode",
                "KiTestMmDone",
                "KiTestFatxDone",
                "KiTestFatxWriteOk",
            ] {
                let line = map
                    .lines()
                    .find(|line| line.contains(&format!("?{name}@@")))
                    .ok_or("self-test symbol missing from the selected map")?;
                let address =
                    u64::from_str_radix(line.split_whitespace().nth(2).ok_or("invalid map")?, 16)?;
                symbols.push((name, address));
            }
        }
    } else {
        fs::write(root.join("flash.bin"), rom)?;
    }
    let config = format!(
        "[general]\nshow_welcome=false\nskip_boot_anim=false\n[general.updates]\ncheck=false\n[display]\nrenderer=\"OPENGL\"\n[sys]\nmem_limit=\"64\"\n[sys.files]\nflashrom_path={}\nbootrom_path=\"\"\nhdd_path=\"\"\ndvd_path=\"\"\neeprom_path={}\n",
        quote(&root.join("flash.bin"))?,
        quote(&root.join("eeprom.bin"))?
    );
    let config = if kernel.is_some() {
        config.replace(
            "hdd_path=\"\"",
            &format!("hdd_path={}", quote(&root.join("hdd.qcow2"))?),
        )
    } else {
        config
    };
    let config = if let Some(disc) = relaunch {
        config.replace(
            "dvd_path=\"\"",
            &format!("dvd_path={}", quote(&disc.canonicalize()?)?),
        )
    } else {
        config
    };
    fs::write(root.join("xemu.toml"), config)?;
    let qmp = root.join("qmp.sock");
    // Unix socket paths are kernel-bounded. Refuse before launching a writer.
    if qmp.as_os_str().len() > 100 {
        return Err("use a shorter new scratch directory for QMP".into());
    }
    let mut log = File::create(root.join("native.log"))?;
    let (mut log_reader, log_writer) = UnixStream::pair()?;
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
        .stdout(Stdio::from(std::os::fd::OwnedFd::from(
            log_writer.try_clone()?,
        )))
        .stderr(Stdio::from(std::os::fd::OwnedFd::from(log_writer)));
    if kernel.is_some() {
        command
            .args(["-device", "lpc47m157", "-serial"])
            .arg(format!("file:{}", root.join("kernel.log").display()));
    }
    lease.begin_writes()?;
    lease.inherit_for(&mut command);
    let mut owned = OwnedChild(command.spawn()?);
    drop(command);
    let drain = std::thread::spawn(move || {
        let mut remaining = 4 * 1024 * 1024;
        let mut buffer = [0u8; 8192];
        loop {
            match log_reader.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let keep = n.min(remaining);
                    let _ = log.write_all(&buffer[..keep]);
                    remaining -= keep;
                }
            }
        }
        let _ = log.sync_all();
    });
    let child = &mut owned.0;
    let deadline = Instant::now() + Duration::from_secs(if kernel.is_some() { 60 } else { 20 });
    let mut booted = false;
    let mut markers = Vec::new();
    while !qmp.exists() && Instant::now() < deadline {
        if child.try_wait()?.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let mut monitor_keepalive = None;
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
            if relaunch.is_some() {
                let address = symbols[0].1;
                let request = format!(
                    "{{\"execute\":\"human-monitor-command\",\"arguments\":{{\"command-line\":\"x/44wx {address:#x}\"}}}}\n"
                );
                socket.write_all(request.as_bytes())?;
                let words = hmp_words(&response(&mut reader)?);
                if words.len() == 44 && words[0] == 0x54544b58 && words[25] == 1 {
                    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
                    let first = String::from_utf8_lossy(&bytes[36..100]);
                    let second = String::from_utf8_lossy(&bytes[112..176]);
                    booted = [1, 2, 3, 4, 5, 7, 8, 25, 26, 27]
                        .iter()
                        .all(|&i| words[i] == 1)
                        && first.contains("default.xbe")
                        && second.contains("media\\second.xbe");
                    fs::write(
                        root.join("relaunch-results.txt"),
                        format!(
                            "words={words:?}\nfirst={first}\nsecond={second}\naccepted={booted}\n"
                        ),
                    )?;
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            if kernel.is_some() {
                markers.clear();
                for (name, address) in &symbols {
                    let request = format!(
                        "{{\"execute\":\"human-monitor-command\",\"arguments\":{{\"command-line\":\"x/1wx {address:#x}\"}}}}\n"
                    );
                    socket.write_all(request.as_bytes())?;
                    let reply = response(&mut reader)?;
                    let tail = reply.split("\\r\\n").next().unwrap_or(&reply);
                    let value = tail
                        .rsplit("0x")
                        .next()
                        .and_then(|s| {
                            let hex: String =
                                s.chars().take_while(char::is_ascii_hexdigit).collect();
                            u32::from_str_radix(&hex, 16).ok()
                        })
                        .unwrap_or(u32::MAX);
                    markers.push((*name, value));
                }
                booted = markers[0].1 == 0x30
                    && markers[1].1 == 0
                    && markers[2].1 == 1
                    && markers[3].1 == 1;
                if booted || (markers[1].1 != 0 && markers[1].1 != u32::MAX) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(500));
                continue;
            }
            socket.write_all(b"{\"execute\":\"human-monitor-command\",\"arguments\":{\"command-line\":\"info registers\"}}\n")?;
            let registers = response(&mut reader)?;
            if registers.contains("EAX=58425821") && registers.contains("EBX=52535421") {
                booted = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        socket.write_all(b"{\"execute\":\"stop\"}\n")?;
        response(&mut reader)?;
        socket.write_all(b"{\"execute\":\"quit\"}\n")?;
        // Keep the transport alive until QEMU has accepted the request. Closing
        // immediately after write can discard a command still queued at EOF.
        response(&mut reader)?;
        monitor_keepalive = Some((socket, reader));
        Ok(())
    })();
    let stop_deadline = Instant::now() + Duration::from_secs(5);
    let mut forced = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if shutdown.is_err() || Instant::now() >= stop_deadline {
            forced = true;
            child.kill()?;
            break child.wait()?;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    drop(monitor_keepalive);
    fs::write(
        root.join("termination.txt"),
        format!("status={status}\nforced={forced}\nqmp={shutdown:?}\n"),
    )?;
    let _ = drain.join();
    lease.sync_storage()?;
    if kernel.is_some() {
        println!("Bounded open-kernel markers: {markers:?}");
    }
    println!(
        "{}: {booted}; normal QMP shutdown: {}",
        if relaunch.is_some() {
            "two-stage title entry, carried data and D: mapping observed"
        } else if kernel.is_some() {
            "selected open-kernel markers observed"
        } else {
            "synthetic reset-vector executed"
        },
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
