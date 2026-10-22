//! The document grant remains the media object. No path reopening or disc copying.
use std::{
    ffi::{CStr, CString, c_char, c_void},
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    os::fd::{AsRawFd, FromRawFd},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    sync::{Condvar, Mutex},
    time::{Duration, Instant},
};

unsafe extern "C" {
    fn xbx_platform_open(uri: *const c_char);
    fn xbx_platform_status(text: *const c_char);
    fn SDL_LoadFile(name: *const c_char, size: *mut usize) -> *mut c_void;
    fn SDL_free(data: *mut c_void);
    fn xbx_digest_new() -> *mut c_void;
    fn xbx_digest_update(digest: *mut c_void, data: *const u8, size: usize);
    fn xbx_digest_string(digest: *mut c_void) -> *const c_char;
    fn xbx_digest_free(digest: *mut c_void);
    fn fcntl(fd: i32, cmd: i32, ...) -> i32;
}
static RESULT: Mutex<Option<Result<(File, String), String>>> = Mutex::new(None);
static DELIVERED: Condvar = Condvar::new();

pub extern "C" fn result(fd: i32, uri: *const c_char, error: *const c_char) {
    let value = if fd >= 0 {
        let file = unsafe { File::from_raw_fd(fd) };
        if uri.is_null() {
            Err("The provider returned no document identity".to_owned())
        } else {
            Ok((
                file,
                unsafe { CStr::from_ptr(uri) }
                    .to_string_lossy()
                    .into_owned(),
            ))
        }
    } else {
        Err(if error.is_null() {
            "No document selected".to_owned()
        } else {
            unsafe { CStr::from_ptr(error) }
                .to_string_lossy()
                .into_owned()
        })
    };
    if let Ok(mut slot) = RESULT.lock()
        && slot.is_none()
    {
        *slot = Some(value);
        DELIVERED.notify_all();
    }
}

pub fn status(text: &str) {
    if let Ok(text) = CString::new(text) {
        unsafe { xbx_platform_status(text.as_ptr()) };
    }
}
fn asset(name: &str) -> std::io::Result<Vec<u8>> {
    let name = CString::new(name).map_err(std::io::Error::other)?;
    let mut size = 0;
    let ptr = unsafe { SDL_LoadFile(name.as_ptr(), &mut size) };
    if ptr.is_null() {
        return Err(std::io::Error::other(
            "The qualification package has no prepared boot profile",
        ));
    }
    if size > 4 * 1024 * 1024 {
        unsafe { SDL_free(ptr) };
        return Err(std::io::Error::other("Unexpected profile asset size"));
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), size) }.to_vec();
    unsafe { SDL_free(ptr) };
    Ok(bytes)
}
fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

pub fn prepare_profile(root: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let identity = asset("profile.identity")?;
    let text = std::str::from_utf8(&identity)?;
    if !text.starts_with("xbx-qualification-profile-v1\n")
        || !text.lines().any(|s| s == "ram_mib=64")
        || !text.lines().any(|s| s == "selftest=0")
    {
        return Err(
            "The package does not identify the normal-play 64 MiB qualification profile".into(),
        );
    }
    let flash = asset("flash.bin")?;
    if root.join("profile.identity").exists() {
        if fs::read(root.join("profile.identity"))? != identity
            || fs::read(root.join("flash.bin"))? != flash
        {
            return Err("The installed boot image differs from this saved profile. Its disk and identity were preserved; a profile migration needs an explicit decision.".into());
        }
    } else {
        if [
            "flash.bin",
            "hdd.qcow2",
            "eeprom.bin",
            "generation",
            "saved-session",
        ]
        .iter()
        .any(|n| root.join(n).exists())
        {
            return Err(
                "An incomplete or older profile was preserved. It cannot be silently replaced."
                    .into(),
            );
        }
        write_new(&root.join("flash.bin"), &flash)?;
        write_new(&root.join("hdd.qcow2"), &asset("hdd.qcow2")?)?;
        write_new(&root.join("profile.identity"), &identity)?;
        File::open(root)?.sync_all()?;
    }
    Ok(text
        .lines()
        .find_map(|s| s.strip_prefix("sha256="))
        .ok_or("missing firmware identity")?
        .to_owned())
}

pub fn owned_profile(
    base: &Path,
) -> Result<(std::path::PathBuf, crate::storage::ProfileLease, String), Box<dyn std::error::Error>>
{
    let identity = asset("profile.identity")?;
    let text = std::str::from_utf8(&identity)?;
    let diagnostic = text.lines().any(|l| l == "mode=launch-diagnostic-v1");
    let root = base.join(if diagnostic {
        "xbx-launch-diagnostic-v1"
    } else {
        "xbx-qualification"
    });
    let lease = if diagnostic && !root.exists() {
        status("Preparing an isolated diagnostic session…");
        crate::branch::create(
            &base.join("xbx-qualification"),
            &root,
            &asset("flash.bin")?,
            &identity,
            &opaque_id()?,
            fingerprint,
        )?
    } else {
        if diagnostic && !root.join("branch.identity").is_file() {
            return Err(
                "An incomplete diagnostic branch was preserved; baseline storage was not replaced"
                    .into(),
            );
        }
        fs::create_dir_all(&root)?;
        crate::storage::ProfileLease::acquire(&root)?
    };
    let firmware = prepare_profile(&root)?;
    Ok((root, lease, firmware))
}
fn fingerprint(file: &mut File) -> std::io::Result<String> {
    file.seek(SeekFrom::Start(0))?;
    let digest = Digest(unsafe { xbx_digest_new() });
    if digest.0.is_null() {
        return Err(std::io::Error::other("digest initialization failed"));
    }
    let mut buffer = [0u8; 128 * 1024];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        unsafe { xbx_digest_update(digest.0, buffer.as_ptr(), n) };
    }
    Ok(unsafe { CStr::from_ptr(xbx_digest_string(digest.0)) }
        .to_str()
        .map_err(std::io::Error::other)?
        .to_owned())
}

pub struct LiveRuntime {
    pub media: File,
    pub disc: crate::disc::Disc,
    pub saved: crate::session::SavedSession,
}
struct Digest(*mut c_void);
impl Drop for Digest {
    fn drop(&mut self) {
        unsafe { xbx_digest_free(self.0) }
    }
}

fn opaque_id() -> std::io::Result<String> {
    let mut bytes = [0; 16];
    File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

pub fn select(root: &Path, firmware: &str) -> Result<LiveRuntime, Box<dyn std::error::Error>> {
    status("Choose Fable from Downloads");
    let mut saved = fs::read_to_string(root.join("media.uri"))
        .ok()
        .and_then(|s| CString::new(s).ok());
    let (mut file, uri) = loop {
        unsafe { xbx_platform_open(saved.as_ref().map_or(std::ptr::null(), |s| s.as_ptr())) };
        let slot = RESULT.lock().map_err(|_| "Media handoff failed")?;
        let (mut slot, wait) = DELIVERED
            .wait_timeout_while(slot, Duration::from_secs(600), |v| v.is_none())
            .map_err(|_| "Media handoff failed")?;
        if wait.timed_out() && slot.is_none() {
            return Err("Game selection timed out; no machine was started".into());
        }
        let value = slot.take().ok_or("No document result")?;
        drop(slot);
        match value {
            Err(error) if error == "grant-unavailable" && saved.is_some() => {
                saved = None;
                status("Locate Fable to restore access");
            }
            other => break other?,
        }
    };
    if !uri.starts_with("content://") || uri.chars().any(char::is_control) {
        return Err("Unsupported document identity".into());
    }
    let flags = unsafe { fcntl(file.as_raw_fd(), 3) }; // F_GETFL
    if flags < 0 || flags & 3 != 0 {
        return Err("The game document must be retained read-only".into());
    }
    let before = file.metadata()?;
    let disc = crate::disc::inspect_open(&mut file)
        .map_err(|e| format!("Choose a complete local Xbox disc image: {e}"))?;
    file.seek(SeekFrom::Start(0))?;
    let digest = Digest(unsafe { xbx_digest_new() });
    if digest.0.is_null() {
        return Err("Media fingerprint initialization failed".into());
    }
    let mut buffer = [0; 256 * 1024];
    let mut read = 0u64;
    let mut last = Instant::now();
    let deadline = Instant::now() + Duration::from_secs(300);
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        unsafe { xbx_digest_update(digest.0, buffer.as_ptr(), n) };
        read += n as u64;
        if last.elapsed() > Duration::from_secs(1) {
            status(&format!(
                "Checking {}… {}%",
                disc.title.name,
                read * 100 / disc.bytes
            ));
            last = Instant::now()
        }
        if Instant::now() > deadline {
            return Err("The document provider is too slow for this bounded qualification. The source was preserved.".into());
        }
    }
    let after = file.metadata()?;
    if read != disc.bytes
        || after.len() != before.len()
        || after.modified().ok() != before.modified().ok()
    {
        return Err(
            "The selected media changed while being checked; no machine was started".into(),
        );
    }
    let hash = unsafe { CStr::from_ptr(xbx_digest_string(digest.0)) }.to_str()?;
    // Permanent entry/profile IDs never contain an FD, PID or native handle.
    let binding = format!(
        "media_sha256={hash}\nbytes={}\npartition_offset={}\nfirmware_sha256={firmware}\nram_mib=64\n",
        disc.bytes, disc.partition_offset
    );
    let session = root.join("saved-session");
    let (game_id, profile_id) = if session.exists() {
        let old = fs::read_to_string(&session)?;
        if !old.ends_with(&binding) {
            return Err("This media or boot profile differs from the saved game. Existing progress was preserved.".into());
        }
        let read_id = |prefix: &str| -> Result<String, Box<dyn std::error::Error>> {
            let id = old
                .lines()
                .find_map(|s| s.strip_prefix(prefix))
                .ok_or("Missing durable identity")?;
            if id.len() != 32 || !id.bytes().all(|v| v.is_ascii_hexdigit()) {
                return Err("Invalid durable identity; prior state preserved".into());
            }
            Ok(id.to_owned())
        };
        (read_id("game_id=")?, read_id("profile_id=")?)
    } else {
        let game = opaque_id()?;
        let profile = opaque_id()?;
        write_new(
            &session,
            format!("xbx-saved-session-v1\ngame_id={game}\nprofile_id={profile}\n{binding}")
                .as_bytes(),
        )?;
        (game, profile)
    };
    let saved = crate::session::SavedSession {
        game: crate::session::GameId(game_id),
        profile: crate::session::VirtualConsoleProfileId(profile_id),
        media_sha256: hash.to_owned(),
        firmware_sha256: firmware.to_owned(),
    };
    let uri_path = root.join("media.uri");
    if fs::read_to_string(&uri_path).ok().as_deref() != Some(uri.as_str()) {
        write_new(&root.join("media.uri.pending"), uri.as_bytes())?;
        fs::rename(root.join("media.uri.pending"), uri_path)?;
    }
    File::open(root)?.sync_all()?;
    file.seek(SeekFrom::Start(0))?;
    status("Starting Fable…");
    Ok(LiveRuntime {
        media: file,
        disc,
        saved,
    })
}
