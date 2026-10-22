//! SDL's retained platform adapter enters Rust here. One engine lifetime per process.
use crate::storage::ProfileLease;
use std::ffi::{CStr, CString, c_char, c_int};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

#[link(name = "xemu")]
unsafe extern "C" {
    fn xbx_engine_abi() -> c_int;
    fn xbx_engine_prepare(config: *const c_char, media_offset: u64) -> c_int;
    fn xbx_engine_run(argc: c_int, argv: *mut *mut c_char) -> c_int;
    fn xbx_engine_display_loop();
    fn xbx_engine_display_wait_ready();
    fn xbx_engine_thread_finished();
    fn xbx_engine_pause();
    fn xbx_engine_resume();
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
static ENGINE_READY: AtomicBool = AtomicBool::new(false);
static FOREGROUND: AtomicBool = AtomicBool::new(true);

/// # Safety
/// JNI supplies an Android lifecycle signal; no object pointers are dereferenced.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_machinerhythm_xbx_XbxActivity_nativeSetForeground(
    _env: *mut std::ffi::c_void,
    _activity: *mut std::ffi::c_void,
    foreground: u8,
) {
    FOREGROUND.store(foreground != 0, Ordering::SeqCst);
    if ENGINE_READY.load(Ordering::SeqCst) {
        if foreground == 0 {
            unsafe { xbx_engine_pause() };
        } else {
            unsafe { xbx_engine_resume() };
        }
    }
}

// This qualification entry accepts only a task-prepared private profile. A game
// picker and real player UI are subsequent work, after native execution is qualified.
fn run() -> Result<i32, Box<dyn std::error::Error>> {
    if STARTED.swap(true, Ordering::SeqCst) {
        return Err("engine already entered in this process".into());
    }
    let base = unsafe { SDL_GetAndroidInternalStoragePath() };
    if base.is_null() {
        return Err("Android did not provide private app storage".into());
    }
    let base = unsafe { CStr::from_ptr(base) }.to_str()?;
    let root = Path::new(base).join("xbx-qualification");
    fs::create_dir_all(&root)?;
    let lease = ProfileLease::acquire(&root)?;
    for name in ["flash.bin", "hdd.qcow2", "disc.xiso"] {
        if !root.join(name).is_file() {
            return Err("The private qualification profile needs a prepared kernel, disk and disc. No game has been started.".into());
        }
    }
    let disc = crate::disc::inspect(&root.join("disc.xiso"))?;
    if unsafe { xbx_engine_abi() } != 2 {
        return Err("The native engine and Rust entry have different ABI versions".into());
    }
    let config = super::profile_config(&root)?;
    let config_path = root.join("xemu.toml");
    // This is generated policy, never an owner-supplied arbitrary QEMU config.
    fs::write(&config_path, config)?;
    let config_path = CString::new(config_path.to_str().ok_or("non-UTF8 profile path")?)?;
    if unsafe { xbx_engine_prepare(config_path.as_ptr(), disc.partition_offset) } != 0 {
        return Err("Native display/configuration initialization failed".into());
    }
    lease.begin_writes()?;
    let lease = std::mem::ManuallyDrop::new(lease);
    let engine = std::thread::spawn(|| {
        let name = CString::new("xbx").unwrap();
        let mut argv = [name.as_ptr().cast_mut(), std::ptr::null_mut()];
        let status = unsafe { xbx_engine_run(1, argv.as_mut_ptr()) };
        ENGINE_READY.store(false, Ordering::SeqCst);
        unsafe { xbx_engine_thread_finished() };
        status
    });
    unsafe {
        xbx_engine_display_wait_ready();
        ENGINE_READY.store(true, Ordering::SeqCst);
        if !FOREGROUND.load(Ordering::SeqCst) {
            xbx_engine_pause();
        }
        xbx_engine_display_loop();
    }
    let status = engine.join().map_err(|_| "engine thread failed")?;
    lease.sync_storage()?;
    // Keep the OS lease until process exit: late donor threads must not race a successor.
    Ok(status)
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
