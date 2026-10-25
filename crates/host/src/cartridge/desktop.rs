// SPDX-License-Identifier: MIT
//! Desktop transport for the shared cartridge/session policy. Native QMP owns
//! machine quiescence and full state. This is not another serializer/player.
use super::*;
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader},
    os::{fd::AsRawFd, unix::net::UnixStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct Process(Child);
impl std::ops::Deref for Process {
    type Target = Child;
    fn deref(&self) -> &Child {
        &self.0
    }
}
impl std::ops::DerefMut for Process {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.0
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
pub struct Native {
    child: Process,
    reader: BufReader<UnixStream>,
    socket: UnixStream,
    sequence: u64,
    socket_path: PathBuf,
    serial_path: PathBuf,
    serial: UnixStream,
    receipt: PathBuf,
    pub directory: PathBuf,
    retired: bool,
}
impl Native {
    fn start(
        r: &resources::Resources,
        p: &store::Profile,
        dir: &Path,
        media: &File,
        partition: u64,
        app_lease: &crate::storage::ProfileLease,
    ) -> io::Result<Self> {
        Self::start_with_uart(r, p, dir, media, partition, app_lease, true)
    }
    fn start_with_uart(
        r: &resources::Resources,
        p: &store::Profile,
        dir: &Path,
        media: &File,
        partition: u64,
        app_lease: &crate::storage::ProfileLease,
        with_uart: bool,
    ) -> io::Result<Self> {
        let run = p.root.join(format!("run-{}", new_id()?));
        mkdir(&run)?;
        let socket_path = PathBuf::from("/tmp").join(format!("xbx-{}.sock", new_id()?));
        let serial_path = PathBuf::from("/tmp").join(format!("xbx-serial-{}.sock", new_id()?));
        let q = |p: &Path| serde_json::to_string(p.to_str().unwrap()).unwrap();
        let dvd = format!(
            "json:{}",
            json!({"driver":"raw","offset":partition,"read-only":true,"file":{"driver":"file","filename":"/dev/fdset/1"}})
        );
        let config = format!(
            "[general]\nshow_welcome=false\n[general.updates]\ncheck=false\n[display]\nrenderer=\"OPENGL\"\n[sys]\nmem_limit=\"64\"\n[input]\nauto_bind=false\n[input.bindings]\nport1_driver=\"usb-xbox-gamepad-s\"\nport1=\"keyboard\"\n[sys.files]\nbootrom_path=\"\"\nflashrom_path={}\nhdd_path={}\neeprom_path={}\ndvd_path={}\n",
            q(&r.root.join("open.bin")),
            q(&dir.join("hdd.qcow2")),
            q(&dir.join("eeprom.bin")),
            serde_json::to_string(&dvd).unwrap()
        );
        write_new(&run.join("xemu.toml"), config.as_bytes())?;
        let incoming = crate::disc::qemu_media_fd(media)?;
        let log = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(run.join("native.log"))?;
        let mut c = Command::new(r.core());
        c.current_dir(&run)
            .env("XBX_ISOLATED_PROCESS_EXIT", "1")
            .env("SDL_AUDIO_DRIVER", "dummy")
            .env("SDL_VIDEO_DRIVER", "x11")
            .env("LIBGL_ALWAYS_SOFTWARE", "0")
            .args([
                "-S",
                "-accel",
                "tcg",
                "-nic",
                "none",
                "-display",
                "none",
                "-config_path",
            ])
            .arg(run.join("xemu.toml"))
            .arg("-qmp")
            .arg(format!("unix:{},server=on,wait=off", socket_path.display()))
            .arg("-add-fd")
            .arg(format!(
                "fd={},set=1,opaque=xbx-readonly-disc",
                incoming.as_raw_fd()
            ))
            .args(["-chardev"])
            .arg(format!(
                "socket,id=guest,path={},server=on,wait=off",
                serial_path.display()
            ))
            .args(["-serial", "chardev:guest", "-monitor", "none"])
            .stdin(Stdio::null())
            .stdout(log.try_clone()?)
            .stderr(log);
        for (key, name) in [
            ("XDG_DATA_HOME", "data"),
            ("XDG_CONFIG_HOME", "config"),
            ("XDG_CACHE_HOME", "cache"),
            ("MESA_SHADER_CACHE_DIR", "mesa"),
            ("XDG_STATE_HOME", "state"),
        ] {
            c.env(key, run.join(name));
        }
        if with_uart {
            c.args(["-device", "lpc47m157"]);
        }
        p.lease.inherit_for(&mut c);
        app_lease.inherit_for(&mut c);
        let mut child = Process(c.spawn()?);
        let deadline = Instant::now() + Duration::from_secs(20);
        let socket = loop {
            match UnixStream::connect(&socket_path) {
                Ok(s) => break s,
                Err(e) => {
                    if child.try_wait()?.is_some() || Instant::now() > deadline {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(io::Error::other(format!(
                            "native monitor unavailable: {e}; see {}",
                            run.display()
                        )));
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
            }
        };
        // Original native/USB fixture showed a 64s restore. Keep a finite
        // acknowledged control boundary; preserve timeout failures and custody.
        socket.set_read_timeout(Some(Duration::from_secs(90)))?;
        socket.set_write_timeout(Some(Duration::from_secs(5)))?;
        let reader = BufReader::new(socket.try_clone()?);
        let serial = UnixStream::connect(&serial_path)?;
        serial.set_read_timeout(Some(Duration::from_millis(10)))?;
        let mut native = Self {
            child,
            reader,
            socket,
            sequence: 0,
            socket_path,
            serial_path,
            serial,
            receipt: run.join("native-return.json"),
            directory: dir.into(),
            retired: false,
        };
        let mut greeting = String::new();
        native.reader.read_line(&mut greeting)?;
        native.call("qmp_capabilities", json!({}))?;
        native.call("query-status", json!({}))?;
        Ok(native)
    }
    pub fn call(&mut self, execute: &str, arguments: Value) -> io::Result<Value> {
        self.sequence += 1;
        let id = self.sequence;
        let request = json!({"execute":execute,"arguments":arguments,"id":id});
        writeln!(self.socket, "{request}")?;
        loop {
            let mut line = String::new();
            if self.reader.read_line(&mut line)? == 0 {
                return Err(io::Error::other("native control closed"));
            }
            if line.len() > 1024 * 1024 {
                return Err(io::Error::other("native control output limit"));
            }
            let v: Value = serde_json::from_str(&line).map_err(io::Error::other)?;
            if v.get("id") == Some(&json!(id)) {
                if let Some(e) = v.get("error") {
                    return Err(io::Error::other(format!("native refused: {e}")));
                }
                return v
                    .get("return")
                    .cloned()
                    .ok_or_else(|| io::Error::other("missing native acknowledgement"));
            }
        }
    }
    fn hmp(&mut self, command: &str) -> io::Result<String> {
        let v = self.call("human-monitor-command", json!({"command-line":command}))?;
        v.as_str()
            .map(str::to_owned)
            .ok_or_else(|| io::Error::other("invalid native HMP response"))
    }
    pub fn pause(&mut self) -> io::Result<()> {
        if self.call("query-status", json!({}))?["running"] == true {
            self.call("stop", json!({}))?;
        }
        let v = self.call("query-status", json!({}))?;
        if v["running"] != false {
            return Err(io::Error::other("machine did not quiesce"));
        }
        Ok(())
    }
    fn resume(&mut self) -> io::Result<()> {
        self.call("cont", json!({}))?;
        Ok(())
    }
    pub fn capture(&mut self) -> io::Result<()> {
        self.pause()?;
        let result = self.hmp("savevm xbx-session")?;
        if !result.trim().is_empty() {
            return Err(io::Error::other(format!("native capture: {result}")));
        }
        let listing = self.hmp("info snapshots")?;
        if !listing.contains("xbx-session") {
            return Err(io::Error::other("native snapshot absent"));
        }
        Ok(())
    }
    fn load(&mut self) -> io::Result<()> {
        self.pause()?;
        let result = self.hmp("loadvm xbx-session")?;
        if !result.trim().is_empty() {
            return Err(io::Error::other(format!("native restore: {result}")));
        }
        self.pause()
    }
    pub fn serial(&mut self) -> io::Result<String> {
        let mut bytes = Vec::new();
        let mut buffer = [0; 4096];
        loop {
            match self.serial.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    bytes.extend_from_slice(&buffer[..n]);
                    if bytes.len() > 65536 {
                        return Err(io::Error::other("guest output limit"));
                    }
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
    /// Original fixture terminal input only; no title memory access/observer.
    pub fn fixture_input(&mut self, text: &str) -> io::Result<()> {
        if text.len() > 64 || !text.is_ascii() {
            return Err(io::Error::other("fixture input bound"));
        }
        for byte in text.bytes() {
            self.serial.write_all(&[byte])?;
            std::thread::sleep(Duration::from_millis(20));
        }
        Ok(())
    }
    pub fn retire(&mut self) -> io::Result<bool> {
        if self.retired {
            return Ok(true);
        }
        let quit = self.call("quit", json!({}));
        let acknowledged = quit.is_ok();
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut forced = false;
        let status = loop {
            if let Some(s) = self.child.try_wait()? {
                break s;
            }
            if Instant::now() > deadline {
                forced = true;
                self.child.kill()?;
                break self.child.wait()?;
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        self.retired = true;
        store::sync_files(&self.directory)?;
        write_new(
            &self.receipt,
            &super::json(&json!({
                "qmp_acknowledged":acknowledged,"qmp_error":quit.err().map(|e|e.to_string()),
                "native_exit":status.to_string(),"forced":forced,"native_reaped":true,
                "storage_synced":true,"clean":acknowledged && !forced && status.success()
            }))?,
        )?;
        let _ = fs::remove_file(&self.socket_path);
        let _ = fs::remove_file(&self.serial_path);
        Ok(acknowledged && !forced && status.success())
    }
}
impl Drop for Native {
    fn drop(&mut self) {
        if !self.retired {
            let _ = self.child.kill();
            let _ = self.child.wait();
            let _ = store::sync_files(&self.directory);
            let _ = fs::remove_file(&self.socket_path);
            let _ = fs::remove_file(&self.serial_path);
        }
    }
}

pub type Player = super::player::Player<Native>;
impl super::player::Engine for Native {
    fn start(
        r: &resources::Resources,
        p: &store::Profile,
        dir: &Path,
        media: &File,
        partition: u64,
        app_lease: &crate::storage::ProfileLease,
    ) -> io::Result<Self> {
        Self::start(r, p, dir, media, partition, app_lease)
    }
    fn pause(&mut self) -> io::Result<()> {
        Self::pause(self)
    }
    fn resume(&mut self) -> io::Result<()> {
        Self::resume(self)
    }
    fn capture(&mut self) -> io::Result<()> {
        Self::capture(self)
    }
    fn load(&mut self) -> io::Result<()> {
        Self::load(self)
    }
    fn retire(&mut self) -> io::Result<bool> {
        Self::retire(self)
    }
}

/// Original-only fault test: omit the saved UART device so QEMU rejects that
/// section after applying the disk snapshot/reset and earlier machine sections.
/// Never reachable from ordinary game launch or commercial-game observation.
pub fn original_partial_load_test(
    app: &Path,
    r: &resources::Resources,
    entry: &library::Entry,
) -> io::Result<serde_json::Value> {
    if !resources::ORIGINAL_FIXTURES.contains(&entry.media_sha256.as_str()) {
        return Err(io::Error::other("original fixture required"));
    }
    let app_lease = crate::storage::ProfileLease::acquire(app)?;
    let p = store::Profile::open(app, entry, r)?;
    let binding = Binding {
        game: entry.id.clone(),
        profile: entry.profile.clone(),
        media: entry.media_sha256.clone(),
        partition: entry.partition,
        runtime: r.identity.clone(),
        state_abi: 1,
    };
    let v = checkpoint::load(&p, r, &binding)?
        .ok_or_else(|| io::Error::other("completed original checkpoint required"))?;
    let before = (
        fs::read(p.root.join("storage.json"))?,
        file_digest(&p.current()?.join("hdd.qcow2"))?,
        file_digest(&p.current()?.join("eeprom.bin"))?,
        file_digest(&v.directory.join("hdd.qcow2"))?,
        p.lease.generation()?,
    );
    let dir = checkpoint::restore_copy(&v, &p)?;
    let (media, _) = library::Library::media(entry)?;
    let mut native =
        Native::start_with_uart(r, &p, &dir, &media, entry.partition, &app_lease, false)?;
    v.recheck(&p, r)?;
    let failure = match native.load() {
        Ok(()) => {
            return Err(io::Error::other(
                "fault setup did not produce a native load failure",
            ));
        }
        Err(e) => e.to_string(),
    };
    let clean = native.retire().unwrap_or(false);
    drop(native);
    store::sync_files(&dir)?;
    let after = (
        fs::read(p.root.join("storage.json"))?,
        file_digest(&p.current()?.join("hdd.qcow2"))?,
        file_digest(&p.current()?.join("eeprom.bin"))?,
        file_digest(&v.directory.join("hdd.qcow2"))?,
        p.lease.generation()?,
    );
    if before != after {
        return Err(io::Error::other("partial-load containment failed"));
    }
    Ok(
        json!({"native_load_failed":true,"failure":failure,"native_return_clean":clean,"attempt_retained":dir,"authoritative_disk_eeprom_checkpoint_pointer_generation_unchanged":true}),
    )
}
