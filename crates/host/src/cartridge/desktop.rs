// SPDX-License-Identifier: MIT
//! Desktop transport for the shared cartridge/session policy. Native QMP owns
//! machine quiescence and full state. This is not another serializer/player.
use super::*;
use serde_json::{Value, json};
use std::{
    os::{
        fd::AsRawFd,
        unix::{net::UnixStream, process::ExitStatusExt},
    },
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
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
// The OS status distinguishes a kill attempt racing normal exit from SIGKILL.
// A reached deadline still fails qualification even if the process exits zero.
fn termination_status(
    status: &ExitStatus,
    acknowledged: bool,
    deadline_reached: bool,
) -> (bool, bool) {
    let forced = status.signal() == Some(9);
    (
        forced,
        acknowledged && !deadline_reached && !forced && status.success(),
    )
}
const RETIRE_GRACE_SECONDS: u64 = 15;

fn cache_directory(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => Ok(()),
        Ok(_) => Err(io::Error::other(
            "host cache path is not an owned directory",
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => mkdir(path),
        Err(error) => Err(error),
    }
}

pub struct Native {
    child: Process,
    monitor: super::monitor::Monitor,
    socket_path: PathBuf,
    serial_path: PathBuf,
    serial: UnixStream,
    receipt: PathBuf,
    pub directory: PathBuf,
    pub(crate) run: PathBuf,
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
            "[general]\nshow_welcome=false\n[general.updates]\ncheck=false\n[display]\nrenderer=\"{}\"\n[display.window]\nvsync={}\n[sys]\nmem_limit=\"64\"\n[input]\nauto_bind=false\n[input.bindings]\nport1_driver=\"usb-xbox-gamepad-s\"\nport1=\"keyboard\"\n[sys.files]\nbootrom_path=\"\"\nflashrom_path={}\nhdd_path={}\neeprom_path={}\ndvd_path={}\n",
            if r.presentation_vulkan {
                "VULKAN"
            } else {
                "OPENGL"
            },
            r.presentation_vsync,
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
            .args(["-S", "-accel", "tcg", "-nic", "none", "-config_path"])
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
        if !r.window {
            c.args(["-display", "none"]);
        }
        // Derived host shader caches survive native process replacement. The
        // profile lease owns their writer; the exact runtime identity separates
        // incompatible builds. They are neither guest saves nor checkpoint RAM.
        let host = p.root.join("host-cache");
        cache_directory(&host)?;
        let cache = host.join(&r.identity);
        cache_directory(&cache)?;
        write_new(
            &run.join("host-cache.json"),
            &super::json(
                &json!({"runtime":r.identity,"root":cache,"scope":"derived host caches; not guest storage or checkpoint state"}),
            )?,
        )?;
        for (key, name, persistent) in [
            ("XDG_DATA_HOME", "data", true),
            ("XDG_CONFIG_HOME", "config", false),
            ("XDG_CACHE_HOME", "cache", true),
            ("MESA_SHADER_CACHE_DIR", "mesa", true),
            ("XDG_STATE_HOME", "state", false),
        ] {
            let path = if persistent {
                cache.join(name)
            } else {
                run.join(name)
            };
            if persistent {
                cache_directory(&path)?;
            }
            c.env(key, path);
        }
        if with_uart {
            c.args(["-device", "lpc47m157"]);
        }
        p.lease.inherit_for(&mut c);
        app_lease.inherit_for(&mut c);
        let mut child = Process(c.spawn()?);
        write_new(&run.join("native.pid"), child.id().to_string().as_bytes())?;
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
        let serial = UnixStream::connect(&serial_path)?;
        serial.set_read_timeout(Some(Duration::from_millis(10)))?;
        let mut native = Self {
            child,
            monitor: super::monitor::Monitor::connected(socket)?,
            socket_path,
            serial_path,
            serial,
            receipt: run.join("native-return.json"),
            directory: dir.into(),
            run,
            retired: false,
        };
        if r.window {
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                let usb = native.hmp("info usb")?;
                if usb.contains("Xbox") && usb.contains("Controller") {
                    write_new(&native.run.join("usb-devices.txt"), usb.as_bytes())?;
                    break;
                }
                if Instant::now() >= deadline {
                    return Err(io::Error::other(
                        "visible runtime Controller S did not attach",
                    ));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        Ok(native)
    }
    pub fn call(&mut self, execute: &str, arguments: Value) -> io::Result<Value> {
        self.monitor.call(execute, arguments)
    }
    fn hmp(&mut self, command: &str) -> io::Result<String> {
        self.call("human-monitor-command", json!({"command-line":command}))?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| io::Error::other("invalid native HMP response"))
    }
    pub fn pause(&mut self) -> io::Result<()> {
        self.monitor.pause()
    }
    fn resume(&mut self) -> io::Result<()> {
        self.monitor.resume()
    }
    pub fn capture(&mut self) -> io::Result<()> {
        self.monitor.capture()
    }
    fn load(&mut self) -> io::Result<()> {
        self.monitor.load()
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
    pub(crate) fn exited(&mut self) -> io::Result<bool> {
        Ok(self.child.try_wait()?.is_some())
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
        let quit_started = Instant::now();
        let quit = self.call("quit", json!({}));
        let quit_ms = quit_started.elapsed().as_millis();
        let acknowledged = quit.is_ok();
        // Original relaunch exposed a5s grace racing an eventual native exit0.
        // Retain a finite grace and report both the decision and actual outcome.
        let reap_started = Instant::now();
        let deadline = reap_started + Duration::from_secs(RETIRE_GRACE_SECONDS);
        let mut deadline_reached = false;
        let status = loop {
            if let Some(s) = self.child.try_wait()? {
                break s;
            }
            if Instant::now() > deadline {
                deadline_reached = true;
                self.child.kill()?;
                break self.child.wait()?;
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        let reap_ms = reap_started.elapsed().as_millis();
        let (forced, clean) = termination_status(&status, acknowledged, deadline_reached);
        self.retired = true;
        let sync_started = Instant::now();
        store::sync_files(&self.directory)?;
        let sync_ms = sync_started.elapsed().as_millis();
        write_new(
            &self.receipt,
            &super::json(&json!({
                "qmp_acknowledged":acknowledged,"qmp_error":quit.err().map(|e|e.to_string()),
                "native_exit":status.to_string(),"forced":forced,"native_reaped":true,
                "kill_attempted":deadline_reached,"shutdown_deadline_reached":deadline_reached,
                "shutdown_grace_seconds":RETIRE_GRACE_SECONDS,"quit_ack_ms":quit_ms,"reap_ms":reap_ms,"storage_sync_ms":sync_ms,
                "storage_synced":true,"clean":clean
            }))?,
        )?;
        let _ = fs::remove_file(&self.socket_path);
        let _ = fs::remove_file(&self.serial_path);
        Ok(clean)
    }
}
impl Drop for Native {
    fn drop(&mut self) {
        if !self.retired {
            let _ = self.child.kill();
            let status = self.child.wait();
            let synced = store::sync_files(&self.directory);
            let _ = write_new(&self.receipt, &super::json(&json!({
                "qmp_acknowledged":false,"native_exit":status.as_ref().ok().map(ToString::to_string),
                "forced":true,"native_reaped":status.is_ok(),"storage_synced":synced.is_ok(),
                "clean":false,"reason":"unacknowledged_scope_exit"
            })).unwrap_or_default());
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn host_cache_preserves_existing_files_and_refuses_symlink_targets() {
        let root = std::env::temp_dir().join(format!("xbx-host-cache-{}", new_id().unwrap()));
        mkdir(&root).unwrap();
        let cache = root.join("cache");
        cache_directory(&cache).unwrap();
        fs::write(cache.join("existing"), b"preserve derived data").unwrap();
        cache_directory(&cache).unwrap();
        let link = root.join("link");
        std::os::unix::fs::symlink(&cache, &link).unwrap();
        assert!(cache_directory(&link).is_err());
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            fs::read(cache.join("existing")).unwrap(),
            b"preserve derived data"
        );
        assert_eq!(fs::read_dir(&cache).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn normal_exit_deadline_and_forced_signal_are_distinct_actual_outcomes() {
        let zero = Command::new("/usr/bin/true").status().unwrap();
        assert_eq!(termination_status(&zero, true, false), (false, true));
        // The observed original-case shape must retain its deadline failure.
        assert_eq!(termination_status(&zero, true, true), (false, false));
        assert_eq!(termination_status(&zero, false, false), (false, false));
        let mut killed = Process(Command::new("/usr/bin/sleep").arg("30").spawn().unwrap());
        killed.kill().unwrap();
        let signal = killed.wait().unwrap();
        assert_eq!(signal.signal(), Some(9));
        assert_eq!(termination_status(&signal, true, true), (true, false));
        assert_eq!(termination_status(&signal, true, false), (true, false));
    }
}
