// SPDX-License-Identifier: MIT
//! Android transport/lifecycle adapter for cartridge::Player. One native machine
//! per process; logical identities and all checkpoint/storage policy are shared.
use super::*;
use crate::cartridge::{
    self as c,
    library::Library,
    player::{Engine, Player},
};
use std::{
    collections::VecDeque,
    fs::File,
    io::{self, Read, Write},
    os::fd::IntoRawFd,
    os::unix::net::UnixStream,
    path::PathBuf,
    sync::{Condvar, mpsc},
    thread::JoinHandle,
};
static ACTIVE: AtomicBool = AtomicBool::new(false);
static INPUT: AtomicBool = AtomicBool::new(false);
static SURFACE: AtomicBool = AtomicBool::new(false);
static STOPPING: AtomicBool = AtomicBool::new(false);
static COMMANDS: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
static CHANGED: Condvar = Condvar::new();
static WAIT: Mutex<()> = Mutex::new(());
static MAIN: OnceLock<mpsc::Sender<Event>> = OnceLock::new();
static BASE: OnceLock<PathBuf> = OnceLock::new();
static ENGINE: Mutex<Option<JoinHandle<i32>>> = Mutex::new(None);
enum Event {
    Launch(Launch),
    Done(Result<i32, String>, i32),
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RestartRequest {
    schema: u32,
    game: String,
    runtime: String,
}
struct Launch {
    config: CString,
    offset: u64,
    fd: File,
    qmp: PathBuf,
    serial: PathBuf,
    create_eeprom: bool,
    answer: mpsc::SyncSender<Result<(), String>>,
}
unsafe extern "C" {
    fn xbx_platform_native_dir() -> *const c_char;
    fn xbx_platform_available(path: *const c_char) -> u64;
    fn xbx_platform_choices(
        title: *const c_char,
        labels: *const *const c_char,
        keys: *const *const c_char,
        count: i32,
    );
    fn xbx_platform_collection_requested() -> bool;
    fn xbx_platform_finish_player(next: i32);
}
pub(super) fn active() -> bool {
    ACTIVE.load(Ordering::SeqCst)
}
pub(super) fn may_input() -> bool {
    INPUT.load(Ordering::SeqCst)
        && STARTUP.may_resume()
        && FOREGROUND.load(Ordering::SeqCst)
        && AUDIO_FOCUS.load(Ordering::SeqCst)
        && SURFACE.load(Ordering::SeqCst)
}
pub(super) fn focus_changed() {
    CHANGED.notify_all();
}
pub(super) extern "C" fn command(value: *const c_char) {
    if value.is_null() {
        return;
    }
    let value = unsafe { CStr::from_ptr(value) }.to_string_lossy();
    if value.len() > 64 || !value.is_ascii() {
        return;
    }
    if let Ok(mut q) = COMMANDS.lock() {
        if q.len() < 32 {
            q.push_back(value.into_owned());
        }
    }
    CHANGED.notify_all();
}
#[unsafe(no_mangle)]
/// # Safety
/// Android supplies only numeric availability; no Java object is dereferenced.
pub unsafe extern "system" fn Java_com_machinerhythm_xbx_XbxActivity_nativeSetSurface(
    _env: *mut std::ffi::c_void,
    _activity: *mut std::ffi::c_void,
    available: u8,
) {
    SURFACE.store(available != 0, Ordering::SeqCst);
    if available == 0 {
        neutral();
    }
    focus_changed();
}
fn next_command() -> Option<String> {
    COMMANDS.lock().ok()?.pop_front()
}
fn wait() {
    if let Ok(lock) = WAIT.lock() {
        drop(CHANGED.wait_timeout(lock, Duration::from_millis(100)));
    }
}
fn choices(title: &str, entries: &[(String, String)]) -> io::Result<()> {
    let title = CString::new(title)?;
    let names: Vec<_> = entries
        .iter()
        .map(|(n, _)| CString::new(n.as_str()))
        .collect::<Result<_, _>>()?;
    let keys: Vec<_> = entries
        .iter()
        .map(|(_, k)| CString::new(k.as_str()))
        .collect::<Result<_, _>>()?;
    let names: Vec<_> = names.iter().map(|s| s.as_ptr()).collect();
    let keys: Vec<_> = keys.iter().map(|s| s.as_ptr()).collect();
    unsafe {
        xbx_platform_choices(
            title.as_ptr(),
            names.as_ptr(),
            keys.as_ptr(),
            names.len() as i32,
        )
    };
    Ok(())
}
fn native_dir() -> io::Result<PathBuf> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let p = unsafe { xbx_platform_native_dir() };
        if !p.is_null() {
            return Ok(PathBuf::from(
                unsafe { CStr::from_ptr(p) }
                    .to_str()
                    .map_err(io::Error::other)?,
            ));
        }
        if Instant::now() > deadline {
            return Err(io::Error::other(
                "Android native library directory unavailable",
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
fn resources(app: &Path) -> io::Result<c::resources::Resources> {
    let native = native_dir()?;
    let id = option_env!("XBX_ANDROID_CORE_SHA256")
        .ok_or_else(|| io::Error::other("private cartridge candidate is not prepared"))?;
    let parent = app.join("resources");
    fs::create_dir_all(&parent)?;
    let _lease = crate::storage::ProfileLease::acquire(&parent)?;
    let root = parent.join(id);
    if !root.exists() {
        let pending = parent.join(format!("pending-{}", c::new_id()?));
        c::mkdir(&pending)?;
        for name in ["runtime.json", "open.bin", "empty.qcow2"] {
            c::write_new(&pending.join(name), &media::asset(name)?)?;
        }
        c::resources::Resources::android(&pending, &native)?;
        fs::rename(&pending, &root)?;
        File::open(&parent)?.sync_all()?;
    }
    c::resources::Resources::android(&root, &native)
}
fn available(app: &Path) -> io::Result<()> {
    let path = CString::new(app.to_str().ok_or_else(|| io::Error::other("app path"))?)?;
    if unsafe { xbx_platform_available(path.as_ptr()) } < 512 * 1024 * 1024 {
        return Err(io::Error::other(
            "Keep at least512MiB free for a coherent session commit; all existing data is retained",
        ));
    }
    Ok(())
}
struct Native {
    monitor: c::monitor::Monitor,
    serial: UnixStream,
    directory: PathBuf,
    run: PathBuf,
    qmp: PathBuf,
    serial_path: PathBuf,
    retired: bool,
}
struct Starting(bool);
impl Drop for Starting {
    fn drop(&mut self) {
        if self.0 {
            STOPPING.store(true, Ordering::SeqCst);
            unsafe { xbx_engine_stop() };
        }
    }
}
impl Engine for Native {
    fn media(app: &Path, e: &c::library::Entry) -> io::Result<File> {
        let (file, uri) =
            media::request(Some(&e.media)).map_err(|e| io::Error::other(e.to_string()))?;
        let file = Library::verify_open(e, file)?;
        if uri != e.media {
            Library::read(app)?.relink_open(app, &e.id, &uri, file.try_clone()?)?;
        }
        Ok(file)
    }
    fn process_reaped(&self) -> bool {
        false
    }
    fn start(
        r: &c::resources::Resources,
        p: &c::store::Profile,
        dir: &Path,
        media: &File,
        partition: u64,
        app_lease: &crate::storage::ProfileLease,
    ) -> io::Result<Self> {
        if STARTED.swap(true, Ordering::SeqCst) {
            return Err(io::Error::other(
                "this engine process has already hosted a machine",
            ));
        }
        p.lease.hold_until_process_exit()?;
        app_lease.hold_until_process_exit()?;
        let mut starting = Starting(true);
        let run = p.root.join(format!("run-{}", c::new_id()?));
        c::mkdir(&run)?;
        let quote = |p: &Path| serde_json::to_string(p.to_str().unwrap()).unwrap();
        let config = format!(
            "[general]\nshow_welcome=false\n[general.updates]\ncheck=false\n[display]\nrenderer=\"vulkan\"\n[sys]\nmem_limit=\"64\"\n[net]\nenable=false\n[input]\nauto_bind=false\n[input.bindings]\nport1_driver=\"usb-xbox-gamepad-s\"\n[sys.files]\nbootrom_path=\"\"\nflashrom_path={}\nhdd_path={}\neeprom_path={}\ndvd_path=\"/dev/fdset/1\"\n",
            quote(&r.root.join("open.bin")),
            quote(&dir.join("hdd.qcow2")),
            quote(&dir.join("eeprom.bin"))
        );
        c::write_new(&run.join("xemu.toml"), config.as_bytes())?;
        let base = BASE
            .get()
            .ok_or_else(|| io::Error::other("platform base missing"))?;
        let qmp = base.join(format!("q-{}.sock", &c::new_id()?[..16]));
        let serial_path = base.join(format!("s-{}.sock", &c::new_id()?[..16]));
        if qmp.as_os_str().len() > 100 || serial_path.as_os_str().len() > 100 {
            return Err(io::Error::other("private control socket path too long"));
        }
        let (tx, rx) = mpsc::sync_channel(1);
        MAIN.get()
            .ok_or_else(|| io::Error::other("platform owner missing"))?
            .send(Event::Launch(Launch {
                config: CString::new(run.join("xemu.toml").to_str().unwrap())?,
                offset: partition,
                fd: crate::disc::qemu_media_fd(media)?.into(),
                qmp: qmp.clone(),
                serial: serial_path.clone(),
                create_eeprom: !dir.join("eeprom.bin").exists(),
                answer: tx,
            }))
            .map_err(io::Error::other)?;
        rx.recv_timeout(Duration::from_secs(65))
            .map_err(io::Error::other)?
            .map_err(io::Error::other)?;
        let mut monitor = c::monitor::Monitor::connected(UnixStream::connect(&qmp)?)?;
        // Retain the older HMP observation for attribution. This Android pin
        // exposes USB topology directly through its native QMP interface;
        // validate that acknowledged result before any restore or guest run.
        let legacy = match monitor.call(
            "human-monitor-command",
            serde_json::json!({"command-line":"info usb"}),
        ) {
            Ok(value) => value,
            Err(error) => serde_json::json!({"error":error.to_string()}),
        };
        c::write_new(&run.join("usb-legacy-monitor.json"), &c::json(&legacy)?)?;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let devices = monitor.call("x-query-usb", serde_json::json!({}))?;
            if devices
                .get("human-readable-text")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|s| s.contains("Xbox") && s.contains("Controller"))
            {
                c::write_new(&run.join("usb-devices.json"), &c::json(&devices)?)?;
                break;
            }
            if Instant::now() > deadline {
                c::write_new(&run.join("usb-devices-rejected.json"), &c::json(&devices)?)?;
                return Err(io::Error::other(
                    "required controller topology did not attach before restore",
                ));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let serial = UnixStream::connect(&serial_path)?;
        serial.set_read_timeout(Some(Duration::from_millis(10)))?;
        starting.0 = false;
        Ok(Self {
            monitor,
            serial,
            directory: dir.into(),
            run,
            qmp,
            serial_path,
            retired: false,
        })
    }
    fn pause(&mut self) -> io::Result<()> {
        INPUT.store(false, Ordering::SeqCst);
        neutral();
        unsafe { xbx_engine_pause() };
        self.monitor.pause()
    }
    fn resume(&mut self) -> io::Result<()> {
        INPUT.store(true, Ordering::SeqCst);
        if !may_input() {
            INPUT.store(false, Ordering::SeqCst);
            return Ok(());
        }
        unsafe { xbx_engine_resume() };
        if let Err(e) = self.monitor.resume() {
            INPUT.store(false, Ordering::SeqCst);
            return Err(e);
        }
        if !may_input() {
            self.pause()?;
        }
        Ok(())
    }
    fn capture(&mut self) -> io::Result<()> {
        self.pause()?;
        self.monitor.capture()
    }
    fn load(&mut self) -> io::Result<()> {
        self.pause()?;
        self.monitor.load()
    }
    fn retire(&mut self) -> io::Result<bool> {
        self.pause()?;
        STOPPING.store(true, Ordering::SeqCst);
        let ack = self.monitor.call("quit", serde_json::json!({})).is_ok();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !FINISHED.load(Ordering::SeqCst) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        if !FINISHED.load(Ordering::SeqCst) {
            return Err(io::Error::other(
                "native machine retirement pending; process custody retained",
            ));
        }
        let status = ENGINE
            .lock()
            .map_err(|_| io::Error::other("engine join owner"))?
            .take()
            .ok_or_else(|| io::Error::other("engine already joined"))?
            .join()
            .map_err(|_| io::Error::other("native machine thread failed"))?;
        c::store::sync_files(&self.directory)?;
        self.retired = true;
        c::write_new(
            &self.run.join("native-return.json"),
            &c::json(
                &serde_json::json!({"qmp_acknowledged":ack,"machine_thread_joined":true,"native_status":status,"process_reaped":false,"writer_leases_retained_until_OS_process_exit":true,"storage_synced":true,"clean":ack&&status==0}),
            )?,
        )?;
        let _ = fs::remove_file(&self.qmp);
        let _ = fs::remove_file(&self.serial_path);
        Ok(ack && status == 0)
    }
}
impl Drop for Native {
    fn drop(&mut self) {
        if !self.retired {
            INPUT.store(false, Ordering::SeqCst);
            STOPPING.store(true, Ordering::SeqCst);
            unsafe { xbx_engine_stop() };
        }
    }
}

fn pick(player: &mut Player<Native>) -> Result<Option<String>, Box<dyn std::error::Error>> {
    loop {
        let library = Library::read(&player.app)?;
        let mut entries: Vec<_> = library
            .entries
            .iter()
            .map(|e| (e.name.clone(), e.id.clone()))
            .collect();
        entries.push(("Add local game".into(), "__add__".into()));
        choices("Your games", &entries)?;
        let deadline = Instant::now() + Duration::from_secs(600);
        let key = loop {
            if let Some(k) = next_command() {
                break k;
            }
            if Instant::now() > deadline {
                return Err("game selection deadline".into());
            }
            wait();
        };
        if key == "__cancel__" {
            return Ok(None);
        }
        if key == "__add__" {
            let (file, uri) = media::request(None)?;
            let mut library = Library::read(&player.app)?;
            return Ok(Some(library.add_open(&player.app, &uri, file)?));
        }
        if c::valid_id(&key) {
            library.resolve(&key)?;
            return Ok(Some(key));
        }
    }
}
fn owner(base: PathBuf) -> Result<i32, Box<dyn std::error::Error>> {
    let app = base.join("xbx-cartridges-v1");
    fs::create_dir_all(&app)?;
    available(&app)?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut player = loop {
        match Player::<Native>::with_resources(&app, resources(&app)?) {
            Ok(v) => break v,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                media::status("Finishing the previous session…");
                wait();
            }
            Err(e) => return Err(e.into()),
        }
    };
    let requested = app.join("restart-request.json");
    let recovery = match c::read_json::<RestartRequest>(&requested) {
        Ok(v) => {
            if v.schema != 1 || !c::valid_id(&v.game) || v.runtime != player.resources.identity {
                return Err("restart request is incompatible; data retained".into());
            }
            player.restart(&v.game)?;
            fs::rename(
                &requested,
                app.join(format!("restart-request-used-{}.json", c::new_id()?)),
            )?;
            File::open(&app)?.sync_all()?;
            Some(v.game)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.into()),
    };
    let selected = if recovery.is_some() {
        recovery
    } else if unsafe { xbx_platform_collection_requested() } {
        pick(&mut player)?
    } else {
        match player.last_game()? {
            Some(id) => Some(id),
            None => pick(&mut player)?,
        }
    };
    let Some(id) = selected else {
        return Ok(0);
    };
    loop {
        media::status("Opening game…");
        match player.launch_quiesced(&id) {
            Ok(_) => break,
            Err(e) => {
                neutral();
                eprintln!("XBX restore refused; data retained: {e}");
                media::status(
                    "This session could not be restored. Your saved session and ordinary saves are retained.",
                );
                choices(
                    "Return needs attention. Your saved session and ordinary saves are retained.",
                    &[
                        ("Keep session and leave".into(), "__keep__".into()),
                        (
                            "Restart using current ordinary saves".into(),
                            "__restart_confirm__".into(),
                        ),
                    ],
                )?;
                let deadline = Instant::now() + Duration::from_secs(600);
                let choice = loop {
                    if let Some(k) = next_command() {
                        break k;
                    }
                    if Instant::now() > deadline {
                        return Err("return decision deadline; data retained".into());
                    }
                    wait();
                };
                if choice != "__restart_confirm__" {
                    return Ok(0);
                }
                if STARTED.load(Ordering::SeqCst) {
                    c::retain_pending(&app, &["restart-request.pending"])?;
                    c::atomic(
                        &requested,
                        &c::json(&RestartRequest {
                            schema: 1,
                            game: id.clone(),
                            runtime: player.resources.identity.clone(),
                        })?,
                    )?;
                    return Ok(1);
                }
                c::retain_pending(&app, &["last-game.pending", "library.pending"])?;
                player.restart(&id)?;
            }
        }
    }
    media::status(if player.live.as_ref().is_some_and(|l| l.paused) {
        "Paused"
    } else {
        "Session ready"
    });
    let mut running = false;
    loop {
        if !STARTUP.ready() {
            return Err(
                "native machine became unhealthy; session and current saves retained".into(),
            );
        }
        let paused = player.live.as_ref().is_some_and(|l| l.paused);
        let allow = FOREGROUND.load(Ordering::SeqCst)
            && AUDIO_FOCUS.load(Ordering::SeqCst)
            && SURFACE.load(Ordering::SeqCst)
            && !paused;
        if allow && !running {
            player.resume()?;
            media::status("");
            running = true;
        }
        if !allow && running {
            available(&app)?;
            media::status("Saving session…");
            player.checkpoint_interruption()?;
            media::status(if paused { "Paused" } else { "Session saved" });
            running = false;
        }
        if let Some(key) = next_command() {
            match key.as_str() {
                "__menu__" => {
                    let mut items = vec![
                        ("Pause".into(), "__pause__".into()),
                        ("Resume".into(), "__resume__".into()),
                        ("Choose game".into(), "__collection__".into()),
                        ("Stop session".into(), "__stop__".into()),
                        ("Leave game".into(), "__leave__".into()),
                    ];
                    if player.live.as_ref().is_some_and(|l| {
                        c::resources::ORIGINAL_FIXTURES.contains(&l.binding.media.as_str())
                    }) {
                        for (k, n) in [
                            ("a", "Advance original state"),
                            ("s", "Write original disk marker"),
                            ("?", "Show original state"),
                        ] {
                            items.push((n.into(), format!("fixture-{k}")));
                        }
                    }
                    choices("Game", &items)?;
                }
                "__pause__" => {
                    available(&app)?;
                    player.pause(true)?;
                    running = false;
                    media::status("Paused");
                }
                "__resume__" => {
                    if let Some(l) = player.live.as_mut() {
                        l.paused = false;
                    }
                }
                "__stop__" => choices(
                    "Stop session? Unsaved execution will be left behind; ordinary saves remain.",
                    &[
                        ("Keep playing".into(), "__cancel__".into()),
                        ("Stop".into(), "__stop_confirm__".into()),
                    ],
                )?,
                "__stop_confirm__" => {
                    player.stop()?;
                    return Ok(0);
                }
                "__collection__" => {
                    available(&app)?;
                    player.depart()?;
                    return Ok(2);
                }
                "__leave__" => {
                    available(&app)?;
                    player.depart()?;
                    return Ok(0);
                }
                value if value.starts_with("fixture-") => {
                    let l = player.live.as_mut().ok_or("no game")?;
                    if c::resources::ORIGINAL_FIXTURES.contains(&l.binding.media.as_str()) {
                        let text = &value[8..];
                        if ["a", "s", "?"].contains(&text) {
                            l.native.serial.write_all(text.as_bytes())?;
                        }
                    }
                }
                _ => {}
            }
        }
        if FINISHED.load(Ordering::SeqCst) {
            return Err("native machine ended before normal departure; data retained".into());
        }
        if let Some(l) = player.live.as_mut() {
            let mut data = [0; 4096];
            match l.native.serial.read(&mut data) {
                Ok(n) if n > 0 => {
                    let text = String::from_utf8_lossy(&data[..n]);
                    eprintln!("{text}");
                    if c::resources::ORIGINAL_FIXTURES.contains(&l.binding.media.as_str()) {
                        media::status(&text);
                    }
                }
                Err(e)
                    if !matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    return Err(e.into());
                }
                _ => {}
            }
        }
        wait();
    }
}

pub(super) fn run() -> Result<i32, Box<dyn std::error::Error>> {
    if unsafe { xbx_engine_abi() } != 4 {
        return Err("cartridge Android native ABI differs; previous data retained".into());
    }
    ACTIVE.store(true, Ordering::SeqCst);
    unsafe { xbx_platform_callbacks(&CALLBACKS) };
    let base = unsafe { SDL_GetAndroidInternalStoragePath() };
    if base.is_null() {
        return Err("private Android storage unavailable".into());
    }
    let base = PathBuf::from(unsafe { CStr::from_ptr(base) }.to_str()?);
    BASE.set(base.clone())
        .map_err(|_| "platform already started")?;
    let _diagnostics = diagnostics(&base)?;
    let (tx, rx) = mpsc::channel();
    MAIN.set(tx.clone())
        .map_err(|_| "platform owner already started")?;
    let worker = std::thread::spawn(move || {
        let result = owner(base).map_err(|e| e.to_string());
        let next = result.as_ref().copied().unwrap_or(0);
        let _ = tx.send(Event::Done(result, next));
    });
    loop {
        match rx.recv()? {
            Event::Done(result, next) => {
                let _ = worker.join();
                if result.is_ok() {
                    unsafe { xbx_platform_finish_player(next) }
                }
                return result.map(|_| 0).map_err(Into::into);
            }
            Event::Launch(request) => {
                let prepared = (|| -> Result<(), String> {
                    if unsafe {
                        xbx_engine_prepare(
                            request.config.as_ptr(),
                            request.offset,
                            request.create_eeprom,
                            resume_allowed,
                        )
                    } != 0
                    {
                        return Err("native display/configuration initialization failed".into());
                    }
                    if unsafe { xbx_pad_create() } != 0 {
                        return Err("touch controller initialization failed".into());
                    }
                    neutral();
                    let fd = request.fd.into_raw_fd();
                    let qmp = request.qmp.clone();
                    let serial = request.serial.clone();
                    let engine = std::thread::spawn(move || {
                        let args = vec![
                            "xbx".into(),
                            "-S".into(),
                            "-accel".into(),
                            "tcg".into(),
                            "-nic".into(),
                            "none".into(),
                            "-add-fd".into(),
                            format!("fd={fd},set=1,opaque=xbx-readonly-disc"),
                            "-device".into(),
                            "lpc47m157".into(),
                            "-qmp".into(),
                            format!("unix:{},server=on,wait=off", qmp.display()),
                            "-chardev".into(),
                            format!(
                                "socket,id=guest,path={},server=on,wait=off",
                                serial.display()
                            ),
                            "-serial".into(),
                            "chardev:guest".into(),
                            "-monitor".into(),
                            "none".into(),
                        ];
                        let args: Vec<_> = args
                            .into_iter()
                            .map(|s: String| CString::new(s).unwrap())
                            .collect();
                        let mut pointers: Vec<_> =
                            args.iter().map(|s| s.as_ptr().cast_mut()).collect();
                        pointers.push(std::ptr::null_mut());
                        let result =
                            unsafe { xbx_engine_run(args.len() as i32, pointers.as_mut_ptr()) };
                        STARTUP.finish();
                        FINISHED.store(true, Ordering::SeqCst);
                        unsafe { xbx_engine_thread_finished() };
                        result
                    });
                    *ENGINE.lock().map_err(|_| "engine owner")? = Some(engine);
                    let deadline = Instant::now() + Duration::from_secs(60);
                    while !STARTUP.ready() {
                        let status = unsafe { xbx_engine_status() };
                        STARTUP.report(status);
                        if unsafe { xbx_engine_display_poll_ready() } {
                            STARTUP.display_ready();
                        }
                        if status < 0
                            || FINISHED.load(Ordering::SeqCst)
                            || Instant::now() > deadline
                        {
                            return Err(
                                "native initialization failed or timed out; data retained".into()
                            );
                        }
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    Ok(())
                })();
                let good = prepared.is_ok();
                let _ = request.answer.send(prepared);
                if good {
                    std::thread::spawn(|| {
                        let mut stopping = None;
                        while !FINISHED.load(Ordering::SeqCst) {
                            if STOPPING.load(Ordering::SeqCst) {
                                let since = stopping.get_or_insert_with(Instant::now);
                                if since.elapsed() > Duration::from_secs(10) {
                                    unsafe { _exit(125) }
                                }
                            }
                            std::thread::sleep(Duration::from_millis(100));
                        }
                    });
                    unsafe { xbx_engine_display_loop() };
                } else {
                    STOPPING.store(true, Ordering::SeqCst);
                    unsafe { xbx_engine_stop() };
                    if !FINISHED.load(Ordering::SeqCst) {
                        unsafe { _exit(124) }
                    }
                }
            }
        }
    }
}
