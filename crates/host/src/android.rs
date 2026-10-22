//! SDL's retained platform adapter enters Rust here. One engine lifetime per process.
use crate::startup::Startup;
use std::ffi::{CStr, CString, c_char, c_int};
use std::fs;
use std::os::fd::IntoRawFd;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
#[path = "android_media.rs"]
mod media;

#[link(name = "xemu")]
unsafe extern "C" {
    fn xbx_engine_abi() -> c_int;
    fn xbx_engine_prepare(
        config: *const c_char,
        media_offset: u64,
        create_eeprom: bool,
        gate: extern "C" fn() -> bool,
    ) -> c_int;
    fn xbx_engine_run(argc: c_int, argv: *mut *mut c_char) -> c_int;
    fn xbx_engine_display_loop();
    fn xbx_engine_status() -> c_int;
    fn xbx_engine_display_poll_ready() -> bool;
    fn xbx_engine_exit_requested() -> bool;
    fn xbx_engine_guest_flips() -> u32;
    fn xbx_engine_stop();
    fn xbx_engine_thread_finished();
    fn xbx_engine_pause();
    fn xbx_engine_resume();
    fn xbx_platform_callbacks(callbacks: *const Callbacks);
    fn xbx_pad_create() -> c_int;
    fn xbx_pad_apply(buttons: u32, axes: *const i16);
}

#[link(name = "SDL3")]
unsafe extern "C" {
    fn SDL_GetAndroidInternalStoragePath() -> *const c_char;
    fn SDL_ShowSimpleMessageBox(
        flags: u32,
        title: *const c_char,
        message: *const c_char,
        window: *mut (),
    ) -> bool;
}

unsafe extern "C" {
    fn _exit(status: c_int) -> !;
}
static STARTED: AtomicBool = AtomicBool::new(false);
static STARTUP: Startup = Startup::new();
static FINISHED: AtomicBool = AtomicBool::new(false);
static FOREGROUND: AtomicBool = AtomicBool::new(true);
static AUDIO_FOCUS: AtomicBool = AtomicBool::new(false);
static VIEWPORT: Mutex<Option<crate::input::Viewport>> = Mutex::new(None);
static TOUCH: Mutex<crate::input::Touch> = Mutex::new(crate::input::Touch::new());
#[repr(C)]
struct Callbacks {
    media: extern "C" fn(i32, *const c_char, *const c_char),
    touch: extern "C" fn(i32, i32, f32, f32, f32, f32, f32),
    layout: extern "C" fn(i32, f32, f32, f32, *mut f32),
    label: extern "C" fn(i32) -> *const c_char,
    neutral: extern "C" fn(),
}
static CALLBACKS: Callbacks = Callbacks {
    media: media::result,
    touch,
    layout,
    label,
    neutral,
};
extern "C" fn neutral() {
    if let Ok(mut touch) = TOUCH.lock() {
        touch.clear();
    }
    unsafe { xbx_pad_apply(0, [0i16; 6].as_ptr()) };
}
extern "C" fn touch(id: i32, phase: i32, x: f32, y: f32, _w: f32, _h: f32, _d: f32) {
    if !STARTUP.may_resume() {
        return;
    }
    let Some(view) = VIEWPORT.lock().ok().and_then(|v| *v) else {
        return;
    };
    if let Ok(mut touch) = TOUCH.lock() {
        let pad = touch.event_in(id, phase, [x, y], view);
        unsafe { xbx_pad_apply(pad.buttons, pad.axes.as_ptr()) };
    }
}
extern "C" fn layout(index: i32, _w: f32, _h: f32, _d: f32, out: *mut f32) {
    if out.is_null() {
        return;
    }
    let Some(view) = VIEWPORT.lock().ok().and_then(|v| *v) else {
        return;
    };
    let out = unsafe { std::slice::from_raw_parts_mut(out, 5) };
    if index == -2 {
        out[0] = crate::input::COUNT as f32;
    } else if index == -1 {
        out[..4].copy_from_slice(&view.surface());
    } else if (0..crate::input::COUNT as i32).contains(&index) {
        let c = view.control(index as usize);
        out[..3].copy_from_slice(&[c.x, c.y, c.radius]);
        out[3] = if TOUCH.lock().is_ok_and(|t| t.active(index as usize)) {
            1.0
        } else {
            0.0
        };
    }
}
extern "C" fn label(index: i32) -> *const c_char {
    static LABELS: OnceLock<Vec<CString>> = OnceLock::new();
    let labels = LABELS.get_or_init(|| {
        crate::input::LABELS
            .iter()
            .map(|s| CString::new(*s).unwrap())
            .collect()
    });
    labels
        .get(index as usize)
        .map_or(c"".as_ptr(), |s| s.as_ptr())
}
/// # Safety
/// JNI supplies only numeric geometry; Java pointers are not dereferenced.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_machinerhythm_xbx_XbxActivity_nativeGeometry(
    _env: *mut std::ffi::c_void,
    _activity: *mut std::ffi::c_void,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    density: f32,
) {
    let next = crate::input::Viewport::new([left, top, width, height], density);
    let changed = if let Ok(mut current) = VIEWPORT.lock() {
        let changed = *current != next;
        *current = next;
        changed
    } else {
        true
    };
    if changed {
        neutral();
    }
}

fn focus_changed() {
    STARTUP.foreground(FOREGROUND.load(Ordering::SeqCst) && AUDIO_FOCUS.load(Ordering::SeqCst));
    if !STARTUP.may_resume() {
        neutral();
    }
    if STARTUP.ready() {
        unsafe {
            if STARTUP.may_resume() {
                xbx_engine_resume()
            } else {
                xbx_engine_pause()
            }
        }
    }
}
#[unsafe(no_mangle)]
/// # Safety
/// Android supplies the audio-focus signal; no Java pointers are dereferenced.
pub unsafe extern "system" fn Java_com_machinerhythm_xbx_XbxActivity_nativeSetAudioFocus(
    _env: *mut std::ffi::c_void,
    _activity: *mut std::ffi::c_void,
    focused: u8,
) {
    AUDIO_FOCUS.store(focused != 0, Ordering::SeqCst);
    focus_changed();
}
extern "C" fn resume_allowed() -> bool {
    STARTUP.may_resume()
}

/// # Safety
/// JNI supplies an Android lifecycle signal; no object pointers are dereferenced.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_machinerhythm_xbx_XbxActivity_nativeSetForeground(
    _env: *mut std::ffi::c_void,
    _activity: *mut std::ffi::c_void,
    foreground: u8,
) {
    FOREGROUND.store(foreground != 0, Ordering::SeqCst);
    focus_changed();
}

// One cold-boot player, one stable private console profile and a retained grant.
fn run() -> Result<i32, Box<dyn std::error::Error>> {
    if STARTED.swap(true, Ordering::SeqCst) {
        return Err("engine already entered in this process".into());
    }
    let base = unsafe { SDL_GetAndroidInternalStoragePath() };
    if base.is_null() {
        return Err("Android did not provide private app storage".into());
    }
    let base = unsafe { CStr::from_ptr(base) }.to_str()?;

    if unsafe { xbx_engine_abi() } != 3 {
        return Err("The native engine and Rust entry have different ABI versions".into());
    }
    unsafe { xbx_platform_callbacks(&CALLBACKS) };
    let (root, lease, firmware) = media::owned_profile(Path::new(base))?;
    let runtime = media::select(&root, &firmware)?;
    let media_fd = crate::disc::qemu_media_fd(&runtime.media)?.into_raw_fd();
    let config = super::profile_config(&root)?;
    let config_path = root.join("xemu.toml");
    // This is generated policy, never an owner-supplied arbitrary QEMU config.
    fs::write(&config_path, config)?;
    let config_path = CString::new(config_path.to_str().ok_or("non-UTF8 profile path")?)?;
    let _diagnostics = diagnostics(&root)?;
    eprintln!(
        "XBX cold boot: durable game {} profile {}; no checkpoint restore",
        runtime.saved.game.0, runtime.saved.profile.0
    );
    let create_eeprom = lease.generation()? == 0 && !root.join("eeprom.bin").try_exists()?;
    if unsafe {
        xbx_engine_prepare(
            config_path.as_ptr(),
            runtime.disc.partition_offset,
            create_eeprom,
            resume_allowed,
        )
    } != 0
    {
        return Err("Native display/configuration initialization failed".into());
    }
    if unsafe { xbx_pad_create() } != 0 {
        return Err("The touch controller could not be initialized".into());
    }
    neutral();
    lease.begin_writes()?;
    let lease = std::mem::ManuallyDrop::new(lease);
    let engine = std::thread::spawn(move || {
        let name = CString::new("xbx").unwrap();
        let fd = CString::new(format!("fd={media_fd},set=1,opaque=xbx-readonly-disc")).unwrap();
        let mut argv = [
            name.as_ptr().cast_mut(),
            c"-S".as_ptr().cast_mut(),
            c"-add-fd".as_ptr().cast_mut(),
            fd.as_ptr().cast_mut(),
            c"-device".as_ptr().cast_mut(),
            c"lpc47m157".as_ptr().cast_mut(),
            c"-serial".as_ptr().cast_mut(),
            c"stdio".as_ptr().cast_mut(),
            c"-monitor".as_ptr().cast_mut(),
            c"none".as_ptr().cast_mut(),
            std::ptr::null_mut(),
        ];
        let status = unsafe { xbx_engine_run(10, argv.as_mut_ptr()) };
        STARTUP.finish();
        FINISHED.store(true, Ordering::SeqCst);
        unsafe { xbx_engine_thread_finished() };
        status
    });
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let status = unsafe { xbx_engine_status() };
        STARTUP.report(status);
        if FINISHED.load(Ordering::SeqCst) || status < 0 {
            while !engine.is_finished() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(20));
            }
            if !engine.is_finished() {
                unsafe { _exit(124) };
            }
            let _ = engine.join();
            return Err("Machine initialization was rejected or exited; the profile and diagnostic evidence were preserved.".into());
        }
        if unsafe { xbx_engine_display_poll_ready() } {
            STARTUP.display_ready();
        }
        if STARTUP.ready() {
            break;
        }
        if Instant::now() >= deadline {
            eprintln!("XBX startup timeout; requesting termination, retaining profile lease");
            unsafe { xbx_engine_stop() };
            let stop = Instant::now() + Duration::from_secs(5);
            while !engine.is_finished() && Instant::now() < stop {
                std::thread::sleep(Duration::from_millis(20));
            }
            if engine.is_finished() {
                let _ = engine.join();
            }
            // The process owns the lease until all native threads die, including
            // an initializer that cannot safely be unwound. No successor starts.
            unsafe { _exit(124) };
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    media::status("");
    let watchdog = std::thread::spawn(|| {
        let mut stopping = None;
        let mut sample = Instant::now();
        let mut flips = unsafe { xbx_engine_guest_flips() };
        while !FINISHED.load(Ordering::SeqCst) {
            if sample.elapsed() >= Duration::from_secs(5) {
                let now = unsafe { xbx_engine_guest_flips() };
                if STARTUP.may_resume() {
                    eprintln!(
                        "XBX guest flips={} interval_ms={} (flip counter, not emulation-speed certification)",
                        now.wrapping_sub(flips),
                        sample.elapsed().as_millis()
                    );
                }
                sample = Instant::now();
                flips = now;
            }
            if unsafe { xbx_engine_exit_requested() } {
                let since = stopping.get_or_insert_with(Instant::now);
                if since.elapsed() > Duration::from_secs(10) {
                    eprintln!("XBX native teardown timeout; terminating with lease retained");
                    unsafe { _exit(125) };
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });
    unsafe {
        if STARTUP.may_resume() {
            xbx_engine_resume();
        }
        xbx_engine_display_loop();
    }
    unsafe { xbx_engine_stop() };
    let status = engine.join().map_err(|_| "engine thread failed")?;
    let _ = watchdog.join();
    lease.sync_storage()?;
    // Keep the OS lease until process exit: late donor threads must not race a successor.
    Ok(status)
}

fn diagnostics(root: &Path) -> std::io::Result<std::thread::JoinHandle<()>> {
    use std::io::{Read, Write};
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::net::UnixStream;
    unsafe extern "C" {
        fn dup2(old: c_int, new: c_int) -> c_int;
    }
    let serial = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(std::io::Error::other)?
        .as_nanos();
    let mut log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join(format!("native-{serial}.log")))?;
    let (mut reader, writer) = UnixStream::pair()?;
    reader.set_read_timeout(Some(Duration::from_secs(1)))?;
    if unsafe { dup2(writer.as_raw_fd(), 1) } < 0 || unsafe { dup2(writer.as_raw_fd(), 2) } < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(std::thread::spawn(move || {
        let mut remaining = 4 * 1024 * 1024;
        let mut buf = [0; 8192];
        let mut guest_failed = false;
        let mut recent = Vec::new();
        let mut dirty = false;
        let mut flushed = Instant::now();
        loop {
            let n = match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                    ) =>
                {
                    0
                }
                Err(_) => break,
            };
            if !guest_failed && n > 0 {
                recent.extend_from_slice(&buf[..n]);
                if recent
                    .windows(b"*** fatal error ".len())
                    .any(|s| s == b"*** fatal error ")
                {
                    guest_failed = true;
                    STARTUP.report(-4);
                    neutral();
                    unsafe {
                        xbx_engine_pause();
                    }
                    media::status(
                        "The guest reported a boot failure. Diagnostic evidence was preserved.",
                    );
                }
                if recent.len() > 64 {
                    recent.drain(..recent.len() - 64);
                }
            }
            let keep = n.min(remaining);
            if keep > 0 {
                let _ = log.write_all(&buf[..keep]);
                dirty = true;
                remaining -= keep;
            }
            if dirty && flushed.elapsed() >= Duration::from_secs(1) {
                let _ = log.sync_data();
                dirty = false;
                flushed = Instant::now();
            }
            // Continue draining after the cap so diagnostics cannot block QEMU.
        }
        let _ = log.sync_data();
    }))
}

/// # Safety
/// Called once by SDL's Android runtime, with its JVM/surface adapter initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SDL_main(_argc: c_int, _argv: *mut *mut c_char) -> c_int {
    let result = std::panic::catch_unwind(run);
    let status = match result {
        Ok(Ok(status)) => status,
        Ok(Err(error)) => {
            let message = CString::new(error.to_string())
                .unwrap_or_else(|_| CString::new("xbx startup failed").unwrap());
            unsafe {
                SDL_ShowSimpleMessageBox(
                    0x10,
                    c"xbx qualification".as_ptr(),
                    message.as_ptr(),
                    std::ptr::null_mut(),
                );
            }
            1
        }
        Err(_) => 1,
    };
    // Donor explicitly requires process termination: no C++/SDL global destructors
    // while Android surface callbacks may still be alive.
    unsafe { _exit(status) }
}
